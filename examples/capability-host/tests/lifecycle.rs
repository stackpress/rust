//! Observability over real app calls; test hooks keep diagnostics out of public rows.

use std::sync::{Arc, Mutex};

use serde_json::json;
use stackpress_example_capability_host::profiles;
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ErrorHandling, HandleReport, LifecycleOutcome, LifecycleStage, Server, ServerBuilder,
    ServerInvocationExt, StageOutcome, StartupContext,
};

/// One test-hook observation of the metadata actually received by a listener.
#[derive(Clone, Debug)]
struct Observation {
    /// Active host stage, which nested operations must inherit.
    stage: LifecycleStage,
    /// Current event, independently identifying delegated capability entry.
    event: String,
    /// Present only for top-level handling, inherited unchanged by nested work.
    correlation: Option<String>,
    /// Original stage offered to the error listener, rather than the listener's own stage.
    failed_stage: Option<LifecycleStage>,
}

/// Test-owned trace; each lock covers only a synchronous vector access, never an await.
#[derive(Default)]
struct Observations(Mutex<Vec<Observation>>);

/// Add diagnostic hooks without modifying any production profiles code.
fn observability(app: &mut ServerBuilder) -> Result<()> {
    // Keep instrumentation in a separate selected plugin and immutable service slot.
    app.on_config(configure_observations)?;
    app.on("request", observe)?;
    app.on_priority("profile-search", 1000, observe)?;
    app.on("response", observe)?;
    app.on("error", observe)?;
    Ok(())
}

/// Allocate a separate trace for each host, with no shared test-global state.
async fn configure_observations(start: &mut StartupContext) -> Result<Flow> {
    // Explicit Arc ownership lets test assertions inspect the same service after calls complete.
    start.register(Arc::new(Observations::default()))?;
    Ok(Flow::Continue)
}

/// Capture real invocation metadata before yielding control back to the host.
async fn observe(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Read metadata before the lock so no fallible framework work occurs while holding it.
    let observation = Observation {
        stage: inv.stage()?,
        event: inv.event().into(),
        correlation: inv.correlation_id()?.map(str::to_owned),
        failed_stage: inv.failure()?.map(|failure| failure.stage),
    };
    inv.service::<Observations>()?
        .0
        .lock()
        .unwrap()
        .push(observation);
    Ok(Flow::Continue)
}

/// Build the unchanged profiles plugin plus test-only observability hooks.
async fn observed_application() -> Result<Server> {
    let mut app = ServerBuilder::new();
    app.plugin("profiles", profiles::register)?;
    app.plugin("observability", observability)?;
    app.build().await.map_err(|error| error.error)
}

/// Drain completed observations without holding a guard across later async calls.
fn take_observations(app: &Server) -> Result<Vec<Observation>> {
    // Moving the completed trace keeps subsequent calls' observations independent.
    Ok(std::mem::take(
        &mut *app.host().service::<Observations>()?.0.lock().unwrap(),
    ))
}

/// Assert one completed lifecycle and metadata shared by its three callback observations.
fn assert_success_trace(report: &HandleReport, observations: &[Observation]) {
    // Input has a stage record but no lifecycle event callback of its own.
    assert_eq!(
        report
            .stages
            .iter()
            .map(|record| record.stage)
            .collect::<Vec<_>>(),
        [
            LifecycleStage::Input,
            LifecycleStage::Request,
            LifecycleStage::Operation,
            LifecycleStage::Response,
        ]
    );
    assert!(
        report
            .stages
            .iter()
            .all(|record| record.outcome == StageOutcome::Completed)
    );
    assert_eq!(report.outcome, LifecycleOutcome::Completed);
    assert!(report.failures.is_empty());
    assert!(!report.correlation_id.as_str().is_empty());
    assert_eq!(
        observations
            .iter()
            .map(|entry| entry.stage)
            .collect::<Vec<_>>(),
        [
            LifecycleStage::Request,
            LifecycleStage::Operation,
            LifecycleStage::Response
        ]
    );

    // The operation hook sees the capability name even when a route delegated to it.
    assert_eq!(observations[1].event, "profile-search");
    for entry in observations {
        assert_eq!(
            entry.correlation.as_deref(),
            Some(report.correlation_id.as_str())
        );
        assert_eq!(entry.failed_stage, None);
    }
}

/// Routed nested work shares one correlation and does not repeat request/response hooks.
#[tokio::test]
async fn delegation_preserves_one_observable_lifecycle() -> Result<()> {
    let app = observed_application().await?;
    let direct = app.call("profile-search", json!({"id": "abc123"})).await?;
    assert_success_trace(&direct.report, &take_observations(&app)?);

    // A route adds nested dispatch but must not add extra lifecycle callbacks or reports.
    let mut request = Request::builder().url("/profiles/abc123").build()?;
    let mut response = Response::new();
    let report = app.handle_route(&mut request, &mut response).await?;
    assert_success_trace(&report, &take_observations(&app)?);
    assert_ne!(report.correlation_id, direct.report.correlation_id);
    assert_eq!(report.event, "GET /profiles/abc123");
    assert_eq!(response.try_to_json()?, direct.response.try_to_json()?);
    Ok(())
}

