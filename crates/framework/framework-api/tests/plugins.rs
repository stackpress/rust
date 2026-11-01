//! Selected/direct equivalence, method routing, all-matching chains and finite reentrancy.

use std::sync::Arc;

use serde_json::json;
use stackpress_framework_api::{ApiConfig, ApiType, Cors, Service, endpoints, plugin, routes};
use stackpress_lib::{Flow, Invocation, MethodPattern, Request, Response, Result};
use stackpress_server::{ConfigLayers, RouteSpec, ServerBuilder, StartupContext};

use super::endpoints::{State, direct, endpoint, fixture, request, status, tail};

#[tokio::test]
async fn missing_null_empty_and_webhook_only_config_have_no_m2_routes() -> Result<()> {
    for (wire, active) in [
        (json!({}), false),
        (json!({"api": null}), false),
        (json!({"api": []}), true), // Existing serde struct-sequence form defaults all fields.
        (json!({"api": {}}), true),
        (
            json!({"api": {"endpoints": [], "webhooks": [{"event": "later", "uri": "http://configured",
            "method": "POST", "validity": {}, "data": {}}]}}),
            true,
        ),
    ] {
        let mut app = ServerBuilder::new();
        app.plugin("selected", plugin::register)?;
        let mut layers = ConfigLayers::new();
        layers.value("test", wire)?;
        app.config_layers(layers)?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        assert_eq!(server.host().service::<Service>().is_ok(), active);
        let expected = usize::from(
            server
                .host()
                .service::<Service>()
                .is_ok_and(|service| !service.config.webhooks.is_empty()),
        );
        assert_eq!(server.listeners().count(), expected);
    }
    Ok(())
}

/// The selected service exists before Listen; Route registration still remains a later phase.
async fn observes_config(start: &mut StartupContext) -> Result<Flow> {
    assert_eq!(start.service::<Service>()?.config.endpoints.len(), 1);
    Ok(Flow::Continue)
}

/// Ordinary direct composition retains its outer plugin identity.
fn composite(app: &mut ServerBuilder) -> Result<()> {
    plugin::register(app)
}

#[tokio::test]
async fn selected_plugin_and_direct_inventory_match_without_lookup_resources_at_startup()
-> Result<()> {
    for kind in [ApiType::Public, ApiType::App, ApiType::Session] {
        let config = endpoint(kind, "GET", "/api");
        let state = Arc::new(State::default());
        let mut app = fixture(state.clone())?;
        let mut layers = ConfigLayers::new();
        layers.value(
            "test",
            json!({"api": ApiConfig { endpoints: vec![config.clone()], ..ApiConfig::default() }}),
        )?;
        app.config_layers(layers)?;
        app.plugin("aggregate", composite)?;
        app.on_listen(observes_config)?;
        let selected = app.build().await.map_err(|failure| failure.error)?;
        let direct_state = Arc::new(State::default());
        let direct = direct(&[config], direct_state.clone()).await?;
        let mut selected_response = Response::new();
        let mut direct_response = Response::new();
        for (server, response) in [
            (&selected, &mut selected_response),
            (&direct, &mut direct_response),
        ] {
            server
                .emit(
                    "GET /api",
                    &mut request("GET", "/api", Some("Other id"), json!({"caller": true}))?,
                    response,
                )
                .await?;
        }
        assert_eq!(
            selected_response.result().unwrap().to_json()?,
            direct_response.result().unwrap().to_json()?
        );
        assert_eq!(status(&selected_response), status(&direct_response));
        assert_eq!(
            selected
                .listeners()
                .filter(|listener| listener.plugin.as_deref() == Some("aggregate"))
                .count(),
            3 + usize::from(cfg!(feature = "ui-ssr"))
        );
    }
    // No mandatory store/JWT/Profile service is acquired at startup, even for protected routes.
    let mut app = ServerBuilder::new();
    plugin::register(&mut app)?;
    let mut layers = ConfigLayers::new();
    layers.value(
        "no resources",
        json!({"api": {"endpoints": [endpoint(ApiType::Session, "GET", "/api")]}}),
    )?;
    app.config_layers(layers)?;
    assert!(app.build().await.is_ok());
    Ok(())
}

