//! UI contributions retain G42 priority/FIFO/abort and source config deactivation.
use super::*;
use stackpress_framework_session::{auth, plugin};
use stackpress_lib::{Flow, Invocation, MethodPattern, Request, Response, Result};
use stackpress_server::{ConfigLayers, ServerBuilder, StartupContext};

/// Additional application handler can populate exactly the source signup result binding.
async fn augment(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&json!({"name":"Plugin person"}))?;
    res.data_mut()?.set("sequence", "first")?;
    Ok(Flow::Continue)
}
/// Equal priority remains FIFO, even though a view was already registered at -100.
async fn equal(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    assert_eq!(
        res.data()
            .get("sequence")
            .and_then(stackpress_lib::Value::as_str),
        Some("first")
    );
    res.data_mut()?.set("sequence", "second")?;
    Ok(Flow::Continue)
}
/// An independent lower-priority observer sees the rendered output.
async fn after(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    assert_eq!(
        res.data()
            .get("sequence")
            .and_then(stackpress_lib::Value::as_str),
        Some("second")
    );
    assert!(matches!(res.body(), stackpress_lib::Body::Text(_)));
    res.data_mut()?.set("after", true)?;
    Ok(Flow::Continue)
}
/// Any handler may stop the chain, preserving its response instead of rendering a page.
async fn stop(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if req.data.has("stop") {
        res.html("Stopped")?;
        return Ok(Flow::Abort);
    }
    Ok(Flow::Continue)
}
/// Install ordinary cooperating handlers; no route ownership is claimed.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    start.get("/auth/signup", augment)?;
    start.get("/auth/signup", equal)?;
    start.route_with(
        stackpress_server::RouteSpec::new(MethodPattern::GET, "/auth/signup").priority(100),
        stop,
    )?;
    start.route_with(
        stackpress_server::RouteSpec::new(MethodPattern::GET, "/auth/signup").priority(-200),
        after,
    )?;
    Ok(Flow::Continue)
}
#[tokio::test]
async fn rendered_route_augmentation_fifo_and_abort() {
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value("fixture", json!({"auth":{},"session":{}}))
        .unwrap();
    app.config_layers(layers).unwrap();
    app.on_config(stackpress_lib::queue::task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.register(std::sync::Arc::new(
                stackpress_framework_csrf::Service::default(),
            ))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.plugin("auth", auth::register).unwrap();
    app.plugin("identity", plugin::register).unwrap();
    app.on_route(routes).unwrap();
    let host = app.build().await.unwrap();
    assert!(
        !host
            .listeners()
            .any(|listener| listener.pattern == "request")
    );
    assert_eq!(
        host.listeners()
            .filter(|listener| listener.priority == -100)
            .count(),
        29
    );
    let mut req = Request::new();
    let mut res = Response::new();
    host.emit("GET /auth/signup", &mut req, &mut res)
        .await
        .unwrap();
    assert_eq!(
        res.data()
            .get("after")
            .and_then(stackpress_lib::Value::as_bool),
        Some(true)
    );
    let stackpress_lib::Body::Text(html) = res.body() else {
        panic!("HTML")
    };
    assert!(html.contains("Plugin person"));
    let mut req = Request::builder()
        .data(json!({"stop":true}))
        .build()
        .unwrap();
    let mut res = Response::new();
    assert_eq!(
        host.emit("GET /auth/signup", &mut req, &mut res)
            .await
            .unwrap(),
        stackpress_lib::DispatchOutcome::Aborted
    );
    assert!(res.data().get("after").is_none());
}
#[tokio::test]
async fn source_missing_config_disables_only_its_contributions() {
    for (session, auth_config, account_routes, auth_routes) in [
        (json!({}), Value::Null, 0, 0),
        (Value::Null, json!({}), 0, 16),
        (json!({}), json!({}), 13, 16),
    ] {
        let mut app = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers
            .value("fixture", json!({"session":session,"auth":auth_config}))
            .unwrap();
        app.config_layers(layers).unwrap();
        app.plugin("auth", auth::register).unwrap();
        app.plugin("identity", plugin::register).unwrap();
        let host = app.build().await.unwrap();
        assert_eq!(
            host.listeners()
                .filter(
                    |listener| listener.priority == -100 && listener.pattern.contains("/account")
                )
                .count(),
            account_routes
        );
        assert_eq!(
            host.listeners()
                .filter(
                    |listener| listener.priority == -100 && !listener.pattern.contains("/account")
                )
                .count(),
            auth_routes
        );
        assert!(
            host.listeners()
                .any(|listener| listener.pattern == "auth-signin")
        );
    }
}

/// The source generic POST view remains available without a matching auth process handler.
#[tokio::test]
async fn source_view_only_signin_fallback_and_concrete_view_precedence() {
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value("fixture", json!({"auth":{},"session":{}}))
        .unwrap();
    app.config_layers(layers).unwrap();
    app.on_config(stackpress_lib::queue::task(|start: &mut StartupContext| {
        Box::pin(async move {
            start.register(std::sync::Arc::new(
                stackpress_framework_csrf::Service::default(),
            ))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.plugin("auth", auth::register).unwrap();
    app.plugin("identity", plugin::register).unwrap();
    let host = app.build().await.unwrap();
    let views: Vec<_> = host
        .listeners()
        .filter(|listener| {
            listener.priority == -100 && listener.pattern.starts_with("POST /auth/signin")
        })
        .map(|listener| listener.pattern.as_str())
        .collect();
    assert_eq!(
        views,
        [
            "POST /auth/signin/2fa/:profile/:auth/:challenge",
            "POST /auth/signin/link/:auth/:challenge",
            "POST /auth/signin/email",
            "POST /auth/signin/otp/:auth/:challenge",
            "POST /auth/signin/phone",
            "POST /auth/signin/username",
            "POST /auth/signin/:type",
        ]
    );
    for (path, title) in [("extension", "Sign In Options"), ("email", "Email Sign In")] {
        let mut req = Request::builder()
            .method("POST")
            .url(format!("https://session.invalid/auth/signin/{path}"))
            .build()
            .unwrap();
        let mut res = Response::new();
        host.handle_route(&mut req, &mut res).await.unwrap();
        let stackpress_lib::Body::Text(html) = res.body() else {
            panic!("missing {path} view")
        };
        assert!(html.contains(&format!("<title>{title}</title>")), "{path}");
        assert_eq!(
            html.matches("<main").count(),
            1,
            "fallback must not replace a concrete view"
        );
    }
}
