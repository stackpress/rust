//! Finite source-order host: real session/CSRF/view/API registrars and SQLite, no listener.
use serde_json::{Value as Json, json};
use stackpress_framework_api::{
    self as api,
    models::{ApplicationActions, Record, SessionActions},
};
use stackpress_framework_session::{
    Service, SessionPermission, auth::Hooks, composite, storage::ProfileActions,
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{ConfigLayers, ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::{Arc, Mutex};

/// Application observations are native resources, not a caller/permission carrier.
#[derive(Default)]
struct Trace {
    phases: Mutex<Vec<&'static str>>,
    calls: Mutex<Vec<(String, bool)>>,
}
/// Record real Config callback order around independently selected ordinary functions.
fn phase(app: &mut ServerBuilder, name: &'static str) -> Result<()> {
    app.on_config(task(move |start: &mut StartupContext| {
        Box::pin(async move {
            start.service::<Trace>()?.phases.lock().unwrap().push(name);
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}
/// Select the existing presentation constituent; the host explicitly supplies its prerequisites.
fn view(app: &mut ServerBuilder) -> Result<()> {
    phase(app, "view")?;
    stackpress_framework_view::plugin::register(app)
}
/// Keep the source session grouping after views and before API/admin.
fn session(app: &mut ServerBuilder) -> Result<()> {
    phase(app, "session")?;
    composite::register(app)
}
/// API acquisition is still source me/session-detail, without a new universal caller policy.
fn api(app: &mut ServerBuilder) -> Result<()> {
    phase(app, "api")?;
    api::register(app)
}
/// Admin registration is application-generated in Rust; this finite app exposes its real Draft.
fn admin(app: &mut ServerBuilder) -> Result<()> {
    phase(app, "admin")?;
    app.get("/consumer/admin", admin_draft)?;
    Ok(())
}
/// Exercise the actual admin library instead of inventing an empty framework registrar.
async fn admin_draft(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let draft = stackpress_framework_admin::Draft::default()
        .snapshot()
        .map_err(Error::from_source)?;
    res.results(&draft)?;
    Ok(Flow::Continue)
}
/// Observe whether a source nested event retained the original browser cookie or got fresh data.
async fn observe(req: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Trace>()?
        .calls
        .lock()
        .unwrap()
        .push((inv.event().into(), req.session.get("identity").is_some()));
    Ok(Flow::Continue)
}
/// The endpoint's own process reads the source-mutated profileId; it adds no identity policy.
async fn read(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&json!({"profileId":req.data.get("profileId").and_then(|value|value.as_str())}))?;
    Ok(Flow::Continue)
}
/// Preserve rich SQL values until the existing public wire projection is requested.
fn result(response: &Response) -> Json {
    response.result().unwrap().to_json().unwrap()
}

/// Build only an in-process application; SQLite files, tasks and native handles close before exit.
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = tempfile::tempdir()?;
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(directory.path().join("consumer.sqlite")).create_if_missing(true),
        )
        .await?,
    ));
    let profiles = ProfileActions::new(engine.clone(), "");
    let applications = ApplicationActions::new(engine.clone(), "");
    let sessions = SessionActions::new(engine.clone(), "");
    profiles.install().await?;
    applications.install().await?;
    sessions.install().await?;
    profiles
        .create(&serde_json::from_value(
            json!({"id":"person","name":"Consumer","roles":["USER"]}),
        )?)
        .await?;
    let mut application: Record = serde_json::from_value(
        json!({"id":"app","profileId":"person","name":"Finite app","secret":"source-app-secret","scopes":["read"]}),
    )?;
    application.insert("expires".into(), api::models::date(0));
    applications.create(&application).await?;

    let trace = Arc::new(Trace::default());
    let resource = engine.clone();
    let observation = trace.clone();
    let mut app = ServerBuilder::new();
    app.on_config(task(move |start: &mut StartupContext| {
        let resource = resource.clone();
        let observation = observation.clone();
        Box::pin(async move {
            start.register(resource)?;
            start.register(observation)?;
            // This separate preference policy denies writes only to its own unused settings paths.
            start.register(Arc::new(
                stackpress_framework_view::preferences::Admission::new(|_| false),
            ))?;
            Ok(Flow::Continue)
        })
    }))?;
    let mut layers = ConfigLayers::new();
    layers.value("consumer",json!({
        "session":{"key":"identity","seed":"x"},"auth":{},"email":{"jsonTransport":true},
        "view_preferences":{"origin":"https://consumer.invalid","base":"/","locales":["en"],"default_locale":"en","aliases":{},"settings_path":"/settings","confirmation_path":"/confirm","serve_theme_bootstrap":false},
        "view_flash":{"origin":"https://consumer.invalid","base":"/","pages":["/consumer/view"]},
        "api":{"endpoints":[{"method":"GET","route":"/api/records","type":"session","event":"record-read","scopes":["read"]}]}
    }))?;
    app.config_layers(layers)?;
    // Source main aggregate selects CSRF before view, then session, API and application admin.
    app.plugin("csrf", stackpress_framework_csrf::plugin::register)?;
    app.plugin("sql", api::sql::register)?;
    app.plugin("view", view)?;
    app.plugin("session", session)?;
    app.plugin("api", api)?;
    app.plugin("admin", admin)?;
    for event in [
        "me",
        "application-detail",
        "session-create",
        "session-detail",
    ] {
        app.on_priority(event, 10, observe)?;
    }
    app.on("record-read", read)?;
    let server = Arc::new(app.build().await.map_err(|failure| failure.error)?);
    server
        .host()
        .service::<Hooks>()?
        .bind(&server, tokio::runtime::Handle::current());
    assert_eq!(
        *trace.phases.lock().unwrap(),
        ["view", "session", "api", "admin"]
    );
    assert!(
        !server
            .listeners()
            .any(|listener| listener.pattern == "request"
                && listener.plugin.as_deref() == Some("session"))
    );
    let identity = server.host().service::<Service>()?;
    let token = identity
        .create(serde_json::from_value(
            json!({"id":"person","roles":["USER"]}),
        )?)
        .await?;

    // API consent reads only the cookie via me. A bearer JWT cannot replace that cookie.
    let mut guest = Request::builder()
        .url("/auth/oauth")
        .data(json!({"client_id":"app","redirect_uri":"/return"}))
        .build()?;
    guest
        .headers
        .set("authorization", &format!("Bearer {token}"))?;
    let mut response = Response::new();
    server.handle_route(&mut guest, &mut response).await?;
    assert!(
        response
            .headers()
            .get("location")
            .unwrap()
            .starts_with("/auth/signin?")
    );
    trace.calls.lock().unwrap().clear();

    // The original mutable Request reaches me; fresh SQL calls get no implicit cookie transfer.
    let mut request=Request::builder().method("POST").url("/auth/oauth").data(json!({"client_id":"app","redirect_uri":"/return","id":"issued","secret":"source-code-secret","scopes":["read"],"profileId":"forged"})).build()?;
    request.session.set("identity", token.clone(), None)?;
    let caller = identity.load_request(&request);
    assert!(!caller.guest().await);
    assert!(caller.can(&[]).await?);
    assert!(
        !caller
            .can(&[SessionPermission::Event("record-read".into())])
            .await?
    );
    let mut response = Response::new();
    server.handle_route(&mut request, &mut response).await?;
    assert_eq!(
        response.headers().get("location"),
        Some("/return?code=issued")
    );
    assert_eq!(
        *trace.calls.lock().unwrap(),
        [
            ("me".into(), true),
            ("application-detail".into(), false),
            ("session-create".into(), false)
        ]
    );
    assert_eq!(
        request
            .session
            .get("identity")
            .and_then(|value| value.as_str()),
        Some(token.as_str())
    );
    let stored = server
        .resolve("session-detail", json!({"id":"issued"}))
        .await?;
    let stored = stored.response.results.unwrap().to_json()?;
    assert_eq!(stored["profileId"], "person");
    assert_eq!(stored["applicationId"], "app");

    // API Session credentials/scopes remain separate from cookie can: source GET accepts id only.
    let mut request = Request::builder()
        .url("/api/records")
        .data(json!({"profileId":"forged"}))
        .build()?;
    request.headers.set("authorization", "source issued")?;
    let mut response = Response::new();
    server.handle_route(&mut request, &mut response).await?;
    assert_eq!(result(&response)["profileId"], "person");
    assert!(request.session.get("identity").is_none());
    assert_eq!(
        trace.calls.lock().unwrap().last(),
        Some(&("session-detail".into(), false))
    );

    let mut request = Request::builder().url("/consumer/admin").build()?;
    let mut response = Response::new();
    server.handle_route(&mut request, &mut response).await?;
    assert_eq!(result(&response), json!({}));
    server.host().service::<Hooks>()?.shutdown().await;
    engine.close().await?;
    println!("slice9a plugin consumer passed");
    Ok(())
}
