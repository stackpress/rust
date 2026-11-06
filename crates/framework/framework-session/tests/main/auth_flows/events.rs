//! Real SQLite auth events and externally supervised owned signup hooks.
use super::*;

/// Real file-backed engine, explicitly installed before any registrar executes.
async fn database() -> (tempfile::TempDir, Arc<Engine>, AuthActions) {
    let dir = tempfile::tempdir().unwrap();
    let db = SqliteDatabase::open(
        SqliteOptions::file(dir.path().join("auth.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(db));
    ProfileActions::new(
        engine.clone(),
        stackpress_framework_session::storage::ProfileStore::new("database-seed".into(), 0),
    )
    .install()
    .await
    .unwrap();
    let auth = AuthActions::new(
        engine.clone(),
        stackpress_framework_session::storage::AuthStore::new("database-seed".into(), 0),
    );
    auth.install().await.unwrap();
    (dir, engine, auth)
}
/// Actual SQL handlers record source event invocation order before executing their actions.
#[derive(Default)]
struct Calls(Mutex<Vec<String>>);
async fn sql_search(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Calls>()?
        .0
        .lock()
        .unwrap()
        .push("auth-search".into());
    events::search(req, res, inv).await
}
/// Explicit host resources; none is eagerly demanded by the auth registrar itself.
async fn sql_host(engine: Arc<Engine>, auth_config: Option<Json>) -> Arc<Server> {
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value(
            "fixture",
            json!({"auth":auth_config,"session":{"key":"identity"}}),
        )
        .unwrap();
    app.config_layers(layers).unwrap();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = engine.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(Environment {
                    database_seed: Some("database-seed".into()),
                    ..Default::default()
                }))?;
                start.register(Arc::new(Csrf::default()))?;
                start.register(Arc::new(Calls::default()))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    app.on("auth-search", sql_search).unwrap();
    app.on("auth-update", events::update).unwrap();
    // Same order as the source constituents; Lane B's email registrar is integrated by its owner.
    app.plugin("auth", auth::register).unwrap();
    app.plugin("identity", stackpress_framework_session::plugin::register)
        .unwrap();
    let server = Arc::new(app.build().await.unwrap());
    server
        .host()
        .service::<Hooks>()
        .unwrap()
        .bind(&server, tokio::runtime::Handle::current());
    server
}
#[tokio::test]
async fn real_signup_optional_phone_roles_override_secret_removal_and_errors() {
    let (_dir, engine, actions) = database().await;
    let host = sql_host(
        engine.clone(),
        Some(json!({"roles":["USER"],"password":{"min":6}})),
    )
    .await;
    let result = invoke(
        &host,
        "auth-signup",
        json!({"name":"Ada","username":"ada-user","secret":"secret","roles":["CUSTOM"]}),
    )
    .await;
    assert_eq!(result.status().unwrap().code, 200);
    let profile = events::json_result(&result).unwrap();
    assert_eq!(profile["roles"], json!(["CUSTOM"]));
    assert!(profile["auth"]["username"].get("secret").is_none());
    assert!(profile["auth"].get("phone").is_none());
    assert_ne!(profile["auth"]["username"]["token"], json!("ada-user")); // Source create-return codec branch remains unchanged.
    assert_eq!(actions.count(&StoreQuery::default()).await.unwrap(), 1);
    let invalid = invoke(
        &host,
        "auth-signup",
        json!({"name":"Ada","username":"another","secret":"short"}),
    )
    .await;
    assert_eq!(invalid.error(), Some("Invalid Parameters"));
    assert_eq!(
        invalid.errors().to_json().unwrap()["secret"],
        "Password must be at least 6 characters"
    );
    let duplicate = invoke(
        &host,
        "auth-signup",
        json!({"name":"Ada","username":"ada-user","secret":"secret"}),
    )
    .await;
    assert_eq!(duplicate.status().unwrap().code, 400);
    assert_eq!(
        duplicate.errors().to_json().unwrap()["username"],
        "Already exists"
    );
    assert!(duplicate.errors().get("token").is_some()); // Original token error is retained too.
    assert_eq!(
        account(&actions)
            .profile()
            .count(&StoreQuery::default())
            .await
            .unwrap(),
        2
    ); // Sequential partial profile survives duplicate identifier.
    host.host().service::<Hooks>().unwrap().shutdown().await;
    engine.close().await.unwrap();
}
#[tokio::test]
async fn real_signin_flags_2fa_search_inactive_profile_and_literal_signout() {
    let (_dir, engine, actions) = database().await;
    let host = sql_host(engine.clone(), Some(json!({"base":"/login"}))).await;
    let profile = account(&actions)
        .signup(&record(
            json!({"name":"Ada","email":"ada@example.test","secret":"correct"}),
        ))
        .await
        .unwrap();
    let profile_id = profile["id"].clone();
    let wrong = invoke(
        &host,
        "auth-signin",
        json!({"type":"email","email":"ada@example.test","secret":"wrong"}),
    )
    .await;
    assert_eq!(wrong.status().unwrap().code, 401);
    assert!(wrong.diagnostic().is_some());
    assert_eq!(
        wrong.errors().to_json().unwrap(),
        json!({"secret":"Invalid Password"})
    );
    let unknown = invoke(
        &host,
        "auth-signin",
        json!({"type":"email","email":"missing@example.test"}),
    )
    .await;
    assert_eq!(unknown.status().unwrap().code, 404);
    assert_eq!(
        unknown.errors().to_json().unwrap(),
        json!({"email":"User Not Found"})
    );
    // Direct event has no page CSRF and honors all false-ish password flags.
    for password in [json!(false), json!(null), json!(0), json!("")] {
        let result = invoke(
            &host,
            "auth-signin",
            json!({"type":"email","email":"ada@example.test","password":password,"2fa":false}),
        )
        .await;
        assert_eq!(result.status().unwrap().code, 200);
        assert!(result.session().get("identity").is_some());
        assert!(
            events::json_result(&result)
                .unwrap()
                .get("secret")
                .is_none()
        );
    }
    assert!(
        host.host()
            .service::<Calls>()
            .unwrap()
            .0
            .lock()
            .unwrap()
            .is_empty()
    ); // Strict false skipped search entirely.
    actions.create(&record(json!({"profileId":profile_id,"type":"2fa","token":"JBSWY3DPEHPK3PXP","secret":"factor"}))).await.unwrap();
    let challenge=invoke(&host,"auth-signin",json!({"type":"email","email":"ada@example.test","password":false,"2fa":null,"redirect_uri":"/after?x=1"})).await;
    assert_eq!(challenge.status().unwrap().code, 302);
    assert!(
        challenge
            .headers()
            .get("Location")
            .unwrap()
            .starts_with("/login/signin/2fa/")
    );
    assert!(challenge.session().get("identity").is_none());
    assert_eq!(
        *host.host().service::<Calls>().unwrap().0.lock().unwrap(),
        vec!["auth-search"]
    );
    account(&actions)
        .profile()
        .update(
            &StoreQuery::eq("id", profile_id),
            &mut record(json!({"active":false})),
        )
        .await
        .unwrap();
    let inactive = invoke(
        &host,
        "auth-signin",
        json!({"type":"email","email":"ada@example.test","password":false,"2fa":false}),
    )
    .await;
    assert_eq!(inactive.status().unwrap().code, 200); // Source joined profile has no added active check.
    let token = inactive
        .session()
        .get("identity")
        .unwrap()
        .as_str()
        .unwrap();
    let identity = host.host().service::<Service>().unwrap().load(token);
    let claims = identity.data().await.unwrap();
    assert_eq!(claims["roles"], json!(["GUEST"]));
    let mut response = Response::new();
    response
        .session_mut()
        .unwrap()
        .set("identity", token.to_owned(), None)
        .unwrap();
    response
        .session_mut()
        .unwrap()
        .set("session", "old", None)
        .unwrap();
    host.emit("auth-signout", &mut Request::new(), &mut response)
        .await
        .unwrap();
    assert!(response.session().get("identity").is_some());
    assert!(response.session().get("session").is_none());
    host.host().service::<Hooks>().unwrap().shutdown().await;
    engine.close().await.unwrap();
}
#[tokio::test]
async fn missing_config_modes_and_direct_events_need_no_global_guard() {
    let mut app = ServerBuilder::new();
    app.plugin("auth", auth::register).unwrap();
    let host = app.build().await.unwrap();
    let patterns: Vec<_> = host
        .listeners()
        .map(|listener| listener.pattern.as_str())
        .collect();
    assert_eq!(patterns, vec!["auth-signup", "auth-signin", "auth-signout"]);
    assert!(
        invoke(&host, "auth-signout", json!({}))
            .await
            .error()
            .is_none()
    );
    assert!(
        host.emit("auth-signin", &mut Request::new(), &mut Response::new())
            .await
            .is_err()
    ); // Missing Engine is consumed only on invocation.
    let (_dir, engine, actions) = database().await;
    let host = sql_host(engine.clone(), None).await;
    let signup = invoke(
        &host,
        "auth-signup",
        json!({"name":"Ada","username":"ada-user","secret":"secret"}),
    )
    .await;
    assert_eq!(signup.status().unwrap().code, 200);
    assert!(
        !host
            .listeners()
            .any(|listener| listener.pattern.contains("/auth/"))
    );
    assert!(
        !host
            .listeners()
            .any(|listener| listener.pattern == "request")
    );
    let mut req = Request::builder()
        .data(json!({"username":"ada-user","password":false}))
        .build()
        .unwrap();
    assert!(
        host.emit("auth-signin", &mut req, &mut Response::new())
            .await
            .is_err()
    ); // Source auth config is read after successful action consumption.
    let auth = actions
        .find(&StoreQuery::eq("token", "ada-user"))
        .await
        .unwrap()
        .unwrap();
    assert!(columns::as_date(&auth["consumed"]).unwrap() > 0);
    engine.close().await.unwrap();
}

/// Hook suspension/barrier fixture, entered only by the external watchdog process.
struct HookBarrier {
    entered: tokio::sync::mpsc::UnboundedSender<Record>,
    release: tokio::sync::Semaphore,
}
/// Keep a submitted hook suspended after capturing its independent payload.
async fn suspended_hook(req: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<HookBarrier>()?;
    state
        .entered
        .send(
            req.data
                .entries()
                .iter()
                .map(|(key, value)| (key.clone(), Value::from(json_value(value.clone()))))
                .collect(),
        )
        .unwrap();
    state.release.acquire().await.unwrap().forget();
    Ok(Flow::Continue)
}
/// Prove source nonawaiting semantics, Send ownership and nested reuse without runtime hangs.
pub(crate) async fn concurrency_child() {
    let (_dir, engine, _) = database().await;
    let (entered, mut receiver) = tokio::sync::mpsc::unbounded_channel();
    let barrier = Arc::new(HookBarrier {
        entered,
        release: tokio::sync::Semaphore::new(0),
    });
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value("fixture", json!({"auth":{},"session":{}}))
        .unwrap();
    app.config_layers(layers).unwrap();
    let state = barrier.clone();
    let database = engine.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let state = state.clone();
            let engine = database.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(state)?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    app.on("email-email-send", suspended_hook).unwrap();
    app.on("auth-phone-verify", suspended_hook).unwrap();
    app.plugin("auth", auth::register).unwrap();
    app.plugin("identity", stackpress_framework_session::plugin::register)
        .unwrap();
    let host = Arc::new(app.build().await.unwrap());
    let hooks = host.host().service::<Hooks>().unwrap();
    let response = invoke(
        &host,
        "auth-signup",
        json!({"name":"Ada","email":"ada@example.test","phone":"+1 (234) 567","secret":"secret"}),
    )
    .await;
    assert_eq!(response.status().unwrap().code, 200); // Parent completed while both hook futures are blocked.
    assert_eq!(hooks.pending(), 2);
    assert!(matches!(
        receiver.try_recv(),
        Err(tokio::sync::mpsc::error::TryRecvError::Empty)
    ));
    hooks.bind(&host, tokio::runtime::Handle::current());
    assert_eq!(hooks.pending(), 0);
    let first = receiver.recv().await.unwrap();
    let second = receiver.recv().await.unwrap();
    for payload in [first, second] {
        assert!(payload["auth"]["email"].get("secret").is_some());
        assert_eq!(
            payload.get("phone").and_then(Value::as_str).unwrap_or(""),
            if payload.contains_key("phone") {
                "+1234567"
            } else {
                ""
            }
        );
    }
    assert!(
        events::json_result(&response).unwrap()["auth"]["email"]
            .get("secret")
            .is_none()
    );
    barrier.release.add_permits(2);
    let completions = hooks.drain().await;
    assert_eq!(completions.len(), 2);
    assert!(
        completions
            .into_iter()
            .all(|result| result.unwrap().is_ok())
    );
    // Binding is explicit: queued hooks remain owned, start on bind, and can be aborted/reaped.
    let unbound = Hooks::default();
    assert_eq!(unbound.pending(), 0);
    unbound.bind(&host, tokio::runtime::Handle::current());
    unbound.shutdown().await;
    // A second submission remains blocked until explicit shutdown aborts and joins it.
    let second = invoke(
        &host,
        "auth-signup",
        json!({"name":"Grace","email":"grace@example.test","secret":"secret"}),
    )
    .await;
    assert_eq!(second.status().unwrap().code, 200);
    receiver.recv().await.unwrap();
    hooks.shutdown().await;
    let weak = Arc::downgrade(&host);
    drop(host);
    assert!(weak.upgrade().is_none());
    engine.close().await.unwrap();
}

#[tokio::test]
async fn source_timezone_column_sql_and_profile_collaborator_vectors() {
    let (_dir, engine, actions) = database().await;
    let fixture: Json = serde_json::from_str(include_str!("../../auth-flows.json")).unwrap();
    for vector in fixture["timezone"].as_array().unwrap() {
        let offset = vector["offset"].as_i64().unwrap() as i32;
        let actions = AuthActions::new(
            actions.engine.clone(),
            stackpress_framework_session::storage::AuthStore::new("database-seed".into(), offset),
        );
        let value = columns::date(1_700_000_000_000);
        let serialized = actions
            .store
            .schema
            .column("consumed")
            .unwrap()
            .serialize(Some(&value), "", false, offset)
            .unwrap()
            .unwrap();
        assert_eq!(serialized, vector["serialized"]);
        let scalar = account(&actions)
            .profile()
            .store
            .scalarize(&Record::from_iter([("created".into(), value)]))
            .unwrap();
        assert_eq!(
            scalar[0].1,
            stackpress_sql::SqlValue::Text(vector["sql"].as_str().unwrap().into())
        );
    }
    engine.close().await.unwrap();
}

/// Finite email-send event proves the caller contract without selecting a live transport.
#[derive(Default)]
struct MailSink(Mutex<Vec<Json>>);
async fn capture_mail(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<MailSink>()?
        .0
        .lock()
        .unwrap()
        .push(req.data.to_json()?);
    res.results(&json!({"sent":true}))?;
    Ok(Flow::Continue)
}
/// Drive actual registered routes with owned request/cookie state; no socket is involved.
async fn route(host: &Server, method: &str, url: &str, data: Json, csrf: Option<&str>) -> Response {
    #[cfg(feature = "leptos-ssr")]
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    let mut request = Request::builder()
        .method(method)
        .url(url)
        .data(data)
        .build()
        .unwrap();
    if let Some(csrf) = csrf {
        request.session.set("csrf", csrf, None).unwrap();
    }
    let mut response = Response::new();
    host.handle_route(&mut request, &mut response)
        .await
        .unwrap();
    response
}
#[tokio::test]
async fn real_sql_otp_magic_handoff_and_session_creation_through_registered_routes() {
    let (_dir, engine, actions) = database().await;
    account(&actions)
        .signup(&record(
            json!({"name":"Ada","email":"ada@example.test","secret":"secret"}),
        ))
        .await
        .unwrap();
    let sink = Arc::new(MailSink::default());
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers.value("fixture",json!({"auth":{"base":"/auth","email":{"name":"Sender","address":"sender@example.test"}},"session":{"key":"identity"}})).unwrap();
    app.config_layers(layers).unwrap();
    let database = engine.clone();
    let mail = sink.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let database = database.clone();
            let mail = mail.clone();
            Box::pin(async move {
                start.register(database)?;
                start.register(mail)?;
                start.register(Arc::new(Csrf::default()))?;
                start.register(Arc::new(Environment {
                    database_seed: Some("database-seed".into()),
                    email_configured: true,
                    host: Some("https://example.test".into()),
                    ..Default::default()
                }))?;
                Ok(Flow::Continue)
            })
        },
    ))
    .unwrap();
    app.on("auth-search", events::search).unwrap();
    app.on("auth-update", events::update).unwrap();
    app.on("email-send", capture_mail).unwrap();
    app.plugin("auth", auth::register).unwrap();
    app.plugin("identity", stackpress_framework_session::plugin::register)
        .unwrap();
    let host = Arc::new(app.build().await.unwrap());
    host.host()
        .service::<Hooks>()
        .unwrap()
        .bind(&host, tokio::runtime::Handle::current());
    let issued = route(
        &host,
        "POST",
        "/auth/signin/email",
        json!({"auth":"otp","email":"ada@example.test","redirect_uri":"/after","csrf":"same"}),
        Some("same"),
    )
    .await;
    assert_eq!(issued.status().unwrap().code, 302);
    let otp_url = issued.headers().get("Location").unwrap().to_owned();
    let mail = sink.0.lock().unwrap()[0].clone();
    let code = mail["text"]
        .as_str()
        .unwrap()
        .split("Your OTP is: ")
        .nth(1)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_owned();
    assert_eq!(code.len(), 6);
    let get = route(&host, "GET", &otp_url, json!({}), None).await;
    assert_eq!(get.status().unwrap().code, 200);
    assert!(events::json_result(&get).unwrap().get("secret").is_none());
    let csrf = get
        .session()
        .get("csrf")
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    let verified = route(
        &host,
        "POST",
        &otp_url,
        json!({"code":code,"csrf":csrf}),
        Some(&csrf),
    )
    .await;
    assert_eq!(verified.status().unwrap().code, 302);
    let two_factor = verified.headers().get("Location").unwrap().to_owned();
    // No factor record: a GET finalizes with no POST/CSRF/code requirement.
    let signed = route(&host, "GET", &two_factor, json!({}), None).await;
    assert_eq!(signed.headers().get("Location"), Some("/after"));
    let token = signed.session().get("identity").unwrap().as_str().unwrap();
    let session = host.host().service::<Service>().unwrap().load(token);
    assert!(!session.guest().await);
    let magic = route(
        &host,
        "POST",
        "/auth/signin/email",
        json!({"auth":"magic","email":"ada@example.test","csrf":"same"}),
        Some("same"),
    )
    .await;
    assert_eq!(magic.status().unwrap().code, 200);
    assert!(!magic.redirected());
    assert!(magic.session().get("csrf").is_some());
    let mail = sink.0.lock().unwrap()[1].clone();
    let link = mail["text"]
        .as_str()
        .unwrap()
        .split("Open ")
        .nth(1)
        .unwrap()
        .split(" to continue")
        .next()
        .unwrap();
    let proof = route(&host, "GET", link, json!({}), None).await;
    assert_eq!(proof.status().unwrap().code, 302);
    assert!(
        proof
            .headers()
            .get("Location")
            .unwrap()
            .starts_with("/auth/signin/2fa/")
    );
    host.host().service::<Hooks>().unwrap().shutdown().await;
    engine.close().await.unwrap();
}

use stackpress_framework_schema::source::SourceSchema;

/// Bind account orchestration to the same generated store environment.
fn account(
    actions: &AuthActions,
) -> stackpress_framework_session::storage::AccountActions<stackpress_framework_models::Models> {
    stackpress_framework_session::storage::AccountActions::new(
        actions.engine.clone(),
        actions.store.schema.seed().into(),
        actions.store.schema.offset_minutes(),
        Default::default(),
    )
}