#[tokio::test]
async fn all_standard_methods_case_normalization_and_extension_methods_route() -> Result<()> {
    let methods = [
        "GET", "HEAD", "POST", "PUT", "DELETE", "CONNECT", "OPTIONS", "TRACE", "PATCH", "PURGE",
    ];
    for config_method in methods.into_iter().chain(["get", "ALL", "all"]) {
        for actual in methods {
            let state = Arc::new(State::default());
            let server = direct(
                &[endpoint(ApiType::Public, config_method, "/api")],
                state.clone(),
            )
            .await?;
            server
                .emit(
                    &format!("{actual} /api"),
                    &mut request(actual, "/api", None, json!({}))?,
                    &mut Response::new(),
                )
                .await?;
            let matches = config_method.eq_ignore_ascii_case("ALL")
                || config_method.eq_ignore_ascii_case(actual);
            assert_eq!(
                state.events().contains(&"target".into()),
                matches,
                "{config_method}/{actual}"
            );
        }
    }
    Ok(())
}

/// Direct helper selection is explicit, regardless of the data-only type field.
type Registrar = fn(&mut ServerBuilder, stackpress_framework_api::ApiEndpoint) -> Result<()>;

#[tokio::test]
async fn direct_route_helpers_choose_the_named_source_adapter() -> Result<()> {
    for (register, expected) in [
        (routes::open as Registrar, None),
        (routes::app as Registrar, Some("application-detail")),
        (routes::session as Registrar, Some("session-detail")),
    ] {
        let state = Arc::new(State::default());
        let mut app = fixture(state.clone())?;
        register(&mut app, endpoint(ApiType::Public, "GET", "/api"))?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        server
            .emit(
                "GET /api",
                &mut request("GET", "/api", Some("X id"), json!({}))?,
                &mut Response::new(),
            )
            .await?;
        let observed: Vec<_> = state
            .events()
            .into_iter()
            .filter(|event| event.ends_with("-detail"))
            .collect();
        assert_eq!(observed, expected.into_iter().collect::<Vec<_>>());
    }
    Ok(())
}

#[tokio::test]
async fn duplicate_paths_priorities_and_registration_ties_remain_all_matching() -> Result<()> {
    let state = Arc::new(State::default());
    let mut config = Vec::new();
    for (name, priority) in [("low", -3), ("first high", 7), ("second high", 7)] {
        let mut endpoint = endpoint(ApiType::Public, "GET", "/api");
        endpoint.priority = priority;
        endpoint.data.insert("source".into(), json!(name));
        config.push(endpoint);
    }
    let mut app = fixture(state.clone())?;
    app.get("/api", tail)?;
    endpoints::routes(&mut app, &config)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    server
        .emit(
            "GET /api",
            &mut request("GET", "/api", None, json!({}))?,
            &mut Response::new(),
        )
        .await?;
    let calls = state.calls();
    let targets: Vec<_> = calls
        .iter()
        .filter(|call| call.event == "target")
        .map(|call| &call.data["source"])
        .collect();
    assert_eq!(
        targets,
        [&json!("first high"), &json!("second high"), &json!("low")]
    );
    assert_eq!(
        state.events(),
        [
            "target",
            "target-tail",
            "target",
            "target-tail",
            "route-tail",
            "target",
            "target-tail"
        ]
    );
    Ok(())
}

#[tokio::test]
async fn child_abort_does_not_stop_outer_route_but_an_outer_abort_does() -> Result<()> {
    let state = Arc::new(State {
        abort_target: true,
        ..State::default()
    });
    let server = direct(&[endpoint(ApiType::Public, "GET", "/api")], state.clone()).await?;
    server
        .emit(
            "GET /api",
            &mut request("GET", "/api", None, json!({}))?,
            &mut Response::new(),
        )
        .await?;
    assert_eq!(state.events(), ["target", "route-tail"]);
    let state = Arc::new(State::default());
    let mut app = fixture(state.clone())?;
    app.route_with(
        RouteSpec::new(MethodPattern::GET, "/api").priority(2000),
        stop,
    )?;
    let mut config = endpoint(ApiType::Public, "GET", "/api");
    config.cors = Some(Cors::Enabled(true));
    endpoints::routes(&mut app, &[config])?;
    app.get("/api", tail)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let mut res = Response::new();
    server
        .emit(
            "GET /api",
            &mut request("GET", "/api", None, json!({}))?,
            &mut res,
        )
        .await?;
    assert!(state.calls().is_empty());
    assert!(res.headers().get("Access-Control-Allow-Origin").is_none());
    Ok(())
}

