//! Source config gates, independent handlers, priority/FIFO and raw guard behavior.
use super::*;

/// Additional handlers can observe/stop the exact same source event and route.
async fn early(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.status_code(401, None)?;
    res.data_mut()?.set("early", true)?;
    Ok(Flow::Continue)
}
async fn later(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    assert_eq!(res.status().unwrap().code, 200);
    res.data_mut()?.set("later", true)?;
    Ok(Flow::Continue)
}
async fn route_later(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("route_later", true)?;
    Ok(Flow::Continue)
}
async fn stop(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if req.data.get("stop").and_then(Lib::as_bool) == Some(true) {
        res.status_code(418, None)?;
        return Ok(Flow::Abort);
    }
    Ok(Flow::Continue)
}
/// Extend the source event family after its own listener contribution.
async fn extra_events(start: &mut StartupContext) -> Result<Flow> {
    start.on_priority("auth-signout", 10, early)?;
    start.on("auth-signout", later)?;
    start.on_priority("auth-signout", 20, stop)?;
    Ok(Flow::Continue)
}
/// Equal route priorities retain the later independent contribution.
async fn extra_routes(start: &mut StartupContext) -> Result<Flow> {
    start.get("/signout", route_later)?;
    Ok(Flow::Continue)
}
#[tokio::test]
async fn source_registration_modes_priority_fifo_and_any_handler_abort() {
    for config in [Json::Null, json!(false), json!(0), json!("")] {
        let mut app = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers.value("fixture", json!({"auth":config})).unwrap();
        app.config_layers(layers).unwrap();
        app.plugin("auth", auth::register).unwrap();
        let host = app.build().await.unwrap();
        assert_eq!(host.listeners().count(), 3);
        assert!(
            !host
                .listeners()
                .any(|listener| listener.pattern.contains('/'))
        );
    }
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value("fixture", json!({"auth":{"base":""}}))
        .unwrap();
    app.config_layers(layers).unwrap();
    app.plugin("auth-with-extensions", |app| {
        auth::register(app)?;
        app.on_listen(extra_events)?;
        app.on_route(extra_routes)?;
        Ok(())
    })
    .unwrap();
    let host = app.build().await.unwrap();
    let patterns: Vec<_> = host
        .listeners()
        .filter(|listener| listener.priority == 0)
        .map(|listener| listener.pattern.clone())
        .collect();
    let expected = [
        "GET /signup",
        "GET /signin",
        "GET /signin/2fa/:profile/:auth/:challenge",
        "GET /signin/link/:auth/:challenge",
        "GET /signin/email",
        "GET /signin/otp/:auth/:challenge",
        "GET /signin/phone",
        "GET /signin/username",
        "GET /signout",
        "POST /signup",
        "POST /signin/2fa/:profile/:auth/:challenge",
        "POST /signin/link/:auth/:challenge",
        "POST /signin/email",
        "POST /signin/otp/:auth/:challenge",
        "POST /signin/phone",
        "POST /signin/username",
    ];
    let routes: Vec<_> = patterns
        .iter()
        .filter(|pattern| pattern.contains(" /"))
        .map(String::as_str)
        .collect();
    assert_eq!(&routes[..16], expected);
    assert_eq!(routes[16], "GET /signout");
    let response = invoke(&host, "auth-signout", json!({})).await;
    assert_eq!(response.status().unwrap().code, 200);
    assert_eq!(
        response.data().get("early").and_then(Lib::as_bool),
        Some(true)
    );
    assert_eq!(
        response.data().get("later").and_then(Lib::as_bool),
        Some(true)
    );
    let mut req = Request::builder()
        .data(json!({"stop":true}))
        .build()
        .unwrap();
    let mut res = Response::new();
    assert_eq!(
        host.emit("auth-signout", &mut req, &mut res).await.unwrap(),
        stackpress_lib::DispatchOutcome::Aborted
    );
    assert_eq!(res.status().unwrap().code, 418);
    assert!(res.data().get("later").is_none());
    let mut req = Request::builder().url("/signout").build().unwrap();
    let mut res = Response::new();
    host.emit("GET /signout", &mut req, &mut res).await.unwrap();
    assert_eq!(res.headers().get("Location"), Some("/"));
    assert_eq!(
        res.data().get("route_later").and_then(Lib::as_bool),
        Some(true)
    );
}
/// Empty-map authorize returns no code, so the source guard still offers the error event.
async fn offered(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("offered", true)?;
    Ok(Flow::Continue)
}
#[tokio::test]
async fn source_empty_access_guard_offers_zero_status_and_falsy_bodies_do_not_skip() {
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value("fixture", json!({"session":{"access":{}}}))
        .unwrap();
    app.config_layers(layers).unwrap();
    app.plugin("identity", stackpress_framework_session::plugin::register)
        .unwrap();
    app.on("error", offered).unwrap();
    let host = app.build().await.unwrap();
    for body in [Json::Null, json!(false), json!(0), json!("")] {
        let mut res = Response::new();
        res.results(&body).unwrap();
        host.emit("request", &mut Request::new(), &mut res)
            .await
            .unwrap();
        assert_eq!(res.data().get("offered").and_then(Lib::as_bool), Some(true));
        assert_eq!(res.status().unwrap().code, 200);
        assert_eq!(res.result().unwrap().to_json().unwrap(), body);
    }
    let mut res = Response::new();
    res.results(&json!({})).unwrap();
    host.emit("request", &mut Request::new(), &mut res)
        .await
        .unwrap();
    assert!(res.data().get("offered").is_none());
}