/// An operation error reaches a real error listener with its original failing stage and ID.
#[tokio::test]
async fn operation_error_reaches_the_error_listener_and_report() -> Result<()> {
    let app = observed_application().await?;
    let reply = app.call("profile-search", json!({"id": ""})).await?;
    let report = &reply.report;
    let observations = take_observations(&app)?;

    // The error listener observes rather than recovers; the response remains a public failure.
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].stage, LifecycleStage::Operation);
    assert_eq!(report.error_offers.len(), 1);
    assert_eq!(report.error_offers[0].outcome, ErrorHandling::Completed);
    assert!(!report.is_success(&reply.response));
    assert_eq!(reply.response.try_to_json()?["code"], 400);
    assert_eq!(
        report
            .stages
            .iter()
            .map(|record| record.stage)
            .collect::<Vec<_>>(),
        [
            LifecycleStage::Input,
            LifecycleStage::Request,
            LifecycleStage::Operation,
            LifecycleStage::Error,
            LifecycleStage::Response,
        ]
    );
    assert_eq!(report.stages[2].outcome, StageOutcome::Failed);

    // Assert inherited metadata where the actual listener ran, not just in the final report.
    let error = observations
        .iter()
        .find(|entry| entry.stage == LifecycleStage::Error)
        .unwrap();
    assert_eq!(error.failed_stage, Some(LifecycleStage::Operation));
    assert_eq!(
        error.correlation.as_deref(),
        Some(report.correlation_id.as_str())
    );
    assert_eq!(error.event, "error");
    let frames = report.failures[0].error.frames();
    assert!(frames.iter().any(|frame| {
        frame.event == "profile-search"
            && frame.plugin.as_deref() == Some("profiles")
            && frame.registered.file.ends_with("profiles.rs")
    }));
    assert!(
        !reply
            .response
            .try_to_json()?
            .as_object()
            .unwrap()
            .contains_key("stack")
    );
    Ok(())
}

/// Reject a request before operation entry to prove Request failure skips delegated search.
async fn reject_request(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A safe public message is independent of the diagnostic developer detail.
    Err(Error::new("test policy rejected this request")
        .with_code(403)
        .with_public_message("Forbidden"))
}

/// Request-stage failure is observable and never enters the route or nested operation.
#[tokio::test]
async fn request_failure_skips_search_and_is_offered_once() -> Result<()> {
    let mut builder = ServerBuilder::new();
    builder.plugin("profiles", profiles::register)?;
    builder.plugin("observability", observability)?;
    builder.on_priority("request", -100, reject_request)?;
    let app = builder.build().await.map_err(|error| error.error)?;

    // Route selection exists, but the failing guard prevents the selected operation from running.
    let mut request = Request::builder().url("/profiles/abc123").build()?;
    let mut response = Response::new();
    let report = app.handle_route(&mut request, &mut response).await?;
    let observations = take_observations(&app)?;
    assert_eq!(report.operation, None);
    assert_eq!(report.failures[0].stage, LifecycleStage::Request);
    assert_eq!(report.error_offers.len(), 1);
    assert_eq!(response.try_to_json()?["code"], 403);
    assert!(
        !observations
            .iter()
            .any(|entry| entry.event == "profile-search")
    );
    assert_eq!(
        observations
            .iter()
            .map(|entry| entry.stage)
            .collect::<Vec<_>>(),
        [
            LifecycleStage::Request,
            LifecycleStage::Error,
            LifecycleStage::Response
        ]
    );
    assert_eq!(observations[1].failed_stage, Some(LifecycleStage::Request));
    Ok(())
}

/// G17 input failure has a correlation/report but runs no callbacks without a normalized request.
#[tokio::test]
async fn malformed_top_level_input_runs_no_lifecycle_hooks() -> Result<()> {
    let app = observed_application().await?;

    // An array cannot become the object-root request expected by Server::call.
    let reply = app.call("profile-search", json!(["abc123"])).await?;
    assert!(!reply.report.correlation_id.as_str().is_empty());
    assert_eq!(reply.report.stages.len(), 1);
    assert_eq!(reply.report.stages[0].stage, LifecycleStage::Input);
    assert_eq!(reply.report.stages[0].outcome, StageOutcome::Failed);
    assert_eq!(reply.report.failures[0].stage, LifecycleStage::Input);
    assert!(reply.report.error_offers.is_empty());
    assert_eq!(reply.report.operation, None);
    assert!(take_observations(&app)?.is_empty());
    assert_eq!(reply.response.try_to_json()?["code"], 400);
    Ok(())
}
