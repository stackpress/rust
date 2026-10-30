//! Optional activation, me, cooperative guard and response/cookie parity.
use super::native::service;
use serde_json::json;
use stackpress_framework_session::{Service, SessionPermission, SessionPermissionList, plugin};
use stackpress_lib::{DispatchOutcome, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ConfigLayers, ServerBuilder};

async fn host(section: Option<serde_json::Value>) -> stackpress_server::Server {
    let mut app = ServerBuilder::new();
    if let Some(section) = section {
        let mut layers = ConfigLayers::new();
        layers.value("fixture", json!({"session":section})).unwrap();
        app.config_layers(layers).unwrap();
    }
    app.plugin("identity", plugin::register).unwrap();
    app.build().await.unwrap()
}
#[tokio::test]
async fn missing_empty_and_absent_access_have_distinct_contributions() {
    let absent = host(None).await;
    assert!(absent.host().service::<Service>().is_err());
    assert!(!absent.listeners().any(|l| l.pattern == "me"));
    let unguarded = host(Some(json!({}))).await;
    assert!(unguarded.host().service::<Service>().is_ok());
    assert!(unguarded.listeners().any(|l| l.pattern == "me"));
    assert!(!unguarded.listeners().any(|l| l.pattern == "request"));
    let guarded = host(Some(json!({"access":{}}))).await;
    assert!(guarded.listeners().any(|l| l.pattern == "request"));
    let defaults = host(Some(json!({"key":"","seed":""}))).await;
    assert_eq!(
        defaults.host().service::<Service>().unwrap().config().seed,
        "abc123"
    );
}
#[tokio::test]
async fn me_returns_original_authorization_and_only_reads_configured_cookie() {
    let host = host(Some(json!({"key":"identity"}))).await;
    let svc = host.host().service::<Service>().unwrap();
    let token = svc
        .create(
            json!({"id":"u1","roles":[],"custom":1})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .unwrap();
    let mut request = Request::new();
    request
        .headers
        .set("Authorization", &format!("Bearer {token}"))
        .unwrap();
    let mut response = Response::new();
    host.emit("me", &mut request, &mut response).await.unwrap();
    assert_eq!(response.result().unwrap().to_json().unwrap()["id"], 0);
    request
        .session
        .set("identity", token.clone(), None)
        .unwrap();
    let mut response = Response::new();
    host.emit("me", &mut request, &mut response).await.unwrap();
    let result = response.result().unwrap().to_json().unwrap();
    assert_eq!(result["id"], "u1");
    assert_eq!(result["token"], token);
    assert_eq!(result["custom"], 1);
    svc.load_request(&request).save(&mut response).unwrap();
    assert_eq!(
        response.session().get("identity").unwrap().as_str(),
        Some(token.as_str())
    );
}
#[tokio::test]
async fn authorize_empty_shortcut_route_mutation_and_source_failure() {
    let (svc, _) = service(None);
    let request = Request::builder().url("/private").build().unwrap();
    let mut response = Response::new();
    let mut permits = vec![SessionPermission::Event("extra".into())];
    assert!(
        svc.authorize(&request, &mut response, &mut permits)
            .await
            .unwrap()
    );
    assert_eq!(permits.len(), 1);
    assert!(response.result().is_none());
    let access = SessionPermissionList::from_iter([(
        "GUEST".into(),
        vec![
            SessionPermission::route("GET", "/private"),
            SessionPermission::Event("extra".into()),
        ],
    )]);
    let mut config = svc.config();
    config.access = Some(access);
    svc.configure(config);
    assert!(
        svc.authorize(&request, &mut response, &mut permits)
            .await
            .unwrap()
    );
    assert_eq!(permits.len(), 2);
    let request = Request::builder().url("/forbidden").build().unwrap();
    let mut response = Response::new();
    assert!(
        !svc.authorize(&request, &mut response, &mut vec![])
            .await
            .unwrap()
    );
    assert_eq!(response.status().unwrap().code, 401);
    assert_eq!(response.error(), Some("Unauthorized"));
}
async fn after(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.data_mut()?.set("after", true)?;
    Ok(Flow::Continue)
}
async fn abort(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Ok(Flow::Abort)
}
#[tokio::test]
async fn guard_is_cooperative_and_only_copies_failed_authorization() {
    let h = host(Some(
        json!({"access":{"GUEST":[{"method":"ALL","route":"/open"}]}}),
    ))
    .await;
    let mut req = Request::builder().url("/open").build().unwrap();
    let mut res = Response::new();
    assert_eq!(
        h.emit("request", &mut req, &mut res).await.unwrap(),
        DispatchOutcome::Completed
    );
    assert!(res.result().is_none());
    let mut req = Request::builder().url("/closed").build().unwrap();
    assert_eq!(
        h.emit("request", &mut req, &mut res).await.unwrap(),
        DispatchOutcome::Completed
    );
    assert_eq!(res.status().unwrap().code, 401);
    let mut app = ServerBuilder::new();
    app.on_priority("same", 1, abort).unwrap();
    app.on("same", after).unwrap();
    let app = app.build().await.unwrap();
    assert_eq!(
        app.emit("same", &mut Request::new(), &mut Response::new())
            .await
            .unwrap(),
        DispatchOutcome::Aborted
    );
}
