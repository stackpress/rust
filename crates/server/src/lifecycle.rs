//! Finite top-level lifecycle; nested dispatch remains raw and output belongs to adapters.

use crate::builder::Server;
use crate::context::CallContext;
use crate::report::{
    CorrelationId, ErrorHandling, ErrorOffer, HandleReport, HostReply, LifecycleStage,
    StageFailure, StageOutcome,
};
use stackpress_lib::{
    Body, DeliveryState, DispatchOutcome, Error, HandlerFuture, IntoRequest, LoadResult, Method,
    Request, RequestView, Resolution, Response, Result,
};
use std::any::Any;
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::Ordering;

/// Trusted per-call adapter metadata, distinct from caller-controlled input fields.
#[derive(Default)]
pub struct HandleOptions {
    /// Native I/O handles should stay in the adapter; this slot carries typed metadata only.
    pub adapter_context: Option<Arc<dyn Any + Send + Sync>>,
}

impl Server {
    /// Handle the exact caller pair once, finalizing buffered input without native output.
    ///
    /// Normal stage errors are retained in the returned report with chosen public output.
    /// Outer errors mean handling could not start (for example correlation exhaustion).
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
    /// let mut req = Request::new();
    /// let mut res = Response::new();
    /// let report = server.handle("greet", &mut req, &mut res).await?;
    /// assert!(req.loaded());
    /// assert!(report.is_success(&res));
    /// assert_eq!(res.delivery_state(), stackpress_lib::DeliveryState::Open);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle(
        &self,
        event: &str,
        req: &mut Request,
        res: &mut Response,
    ) -> Result<HandleReport> {
        // Default metadata cannot be replaced by a field in Request.data.
        self.handle_with_options(event, req, res, HandleOptions::default())
            .await
    }

    /// Run one lifecycle with trusted adapter metadata inherited by every nested dispatch.
    pub async fn handle_with_options(
        &self,
        event: &str,
        req: &mut Request,
        res: &mut Response,
        options: HandleOptions,
    ) -> Result<HandleReport> {
        // Buffered finalization uses the same Input runner as actual adapter loaders.
        self.handle_input(event, req, res, options, |_| async {
            Ok(LoadResult::default())
        })
        .await
    }

    /// Convert owned input inside the correlation/report boundary and return its owned response.
    ///
    /// Conversion failure before a normalized Request exists returns sanitized status 400 and
    /// an Input failure without invoking any callback or fabricating a Request (G17).
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
    /// assert_eq!(reply.response.result().and_then(|value| value.as_str()), Some("hello"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call<I: IntoRequest>(&self, event: &str, input: I) -> Result<HostReply> {
        // Reserve one identity before conversion can fail; the normalized path reuses this report.
        let report = self.start_report(event)?;
        // The owned reply keeps its own budgets and delivery state rather than only an envelope.
        let mut response = Response::new();
        // Conversion decides whether a real Request exists and therefore whether hooks may run.
        match input.into_request() {
            // A normalized request may enter Input and the ordinary error/Response table.
            Ok(mut request) => {
                // Reuse the already allocated report; one top-level call allocates exactly one ID.
                let report = self
                    .run_input(
                        event,
                        &mut request,
                        &mut response,
                        HandleOptions::default(),
                        report,
                        |_| async { Ok(LoadResult::default()) },
                    )
                    .await;
                Ok(HostReply { response, report })
            }
            // Without normalized input, G17 returns 400 and never fabricates callback arguments.
            Err(error) => Ok(no_request_failure(report, response, error)),
        }
    }

    /// Select only the normalized request's method plus pathname; query/data never select events.
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
    /// app.get("/hello/:id", greet)?;
    /// let server = app.build().await?;
    /// let mut req = Request::builder().url("/hello/7?event=ignored").build()?;
    /// let mut res = Response::new();
    /// let report = server.handle_route(&mut req, &mut res).await?;
    /// assert_eq!(report.event, "GET /hello/7");
    /// assert_eq!(req.data.path_as::<String>("id")?, Some("7".into()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle_route(
        &self,
        req: &mut Request,
        res: &mut Response,
    ) -> Result<HandleReport> {
        // Build the selector from checked method/path only; input fields cannot choose a
        // capability.
        let event = route_event(&req.method, req.url.path());
        // Informational URL query values are never copied into route diagnostics.
        self.handle(&event, req, res).await
    }

    /// Resolve a method/path event with host context, without lifecycle/loading/default status.
    pub fn resolve_route<'a, I: IntoRequest + 'a>(
        &'a self,
        method: Method,
        path: &'a str,
        input: I,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        Box::pin(async move {
            // Raw convenience retains raw semantics, including no fabricated correlation.
            let event = route_event(&method, path);
            validate_event(&event)?;
            self.resolve(&event, input).await
        })
    }

    /// Common adapter Input path: allocate once, load once, then enter the same stage machine.
    pub(crate) async fn handle_input<F, Fut>(
        &self,
        event: &str,
        req: &mut Request,
        res: &mut Response,
        options: HandleOptions,
        loader: F,
    ) -> Result<HandleReport>
    where
        F: FnOnce(RequestView) -> Fut + Send,
        Fut: Future<Output = Result<LoadResult>> + Send,
    {
        let report = self.start_report(event)?;
        // The adapter's reader remains caller-owned and is never detached or retried.
        Ok(self
            .run_input(event, req, res, options, report, loader)
            .await)
    }

    /// Construct a no-request Input failure for malformed adapter normalization (G17).
    #[cfg(any(feature = "http", test))] // Only the HTTP adapter and seam tests need this entry.
    pub(crate) fn input_failure(&self, event: &str, error: Error) -> Result<HostReply> {
        // A normalized Request does not exist, so no hook receives invented input.
        Ok(no_request_failure(
            self.start_report(event)?,
            Response::new(),
            error,
        ))
    }

    /// Allocate a checked host-local ID before input validation or callbacks.
    fn start_report(&self, event: &str) -> Result<HandleReport> {
        // A checked atomic reservation cannot reuse an ID after overflow or cancellation.
        let sequence = self
            .next_correlation
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| Error::limit("host correlation exhausted"))?;
        // Relaxed atomic ordering establishes uniqueness; it does not publish handler state.
        // An explicit prefix helps identify this host but adds no cross-process uniqueness.
        let id = match &self.prefix {
            Some(prefix) => format!("{prefix}:{sequence}"),
            None => sequence.to_string(),
        };
        // Invalid/oversized event input is represented with a fixed marker, never copied unbounded.
        let safe_event = if validate_event(event).is_ok() {
            event
        } else {
            "<invalid event>"
        };
        Ok(HandleReport::new(CorrelationId(id), safe_event))
    }

    /// Validate/load one normalized Request before deciding whether later stages may run.
    async fn run_input<F, Fut>(
        &self,
        event: &str,
        req: &mut Request,
        res: &mut Response,
        options: HandleOptions,
        mut report: HandleReport,
        loader: F,
    ) -> HandleReport
    where
        F: FnOnce(RequestView) -> Fut + Send,
        Fut: Future<Output = Result<LoadResult>> + Send,
    {
        // Inspect delivery before loading: frozen or interrupted output cannot reopen handling.
        if res.delivery_state() != DeliveryState::Open {
            // Finalized entry does not validate/load or invoke any lifecycle callback.
            report.observe(LifecycleStage::Input, None, StageOutcome::Finalized, res);
            check_delivery(&mut report, res);
            return report;
        }
        // Validate bounded diagnostic/event input before entering a possibly side-effecting loader.
        let loaded = match validate_event(event) {
            // Loading consumes the request attempt before suspension; cancellation grants no retry.
            Ok(()) => req.load_with(loader).await,
            // An invalid selector fails Input without calling the loader or selected operation.
            Err(error) => Err(error),
        };
        // Only successful Input can enter Request/Operation; normalized failures get one error
        // offer.
        match loaded {
            // Record Input completion before Request begins so stage history reflects execution
            // order.
            Ok(()) => {
                report.observe(LifecycleStage::Input, None, StageOutcome::Completed, res);
                self.run_stages(event, req, res, &options, &mut report)
                    .await;
            }
            Err(error) => {
                let failure = retain(&mut report, LifecycleStage::Input, None, error, res);
                // A normalized request exists, so load/parse failure gets the full G12 offer.
                // Continue to Response only when the error offer permits it and output remains
                // Open.
                if self.offer(req, res, &options, &mut report, failure).await {
                    self.response_stage(req, res, &options, &mut report).await;
                }
            }
        }
        // Apply final empty/body defaults once; this never repeats operation not-found detection.
        normalize(&mut report, res);
        report
    }

    /// Execute request/operation/response once, with explicit short-circuit decisions.
    async fn run_stages(
        &self,
        event: &str,
        req: &mut Request,
        res: &mut Response,
        options: &HandleOptions,
        report: &mut HandleReport,
    ) {
        // Await the guard stage completely before deciding whether the capability may start.
        let request = self
            .stage(
                "request",
                LifecycleStage::Request,
                req,
                res,
                options,
                report,
            )
            .await;
        // A guard Abort stops everything; guard failure may recover output but cannot run
        // Operation.
        match request {
            // Preserve deliberate early output and skip Operation and Response.
            Ok(DispatchOutcome::Aborted) => return,
            Err(failure) => {
                // A failed guard must never run the operation, even after successful recovery.
                if self.offer(req, res, options, report, failure).await {
                    self.response_stage(req, res, options, report).await;
                }
                // The failing guard path ends here even when Error/Response repaired public output.
                return;
            }
            // Completed and missing optional Request listeners both permit ordinary operation
            // selection.
            Ok(_) => {}
        }
        // Even a continuing callback may finalize output; inspect its value before another stage.
        if !check_delivery(report, res) {
            return;
        }
        // Operation policy decides whether Response is still eligible; it never runs Response
        // itself.
        if self.operation_stage(event, req, res, options, report).await {
            self.response_stage(req, res, options, report).await;
        }
    }

    /// Run the selected operation and resolve its result/error/Abort into Response eligibility.
    async fn operation_stage(
        &self,
        event: &str,
        req: &mut Request,
        res: &mut Response,
        options: &HandleOptions,
        report: &mut HandleReport,
    ) -> bool {
        // Nested emits stay raw inside this one selected operation and inherit immutable context.
        let operation = self
            .stage(event, LifecycleStage::Operation, req, res, options, report)
            .await;
        // Raw completion, Abort and errors lead to distinct host decisions without operation retry.
        match operation {
            Ok(outcome) => {
                // Retain the raw outcome even when missing output becomes a synthetic 404 later.
                report.operation = Some(outcome);
                // Abort skips output synthesis; finalized delivery also forbids Response and error
                // mutation.
                if outcome == DispatchOutcome::Aborted || !check_delivery(report, res) {
                    // Tell the caller to skip Response while preserving the operation's completed
                    // mutations.
                    return false;
                }
                // Only ordinary Open completion checks missing output or supplies an absent success
                // status.
                match prepare_operation_output(res) {
                    // A deliberate body or status is ready for the single Response pass.
                    Ok(()) => true,
                    // Missing/default-status failure is an Operation diagnostic, offered through
                    // Error once.
                    Err(error) => {
                        let failure =
                            retain(report, LifecycleStage::Operation, Some(event), error, res);
                        self.offer(req, res, options, report, failure).await
                    }
                }
            }
            // A handler error is offered once, never retried as another operation.
            Err(failure) => self.offer(req, res, options, report, failure).await,
        }
    }

    /// Run Response at most once; its failure may offer Error but never re-enters Response.
    async fn response_stage(
        &self,
        req: &mut Request,
        res: &mut Response,
        options: &HandleOptions,
        report: &mut HandleReport,
    ) {
        // Earlier handlers may consume delivery even when their raw outcome permits Response.
        if !check_delivery(report, res) {
            return;
        }
        // Response gets one dispatch; a failure may offer Error but cannot restart this stage.
        if let Err(failure) = self
            .stage(
                "response",
                LifecycleStage::Response,
                req,
                res,
                options,
                report,
            )
            .await
        {
            // Deliberately ignore the continuation answer: Response has already run once.
            self.offer(req, res, options, report, failure).await;
        }
        // Preserve delivery changes made by either Response or its final Error offer.
        check_delivery(report, res);
    }

    /// Dispatch one non-error stage with immutable per-call metadata and ordered observations.
    async fn stage(
        &self,
        event: &str,
        stage: LifecycleStage,
        req: &mut Request,
        res: &mut Response,
        options: &HandleOptions,
        report: &mut HandleReport,
    ) -> std::result::Result<DispatchOutcome, Arc<StageFailure>> {
        // Each stage borrows a local context; no emitter-global current stage or registry lock
        // exists.
        let context = self.context(report, stage, None, options);
        match self
            .emitter
            .emit_with_context(event, req, res, &context)
            .await
        {
            Ok(outcome) => {
                // Observe the lib outcome honestly; no-listener is normal for optional hooks.
                report.observe(stage, Some(event), stage_outcome(outcome), res);
                Ok(outcome)
            }
            // Own the original failure before any error offer can replace response state.
            Err(error) => Err(retain(report, stage, Some(event), error, res)),
        }
    }

    /// Offer exactly the original failure; secondary/setup failure terminates without recursion.
    async fn offer(
        &self,
        req: &mut Request,
        res: &mut Response,
        options: &HandleOptions,
        report: &mut HandleReport,
        failure: Arc<StageFailure>,
    ) -> bool {
        // Error after finalization is retained by the caller, but frozen output gets no offer.
        if !check_delivery(report, res) {
            return false;
        }
        // Prepare sanitized public state before Error listeners can observe or rebuild it.
        if let Err(error) = fresh_error(res, &failure, None) {
            // Retain setup failure independently, then drop all partial output through fallback.
            retain(report, LifecycleStage::Error, Some("error"), error, res);
            terminal(report, res);
            return false;
        }
        // Successful setup appends no failures, so the retained original is still the report tail.
        let index = report.failures.len() - 1;
        // Error callbacks see Error as their active stage and the original failing stage
        // separately.
        let context = self.context(report, LifecycleStage::Error, Some(failure), options);
        match self
            .emitter
            .emit_with_context("error", req, res, &context)
            .await
        {
            Ok(outcome) => {
                // Record the actual offer outcome; Completed alone does not establish public
                // recovery.
                let handling = match outcome {
                    // An absent optional error hook leaves the original public error intact.
                    DispatchOutcome::NoListener => ErrorHandling::NoListener,
                    // A processed offer may permit Response; final public fields determine success
                    // later.
                    DispatchOutcome::Completed => ErrorHandling::Completed,
                    // Error Abort suppresses later hooks while an Open response remains eligible
                    // for output.
                    DispatchOutcome::Aborted => ErrorHandling::Aborted,
                };
                // Keep the offer tied to its original failure, independently of response
                // replacement.
                report.error_offers.push(ErrorOffer {
                    failure: index,
                    outcome: handling,
                });
                report.observe(
                    LifecycleStage::Error,
                    Some("error"),
                    stage_outcome(outcome),
                    res,
                );
                // An aborted error offer stops Response while leaving Open output eligible.
                let open = check_delivery(report, res);
                open && outcome != DispatchOutcome::Aborted
            }
            Err(error) => {
                // Preserve both failures, even if this callback already finalized output.
                retain(report, LifecycleStage::Error, Some("error"), error, res);
                report.error_offers.push(ErrorOffer {
                    failure: index,
                    outcome: ErrorHandling::Failed,
                });
                // A secondary Error failure terminates hook processing; it is never offered
                // recursively.
                terminal(report, res);
                false
            }
        }
    }

    /// Construct immutable borrowed dispatch metadata without retaining Server or its emitter.
    fn context(
        &self,
        report: &HandleReport,
        stage: LifecycleStage,
        failure: Option<Arc<StageFailure>>,
        options: &HandleOptions,
    ) -> CallContext {
        // Borrow metadata from a local owned context; host state has no emitter/callback back-
        // reference.
        CallContext {
            host: Arc::clone(&self.host),
            correlation: Some(report.correlation_id.0.clone()),
            stage,
            failure,
            adapter_context: options.adapter_context.clone(),
        }
    }
}

