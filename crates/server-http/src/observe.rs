//! Bounded redacted observations after exclusive IO/future ownership has ended.
use crate::{
    ConnectionReport, HttpDelivery,
    report::{self, Exchange, Registry, lock},
};
use stackpress_lib::{Error, ErrorKind};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::{Duration, Instant},
};

/// Fixed-size final metadata: no URI, header, body, address, SQL or native error message.
#[derive(Clone, Debug)]
pub struct ExchangeObservation {
    /// Checked host-local identity, never adopted from a request header.
    pub connection_id: u64,
    /// Bounded connection-local exchange identity.
    pub exchange_id: u64,
    /// Selected status, absent for cancellation before a response exists.
    pub status: Option<u16>,
    /// Last local delivery state, never a remote acknowledgement.
    pub delivery: HttpDelivery,
    /// Bytes handed to Hyper, not remotely acknowledged bytes.
    pub bytes: u64,
    /// Final local duration.
    pub elapsed: Duration,
    /// Classification only; no implicit native source formatting.
    pub error_kind: Option<ErrorKind>,
    /// Whether a completed framework lifecycle report exists.
    pub host_report: bool,
}
/// Synchronous nonblocking sink, invoked outside host/registry/body locks and after active IO Drop.
/// Panic follows ordinary unwind policy; callbacks are not retried. No observation queue is created.
///
/// # Examples
/// A closure can update bounded application-owned counters without retaining payloads.
/// ```
/// use stackpress_server_http::{ExchangeObservation, HttpDelivery, NetworkOptions};
/// use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
/// let completed = Arc::new(AtomicU64::new(0));
/// let sink = Arc::clone(&completed);
/// let options = NetworkOptions::http("127.0.0.1:8080".parse()?, "http://localhost:8080/")
///     .with_observer(Arc::new(move |event: ExchangeObservation| {
///         // Count local completion; it is not a remote acknowledgement.
///         if event.delivery == HttpDelivery::Completed {
///             sink.fetch_add(1, Ordering::Relaxed);
///         }
///     }));
/// assert!(options.observer.is_some());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait ExchangeObserver: Send + Sync {
    /// Receive one final bounded observation; retaining/forwarding it is the caller's policy.
    fn observe(&self, observation: ExchangeObservation);
}
impl<F: Fn(ExchangeObservation) + Send + Sync> ExchangeObserver for F {
    /// Closures follow the same nonblocking/unwind contract as explicit observer implementations.
    fn observe(&self, observation: ExchangeObservation) {
        self(observation)
    }
}
/// The inner future is always dropped before the callback, including cancellation and unwind.
pub(crate) struct Observed<F> {
    /// Option permits dropping pinned native work before any callback, without moving its future.
    future: Option<Pin<Box<F>>>,
    /// Retained even when cancellation prevents a normal ConnectionReport return.
    slots: Registry,
    /// Taking this option consumes callback authority before entering user code.
    observer: Option<Arc<dyn ExchangeObserver>>,
    /// Checked host-local identity, shared by this connection's ordered exchange snapshots.
    id: u64,
    /// Native attempts without parsed heads still have a bounded measurable duration.
    started: Instant,
}
impl<F> Observed<F> {
    /// Own the serving future and its diagnostic registry without a sender task/channel.
    pub(crate) fn new(
        future: F,
        slots: Registry,
        observer: Arc<dyn ExchangeObserver>,
        id: u64,
    ) -> Self {
        Self {
            future: Some(Box::pin(future)),
            slots,
            observer: Some(observer),
            id,
            started: Instant::now(),
        }
    }
    /// Remove observer authority before calling user code, so panic cannot replay observations.
    fn notify(&mut self, report: &ConnectionReport) {
        if let Some(observer) = self.observer.take() {
            notify(&*observer, self.id, report);
        }
    }
}
impl<F: Future<Output = ConnectionReport>> Future for Observed<F> {
    type Output = ConnectionReport;
    /// Completion callbacks see no remaining active inner work and hold no registry lock.
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Observe a task poll panic outside its unwind, then resume the original payload unchanged.
        // The future is never resumed after a panic; AssertUnwindSafe permits only its destruction.
        let polled = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.future
                .as_mut()
                .expect("observed future polled after completion")
                .as_mut()
                .poll(cx)
        }));
        let report = match polled {
            Ok(Poll::Pending) => return Poll::Pending,
            Ok(Poll::Ready(report)) => report,
            Err(payload) => {
                self.future.take();
                let report = report::finish(
                    Arc::clone(&self.slots),
                    Err(Error::new("HTTP owned task panicked").with_kind(ErrorKind::External)),
                    false,
                    false,
                );
                self.notify(&report);
                std::panic::resume_unwind(payload)
            }
        };
        self.future.take();
        self.notify(&report);
        Poll::Ready(report)
    }
}
impl<F> Drop for Observed<F> {
    /// Drop native work first; cancelled futures still emit one final observation.
    fn drop(&mut self) {
        self.future.take();
        if self.observer.is_none() || std::thread::panicking() {
            return;
        }
        // Do not call an observer during another panic: normal task failure reporting owns unwind.
        if lock(&self.slots).is_empty() {
            let mut exchange = Exchange::new(1);
            exchange.started = self.started;
            lock(&self.slots).push(Arc::new(Mutex::new(exchange)));
        }
        let report = report::finish(Arc::clone(&self.slots), Ok(()), false, true);
        self.notify(&report);
    }
}
/// Snapshot only fixed-size approved metadata; the report already owns its values without locks.
pub(crate) fn notify(observer: &dyn ExchangeObserver, id: u64, report: &ConnectionReport) {
    for exchange in &report.exchanges {
        observer.observe(ExchangeObservation {
            connection_id: id,
            exchange_id: exchange.exchange_id,
            status: exchange.status,
            delivery: exchange.delivery,
            bytes: exchange.bytes,
            elapsed: exchange.elapsed,
            error_kind: exchange
                .failure
                .as_ref()
                .map(|error| error.kind())
                .or_else(|| report.failure.as_ref().map(|error| error.kind())),
            host_report: exchange.host.is_some(),
        });
    }
}
