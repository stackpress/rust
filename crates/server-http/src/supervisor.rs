//! Generic admission and owned task supervision without any listener or process signal handling.

use crate::{ConnectionReport, HttpServer, external, report::lock};
use stackpress_lib::{Error, Result};
use std::{
    collections::VecDeque,
    future::Future,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::Notify,
    task::JoinSet,
};

/// Caller-selected, idempotent stop signal, also the linearization lock for host admission.
#[derive(Clone, Default)]
pub struct Shutdown(Arc<StopState>);
#[derive(Default)]
/// The sticky flag and spawn gate are one lock, giving stop a precise admission boundary.
struct StopState {
    /// Set once under the same gate held through synchronous task spawn.
    stopped: Mutex<bool>,
    /// No payload or work queue; all waiters observe the sticky flag after waking.
    changed: Notify,
}
impl Shutdown {
    /// Create an unsignalled token; clones control the same admission boundary.
    pub fn new() -> Self {
        Self::default()
    }
    /// Atomically close admission; repeated calls are harmless and wake all drain owners.
    pub fn stop(&self) {
        *lock(&self.0.stopped) = true;
        self.0.changed.notify_waiters();
    }
    /// Inspect the sticky stop state without resetting it.
    pub fn is_stopped(&self) -> bool {
        *lock(&self.0.stopped)
    }
    /// Wait for stop without losing a notification between inspection and suspension.
    pub async fn cancelled(&self) {
        loop {
            let notified = self.0.changed.notified();
            if self.is_stopped() {
                return;
            }
            notified.await;
        }
    }
}
/// One caller-supplied accepted IO and trusted peer; no bind or accept syscall is implied.
pub struct AcceptedIo<I> {
    /// Owned input, dropped promptly on rejection outside the admission lock.
    io: I,
    /// Caller-attested metadata, never inferred from forwarded headers.
    peer: SocketAddr,
    #[cfg(feature = "tls")]
    /// An explicit validated TLS selection; None means deliberate plaintext.
    tls: Option<crate::TlsConfig>,
}
impl<I> AcceptedIo<I> {
    /// Supply explicit plaintext transport.
    pub fn plain(io: I, peer: SocketAddr) -> Self {
        Self {
            io,
            peer,
            #[cfg(feature = "tls")]
            tls: None,
        }
    }
    /// Supply explicit TLS transport; admission is acquired before the handshake task.
    #[cfg(feature = "tls")]
    pub fn tls(io: I, peer: SocketAddr, tls: crate::TlsConfig) -> Self {
        Self {
            io,
            peer,
            tls: Some(tls),
        }
    }
}
/// A cancellable source of owned IO. Dropping an accept future must not detach hidden work.
/// The caller owns any external source/backlog; this trait creates no queue or native listener.
pub trait Acceptor: Send {
    /// Owned IO moved into one admitted connection task.
    type Io: AsyncRead + AsyncWrite + Unpin + Send + 'static;
    /// Return one IO, end of source or original typed failure. Stop may drop this future.
    fn accept(&mut self) -> impl Future<Output = Result<Option<AcceptedIo<Self::Io>>>> + Send;
}
/// Final aggregate after all admitted tasks have completed or been aborted and reaped.
pub struct DrainReport {
    /// Total tasks admitted before the exact stop boundary.
    pub admitted: usize,
    /// Excess IO closed before spawning or starting a TLS handshake.
    pub rejected: usize,
    /// Tasks observed completed, including errors.
    pub completed: usize,
    /// Tasks forcibly aborted and reaped at the common drain deadline.
    pub aborted: usize,
    /// Task panics/cancellations distinct from ordinary connection diagnostics.
    pub task_failures: usize,
    /// Last reports, bounded by max_connections; aggregate counters retain full totals.
    pub reports: VecDeque<ConnectionReport>,
    /// Older reports dropped from the bounded return history.
    pub discarded_reports: usize,
    /// Original accept/task failure after cleanup, without automatic native formatting.
    pub failure: Option<Arc<Error>>,
}
impl DrainReport {
    /// Start with no admitted work or fabricated completion.
    fn new() -> Self {
        Self {
            admitted: 0,
            rejected: 0,
            completed: 0,
            aborted: 0,
            task_failures: 0,
            reports: VecDeque::new(),
            discarded_reports: 0,
            failure: None,
        }
    }
    /// Retain bounded recent reports while preserving complete completion counts.
    fn joined(
        &mut self,
        result: std::result::Result<ConnectionReport, tokio::task::JoinError>,
        capacity: usize,
    ) {
        self.completed += 1;
        match result {
            Ok(report) => {
                if self.reports.len() == capacity {
                    self.reports.pop_front();
                    self.discarded_reports += 1;
                }
                self.reports.push_back(report);
            }
            Err(cause) => {
                if cause.is_cancelled() {
                    self.aborted += 1;
                } else {
                    self.task_failures += 1;
                }
                if self.failure.is_none() {
                    self.failure = Some(Arc::new(external("HTTP owned task failed", cause)));
                }
            }
        }
    }
}
impl HttpServer {
    /// Drive a generic acceptor until a caller-owned stop future resolves, then fully drain/reap.
    ///
    /// This is the same stop-future path as BoundHttpServer::serve_until, without a listener.
    /// Source errors remain in DrainReport::failure after cleanup; no signal task is spawned.
    pub async fn serve_acceptor_until<A: Acceptor, F: Future<Output = ()> + Send>(
        self: Arc<Self>,
        acceptor: A,
        stop: F,
    ) -> DrainReport {
        let stopping = Shutdown::new();
        let work = self.serve_acceptor(acceptor, stopping.clone());
        tokio::pin!(work);
        tokio::pin!(stop);
        tokio::select! {biased;()=&mut stop=>{stopping.stop();work.await} report=&mut work=>report}
    }
    /// Supervise a generic IO source with admission before spawn and bounded stop/drain/reap.
    ///
    /// Stop wins over a ready accept. An admission mutex linearizes stop with spawning;
    /// after stop returns, this run cannot admit another task. End/error also drains.
    /// Returned history is bounded; await this method before closing process services.
    /// Dropping this future aborts owned tasks through JoinSet; awaited shutdown additionally
    /// reaps them. Blocking user polls cannot be preempted by asynchronous deadlines.
    ///
    /// This method requires an active Tokio runtime. The example intentionally supplies
    /// an in-memory source rather than opening a listener:
    /// ```
    /// use stackpress_server_http::{AcceptedIo,Acceptor,DrainReport,HttpServer,Shutdown};
    /// struct Inputs(tokio::sync::mpsc::Receiver<AcceptedIo<tokio::io::DuplexStream>>);
    /// impl Acceptor for Inputs {
    ///     type Io=tokio::io::DuplexStream;
    ///     async fn accept(&mut self)->stackpress_lib::Result<Option<AcceptedIo<Self::Io>>> {
    ///         Ok(self.0.recv().await)
    ///     }
    /// }
    /// # async fn example() -> stackpress_lib::Result<DrainReport> {
    /// let app=stackpress_server::ServerBuilder::new().build().await.map_err(|e|e.error)?;
    /// let host=std::sync::Arc::new(HttpServer::new(std::sync::Arc::new(app),Default::default())?);
    /// let (_sender,receiver)=tokio::sync::mpsc::channel(4);
    /// let stop=Shutdown::new();
    /// // A pre-signalled stop never polls the acceptor or admits IO.
    /// stop.stop();
    /// let report=host.serve_acceptor(Inputs(receiver),stop).await;
    /// assert_eq!(report.admitted,0);
    /// # Ok(report) }
    /// ```
    pub async fn serve_acceptor<A: Acceptor>(
        self: Arc<Self>,
        mut acceptor: A,
        stop: Shutdown,
    ) -> DrainReport {
        let mut tasks = JoinSet::new();
        let draining = Shutdown::new();
        let mut report = DrainReport::new();
        loop {
            // A custom always-ready source must not starve connections or stop on one executor thread.
            tokio::task::consume_budget().await;
            tokio::select! {
                biased;
                ()=stop.cancelled()=>break,
                Some(result)=tasks.join_next(),if !tasks.is_empty()=>report.joined(result,self.limits.max_connections),
                accepted=acceptor.accept()=>{
                    let accepted=match accepted {
                        Ok(Some(accepted))=>accepted,
                        Ok(None)=>break,
                        Err(error)=>{report.failure=Some(Arc::new(error));break;}
                    };
                    match self.spawn_accepted(accepted,&stop,&draining,&mut tasks) {
                        AdmissionOutcome::Admitted=>report.admitted+=1,
                        AdmissionOutcome::Rejected=>report.rejected+=1,
                        AdmissionOutcome::Stopped=>break,
                    }
                }
            }
        }
        // Close the source/listener/backlog before draining connections, outside the admission gate.
        drop(acceptor);
        // Admission is already closed for this run; the group shares one absolute drain budget.
        draining.stop();
        drain_tasks(
            &mut tasks,
            &mut report,
            self.limits.max_connections,
            self.limits.shutdown_grace,
        )
        .await;
        report
    }
    /// Linearize stop against permit acquisition and task creation, without any await or IO poll.
    fn spawn_accepted<I>(
        self: &Arc<Self>,
        accepted: AcceptedIo<I>,
        stop: &Shutdown,
        draining: &Shutdown,
        tasks: &mut JoinSet<ConnectionReport>,
    ) -> AdmissionOutcome
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        // Drop rejected IO only after releasing the gate: arbitrary IO Drop never runs under it.
        let gate = lock(&stop.0.stopped);
        if *gate {
            drop(gate);
            drop(accepted);
            return AdmissionOutcome::Stopped;
        }
        #[cfg(feature = "tls")]
        let tls = accepted.tls.is_some();
        #[cfg(not(feature = "tls"))]
        let tls = false;
        let Some(admission) = self.admit(tls) else {
            drop(gate);
            drop(accepted);
            self.observe_rejection();
            return AdmissionOutcome::Rejected;
        };
        // Spawn is synchronous and remains inside the same gate as stop's sticky transition.
        let host = Arc::clone(self);
        let stopping = draining.clone();
        let started = tokio::time::Instant::now();
        tasks.spawn(async move {
            let registry = crate::report::registry();
            let slots = Arc::clone(&registry);
            let driver = Arc::clone(&host);
            let work = async move {
                #[cfg(feature = "tls")]
                if let Some(tls) = accepted.tls {
                    return driver
                        .serve_tls_admitted(
                            accepted.io,
                            tls,
                            stopping.cancelled(),
                            admission,
                            crate::ConnectionContext {
                                peer: accepted.peer,
                                started,
                                slots,
                            },
                        )
                        .await;
                }
                let _admission = admission;
                driver
                    .serve_connection(
                        accepted.io,
                        "http",
                        stopping.cancelled(),
                        crate::ConnectionContext {
                            peer: accepted.peer,
                            started,
                            slots,
                        },
                    )
                    .await
            };
            host.observed(work, registry).await
        });
        drop(gate);
        AdmissionOutcome::Admitted
    }
}