/// Explicit developer-owned outer control can stop all later matching route contributions.
async fn stop(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Ok(Flow::Abort)
}

#[tokio::test]
async fn rejected_protected_route_can_fall_through_to_another_public_route() -> Result<()> {
    let state = Arc::new(State::default());
    let server = direct(
        &[
            endpoint(ApiType::App, "GET", "/api"),
            endpoint(ApiType::Public, "GET", "/api"),
        ],
        state.clone(),
    )
    .await?;
    server
        .emit(
            "GET /api",
            &mut request("GET", "/api", None, json!({}))?,
            &mut Response::new(),
        )
        .await?;
    let calls = state.calls();
    assert_eq!(calls[0].event, "target");
    assert_eq!(
        calls[0].status,
        Some(401),
        "source adds neither Abort nor prior-status guard"
    );
    assert_eq!(state.events(), ["target", "target-tail", "route-tail"]);
    Ok(())
}

#[tokio::test]
async fn absent_target_is_not_a_startup_allowlist_error_or_nested_no_listener_error() -> Result<()>
{
    let state = Arc::new(State::default());
    let mut config = endpoint(ApiType::Public, "GET", "/api");
    config.event = "configured-without-listener".into();
    config.data.insert("fixed".into(), json!(true));
    let server = direct(&[config], state.clone()).await?;
    let mut res = Response::new();
    res.status_code(409, None)?;
    server
        .emit(
            "GET /api",
            &mut request("GET", "/api", None, json!({}))?,
            &mut res,
        )
        .await?;
    assert_eq!(status(&res), Some(409));
    assert_eq!(state.events(), ["route-tail"]);
    assert_eq!(state.calls()[0].data, json!({"fixed": true}));
    Ok(())
}

#[tokio::test]
async fn invalid_method_is_a_host_mechanics_error_and_root_register_is_the_plugin_alias()
-> Result<()> {
    let mut app = ServerBuilder::new();
    assert!(
        endpoints::routes(&mut app, &[endpoint(ApiType::Public, "bad method", "/api")]).is_err()
    );
    let mut app = ServerBuilder::new();
    stackpress_framework_api::register(&mut app)?;
    assert!(
        plugin::register(&mut app).is_err(),
        "ordinary duplicate config section, not route ownership"
    );
    Ok(())
}

#[test]
#[cfg(unix)]
fn actual_endpoint_reentrancy_and_concurrent_callers_have_process_deadline() {
    assert!(
        super::proof::watchdog(false, "plugins::concurrent_child"),
        "M2 concurrency/reentrancy deadline"
    );
}

#[test]
#[ignore = "invoked only by the external process watchdog parent"]
fn concurrent_child() -> Result<()> {
    std::fs::write(std::env::var("SLICE10_READY").unwrap(), b"ready")
        .map_err(stackpress_lib::Error::from_source)?;
    tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(stackpress_lib::Error::from_source)?
        .block_on(async {
            let state = Arc::new(State {
                barrier: Some(tokio::sync::Barrier::new(2)),
                reenter: true,
                profile_from_id: true,
                ..State::default()
            });
            let server =
                direct(&[endpoint(ApiType::Session, "POST", "/api")], state.clone()).await?;
            let execute = |id: &'static str| {
                let server = &server;
                async move {
                    let mut req = request(
                        "POST",
                        "/api",
                        Some(&format!("Any {id}:stored")),
                        json!({"caller": id}),
                    )?;
                    req.session.set("session", format!("jwt-{id}"), None)?;
                    let mut res = Response::new();
                    server.handle_route(&mut req, &mut res).await?;
                    assert_eq!(
                        req.data.to_json()?,
                        json!({"caller": id, "profileId": format!("profile-{id}")})
                    );
                    assert_eq!(
                        req.session.get("session").unwrap().as_str(),
                        Some(format!("jwt-{id}").as_str())
                    );
                    Ok::<_, stackpress_lib::Error>(())
                }
            };
            let (a, b) = tokio::join!(execute("A"), execute("B"));
            a?;
            b?;
            let calls = state.calls();
            assert_eq!(
                calls
                    .iter()
                    .filter(|call| call.event == "lookup-observer")
                    .count(),
                2
            );
            assert_eq!(
                calls.iter().filter(|call| call.event == "target").count(),
                2
            );
            Ok(())
        })
}
