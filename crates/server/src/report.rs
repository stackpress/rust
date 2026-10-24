//! Ordered lifecycle diagnostics and explicitly local correlation, separate from wire output.

use stackpress_lib::{DeliveryState, DispatchOutcome, Error, Response};
use std::fmt;
use std::sync::Arc;

/// Host stage, independent from a listener's before/action/after phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleStage {
    /// Input conversion, validation or one-attempt loading.
    Input,
    /// General request preprocessing.
    Request,
    /// Selected operation or method/path event.
    Operation,
    /// General response processing.
    Response,
    /// One error offer, never recursively offered to itself.
    Error,
    /// Adapter conversion or delivery, outside lifecycle hooks.
    Output,
    /// Raw dispatch without an invented top-level correlation.
    Dispatch,
}

/// Diagnostic precedence is failure, abort, finalized entry, then ordinary completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleOutcome {
    /// Ordinary lifecycle completed.
    Completed,
    /// Cooperative cancellation stopped subsequent stages.
    Aborted {
        /// Stage that returned Abort.
        stage: LifecycleStage,
    },
    /// First retained failure, even when public recovery succeeded.
    Failed {
        /// First failing stage.
        stage: LifecycleStage,
    },
    /// Already finalized entry skipped handling.
    Finalized {
        /// Stage at which finalization was observed.
        stage: LifecycleStage,
    },
}

/// Result of an actual error-event dispatch, independent of public recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorHandling {
    /// No error listener matched.
    NoListener,
    /// All matching listeners continued; this alone is not recovery.
    Completed,
    /// A listener deliberately stopped remaining lifecycle hooks.
    Aborted,
    /// A listener failed; terminal fallback prevents recursion.
    Failed,
}

/// Actual observation of one host stage; missing hooks are explicitly recorded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StageOutcome {
    /// Successful work (including Input loading).
    Completed,
    /// An optional event matched no listeners.
    NoListener,
    /// Cooperative Abort.
    Aborted,
    /// A caught Result error retained in the report.
    Failed,
    /// Hooks were skipped because delivery authority was already consumed.
    Finalized,
}

/// One ordered lifecycle observation, without request/session/capture payloads.
#[derive(Clone, Debug)]
pub struct StageRecord {
    /// Host phase, separate from listener phases.
    pub stage: LifecycleStage,
    /// Event selected for this dispatch, if any; URLs contain pathname only.
    pub event: Option<String>,
    /// What actually happened in this phase.
    pub outcome: StageOutcome,
    /// Response state after the phase; never an inferred transport success.
    pub delivery_state: DeliveryState,
}

/// Checked per-host identifier; neither globally unique nor an authentication token.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CorrelationId(pub(crate) String);

impl CorrelationId {
    /// Borrow its printable local representation without allocating.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let reply = server.call("greet", serde_json::json!({})).await?;
    /// assert!(!reply.report.correlation_id.as_str().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn as_str(&self) -> &str {
        // The allocator validated the optional prefix before host construction.
        &self.0
    }
}
impl fmt::Display for CorrelationId {
    /// Format only the bounded host-local identity.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The stored identifier was validated/allocated by this host and contains no request
        // payload.
        f.write_str(&self.0)
    }
}

/// Original diagnostic retained independently of mutable/replaced public response state.
#[derive(Clone)]
pub struct StageFailure {
    /// Stage that failed; an error handler sees Error as its own active stage.
    pub stage: LifecycleStage,
    /// Explicit shared ownership preserves original frames, source chain and backtrace.
    pub error: Arc<Error>,
}
impl fmt::Debug for StageFailure {
    /// Keep implicit debug formatting free of developer cause messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Original details remain available through the explicit error accessor.
        f.debug_struct("StageFailure")
            .field("stage", &self.stage)
            .field("frames", &self.error.frames())
            .finish_non_exhaustive()
    }
}

/// One automatic error offer and the failure that it was intended to handle.
#[derive(Clone, Debug)]
pub struct ErrorOffer {
    /// Index of the original failure in HandleReport::failures.
    pub failure: usize,
    /// Actual error-event outcome; none is invented for failed setup.
    pub outcome: ErrorHandling,
}

