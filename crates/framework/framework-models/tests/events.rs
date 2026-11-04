//! Generated event wrappers retain source skips, phase order, input mutations and G42 chaining.
#![cfg(all(feature = "server", not(target_arch = "wasm32")))]
use stackpress_framework_models::generated::stores::{auth, profile};
use stackpress_framework_sql::source::{SourceEnvironment, SourceModel};
use stackpress_lib::{Body, Flow, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

#[tokio::test]
async fn generated_registrars_have_no_startup_io_or_global_guard_and_cooperate() {
    let mut app = ServerBuilder::new();
    profile::register(&mut app).unwrap();
    auth::register(&mut app).unwrap();
    // Multiple identical registrars stay selectable; first response remains a source skip.
    profile::register(&mut app).unwrap();
    let server = app.build().await.unwrap();
    let mut request = Request::new();
    let mut response = Response::new();
    server
        .emit("profile-create", &mut request, &mut response)
        .await
        .unwrap();
    assert!(matches!(response.body(), Body::Empty));
    // No global request guard is installed by either generated family.
    server
        .emit("request", &mut request, &mut response)
        .await
        .unwrap();
    assert!(response.status().is_none());
}

#[tokio::test]
async fn generated_events_real_sqlite_ids_columns_batch_and_earlier_abort() {
    let dir = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("events.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(db));
    profile::ProfileModel::actions(engine.clone(), "event-seed".into(), 0)
        .install()
        .await
        .unwrap();
    let mut app = ServerBuilder::new();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = engine.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(SourceEnvironment {
                    seed: "event-seed".into(),
                    offset_minutes: 0,
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    profile::register(&mut app).unwrap();
    profile::register(&mut app).unwrap();
    app.on_priority("profile-get", -1, stop).unwrap();
    let server = app.build().await.unwrap();
    let mut req = Request::new();
    req.data.set("id", "p").unwrap();
    req.data.set("name", "Ada").unwrap();
    let mut res = Response::new();
    server
        .emit("profile-create", &mut req, &mut res)
        .await
        .unwrap();
    assert_eq!(res.status().unwrap().code, 200);
    assert_eq!(
        res.result()
            .unwrap()
            .as_object()
            .unwrap()
            .get("id")
            .unwrap()
            .as_str(),
        Some("p")
    );
    let mut req = Request::new();
    let mut res = Response::new();
    server
        .emit("profile-detail", &mut req, &mut res)
        .await
        .unwrap();
    assert_eq!(res.status().unwrap().code, 400);
    let mut req = Request::new();
    req.data.set("q", "aD").unwrap();
    let mut res = Response::new();
    server
        .emit("profile-search", &mut req, &mut res)
        .await
        .unwrap();
    assert_eq!(res.status().unwrap().code, 200);
    assert!(req.data.has("columns"));
    let mut req = Request::new();
    req.data.set("id", "p").unwrap();
    req.data.set("name", "New").unwrap();
    let mut res = Response::new();
    server
        .emit("profile-update", &mut req, &mut res)
        .await
        .unwrap();
    assert_eq!(res.status().unwrap().code, 200);
    assert!(req.data.has("updated"));
    let mut req = Request::new();
    let mut res = Response::new();
    server
        .emit("profile-get", &mut req, &mut res)
        .await
        .unwrap();
    assert!(matches!(res.body(),Body::Text(text) if text=="earlier"));
}
/// An earlier selected listener can stop every later generated handler under G42.
async fn stop(
    _: &mut Request,
    res: &mut Response,
    _: stackpress_lib::Invocation<'_>,
) -> Result<Flow> {
    res.set("text/plain", Body::Text("earlier".into()))?;
    Ok(Flow::Abort)
}