/// Exhaustive outcomes keep IO rejection, already-stopped and owned spawn distinct.
enum AdmissionOutcome {
    Admitted,
    Rejected,
    Stopped,
}
/// One absolute drain deadline covers every task; all abort handles are then observed.
async fn drain_tasks(
    tasks: &mut JoinSet<ConnectionReport>,
    report: &mut DrainReport,
    capacity: usize,
    grace: std::time::Duration,
) {
    let deadline = tokio::time::Instant::now() + grace;
    while !tasks.is_empty() {
        match tokio::time::timeout_at(deadline, tasks.join_next()).await {
            Ok(Some(result)) => report.joined(result, capacity),
            Ok(None) => break,
            Err(_) => {
                tasks.abort_all();
                break;
            }
        }
    }
    // Awaiting each cancelled task is required before the caller can close shared services.
    while let Some(result) = tasks.join_next().await {
        report.joined(result, capacity);
    }
}

impl std::fmt::Debug for DrainReport {
    /// Aggregate formatting follows connection/host redaction; native causes stay explicit.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("DrainReport")
            .field("admitted", &self.admitted)
            .field("rejected", &self.rejected)
            .field("completed", &self.completed)
            .field("aborted", &self.aborted)
            .field("task_failures", &self.task_failures)
            .field("reports", &self.reports)
            .field("discarded_reports", &self.discarded_reports)
            .field("failure_kind", &self.failure.as_ref().map(|e| e.kind()))
            .finish()
    }
}
