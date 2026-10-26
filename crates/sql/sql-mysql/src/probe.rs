//! Private test-only transition gates; no production hook or driver escape API.
use stackpress_sql::{ErrorKind, Result, SqlError};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// A precise first-party/native acknowledgement boundary selected by one child scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Phase {
    /// Lease published but no native preparation started.
    BeforePrepare,
    /// Native prepared metadata received; no application execution started.
    AfterPrepare,
    /// Parameters validated/bound, immediately before native stream polling.
    BeforeExecute,
    /// One native row received; the remaining protocol may still be pending.
    AfterRow,
    /// Native stream fully acknowledged, before publishing the owned result.
    AfterExecute,
    /// No native BEGIN command sent yet.
    BeforeBegin,
    /// Native BEGIN acknowledged, before publishing the session.
    AfterBegin,
    /// No native COMMIT command sent yet.
    BeforeCommit,
    /// Native COMMIT acknowledged, before publishing success.
    AfterCommit,
    /// No native rollback command sent yet.
    BeforeRollback,
    /// Native rollback acknowledged.
    AfterRollback,
    /// Consuming close polled once; raw socket owned until its future completes/drops.
    BeforeClose,
    /// Acknowledged success, before reset/reuse.
    BeforeReuse,
    /// Reset acknowledged; connection still guarded, not yet returned to the pool.
    AfterReset,
}
/// One bounded scenario owns one armed gate; child cases execute serially.
static ARMED: Mutex<Option<(Phase, Arc<Gate>, bool)>> = Mutex::new(None);
/// Notification handshake whose release can be withheld to force cancellation/deadline cases.
#[derive(Default)]
pub(crate) struct Gate {
    /// Native/first-party phase reached and the gate removed from the registry.
    entered: Notify,
    /// Explicit test release; no production listener can install a gate.
    release: Notify,
}
impl Gate {
    /// Wait for the operation to reach the exact selected boundary.
    pub(crate) async fn entered(&self) {
        self.entered.notified().await;
    }
    /// Permit acknowledgement to continue in a non-cancellation scenario.
    pub(crate) fn release(&self) {
        self.release.notify_one();
    }
}
/// Arm one pause or injected cleanup error; stale gates are a test failure.
pub(crate) fn arm(phase: Phase, fail: bool) -> Arc<Gate> {
    let gate = Arc::new(Gate::default());
    let mut armed = ARMED.lock().unwrap();
    assert!(armed.is_none(), "previous test gate was not reached");
    *armed = Some((phase, gate.clone(), fail));
    gate
}
/// Pause after taking the registry entry, with no mutex guard held across await.
pub(crate) async fn at(phase: Phase) -> Result<()> {
    let selected = {
        let mut armed = ARMED.lock().unwrap();
        if armed
            .as_ref()
            .is_some_and(|(selected, _, _)| *selected == phase)
        {
            armed.take()
        } else {
            None
        }
    };
    if let Some((_, gate, fail)) = selected {
        gate.entered.notify_one();
        if fail {
            return Err(SqlError::with_source(
                ErrorKind::Driver,
                "injected cleanup transport failure",
                std::io::Error::from(std::io::ErrorKind::BrokenPipe),
            ));
        }
        gate.release.notified().await;
    }
    Ok(())
}
