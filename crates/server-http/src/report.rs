//! Connection-owned exchange diagnostics; publication and transport completion are distinct.

use crate::Cancellation;
use stackpress_lib::Error;
use stackpress_server::HandleReport;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Last observed delivery state after the connection/body owner has released its work.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HttpDelivery {
    /// Hyper rejected input before any normalized host call could start.
    RejectedBeforeHost,
    /// Framework explicitly suppressed output.
    Suppressed,
    /// Adapter published output; producer handoff or native flush remains pending.
    Published,
    /// Producer handoff and native flush completed; this is not a remote acknowledgement.
    Completed,
    /// Producer, adapter or native connection failed; no retry is permitted.
    Failed,
    /// Active loading, handling or delivery was dropped/cancelled.
    Cancelled,
}
/// One final exchange observation; does not automatically serialize developer diagnostics.
pub struct HttpExchangeReport {
    /// Monotonic ID within this connection, independent of caller headers.
    pub exchange_id: u64,
    /// Completed lifecycle/publication history, absent when handling never completed.
    pub host: Option<HandleReport>,
    /// Final delivery observation after connection ownership has ended.
    pub delivery: HttpDelivery,
    /// Bytes yielded to Hyper, never a count of remotely acknowledged bytes.
    pub bytes: u64,
    /// Total local exchange duration.
    pub elapsed: Duration,
    /// First original typed transport/producer failure, retained once.
    pub failure: Option<Arc<Error>>,
    /// Chosen HTTP status, including boundary rejections that only close excess IO.
    pub status: Option<u16>,
}
/// Final result of driving one supplied IO; returned only after its connection is dropped.
pub struct ConnectionReport {
    /// Ordered, final exchange reports, including parser rejection when applicable.
    pub exchanges: Vec<HttpExchangeReport>,
    /// Original connection/handshake failure if any; no disclosure to HTTP output.
    pub failure: Option<Arc<Error>>,
    /// Whether caller stop initiated graceful shutdown.
    pub shutdown: bool,
    /// Grace expired; active work was dropped before returning this report.
    pub forced: bool,
}
/// Mutable exchange state stays private to the connection and its body; no sender task exists.
pub(crate) struct Exchange {
    /// Diagnostic history retained even when the active body or lifecycle is dropped.
    pub(crate) report: HttpExchangeReport,
    /// Monotonic start measures local duration without depending on wall clock adjustments.
    pub(crate) started: Instant,
    /// Shared read-only signal outlives the active work when a handler retains its context.
    pub(crate) cancellation: Cancellation,
    /// A later exchange cannot retroactively fail a previously completed native flush.
    pub(crate) settled: bool,
}
/// Shared only with the one active body; all mutation is short synchronous work.
pub(crate) type ExchangeSlot = Arc<Mutex<Exchange>>;

/// Recover poisoned diagnostic storage during unwind cleanup; never execute user code here.
pub(crate) fn lock<T>(slot: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    slot.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
impl Exchange {
    /// Reserve a report before loading; cancellation can therefore retain an incomplete attempt.
    pub(crate) fn new(id: u64) -> Self {
        Self {
            report: HttpExchangeReport {
                exchange_id: id,
                host: None,
                delivery: HttpDelivery::Cancelled,
                bytes: 0,
                elapsed: Duration::ZERO,
                failure: None,
                status: None,
            },
            started: Instant::now(),
            cancellation: Cancellation::default(),
            settled: false,
        }
    }
    /// First failure wins and cannot be overwritten by subsequent Body Drop or IO errors.
    pub(crate) fn fail(&mut self, error: Arc<Error>) {
        if self.report.failure.is_none() {
            self.report.failure = Some(error);
        }
        self.report.delivery = HttpDelivery::Failed;
        self.cancellation.cancel();
    }
}

/// Consume the registry only after connection/service/body ownership has ended.
pub(crate) fn finish(
    slots: Arc<Mutex<Vec<ExchangeSlot>>>,
    result: stackpress_lib::Result<()>,
    shutdown: bool,
    forced: bool,
) -> ConnectionReport {
    // Drain after Hyper Drop: no body or service future can still mutate these reports.
    let mut exchanges = std::mem::take(&mut *lock(&slots));
    let failure = result.err().map(Arc::new);
    if failure.is_some() && exchanges.iter().all(|slot| lock(slot).settled) {
        let mut rejected = Exchange::new(exchanges.len() as u64 + 1);
        rejected.report.delivery = HttpDelivery::RejectedBeforeHost;
        rejected.report.failure = failure.clone();
        rejected.report.status = failure
            .as_ref()
            .map(|error| error.code())
            .filter(|code| matches!(code, 400 | 408 | 431));
        exchanges.push(Arc::new(Mutex::new(rejected)));
    }
    let reports = exchanges
        .into_iter()
        .map(|slot| {
            let mut state = lock(&slot);
            // Native errors can invalidate a completed handoff; preserve original producer errors.
            if let Some(error) = &failure
                && !state.settled
                && state.report.failure.is_none()
                && !matches!(
                    state.report.delivery,
                    HttpDelivery::Suppressed | HttpDelivery::RejectedBeforeHost
                )
                && !forced
            {
                // Forced drain preserves Cancelled; an ordinary IO failure records Failed.
                state.fail(Arc::clone(error));
            }
            // An ended producer can still have native bytes pending when graceful drain is forced.
            if !state.settled && state.report.delivery == HttpDelivery::Published {
                state.report.delivery = HttpDelivery::Cancelled;
            }
            if !state.settled {
                state.report.elapsed = state.started.elapsed();
            }
            // Moving the report after dropping Hyper avoids clones or completion channels.
            std::mem::replace(&mut state.report, Exchange::new(0).report)
        })
        .collect();
    ConnectionReport {
        exchanges: reports,
        failure,
        shutdown,
        forced,
    }
}

impl std::fmt::Debug for HttpExchangeReport {
    /// Keep implicit report formatting free of native messages and developer source chains.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Original causes remain available only through the explicit diagnostic field.
        formatter
            .debug_struct("HttpExchangeReport")
            .field("exchange_id", &self.exchange_id)
            .field("host", &self.host)
            .field("delivery", &self.delivery)
            .field("bytes", &self.bytes)
            .field("elapsed", &self.elapsed)
            .field("status", &self.status)
            .field(
                "failure_kind",
                &self.failure.as_ref().map(|error| error.kind()),
            )
            .finish()
    }
}
impl std::fmt::Debug for ConnectionReport {
    /// Match HandleReport's redacted default while retaining explicit typed diagnostics.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Formatting report metadata must not implicitly format an underlying native error.
        formatter
            .debug_struct("ConnectionReport")
            .field("exchanges", &self.exchanges)
            .field(
                "failure_kind",
                &self.failure.as_ref().map(|error| error.kind()),
            )
            .field("shutdown", &self.shutdown)
            .field("forced", &self.forced)
            .finish()
    }
}

/// A transport rejection owns no fabricated framework request or lifecycle report.
pub(crate) fn rejected(error: Error, status: u16) -> ConnectionReport {
    let mut exchange = Exchange::new(1);
    exchange.report.delivery = HttpDelivery::RejectedBeforeHost;
    exchange.report.status = Some(status);
    let error = Arc::new(error);
    exchange.report.failure = Some(Arc::clone(&error));
    ConnectionReport {
        exchanges: vec![exchange.report],
        failure: Some(error),
        shutdown: false,
        forced: false,
    }
}
