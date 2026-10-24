//! Public-envelope, configuration, routing and independent customization proofs.

use serde_json::json;
use stackpress_example_capability_host::{build_application, customization, profiles};
use stackpress_lib::{DispatchOutcome, Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ConfigLayers, LifecycleOutcome, LifecycleStage, PluginSpec, ServerBuilder, StageOutcome,
    StartupStage,
};

/// Direct and routed requests produce exactly the same public envelope and capture policy.
#[tokio::test]
async fn direct_call_and_route_share_the_public_envelope() -> Result<()> {
    // Build once so both paths use the same compiled plugin and service allocation.
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    let direct = app.call("profile-search", json!({"id": "abc123"})).await?;

    // Conflicting query, POST and explicit values must yield to the route's captured id.
    let mut request = Request::builder()
        .method("GET")
        .url("/profiles/abc123?id=query")
        .post(json!({"id": "post"}))
        .data(json!({"id": "explicit"}))
        .build()?;
    // Independently constructed values remain editable before handling or native output.
    request.data.set("id", "manually edited")?;
    let mut response = Response::new();
    response
        .headers_mut()?
        .set("x-example", "before handling")?;
    let report = app.handle_route(&mut request, &mut response).await?;

    // Compare the entire public envelope, including status and totals, not just a row field.
    let expected = direct.response.try_to_json()?;
    assert_eq!(response.try_to_json()?, expected);
    assert_eq!(expected["code"], 200);
    assert_eq!(expected["total"], 1);
    assert_eq!(
        expected["results"],
        json!([{"id": "abc123", "name": "Ada Lovelace"}])
    );
    assert!(report.is_success(&response));
    assert_eq!(response.headers().get("x-example"), Some("before handling"));
    // Handling leaves native delivery to the adapter, so callers can still edit response headers.
    response
        .headers_mut()?
        .set("x-reviewed", "after handling")?;
    assert!(response.headers().has("x-reviewed"));
    assert_eq!(
        request.data.path_as::<String>("id")?.as_deref(),
        Some("abc123")
    );
    assert_eq!(
        request.query.path_as::<String>("id")?.as_deref(),
        Some("query")
    );
    assert_eq!(
        request.post.path_as::<String>("id")?.as_deref(),
        Some("post")
    );
    Ok(())
}