/// Distinguish absent output from deliberately supplied body/status before Response hooks.
fn prepare_operation_output(response: &mut Response) -> Result<()> {
    if matches!(response.body(), Body::Empty) && response.status().is_none() {
        // This applies only to ordinary completion, never to cooperative early Abort.
        return Err(Error::new("operation produced no body or status")
            .with_code(404)
            .with_public_message("Not Found"));
    }
    if response.status().is_none() {
        // Null, false, zero and empty text/bytes are supplied bodies and default to success.
        response.status_code(200, None)?;
    }
    Ok(())
}

/// Retain before any response mutation, then record the failed stage's actual delivery state.
fn retain(
    report: &mut HandleReport,
    stage: LifecycleStage,
    event: Option<&str>,
    error: Error,
    response: &Response,
) -> Arc<StageFailure> {
    // Own the error first, so later response setup/recovery cannot erase its diagnostic allocation.
    let failure = report.retain(stage, error);
    // Append the actual failed observation after retaining the first-failure precedence.
    report.observe(stage, event, StageOutcome::Failed, response);
    failure
}

/// Inspect delivery after every stage/offer; incomplete or failed delivery is terminal.
fn check_delivery(report: &mut HandleReport, response: &Response) -> bool {
    // Output eligibility follows actual value state, independent of lifecycle completion or Abort.
    report.output_eligible = response.delivery_state() == DeliveryState::Open;
    if matches!(
        response.delivery_state(),
        DeliveryState::Attempted | DeliveryState::Failed
    ) && !report
        .stages
        .iter()
        .any(|s| s.stage == LifecycleStage::Output)
    {
        // A prior writer may have emitted partial output; retain a diagnostic and never retry.
        report.record_output_failure(
            Error::new("response output was failed or incomplete"),
            response,
        );
    }
    report.output_eligible
}

