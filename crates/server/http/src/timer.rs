//! Hyper-owned complete-head timers with an additional native-read progress bound.
//!
//! Hyper creates/drops this timer at its own head parsing boundaries, including heads
//! already in its read-ahead buffer. No raw byte inspection or framing interpretation occurs.
use crate::{monitor::Monitor, report::lock};
use hyper::rt::{Sleep, Timer};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Every HTTP/1 timer supplied by this connection builder is a request-head timer.
#[derive(Clone)]
pub(crate) struct HeadTimer {
    /// Read progress comes from the sole Hyper-owned IO wrapper.
    monitor: Arc<Monitor>,
    /// Validated gap ceiling, distinct from the parser's immutable absolute budget.
    idle: Duration,
}
impl HeadTimer {
    /// Retain progress metadata without owning or reading the IO a second time.
    pub(crate) fn new(monitor: Arc<Monitor>, idle: Duration) -> Self {
        Self { monitor, idle }
    }
}
impl Timer for HeadTimer {
    /// Delegate relative timing to the same absolute/progress implementation.
    fn sleep(&self, duration: Duration) -> Pin<Box<dyn Sleep>> {
        self.sleep_until(Instant::now() + duration)
    }
    /// Hyper's parser supplies the immutable absolute deadline for each head.
    fn sleep_until(&self, deadline: Instant) -> Pin<Box<dyn Sleep>> {
        // A buffered partial successor starts timing here, even if no fresh IO read follows.
        *lock(&self.monitor.last_read) = tokio::time::Instant::now();
        Box::pin(HeadSleep {
            inner: Box::pin(tokio::time::sleep_until(deadline.into())),
            absolute: deadline.into(),
            monitor: Arc::clone(&self.monitor),
            idle: self.idle,
        })
    }
}
/// The producer is Hyper itself: only it determines when a head timer exists.
struct HeadSleep {
    /// One runtime timer reset to the earliest total/progress deadline at each poll.
    inner: Pin<Box<tokio::time::Sleep>>,
    /// Parser-selected deadline; progress never moves this timestamp.
    absolute: tokio::time::Instant,
    /// Observes reads without interpreting buffered bytes or owning another reader.
    monitor: Arc<Monitor>,
    /// Maximum read-progress gap during this parser-owned head.
    idle: Duration,
}
impl Future for HeadSleep {
    type Output = ();
    /// Progress can move the idle deadline but never the absolute parser deadline.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let deadline = self
            .absolute
            .min(*lock(&self.monitor.last_read) + self.idle);
        self.inner.as_mut().reset(deadline);
        self.inner.as_mut().poll(cx)
    }
}
impl Sleep for HeadSleep {}