/// Defaults, a JSON file-shaped layer, then an explicit code layer change actual results.
#[tokio::test]
async fn configuration_layers_change_the_service_limit() -> Result<()> {
    // Each host is independently built from the prefix of the same ordered layer sequence.
    for (json_layer, code_layer, expected) in [(false, false, 3), (true, false, 2), (true, true, 1)]
    {
        let mut layers = ConfigLayers::new();
        // The JSON layer overrides defaults only when it is explicitly supplied.
        if json_layer {
            layers.json("application.json", r#"{"profiles":{"limit":2}}"#)?;
        }
        // Code is appended last, so it wins over the supplied file-shaped layer.
        if code_layer {
            layers.value("application code", json!({"profiles": {"limit": 1}}))?;
        }
        let app = build_application(layers)
            .await
            .map_err(|error| error.error)?;
        let reply = app.call("profile-search", json!({"id": "abc"})).await?;

        // Observe output length/order as well as the typed config that constructed the service.
        let envelope = reply.response.try_to_json()?;
        assert_eq!(envelope["total"], expected);
        assert_eq!(envelope["results"].as_array().unwrap().len(), expected);
        assert_eq!(envelope["results"][0]["id"], "abc123");
        assert_eq!(
            app.host().config::<profiles::ProfilesConfig>()?.limit,
            expected
        );

        // A later seed row must still match under limit one: filter precedes truncation.
        let later = app.call("profile-search", json!({"id": "abc4"})).await?;
        assert_eq!(
            later.response.try_to_json()?["results"],
            json!([{"id": "abc456", "name": "Grace Hopper"}])
        );
    }
    Ok(())
}

/// Invalid limits and unknown config fields fail before a usable service can be published.
#[tokio::test]
async fn invalid_configuration_is_rejected_before_startup() -> Result<()> {
    // Include both semantic bounds, serde field policy and host-level section ownership.
    for value in [
        json!({"profiles": {"limit": 0}}),
        json!({"profiles": {"limit": 101}}),
        json!({"profiles": {"limit": -1}}),
        json!({"profiles": {"limit": "two"}}),
        json!({"profiles": {"limti": 1}}),
        json!({"unselected-plugin": {}}),
    ] {
        let mut layers = ConfigLayers::new();
        layers.value("invalid config", value)?;
        let error = build_application(layers)
            .await
            .err()
            .expect("invalid startup");
        assert_eq!(error.stage, StartupStage::Config);
    }

    // The upper bound itself is valid, although the demo contains only three rows.
    let mut layers = ConfigLayers::new();
    layers.value("maximum valid limit", json!({"profiles": {"limit": 100}}))?;
    let app = build_application(layers)
        .await
        .map_err(|error| error.error)?;
    let reply = app.call("profile-search", json!({"id": "abc"})).await?;
    assert_eq!(reply.response.try_to_json()?["total"], 3);
    Ok(())
}

/// Missing/empty/non-string IDs fail; a valid unknown or differently cased prefix is empty.
#[tokio::test]
async fn input_validation_and_empty_matches_are_explicit() -> Result<()> {
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    // Each invalid input reaches the operation and its ordinary error lifecycle.
    for input in [
        json!({}),
        json!({"id": ""}),
        json!({"id": 123}),
        json!({"id": null}),
    ] {
        let reply = app.call("profile-search", input).await?;
        assert!(!reply.report.is_success(&reply.response));
        assert_eq!(reply.report.failures[0].stage, LifecycleStage::Operation);
        assert!(reply.response.to_envelope().error.is_some());
    }

    // Empty matches remain successful rows with an explicit zero total.
    for id in ["unknown", "ABC", "abc123-extra"] {
        let reply = app.call("profile-search", json!({"id": id})).await?;
        let envelope = reply.response.try_to_json()?;
        assert_eq!(envelope["code"], 200);
        assert_eq!(envelope["results"], json!([]));
        assert_eq!(envelope["total"], 0);
        assert!(reply.report.is_success(&reply.response));
    }
    Ok(())
}

/// Unsupported methods and paths cannot implicitly select an arbitrary capability.
#[tokio::test]
async fn route_exposure_is_explicit() -> Result<()> {
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    // Neither another method, a bare collection, nor an event-looking query selects search.
    for (method, path) in [
        ("POST", "/profiles/abc123"),
        ("GET", "/profiles"),
        ("GET", "/unknown?event=profile-search&id=abc123"),
    ] {
        let mut request = Request::builder().method(method).url(path).build()?;
        let mut response = Response::new();
        let report = app.handle_route(&mut request, &mut response).await?;
        assert_eq!(report.operation, Some(DispatchOutcome::NoListener));
        assert_eq!(response.try_to_json()?["code"], 404);
    }
    Ok(())
}

/// A response hook would visibly change output if the customization's Abort were discarded.
async fn response_marker(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Observe the lifecycle boundary without changing the original profiles plugin.
    res.headers_mut()?.set("x-response-hook", "ran")?;
    Ok(Flow::Continue)
}

/// An independently selected plugin overrides both entry paths and preserves service policy.
#[tokio::test]
async fn customization_changes_behavior_without_editing_profiles() -> Result<()> {
    let ordinary = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    let original = ordinary
        .call("profile-search", json!({"id": "abc"}))
        .await?;

    // The application supplies selection/dependency order; customization owns its handler.
    let mut layers = ConfigLayers::new();
    layers.value("customized app policy", json!({"profiles": {"limit": 1}}))?;
    let mut builder = ServerBuilder::new();
    builder.config_layers(layers)?;
    builder.plugin("profiles", profiles::register)?;
    builder.plugin_with(
        PluginSpec::new("customization").requires(&["profiles"]),
        customization::register,
    )?;
    builder.on("response", response_marker)?;
    let app = builder.build().await.map_err(|error| error.error)?;

    // The overriding listener must win and stop the original from rewriting these names.
    let direct = app
        .call("profile-search", json!({"id": "abc", "limit": 100}))
        .await?;
    let envelope = direct.response.try_to_json()?;
    assert_eq!(envelope["total"], 1);
    assert_eq!(envelope["results"][0]["name"], "Customized Ada Lovelace");
    // Customization creates output copies; it cannot mutate the original shared service rows.
    let seed = app
        .host()
        .service::<profiles::Profiles>()?
        .search("abc123")
        .await?;
    assert_eq!(seed[0].name, "Ada Lovelace");
    assert_eq!(
        original.response.try_to_json()?["results"][0]["name"],
        "Ada Lovelace"
    );
    assert_eq!(direct.report.operation, Some(DispatchOutcome::Aborted));
    assert_eq!(
        direct.report.outcome,
        LifecycleOutcome::Aborted {
            stage: LifecycleStage::Operation
        }
    );
    assert!(direct.report.is_success(&direct.response));
    assert!(
        !direct
            .report
            .stages
            .iter()
            .any(|record| record.stage == LifecycleStage::Response)
    );
    assert!(!direct.response.headers().has("x-response-hook"));

    // Nested delegate must propagate Abort too, rather than start/finish a second lifecycle.
    let mut request = Request::builder().url("/profiles/abc").build()?;
    let mut response = Response::new();
    let report = app.handle_route(&mut request, &mut response).await?;
    assert_eq!(response.try_to_json()?, envelope);
    assert_eq!(report.operation, Some(DispatchOutcome::Aborted));
    assert!(
        !report
            .stages
            .iter()
            .any(|record| record.stage == LifecycleStage::Response)
    );

    // Failure before Abort still uses the ordinary host error path.
    let invalid = app.call("profile-search", json!({"id": ""})).await?;
    assert_eq!(invalid.report.failures[0].stage, LifecycleStage::Operation);
    assert_eq!(invalid.report.stages[2].outcome, StageOutcome::Failed);
    Ok(())
}

/// Declared plugin order errors remain visible rather than silently reordering the app.
#[tokio::test]
async fn customization_requires_the_profiles_prerequisite() -> Result<()> {
    // No profiles selection means no typed config/service can satisfy customization.
    let mut builder = ServerBuilder::new();
    builder.plugin_with(
        PluginSpec::new("customization").requires(&["profiles"]),
        customization::register,
    )?;
    let error = builder.build().await.err().expect("missing profiles");
    assert_eq!(error.stage, StartupStage::Registration);
    Ok(())
}