/// Replace all buffered success/recovery state with the original error's public view and cause.
fn fresh_error(response: &mut Response, failure: &StageFailure, code: Option<u16>) -> Result<()> {
    // Select intentionally public fields; report ownership retains the original private diagnostic.
    let public = failure.error.public_view();
    // Recreate body/header/session/data state under the caller's retained budgets.
    let mut fresh = Response::with_limits(response.limits());
    let error = Error::new("retained host failure")
        .with_cause(SharedError(Arc::clone(&failure.error)))
        .with_code(code.unwrap_or(public.code))
        .with_public_message(public.error)
        .with_errors(public.errors);
    // Validate the complete public view and retained source before replacing caller-visible output.
    fresh.apply_error(error)?;
    // Commit only complete error state; no old body/header/cookie/data/stack survives replacement.
    // Drop all partial success/recovery state together; no asynchronous rollback is implied.
    *response = fresh;
    Ok(())
}

/// Terminal fallback is fresh sanitized status 500, with no second error offer or hook pass.
fn terminal(report: &mut HandleReport, response: &mut Response) {
    // Terminal failure prevents adapters from treating a cosmetic status change as recovery.
    report.terminal_failure = true;
    // Already consumed delivery authority forbids even fallback mutation or another write.
    if !check_delivery(report, response) {
        return;
    }
    let mut fresh = Response::with_limits(response.limits());
    // Generic public error is best effort under caller budgets; status-only 500 always remains.
    let _ = fresh.apply_error(
        Error::new("terminal host failure").with_public_message("Internal Server Error"),
    );
    // A fresh Open value plus standard status cannot fail; tiny budgets may leave the body empty.
    let _ = fresh.status_code(500, None);
    *response = fresh;
}