/// Retained diagnostic history; output success must also inspect the final public Response.
#[derive(Debug)]
pub struct HandleReport {
    /// Allocated once before input conversion/loading.
    pub correlation_id: CorrelationId,
    /// Validated selected event (or a fixed redacted marker for invalid input).
    pub event: String,
    /// Highest-precedence diagnostic outcome, independent of public recovery.
    pub outcome: LifecycleOutcome,
    /// Actual observations in execution order.
    pub stages: Vec<StageRecord>,
    /// Original then secondary failures in observation order.
    pub failures: Vec<StageFailure>,
    /// Actual offers, with no recursive offer for a failed Error stage.
    pub error_offers: Vec<ErrorOffer>,
    /// Selected operation's raw result, absent if skipped or failed.
    pub operation: Option<DispatchOutcome>,
    /// Open response can be attempted by an adapter; Abort does not suppress output.
    pub output_eligible: bool,
    /// Error setup/error-listener/output failure required a terminal fallback or stop.
    pub terminal_failure: bool,
}

/// Owned call output retains budgets/private diagnostics absent from a bare envelope.
#[derive(Debug)]
pub struct HostReply {
    /// Final framework response, still independent of any native delivery.
    pub response: Response,
    /// Ordered diagnostic observations.
    pub report: HandleReport,
}

impl HandleReport {
    /// Start a fresh report before input work; no stage is invented as already completed.
    pub(crate) fn new(correlation_id: CorrelationId, event: &str) -> Self {
        // Initialize an empty history; no stage is invented as completed before it actually runs.
        Self {
            correlation_id,
            event: event.into(),
            outcome: LifecycleOutcome::Completed,
            stages: Vec::new(),
            failures: Vec::new(),
            error_offers: Vec::new(),
            operation: None,
            output_eligible: true,
            terminal_failure: false,
        }
    }

    /// Record an actual observation and apply failure/abort/finalization precedence.
    pub(crate) fn observe(
        &mut self,
        stage: LifecycleStage,
        event: Option<&str>,
        outcome: StageOutcome,
        response: &Response,
    ) {
        // A later abort/finalization never erases an earlier retained failure.
        if self.failures.is_empty() && self.outcome == LifecycleOutcome::Completed {
            // Only the first non-failure stop may establish an abort/finalized diagnostic outcome.
            self.outcome = match outcome {
                // A cooperative stop is diagnostic cancellation rather than output suppression.
                StageOutcome::Aborted => LifecycleOutcome::Aborted { stage },
                // Already-finalized entry skips work without fabricating transport success.
                StageOutcome::Finalized => LifecycleOutcome::Finalized { stage },
                // Completed and missing optional listeners do not themselves stop the lifecycle.
                _ => LifecycleOutcome::Completed,
            };
        }
        // Append immutable observations in execution order, even after an earlier failure/abort.
        self.stages.push(StageRecord {
            stage,
            event: event.map(str::to_owned),
            outcome,
            delivery_state: response.delivery_state(),
        });
        // Eligibility is a snapshot of the current response, not permission to reset adapter
        // authority.
        self.output_eligible = response.delivery_state() == DeliveryState::Open;
    }

    /// Retain the original Error before any fresh-response preparation or error dispatch.
    pub(crate) fn retain(&mut self, stage: LifecycleStage, error: Error) -> Arc<StageFailure> {
        // Explicit Arc ownership lets the report and Error context retain exactly the same Error.
        let failure = StageFailure {
            stage,
            error: Arc::new(error),
        };
        if self.failures.is_empty() {
            // First failure wins even if an error listener later recovers the public response.
            self.outcome = LifecycleOutcome::Failed { stage };
        }
        // Keep every secondary failure while retaining the first failure as the report outcome.
        self.failures.push(failure.clone());
        // The context shares diagnostic ownership only; it contains no Server or callback cycle.
        Arc::new(failure)
    }

    /// Record adapter conversion/write failure without rerunning lifecycle hooks or output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let mut reply = server.call("greet", serde_json::json!({})).await?;
    /// let failure = stackpress_lib::Error::new("writer failed");
    /// reply.report.record_output_failure(failure, &reply.response);
    /// assert!(!reply.report.is_success(&reply.response));
    /// # Ok(())
    /// # }
    /// ```
    pub fn record_output_failure(&mut self, error: Error, response: &Response) {
        // Output errors are terminal observations after lifecycle work; no hook retry is allowed.
        self.retain(LifecycleStage::Output, error);
        // A later public status/body edit cannot make a failed native attempt successful.
        self.terminal_failure = true;
        // Output is a terminal observation; adapters retain their own private attempt token.
        self.observe(LifecycleStage::Output, None, StageOutcome::Failed, response);
    }

