//! Real Lane A M2 registrars, SQL actions and auth processes join CSRF/email/SSR.
//! Account views remain Lane A M3; only this test's explicit public projection renders.

mod rendering;
mod signup;
mod support;
mod transports;

use serde_json::{Value as Json, json};
use stackpress_framework_csrf::Service as Csrf;
use stackpress_framework_email::{events::EMAIL_SEND, plugin};
use stackpress_framework_session::{
    auth::{self, Environment, Hooks},
    storage::{AuthActions, ProfileActions},
};
use stackpress_lib::{
    Flow, Invocation, PublicError, Request, Response, Result, handler, queue::task,
};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::{Arc, Mutex};
use support::Factory;

/// Original source output defines page invocation order and complete template fields.
fn vectors() -> Json {
    serde_json::from_str(include_str!("../../fixtures/source-page-boundaries.json")).unwrap()
}
/// An owned test directory uses the already selected OS entropy provider, without a new crate.
struct DatabaseDirectory(std::path::PathBuf);
impl DatabaseDirectory {
    /// Create only this test's nonce-named directory; never reuse another fixture's file.
    fn new() -> Self {
        use ring::rand::SecureRandom;
        let mut nonce = [0; 16];
        ring::rand::SystemRandom::new().fill(&mut nonce).unwrap();
        let path = std::env::temp_dir().join(format!("slice9b-m4-{}", u128::from_be_bytes(nonce)));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for DatabaseDirectory {
    /// SQL engines are explicitly closed before this owned test resource is removed.
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Freeze Date.now independently from real SQL timestamps, as the source oracle does.
struct FixedClock;
impl stackpress_framework_session::Clock for FixedClock {
    fn now(&self) -> f64 {
        1_700_000_000.0
    }
}
/// Owned finite database and transport observations; no alternative auth implementation.
struct Fixture {
    case: Json,
    csrf: Csrf,
    trace: Mutex<Vec<String>>,
    factory: Arc<Factory>,
    offered: Mutex<Vec<Json>>,
    hook_entered: tokio::sync::Semaphore,
    hook_release: tokio::sync::Semaphore,
    hook_input: Mutex<Vec<Json>>,
    engine: Arc<Engine>,
    _directory: DatabaseDirectory,
}
impl Fixture {
    /// Trace observable event boundaries rather than instrumenting production services.
    fn record(&self, event: &str) {
        self.trace.lock().unwrap().push(event.into());
    }
}
/// Establish native operation status before Lane A's independently selected process.
async fn begin(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.status_code(200, None)?;
    res.data_mut()?.set("finite_form", true)?;
    Ok(Flow::Continue)
}
/// Observe reusable events without intercepting their production handlers or responses.
async fn observe(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Fixture>()?.record(inv.event());
    Ok(Flow::Continue)
}
/// Build real SQL-backed auth constituents; only external delivery and the clock are finite.
async fn host(case: Json, config: Option<Json>) -> (Arc<Server>, Arc<Fixture>) {
    rendering::initialize();
    let directory = DatabaseDirectory::new();
    let database = SqliteDatabase::open(
        SqliteOptions::file(directory.0.join("auth.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(database));
    let profiles = ProfileActions::new(engine.clone(), "database-seed");
    profiles.install().await.unwrap();
    let actions = AuthActions::new(engine.clone(), "database-seed");
    actions.install().await.unwrap();
    // Explicit IDs and consumed time let Rust compare original-source templates byte for byte.
    profiles
        .create(&serde_json::from_value(json!({"id":"profile-id","name":"Recipient"})).unwrap())
        .await
        .unwrap();
    actions.create(&serde_json::from_value(json!({"id":"auth-id","profileId":"profile-id","type":"email","token":"recipient@example.test","secret":"correct"})).unwrap()).await.unwrap();
    let state = Arc::new(Fixture {
        factory: Factory::new(case["throws"] == true),
        case,
        csrf: Csrf::default(),
        trace: Mutex::new(vec![]),
        offered: Mutex::new(vec![]),
        hook_entered: tokio::sync::Semaphore::new(0),
        hook_release: tokio::sync::Semaphore::new(0),
        hook_input: Mutex::new(vec![]),
        engine,
        _directory: directory,
    });
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    let mut settings = json!({"session":{},"auth":{"base":"/auth","email":if state.case["sender"]==false {Json::Null}else{json!({"name":"Sender","address":"sender@example.test"})}}});
    if let Some(config) = config {
        settings["email"] = config;
    }
    layers.value("fixture", settings).unwrap();
    app.config_layers(layers).unwrap();
    let resource = state.clone();
    app.on_config(task(move |start: &mut StartupContext| {
        let resource = resource.clone();
        Box::pin(async move {
            start.register(resource.engine.clone())?;
            start.register(Arc::new(Environment {
                database_seed: Some("database-seed".into()),
                email_configured: resource.case["configured"] != false,
                host: Some("https://fixture.example.test".into()),
                brand: "Fixture".into(),
                clock: Arc::new(FixedClock),
                random: Arc::new(|| Ok(0.123456)),
                ..Default::default()
            }))?;
            start.register(resource)?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    // Public registrars retain constituent order; none installs a global request gate here.
    app.plugin("csrf", stackpress_framework_csrf::plugin::register)
        .unwrap();
    app.plugin("auth", auth::register).unwrap();
    plugin::register_with_factory(&mut app, state.factory.clone()).unwrap();
    app.plugin("identity", stackpress_framework_session::plugin::register)
        .unwrap();
    app.on("auth-search", auth::events::search).unwrap();
    app.on("auth-update", auth::events::update).unwrap();
    for event in ["auth-signin", "auth-search", "auth-update"] {
        app.on_priority(event, 10, observe).unwrap();
    }
    app.on_priority(
        EMAIL_SEND,
        10,
        handler(
            |req: &mut Request, res: &mut Response, inv: Invocation<'_>| {
                Box::pin(async move {
                    let state = inv.service::<Fixture>()?;
                    state.record(EMAIL_SEND);
                    state
                        .offered
                        .lock()
                        .unwrap()
                        .push(serde_json::to_value(req.data.entries())?);
                    if state.case["sendStatus"] == 419 {
                        res.set_error(PublicError {
                            code: 419,
                            status: "Page Expired".into(),
                            error: "provider public failure".into(),
                            errors: json!({}),
                        })?;
                    }
                    Ok(Flow::Continue)
                })
            },
        ),
    )
    .unwrap();
    // Native top-level operation selection precedes real route dispatch; views are not faked.
    for event in ["GET /auth/signin/email", "POST /auth/signin/email"] {
        app.on_priority(event, 20, begin).unwrap();
    }
    app.on(
        "error",
        handler(|_: &mut Request, _: &mut Response, inv: Invocation<'_>| {
            Box::pin(async move {
                inv.service::<Fixture>()?.record("error");
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    app.on("response", rendering::response).unwrap();
    if state.case["signupHook"] == true {
        app.on("email-email-send", signup::delegate).unwrap();
    }
    let server = Arc::new(app.build().await.unwrap());
    server
        .host()
        .service::<Hooks>()
        .unwrap()
        .bind(&server, tokio::runtime::Handle::current());
    assert!(
        !server
            .listeners()
            .any(|listener| listener.pattern == "request")
    );
    (server, state)
}
/// Fresh incoming form state is independent from the outgoing token and SQL row.
fn request(case: &Json) -> Request {
    let mut req=Request::builder().method(case["method"].as_str().unwrap()).url("/auth/signin/email")
        .data(json!({"email":"recipient@example.test","auth":case["mode"],"secret":if case["signinStatus"]==401 {"wrong"}else{"correct"},"2fa":false,"csrf":if case["valid"]==false {"wrong"}else{"incoming"},"redirect_uri":"/after"})).build().unwrap();
    req.session.set("csrf", "incoming", None).unwrap();
    req
}
/// Invoke only a registrar-provided public route name, without calling Lane A private helpers.
async fn handle(
    server: &Server,
    req: &mut Request,
    res: &mut Response,
) -> stackpress_lib::Result<stackpress_server::HandleReport> {
    let event = format!("{} /auth/signin/email", req.method.as_str());
    server.handle(&event, req, res).await
}
/// Original page/process traces now exercise real lookup/update/reload/templates and signin.
#[tokio::test]
async fn source_page_csrf_email_render_and_error_boundaries() {
    for case in vectors()["cases"].as_array().unwrap() {
        let (server, state) = host(
            case.clone(),
            (case["configured"] != false)
                .then(|| json!({"jsonTransport":true,"skipEncoding":true})),
        )
        .await;
        let mut req = request(case);
        if case["guest"] == false {
            let token = server
                .host()
                .service::<stackpress_framework_session::Service>()
                .unwrap()
                .create(serde_json::from_value(json!({"id":"profile-id"})).unwrap())
                .await
                .unwrap();
            req.session.set("session", token, None).unwrap();
        }
        let mut res = Response::new();
        let report = handle(&server, &mut req, &mut res).await.unwrap();
        assert!(
            report.failures.is_empty(),
            "{} {}",
            case["name"],
            report.developer()
        );
        let actual: Vec<_> = state
            .trace
            .lock()
            .unwrap()
            .iter()
            .filter(|step| step.as_str() != "render")
            .cloned()
            .collect();
        let expected: Vec<_> = case["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|step| {
                matches!(
                    step.as_str(),
                    Some("auth-search" | "auth-update" | "auth-signin" | "email-send")
                )
            })
            .cloned()
            .collect();
        assert_eq!(json!(actual), json!(expected), "{}", case["name"]);
        assert_eq!(
            res.status().unwrap().code as u64,
            case["code"].as_u64().unwrap(),
            "{}",
            case["name"]
        );
        assert_eq!(res.headers().get("location"), case["redirect"].as_str());
        assert_eq!(
            res.data().has("csrf"),
            case["retainedCsrf"].as_bool().unwrap()
        );
        assert_eq!(req.session.get("csrf").unwrap().as_str(), Some("incoming"));
        // The production SQL update leaves a body, so email's exact source skip wins.
        assert_eq!(
            state
                .factory
                .created
                .load(std::sync::atomic::Ordering::SeqCst),
            0
        );
        let rendered = state
            .trace
            .lock()
            .unwrap()
            .iter()
            .any(|step| step == "render");
        assert_eq!(
            rendered,
            matches!(
                case["name"].as_str(),
                Some("guest-get" | "invalid-csrf" | "password-status")
            ),
            "{}",
            case["name"]
        );
        if res.data().has("csrf") {
            let token = res.data().path_as::<String>("csrf.token").unwrap().unwrap();
            assert_eq!(token.len(), 64);
            assert_ne!(token, "incoming");
            assert_eq!(
                res.session().get("csrf").unwrap().as_str(),
                Some(token.as_str())
            );
            if matches!(res.body(), stackpress_lib::Body::Text(_)) {
                rendering::assert_form(&res, &token);
            }
        }
        if case["mode"] == "pass" && case["valid"] != false {
            assert_eq!(
                req.data.get("password").and_then(|v| v.as_bool()),
                Some(true)
            );
            assert_eq!(req.data.get("type").and_then(|v| v.as_str()), Some("email"));
            assert_eq!(
                res.session().get("session").is_some(),
                case["signinStatus"] == 200
            );
        }
        if !case["mail"].is_null() {
            assert_eq!(
                state.offered.lock().unwrap()[0],
                case["mail"],
                "{}",
                case["name"]
            );
        }
        server.host().service::<Hooks>().unwrap().shutdown().await;
        state.engine.close().await.unwrap();
    }
}
/// Fresh events send independently of the page's prior SQL result, without a global gate.
#[tokio::test]
async fn direct_email_reuse_has_no_automatic_csrf_or_auth_guard() {
    let (server, state) = host(
        vectors()["cases"][0].clone(),
        Some(json!({"jsonTransport":true})),
    )
    .await;
    let reply = server
        .resolve(EMAIL_SEND, json!({"text":"direct unprotected process"}))
        .await
        .unwrap();
    assert_eq!(reply.response.code, Some(200));
    assert_eq!(*state.trace.lock().unwrap(), vec![EMAIL_SEND]);
    assert_eq!(
        state
            .factory
            .created
            .load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    state.engine.close().await.unwrap();
}
/// A raw delivery exception reaches the real G12 host error path on a fresh event response.
#[tokio::test]
async fn fresh_email_provider_error_uses_existing_host_error_lifecycle() {
    let (server, state) = host(json!({"throws":true}), Some(json!({"jsonTransport":true}))).await;
    let mut req = Request::builder()
        .data(json!({"text":"failure"}))
        .build()
        .unwrap();
    let mut res = Response::new();
    let report = server.handle(EMAIL_SEND, &mut req, &mut res).await.unwrap();
    assert_eq!(res.status().unwrap().code, 500);
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.error_offers.len(), 1);
    assert!(
        report
            .developer()
            .to_string()
            .contains("provider fixture failure")
    );
    assert!(
        !res.error()
            .unwrap_or_default()
            .contains("SMTP_PRIVATE_SENTINEL")
    );
    assert_eq!(*state.trace.lock().unwrap(), vec![EMAIL_SEND, "error"]);
    state.engine.close().await.unwrap();
}