/// Normalize Open aborted/recovered/response-hook output without rerunning missing-result checks.
fn normalize(report: &mut HandleReport, response: &mut Response) {
    // Only Open output with absent status receives a default; explicit 0/custom codes are retained.
    if !check_delivery(report, response) || response.status().is_some() {
        return;
    }
    let code = if matches!(response.body(), Body::Empty) {
        // Empty early Abort or a cleared Response hook means no content, not another 404.
        204
    } else {
        // Any supplied body, including false/null/zero or empty buffers, is deliberate success
        // output.
        200
    };
    // Open response and a standard status cannot fail the setter's state/label checks.
    if let Err(error) = response.status_code(code, None) {
        report.record_output_failure(error, response);
        terminal(report, response);
    }
}

/// Conversion failure has no normalized input, so return 400 without invoking any callback.
fn no_request_failure(mut report: HandleReport, mut response: Response, error: Error) -> HostReply {
    // Keep the conversion/normalization cause before preparing the G17 public 400 response.
    let failure = retain(&mut report, LifecycleStage::Input, None, error, &response);
    // If public error setup itself fails, retain both diagnostics and take terminal fallback.
    if let Err(error) = fresh_error(&mut response, &failure, Some(400)) {
        retain(&mut report, LifecycleStage::Error, None, error, &response);
        terminal(&mut report, &mut response);
    }
    report.output_eligible = response.delivery_state() == DeliveryState::Open;
    HostReply { response, report }
}

