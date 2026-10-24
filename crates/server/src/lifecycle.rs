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
    pub async fn call<I: IntoRequest>(&self, event: &str, input: I) -> Result<HostReply> {
        let report = self.start_report(event)?;
        let mut response = Response::new();
        match input.into_request() {
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
            Err(error) => Ok(no_request_failure(report, response, error)),
        }
    }

    /// Select only the normalized request's method plus pathname; query/data never select events.
    pub async fn handle_route(
        &self,
        req: &mut Request,
        res: &mut Response,
    ) -> Result<HandleReport> {
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
    #[allow(dead_code)] // The buffered adapter consumes this seam in the following milestone.
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
        let sequence = self
            .next_correlation
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| Error::limit("host correlation exhausted"))?;
        // Relaxed atomic ordering establishes uniqueness; it does not publish handler state.
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
        if res.delivery_state() != DeliveryState::Open {
            // Finalized entry does not validate/load or invoke any lifecycle callback.
            report.observe(LifecycleStage::Input, None, StageOutcome::Finalized, res);
            check_delivery(&mut report, res);
            return report;
        }
        let loaded = match validate_event(event) {
            Ok(()) => req.load_with(loader).await,
            Err(error) => Err(error),
        };
        match loaded {
            Ok(()) => {
                report.observe(LifecycleStage::Input, None, StageOutcome::Completed, res);
                self.run_stages(event, req, res, &options, &mut report)
                    .await;
            }
            Err(error) => {
                let failure = retain(&mut report, LifecycleStage::Input, None, error, res);
                // A normalized request exists, so load/parse failure gets the full G12 offer.
                if self.offer(req, res, &options, &mut report, failure).await {
                    self.response_stage(req, res, &options, &mut report).await;
                }
            }
        }
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
        match request {
            Ok(DispatchOutcome::Aborted) => return,
            Err(failure) => {
                // A failed guard must never run the operation, even after successful recovery.
                if self.offer(req, res, options, report, failure).await {
                    self.response_stage(req, res, options, report).await;
                }
                return;
            }
            Ok(_) => {}
        }
        if !check_delivery(report, res) {
            return;
        }
        let operation = self
            .stage(event, LifecycleStage::Operation, req, res, options, report)
            .await;
        match operation {
            Ok(outcome) => {
                report.operation = Some(outcome);
                if outcome == DispatchOutcome::Aborted || !check_delivery(report, res) {
                    return;
                }
                if matches!(res.body(), Body::Empty) && res.status().is_none() {
                    // Typed emptiness catches no-ops and preserves false/null/zero bodies.
                    let error = Error::new("operation produced no body or status")
                        .with_code(404)
                        .with_public_message("Not Found");
                    let failure =
                        retain(report, LifecycleStage::Operation, Some(event), error, res);
                    if !self.offer(req, res, options, report, failure).await {
                        return;
                    }
                } else if res.status().is_none() {
                    // Existing redirect, status-only and custom status values are authoritative.
                    if let Err(error) = res.status_code(200, None) {
                        let failure =
                            retain(report, LifecycleStage::Operation, Some(event), error, res);
                        if !self.offer(req, res, options, report, failure).await {
                            return;
                        }
                    }
                }
            }
            Err(failure) => {
                if !self.offer(req, res, options, report, failure).await {
                    return;
                }
            }
        }
        self.response_stage(req, res, options, report).await;
    }

    /// Run Response at most once; its failure may offer Error but never re-enters Response.
    async fn response_stage(
        &self,
        req: &mut Request,
        res: &mut Response,
        options: &HandleOptions,
        report: &mut HandleReport,
    ) {
        if !check_delivery(report, res) {
            return;
        }
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
        let context = self.context(report, stage, None, options);
        match self
            .emitter
            .emit_with_context(event, req, res, &context)
            .await
        {
            Ok(outcome) => {
                report.observe(stage, Some(event), stage_outcome(outcome), res);
                Ok(outcome)
            }
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
        if !check_delivery(report, res) {
            return false;
        }
        if let Err(error) = fresh_error(res, &failure, None) {
            // Retain setup failure independently, then drop all partial output through fallback.
            retain(report, LifecycleStage::Error, Some("error"), error, res);
            terminal(report, res);
            return false;
        }
        let index = report.failures.len() - 1;
        let context = self.context(report, LifecycleStage::Error, Some(failure), options);
        match self
            .emitter
            .emit_with_context("error", req, res, &context)
            .await
        {
            Ok(outcome) => {
                let handling = match outcome {
                    DispatchOutcome::NoListener => ErrorHandling::NoListener,
                    DispatchOutcome::Completed => ErrorHandling::Completed,
                    DispatchOutcome::Aborted => ErrorHandling::Aborted,
                };
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
                retain(report, LifecycleStage::Error, Some("error"), error, res);
                report.error_offers.push(ErrorOffer {
                    failure: index,
                    outcome: ErrorHandling::Failed,
                });
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
        CallContext {
            host: Arc::clone(&self.host),
            correlation: Some(report.correlation_id.0.clone()),
            stage,
            failure,
            adapter_context: options.adapter_context.clone(),
        }
    }
}

/// Retain before any response mutation, then record the failed stage's actual delivery state.
fn retain(
    report: &mut HandleReport,
    stage: LifecycleStage,
    event: Option<&str>,
    error: Error,
    response: &Response,
) -> Arc<StageFailure> {
    let failure = report.retain(stage, error);
    report.observe(stage, event, StageOutcome::Failed, response);
    failure
}

/// Inspect delivery after every stage/offer; incomplete or failed delivery is terminal.
fn check_delivery(report: &mut HandleReport, response: &Response) -> bool {
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
    let public = failure.error.public_view();
    let mut fresh = Response::with_limits(response.limits());
    let error = Error::new("retained host failure")
        .with_cause(SharedError(Arc::clone(&failure.error)))
        .with_code(code.unwrap_or(public.code))
        .with_public_message(public.error)
        .with_errors(public.errors);
    fresh.apply_error(error)?;
    // Commit only complete error state; no old body/header/cookie/data/stack survives replacement.
    *response = fresh;
    Ok(())
}

/// Terminal fallback is fresh sanitized status 500, with no second error offer or hook pass.
fn terminal(report: &mut HandleReport, response: &mut Response) {
    report.terminal_failure = true;
    if !check_delivery(report, response) {
        return;
    }
    let mut fresh = Response::with_limits(response.limits());
    // Generic public error is best effort under caller budgets; status-only 500 always remains.
    let _ = fresh.apply_error(
        Error::new("terminal host failure").with_public_message("Internal Server Error"),
    );
    let _ = fresh.status_code(500, None);
    *response = fresh;
}

/// Normalize Open aborted/recovered/response-hook output without rerunning missing-result checks.
fn normalize(report: &mut HandleReport, response: &mut Response) {
    if !check_delivery(report, response) || response.status().is_some() {
        return;
    }
    let code = if matches!(response.body(), Body::Empty) {
        204
    } else {
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
    let failure = retain(&mut report, LifecycleStage::Input, None, error, &response);
    if let Err(error) = fresh_error(&mut response, &failure, Some(400)) {
        retain(&mut report, LifecycleStage::Error, None, error, &response);
        terminal(&mut report, &mut response);
    }
    report.output_eligible = response.delivery_state() == DeliveryState::Open;
    HostReply { response, report }
}

/// Translate the lib outcome without conflating application status or delivery state.
fn stage_outcome(outcome: DispatchOutcome) -> StageOutcome {
    match outcome {
        DispatchOutcome::Completed => StageOutcome::Completed,
        DispatchOutcome::NoListener => StageOutcome::NoListener,
        DispatchOutcome::Aborted => StageOutcome::Aborted,
    }
}

/// Route authority is a checked method and pathname; query values are excluded.
fn route_event(method: &Method, path: &str) -> String {
    format!("{} {path}", method.as_str())
}

/// Bound event metadata and reject control characters before any callback entry.
pub(crate) fn validate_event(event: &str) -> Result<()> {
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
        fmt::Display::fmt(&self.0, f)
    }
}
impl std::error::Error for SharedError {
    /// Expose the original allocation through an ordinary retained source chain.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
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
