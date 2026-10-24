//! In-process HTTP proofs for the compiled profiles application; no listener is opened.

use serde_json::{Value as Json, json};
use stackpress_example_capability_host::{build_application, customization, profiles};
use stackpress_lib::{
    DeliveryState, DispatchOutcome, Error, Flow, Invocation, Request, Response, Result,
};
use stackpress_server::http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput};
use stackpress_server::{
    ConfigLayers, LifecycleOutcome, LifecycleStage, PluginSpec, ServerBuilder, StageOutcome,
};

/// Construct a native GET request through the adapter's buffered alias, without another dependency.
fn get_request(uri: &str) -> Result<BufferedRequest> {
    // http::Request::new starts with GET; only the explicitly supplied URI is changed.
    let mut input = BufferedRequest::new(Vec::new());
    *input.uri_mut() = uri.parse().map_err(Error::from_source)?;
    Ok(input)
}

/// Decode only an actual native Response and check its status independently from its JSON body.
fn public_envelope(output: HttpOutput, expected_status: u16) -> Result<Json> {
    // Suppressed and unavailable output cannot stand in for a produced HTTP response.
    let HttpOutput::Response(response) = output else {
        panic!("expected the HTTP adapter to publish a native response, got {output:?}");
    };
    assert_eq!(response.status().as_u16(), expected_status);

    // JSON decoding observes wire bytes, independently of the host's in-process response value.
    Ok(serde_json::from_slice(response.body())?)
}

/// The real application operation produces the same complete public envelope through HTTP.
#[tokio::test]
async fn http_and_direct_call_return_the_same_public_envelope() -> Result<()> {
    // Both entries share one compiled application and its immutable profiles service.
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    // Empty matches must stay a successful empty array/zero total, distinct from a missing route.
    for (id, rows, total) in [
        (
            "abc123",
            json!([{"id": "abc123", "name": "Ada Lovelace"}]),
            1,
        ),
        ("missing", json!([]), 0),
    ] {
        let direct = app.call("profile-search", json!({"id": id})).await?;
        let reply = adapter
            .handle(&app, get_request(&format!("/profiles/{id}"))?)
            .await?;

        // Pin a successful result as well as equality, so matching error envelopes cannot pass.
        let envelope = public_envelope(reply.output, 200)?;
        assert_eq!(envelope, direct.response.try_to_json()?);
        assert_eq!(envelope["code"], 200);
        assert_eq!(envelope["total"], total);
        assert_eq!(envelope["results"], rows);
        assert_eq!(reply.report.event, format!("GET /profiles/{id}"));
        assert!(reply.report.failures.is_empty());
        assert!(!reply.report.correlation_id.as_str().is_empty());
        assert_ne!(reply.report.correlation_id, direct.report.correlation_id);
        assert_eq!(
            reply.report.stages.last().unwrap().stage,
            LifecycleStage::Output
        );
        assert_eq!(
            reply.report.stages.last().unwrap().delivery_state,
            DeliveryState::Sent
        );
    }
    Ok(())
}

/// HTTP exposes only registered routes, even when query data names the internal operation.
#[tokio::test]
async fn unknown_http_route_returns_a_public_404_envelope() -> Result<()> {
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;

    // A caller-provided event/id cannot turn an unknown pathname into a capability endpoint.
    let reply = adapter
        .handle(
            &app,
            get_request("/unknown?event=profile-search&id=abc123")?,
        )
        .await?;
    let envelope = public_envelope(reply.output, 404)?;
    assert_eq!(envelope["code"], 404);
    assert!(envelope["error"].is_string());
    assert!(envelope.get("results").is_none());
    assert_eq!(reply.report.operation, Some(DispatchOutcome::NoListener));
    assert_eq!(reply.report.event, "GET /unknown");
    assert!(!reply.report.terminal_failure);
    Ok(())
}

/// Fail the proof immediately if pre-normalization input reaches any lifecycle callback.
async fn forbidden_callback(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // G17 has no normalized Request to give user code, including error handlers.
    panic!("malformed HTTP input must not enter lifecycle callbacks")
}

/// An invalid request target produces G17's 400 and Input/Output report without any callbacks.
#[tokio::test]
async fn malformed_http_input_returns_400_without_callbacks() -> Result<()> {
    let mut builder = ServerBuilder::new();
    builder.plugin("profiles", profiles::register)?;
    // All callbacks would fail the test if normalization fabricated a request for them.
    for event in ["request", "profile-search", "error", "response"] {
        builder.on(event, forbidden_callback)?;
    }
    let app = builder.build().await.map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;

    // Asterisk-form has no pathname. Dot normalization would silently select a real profiles route.
    for uri in ["*", "/ignored/../profiles/abc123"] {
        let reply = adapter.handle(&app, get_request(uri)?).await?;
        let envelope = public_envelope(reply.output, 400)?;
        assert_eq!(envelope["code"], 400);
        assert!(envelope["error"].is_string());
        assert!(envelope.get("results").is_none());
        assert_eq!(reply.report.operation, None);
        assert_eq!(reply.report.failures.len(), 1);
        assert_eq!(reply.report.failures[0].stage, LifecycleStage::Input);
        assert!(reply.report.error_offers.is_empty());
        assert_eq!(
            reply
                .report
                .stages
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            [LifecycleStage::Input, LifecycleStage::Output]
        );
        assert_eq!(reply.report.stages[0].outcome, StageOutcome::Failed);
        assert_eq!(reply.report.stages[1].outcome, StageOutcome::Completed);
        assert_eq!(
            reply.report.outcome,
            LifecycleOutcome::Failed {
                stage: LifecycleStage::Input
            }
        );
        assert!(!reply.report.correlation_id.as_str().is_empty());
    }
    Ok(())
}

/// An independently selected customization changes HTTP output just as it changes direct calls.
#[tokio::test]
async fn customization_is_visible_through_http() -> Result<()> {
    let mut layers = ConfigLayers::new();
    layers.value("HTTP application policy", json!({"profiles": {"limit": 1}}))?;
    let mut builder = ServerBuilder::new();
    builder.config_layers(layers)?;
    builder.plugin("profiles", profiles::register)?;
    builder.plugin_with(
        PluginSpec::new("customization").requires(&["profiles"]),
        customization::register,
    )?;
    let app = builder.build().await.map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;

    // Query input yields to the captured id; request limits cannot widen the host's policy.
    let reply = adapter
        .handle(&app, get_request("/profiles/abc?id=unknown&limit=100")?)
        .await?;
    let envelope = public_envelope(reply.output, 200)?;
    let direct = app.call("profile-search", json!({"id": "abc"})).await?;
    assert_eq!(envelope, direct.response.try_to_json()?);
    assert_eq!(envelope["total"], 1);
    assert_eq!(
        envelope["results"],
        json!([{"id": "abc123", "name": "Customized Ada Lovelace"}])
    );

    // Delegated Abort skips Response hooks but still permits the adapter's sole native output.
    assert_eq!(reply.report.operation, Some(DispatchOutcome::Aborted));
    assert_eq!(
        reply.report.outcome,
        LifecycleOutcome::Aborted {
            stage: LifecycleStage::Operation
        }
    );
    assert!(
        !reply
            .report
            .stages
            .iter()
            .any(|record| record.stage == LifecycleStage::Response)
    );
    assert_eq!(
        reply.report.stages.last().unwrap().stage,
        LifecycleStage::Output
    );
    assert_eq!(
        reply.report.stages.last().unwrap().delivery_state,
        DeliveryState::Sent
    );
    assert!(reply.report.failures.is_empty());
    Ok(())
}