/// Translate the lib outcome without conflating application status or delivery state.
fn stage_outcome(outcome: DispatchOutcome) -> StageOutcome {
    // Translate execution only; status and delivery are deliberately outside DispatchOutcome.
    match outcome {
        DispatchOutcome::Completed => StageOutcome::Completed,
        DispatchOutcome::NoListener => StageOutcome::NoListener,
        DispatchOutcome::Aborted => StageOutcome::Aborted,
    }
}

/// Route authority is a checked method and pathname; query values are excluded.
fn route_event(method: &Method, path: &str) -> String {
    // The method is already checked and the caller supplies pathname rather than URL query data.
    format!("{} {path}", method.as_str())
}

/// Bound event metadata and reject control characters before any callback entry.
pub(crate) fn validate_event(event: &str) -> Result<()> {
    // Reject unbounded/control-bearing selectors before they can enter callbacks or diagnostic
    // frames.
    if event.is_empty() || event.len() > 8192 || event.chars().any(char::is_control) {
        return Err(Error::invalid(
            "event is empty, oversized or contains control characters",
        ));
    }
    Ok(())
}

/// Private source wrapper shares the original Error without inventing an Error Clone contract.
#[derive(Debug)]
struct SharedError(Arc<Error>);
impl fmt::Display for SharedError {
    /// Developer source formatting is explicit; public response construction uses public_view.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Only an explicitly chosen developer/source formatter reaches the original message.
        fmt::Display::fmt(&self.0, f)
    }
}
impl std::error::Error for SharedError {
    /// Expose the original allocation through an ordinary retained source chain.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Borrow the retained original allocation without inventing an Error Clone contract.
        Some(self.0.as_ref())
    }
}

