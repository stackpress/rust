//! Connection-local phase/deadline state; IO progress and producer handoff are distinct.

use crate::{
    NetworkLimits,
    report::{ExchangeSlot, HttpDelivery, lock},
};
use stackpress_server::http::stream::StreamLimits;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{sync::Notify, time::Instant};

/// The phase determines whether a deadline may still publish an empty rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    Head,
    Body,
    Lifecycle,
    Output,
    Idle,
}
/// Short-lived synchronous state, never held across IO or application callbacks.
pub(crate) struct State {
    /// Current lifecycle/output phase; native Hyper separately owns its head timer.
    pub(crate) phase: Phase,
    /// Immutable total deadline for the current phase; progress cannot extend it.
    pub(crate) absolute: Instant,
    /// Next progress deadline, reset only by the phase-specific progress source.
    pub(crate) idle: Instant,
    /// Maximum progress gap for head/output, independent from total time.
    pub(crate) idle_budget: Duration,
    /// Active exchange retained until producer completion and native flush both occur.
    pub(crate) current: Option<ExchangeSlot>,
    /// The producer ended or output is known empty; this alone does not release reuse.
    pub(crate) body_done: bool,
    /// Whether this exchange wrote any native bytes; true prevents a timeout fallback.
    pub(crate) wrote: bool,
}
/// Only a connection and its exclusive body/IO share this local wakeup and state.
pub(crate) struct Monitor {
    /// Connection-only diagnostic/timing state; never held across a native poll.
    pub(crate) state: Mutex<State>,
    /// Wakes the supervisor after phase/progress transitions without a sender task.
    pub(crate) changed: Notify,
    /// Validated immutable phase defaults, copied once per admitted connection.
    limits: NetworkLimits,
    /// Hyper's head timer reads this progress clock, including for buffered partial successors.
    pub(crate) last_read: Mutex<Instant>,
}
impl Monitor {
    /// Begin head timing at admission, even when the peer sends no bytes.
    pub(crate) fn new(limits: &NetworkLimits) -> Arc<Self> {
        let now = Instant::now();
        Arc::new(Self {
            state: Mutex::new(State {
                phase: Phase::Head,
                absolute: now + limits.head_timeout,
                idle: now + limits.head_idle_timeout,
                idle_budget: limits.head_idle_timeout,
                current: None,
                body_done: false,
                wrote: false,
            }),
            changed: Notify::new(),
            limits: limits.clone(),
            last_read: Mutex::new(now),
        })
    }
    /// Wait for the previous body and native flush, so a second lifecycle cannot overlap.
    pub(crate) async fn turn(&self) {
        loop {
            let notified = self.changed.notified();
            if lock(&self.state).current.is_none() {
                return;
            }
            notified.await;
        }
    }
    /// A parsed head starts a fresh exchange; Hyper alone produced that request.
    pub(crate) fn begin(&self, slot: ExchangeSlot) {
        let now = Instant::now();
        let mut state = lock(&self.state);
        state.phase = Phase::Body;
        state.absolute = now + self.limits.body_timeout;
        state.idle = now + self.limits.body_idle_timeout;
        state.idle_budget = self.limits.body_idle_timeout;
        state.current = Some(slot);
        state.body_done = false;
        state.wrote = false;
        drop(state);
        self.changed.notify_waiters();
    }
    /// Loader completion/failure starts the separate absolute lifecycle budget.
    pub(crate) fn lifecycle(&self) {
        let now = Instant::now();
        let mut state = lock(&self.state);
        state.phase = Phase::Lifecycle;
        state.absolute = now + self.limits.lifecycle_timeout;
        state.idle = state.absolute;
        drop(state);
        self.changed.notify_waiters();
    }
    /// Publication starts total output timing and actual-write progress timing.
    pub(crate) fn output(&self, stream: Option<StreamLimits>) {
        let now = Instant::now();
        let mut state = lock(&self.state);
        let total = stream.map_or(self.limits.response_timeout, |s| {
            s.total_timeout.min(self.limits.response_timeout)
        });
        let idle = stream.map_or(self.limits.write_idle_timeout, |s| {
            s.idle_timeout.min(self.limits.write_idle_timeout)
        });
        state.phase = Phase::Output;
        state.absolute = now + total;
        state.idle = now + idle;
        state.idle_budget = idle;
        state.body_done = false;
        drop(state);
        self.changed.notify_waiters();
    }
    /// Producer completion does not release the next lifecycle before buffered writes flush.
    pub(crate) fn body_done(&self) {
        lock(&self.state).body_done = true;
        self.changed.notify_waiters();
    }
    /// Count actual IO read progress, without letting body/pipelined reads extend write budgets.
    pub(crate) fn read(&self) {
        let now = Instant::now();
        *lock(&self.last_read) = now;
        let mut state = lock(&self.state);
        if state.phase == Phase::Idle {
            state.phase = Phase::Head;
            state.absolute = now + self.limits.head_timeout;
            state.idle_budget = self.limits.head_idle_timeout;
        }
        if state.phase == Phase::Head {
            state.idle = now + state.idle_budget;
        }
        drop(state);
        self.changed.notify_waiters();
    }
    /// Native writes reset only the progress deadline; total output time never moves.
    pub(crate) fn wrote(&self) {
        let mut state = lock(&self.state);
        state.wrote = true;
        if state.phase == Phase::Output {
            state.idle = Instant::now() + state.idle_budget;
        }
        drop(state);
        self.changed.notify_waiters();
    }
    /// A completed producer plus flush establishes the previous exchange's final local boundary.
    pub(crate) fn flushed(&self) {
        let mut state = lock(&self.state);
        if state.phase == Phase::Output && state.body_done {
            if let Some(slot) = state.current.take() {
                let mut exchange = lock(&slot);
                exchange.settled = true;
                exchange.report.elapsed = exchange.started.elapsed();
                // Completed here still means local handoff/flush, never a remote ACK.
                if exchange.report.delivery == HttpDelivery::Published {
                    exchange.report.delivery = HttpDelivery::Completed;
                }
            }
            state.phase = Phase::Idle;
            state.absolute = Instant::now() + self.limits.keep_alive_idle;
            state.idle = state.absolute;
            state.wrote = false;
        }
        drop(state);
        self.changed.notify_waiters();
    }
    /// Sleep until a phase deadline, rechecking after progress and state transitions.
    pub(crate) async fn expired(&self) -> Phase {
        loop {
            let notified = self.changed.notified();
            let deadline = {
                let state = lock(&self.state);
                // The loader owns Body timeouts so G12 can offer an Input error exactly once.
                if state.phase == Phase::Body {
                    None
                } else {
                    let deadline = state.absolute.min(state.idle);
                    // Progress wakeups cannot postpone an already-expired absolute clock.
                    if Instant::now() >= deadline {
                        return state.phase;
                    }
                    Some(deadline)
                }
            };
            let Some(deadline) = deadline else {
                notified.await;
                continue;
            };
            tokio::select! {
                biased;
                ()=notified=>{},
                ()=tokio::time::sleep_until(deadline)=>{
                    let state=lock(&self.state);
                    if state.phase!=Phase::Body && Instant::now()>=state.absolute.min(state.idle) { return state.phase; }
                }
            }
        }
    }
}