    /// Record successful/suppressed native output state after an adapter's sole attempt.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let mut reply = server.call("greet", serde_json::json!({})).await?;
    /// reply.response.stop()?; // A real adapter would instead record its own successful delivery.
    /// reply.report.record_output(&reply.response);
    /// assert!(!reply.report.output_eligible);
    /// # Ok(())
    /// # }
    /// ```
    pub fn record_output(&mut self, response: &Response) {
        // Suppression is intentional, whereas foreign/incomplete output needs a failure record.
        self.observe(
            LifecycleStage::Output,
            None,
            StageOutcome::Completed,
            response,
        );
    }

    /// Evaluate final public success independently from diagnostic failure/recovery history.
    ///
    /// Requires status 200..400, no public error, no terminal failure and no failed/incomplete
    /// delivery. This may be true while output is still Open; it is not native-delivery proof.
    /// Deliberate public recovery may succeed while the diagnostic outcome remains Failed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let reply = server.call("greet", serde_json::json!({})).await?;
    /// assert!(reply.report.is_success(&reply.response));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_success(&self, response: &Response) -> bool {
        // Completed error dispatch alone never establishes recovery or native delivery success.
        !self.terminal_failure
            && response.error().is_none()
            && response
                .status()
                .is_some_and(|s| (200..400).contains(&s.code))
            && !matches!(
                response.delivery_state(),
                DeliveryState::Failed | DeliveryState::Attempted
            )
    }

    /// Opt into developer messages and cause chains, keeping default Display metadata-only.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// let server = app.build().await?;
    /// let reply = server.call("missing", serde_json::json!({})).await?;
    /// let diagnostic = reply.report.developer().to_string();
    /// assert!(diagnostic.contains("missing"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn developer(&self) -> DeveloperReport<'_> {
        // This explicit wrapper marks a deliberate disclosure boundary for CLI verbosity.
        DeveloperReport(self)
    }
}

impl fmt::Display for HandleReport {
    /// Render correlation, host stage and available listener identities without payload values.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Start with bounded correlation/event/outcome metadata; payload and cause text stay
        // private.
        write!(
            f,
            "call {} event {:?}: {:?}",
            self.correlation_id, self.event, self.outcome
        )?;
        // Render failures in observation order without formatting their developer messages.
        for failure in &self.failures {
            write!(f, "; {:?}", failure.stage)?;
            // Walk the retained listener path, using identities and sites rather than captured
            // values.
            for frame in failure.error.frames() {
                // Frames contain identity and sites, never capture/request/service contents.
                write!(
                    f,
                    " [{} plugin {:?} listener {} at {}:{}]",
                    frame.event,
                    frame.plugin,
                    frame.listener,
                    frame.registered.file,
                    frame.registered.line
                )?;
            }
        }
        Ok(())
    }
}

/// Explicit developer-only formatter; never used for an automatic public envelope.
pub struct DeveloperReport<'a>(&'a HandleReport);
impl fmt::Display for DeveloperReport<'_> {
    /// Include retained developer messages and ordinary source-chain traversal.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Developer formatting first includes the same metadata-only summary as normal Display.
        write!(f, "{}", self.0)?;
        // Each explicitly requested developer failure includes its original message and source
        // chain.
        for failure in &self.0.failures {
            write!(f, "\n{:?}: {}", failure.stage, failure.error)?;
            let mut cause = std::error::Error::source(failure.error.as_ref());
            // Bound traversal against application-defined cyclic or excessively long sources.
            for _ in 0..32 {
                // A naturally terminated source chain needs no further formatting or synthetic
                // cause.
                let Some(source) = cause else {
                    break;
                };
                write!(f, "\ncaused by: {source}")?;
                // Advance only after formatting the current source; the finite loop bounds cyclic
                // chains.
                cause = source.source();
            }
        }
        Ok(())
    }
}