#[cfg(test)]
mod tests {
    //! Unit seams cover forced atomic exhaustion and the adapter-only Input runner.
    use super::*;
    use crate::{ServerBuilder, ServerInvocationExt};
    use stackpress_lib::{Flow, Invocation, Nest};

    /// An error listener sees the real normalized request and original loader failure.
    async fn input_error(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        // Assert the original normalized input/cause, then deliberately rebuild successful public
        // output.
        assert_eq!(req.url.path(), "/normalized");
        assert_eq!(
            inv.failure()?.unwrap().error.message(),
            "private parse failure"
        );
        *res = Response::new();
        res.results(&true)?;
        Ok(Flow::Continue)
    }

    /// Adapter loading fails inside Input, while pre-normalization failures invoke no callback.
    #[tokio::test]
    async fn common_adapter_input_boundary() -> Result<()> {
        // Prepare one real error listener to distinguish normalized load failure from G17 denial.
        let mut builder = ServerBuilder::new();
        builder.on("error", input_error)?;
        let app = builder.build().await.map_err(|e| e.error)?;
        let mut req = Request::builder().url("/normalized").build()?;
        let mut res = Response::new();
        let report = app
            .handle_input(
                "GET /normalized",
                &mut req,
                &mut res,
                HandleOptions::default(),
                |_| async { Err(Error::new("private parse failure")) },
            )
            .await?;
        assert_eq!(report.failures[0].stage, LifecycleStage::Input);
        assert_eq!(report.error_offers.len(), 1);
        assert!(report.is_success(&res));
        let reply =
            app.input_failure("GET /unrepresentable", Error::new("private malformed URI"))?;
        assert_eq!(reply.response.status().unwrap().code, 400);
        assert!(reply.report.error_offers.is_empty());
        assert!(
            !reply
                .response
                .try_to_json()?
                .to_string()
                .contains("private malformed URI")
        );
        Ok(())
    }

    /// Exhaustion is an outer inability-to-start error before conversion/loading/handler entry.
    #[tokio::test]
    async fn checked_correlation_exhaustion_and_host_scope() -> Result<()> {
        let app = ServerBuilder::new().build().await.map_err(|e| e.error)?;
        app.next_correlation.store(u64::MAX, Ordering::Relaxed);
        let mut req = Request::new();
        let mut res = Response::new();
        assert!(app.handle("work", &mut req, &mut res).await.is_err());
        assert!(!req.loaded());
        assert!(res.status().is_none());
        assert!(app.call("work", Nest::new()).await.is_err());
        let first = ServerBuilder::new().build().await.map_err(|e| e.error)?;
        let second = ServerBuilder::new().build().await.map_err(|e| e.error)?;
        // IDs intentionally repeat across independent hosts; they are not distributed IDs.
        assert_eq!(
            first.call("work", Nest::new()).await?.report.correlation_id,
            second
                .call("work", Nest::new())
                .await?
                .report
                .correlation_id
        );
        Ok(())
    }
}
