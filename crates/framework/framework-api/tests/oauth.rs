//! Source process branch matrix, real JWT/SQL composition and supervised nested cancellation.
use serde_json::{Value as Json, json};
use stackpress_framework_api::{pages, register};
#[cfg(feature = "native")]
use stackpress_lib::Error;
#[cfg(feature = "sql")]
use stackpress_lib::Nest;
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result, Timestamp, Value};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Each fake observes the actual request/response boundary and returns a chosen source branch.
struct State {
    caller: Json,
    app: Json,
    app_status: u16,
    created: Option<Json>,
    create_status: u16,
    detail: Option<Value>,
    calls: Mutex<Vec<(String, Json, Option<u16>)>>,
    pending: bool,
    dropped: Arc<AtomicUsize>,
}
impl Default for State {
    fn default() -> Self {
        Self { caller: json!({"id":"p"}), app: json!({"id":"a","name":"App","secret":"visible"}),
            app_status: 200, created: Some(json!({"id":"s"})), create_status: 200,
            detail: Some(Value::try_from(json!({"id":"s","applicationId":"a","secret":"stored","profile":{"id":"p","name":"Ada","image":null,"created":"epoch"}})).unwrap()),
            calls: Mutex::new(Vec::new()), pending:false, dropped:Arc::new(AtomicUsize::new(0)) }
    }
}
/// Only serializable JSON is recorded; Date request fields are inspected separately in create.
fn capture(state: &State, event: &str, req: &Request, res: &Response) {
    let mut data = req.data.clone();
    if let Some(Value::DateTime(time)) = data.get("expires").cloned() {
        data.set(
            "expires",
            time.seconds * 1000 + i64::from(time.nanoseconds() / 1_000_000),
        )
        .unwrap();
    }
    state.calls.lock().unwrap().push((
        event.into(),
        data.to_json().unwrap(),
        res.status().map(|status| status.code),
    ));
}
async fn me(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<State>()?;
    capture(state, "me", req, res);
    res.data_mut()?.set(
        "original_cookie",
        req.session
            .get("jwt")
            .and_then(|value| value.as_str())
            .unwrap_or(""),
    )?;
    res.results(&state.caller)?;
    Ok(Flow::Abort) // Child Abort must never become the outer page's Abort.
}
async fn application(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<State>()?;
    assert!(req.session.get("jwt").is_none());
    assert!(req.headers.get("authorization").is_none());
    capture(state, "app", req, res);
    res.data_mut()?.set("shared", true)?;
    res.results(&state.app)?;
    res.status_code(state.app_status, None)?;
    Ok(Flow::Continue)
}
struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
async fn create(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<State>()?;
    capture(state, "create", req, res);
    assert!(req.session.get("jwt").is_none());
    if state.pending {
        let _drop = Dropped(state.dropped.clone());
        std::future::pending::<()>().await;
    }
    if let Some(value) = &state.created {
        res.results(value)?;
    }
    res.status_code(state.create_status, None)?;
    Ok(Flow::Continue)
}
async fn detail(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<State>()?;
    capture(state, "detail", req, res);
    assert!(req.headers.get("authorization").is_none());
    assert!(req.session.get("jwt").is_none());
    if let Some(value) = &state.detail {
        res.results_value(value.clone())?;
    }
    // Source token page checks the row, not the lookup's status.
    res.status_code(503, None)?;
    Ok(Flow::Continue)
}
async fn host(state: Arc<State>, config: Json, env: pages::Environment) -> Result<Server> {
    let mut app = ServerBuilder::new();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let state = state.clone();
            let env = env.clone();
            Box::pin(async move {
                start.register(state)?;
                start.register(Arc::new(env))?;
                Ok(Flow::Continue)
            })
        },
    ))?;
    let mut layers = ConfigLayers::new();
    layers.value("fixture", json!({"api":config}))?;
    app.config_layers(layers)?;
    register(&mut app)?;
    pages::register(&mut app)?; // direct registration is also independently usable
    app.on("me", me)?;
    app.on("application-detail", application)?;
    app.on("session-create", create)?;
    app.on("session-detail", detail)?;
    app.build().await.map_err(|failure| failure.error)
}
fn environment(now: i64) -> pages::Environment {
    pages::Environment {
        now: Arc::new(move || Ok(now)),
        ..Default::default()
    }
}
fn request(method: &str, data: Json) -> Request {
    let mut req = Request::builder()
        .method(method)
        .url("/auth/oauth?scope=x%20y&state=a%26b")
        .data(data)
        .build()
        .unwrap();
    req.session.set("jwt", "original", None).unwrap();
    req
}
async fn invoke(server: &Server, path: &str, req: &mut Request) -> Result<Response> {
    let mut res = Response::new();
    server
        .emit(&format!("{} {path}", req.method.as_str()), req, &mut res)
        .await?;
    Ok(res)
}
fn object(value: &mut Value) -> &mut stackpress_lib::DataMap<String, Value> {
    match value {
        Value::Object(object) => object,
        _ => panic!("expected object"),
    }
}
fn code(res: &Response) -> Option<u16> {
    res.status().map(|status| status.code)
}
fn value(res: &Response) -> Json {
    res.result().unwrap().to_json().unwrap()
}
fn input() -> Json {
    json!({"client_id":"a","redirect_uri":"http://external/callback?x=one+two&code=old&code=again&state=old?discard","state":"a &β"})
}

#[tokio::test]
async fn oauth_prior_body_status_and_required_input_branches() -> Result<()> {
    let state = Arc::new(State::default());
    let server = host(state.clone(), json!({}), environment(1000)).await?;
    for (body, status, skip) in [
        (Some(json!({})), 200, true),
        (Some(json!([])), 200, true),
        (Some(json!("body")), 200, true),
        (None, 201, true),
        (None, 401, true),
        (Some(json!(false)), 200, false),
        (Some(json!(0)), 200, false),
        (Some(json!("")), 200, false),
        (Some(Json::Null), 200, false),
        (None, 200, false),
    ] {
        let mut res = Response::new();
        if let Some(body) = body {
            res.set("application/json", Body::Value(Value::try_from(body)?))?;
        }
        res.status_code(status, None)?;
        server
            .emit("GET /auth/oauth", &mut request("GET", json!({})), &mut res)
            .await?;
        assert_eq!(code(&res), Some(if skip { status } else { 401 }));
        assert_eq!(res.data().get("api").is_none(), skip);
    }
    for data in [
        json!({}),
        json!({"client_id":"a"}),
        json!({"redirect_uri":"/cb"}),
        json!({"client_id":false,"redirect_uri":"/cb"}),
    ] {
        let res = invoke(&server, "/auth/oauth", &mut request("GET", data)).await?;
        assert_eq!(code(&res), Some(401));
        assert_eq!(
            res.data().get("brand").unwrap().to_json()?["name"],
            "Stackpress"
        );
    }
    assert!(state.calls.lock().unwrap().is_empty());
    Ok(())
}
#[tokio::test]
async fn guest_continuation_and_shared_application_response_preserve_source_config() -> Result<()> {
    for caller in [json!({}), json!({"id":0}), json!({"id":false})] {
        let state = Arc::new(State {
            caller,
            ..State::default()
        });
        let server = host(state.clone(), json!({}), environment(0)).await?;
        let res = invoke(&server, "/auth/oauth", &mut request("GET", input())).await?;
        assert_eq!(
            res.headers().get("location"),
            Some("/auth/signin?redirect_uri=%2Fauth%2Foauth%3Fscope%3Dx%2520y%26state%3Da%2526b")
        );
        assert_eq!(state.calls.lock().unwrap().len(), 1);
    }
    let state = Arc::new(State::default());
    let env = pages::Environment {
        view: json!({"base":"","props":false,"notify":{"hello":1},"ignored":"no"}),
        brand: json!({"name":"Custom","logo":"","icon":false,"favicon":"fav"}),
        ..environment(0)
    };
    let server = host(
        state.clone(),
        json!({"scopes":{"read":{"name":"Read","description":"Source"}}}),
        env,
    )
    .await?;
    let res = invoke(&server, "/auth/oauth", &mut request("PUT", input())).await?;
    assert_eq!(value(&res)["secret"], "visible");
    assert_eq!(res.data().get("shared").unwrap().to_json()?, true);
    let data = res.data().to_json()?;
    assert_eq!(
        data["view"],
        json!({"base":"/","props":{},"notify":{"hello":1}})
    );
    assert_eq!(
        data["brand"],
        json!({"name":"Custom","logo":"/logo.png","icon":"/icon.png","favicon":"fav"})
    );
    let calls = state.calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].1, json!({"id":"a"}));
    Ok(())
}
#[tokio::test]
async fn post_spreads_input_overrides_three_fields_and_preserves_creation_status_distinction()
-> Result<()> {
    for config in [
        json!({}),
        json!({"expires":null}),
        json!({"expires":0}),
        json!({"expires":-1.9}),
        json!({"expires":0.9}),
    ] {
        let state = Arc::new(State {
            create_status: 500,
            ..State::default()
        });
        let server = host(state.clone(), config.clone(), environment(1000)).await?;
        let mut data = input();
        data.as_object_mut().unwrap().extend(json!({"id":"caller-id","secret":"caller-secret","active":false,"scopes":["unregistered"],"applicationId":"forged","profileId":"forged","expires":-999}).as_object().unwrap().clone());
        let original = data.clone();
        let mut req = request("POST", data);
        let res = invoke(&server, "/auth/oauth", &mut req).await?;
        assert_eq!(code(&res), Some(302));
        assert_eq!(value(&res)["id"], "a");
        assert_eq!(
            res.headers().get("location"),
            Some("http://external/callback?x=one+two&code=s&state=a+%26%CE%B2")
        );
        assert_eq!(
            req.data.get("profileId").and_then(Value::as_str),
            Some("forged")
        );
        let calls = state.calls.lock().unwrap();
        let sent = &calls[2].1;
        assert_eq!(sent["applicationId"], "a");
        assert_eq!(sent["profileId"], "p");
        for field in [
            "id",
            "secret",
            "active",
            "scopes",
            "client_id",
            "redirect_uri",
            "state",
        ] {
            assert_eq!(sent[field], original[field]);
        }
        let config: stackpress_framework_api::ApiConfig = serde_json::from_value(config)?;
        assert_eq!(sent["expires"], config.expires_at(1000).unwrap());
    }
    for (app_status, created, expected, calls) in [
        (404, Some(json!({"id":"s"})), 401, 2),
        (200, None, 200, 3),
        (200, Some(Json::Null), 200, 3),
        (200, Some(json!({})), 302, 3),
    ] {
        let state = Arc::new(State {
            app_status,
            created,
            ..State::default()
        });
        let server = host(state.clone(), json!({}), environment(1000)).await?;
        let res = invoke(&server, "/auth/oauth", &mut request("POST", input())).await?;
        assert_eq!(code(&res), Some(expected));
        assert_eq!(state.calls.lock().unwrap().len(), calls);
        if expected == 302 {
            assert!(
                res.headers()
                    .get("location")
                    .unwrap()
                    .contains("code=undefined")
            );
        }
    }
    Ok(())
}
#[tokio::test]
async fn redirect_state_split_encoding_and_late_mechanics_errors() -> Result<()> {
    for (redirect, state, expected) in [
        ("/cb", json!(false), "/cb?code=s"),
        (
            "/cb?state=existing&code=old&code=two",
            json!(0),
            "/cb?state=existing&code=s",
        ),
        (
            "/cb?x=%zz&x=%FF&line=\t\n\r",
            json!(["a", null, "b"]),
            "/cb?x=%25zz&x=%EF%BF%BD&line=%09%0A%0D&code=s&state=a%2C%2Cb",
        ),
        (
            "/cb#frag",
            json!({}),
            "/cb#frag?code=s&state=%5Bobject+Object%5D",
        ),
    ] {
        let server = host(Arc::new(State::default()), json!({}), environment(0)).await?;
        let res = invoke(
            &server,
            "/auth/oauth",
            &mut request(
                "POST",
                json!({"client_id":"a","redirect_uri":redirect,"state":state}),
            ),
        )
        .await?;
        assert_eq!(res.headers().get("location"), Some(expected));
    }
    let state = Arc::new(State::default());
    let server = host(state.clone(), json!({}), environment(0)).await?;
    assert!(
        invoke(
            &server,
            "/auth/oauth",
            &mut request("POST", json!({"client_id":"a","redirect_uri":true}))
        )
        .await
        .is_err()
    );
    assert_eq!(
        state.calls.lock().unwrap().len(),
        3,
        "redirect type failure follows persistence"
    );
    let server = host(
        Arc::new(State::default()),
        json!({"expires":9e20}),
        environment(0),
    )
    .await?;
    assert!(
        invoke(&server, "/auth/oauth", &mut request("POST", input()))
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn token_header_code_app_join_and_repeated_exchange_branches() -> Result<()> {
    let state = Arc::new(State::default());
    let server = host(state.clone(), json!({}), environment(1000)).await?;
    for (method, header, data, expected) in [
        ("GET", None, json!({"code":"s"}), 401),
        ("GET", Some("Any a"), json!({"code":"s"}), 401),
        ("POST", Some("Any a:"), json!({"code":"s"}), 401),
        ("GET", Some("Any a:unused"), json!({}), 401),
        ("GET", Some("Any a:unused"), json!({"code":false}), 401),
        ("GET", Some("Any wrong:unused"), json!({"code":"s"}), 401),
        ("GET", Some("Any a:wrong"), json!({"code":"s"}), 200),
        ("HEAD", Some("Other a:wrong"), json!({"code":"s"}), 200),
        ("DELETE", Some("a a:wrong"), json!({"code":"s"}), 200),
    ] {
        let mut req = request(method, data);
        if let Some(header) = header {
            req.headers.set("authorization", header)?;
        }
        let res = invoke(&server, "/auth/oauth/token", &mut req).await?;
        assert_eq!(code(&res), Some(expected));
        if expected == 200 {
            assert_eq!(
                value(&res),
                json!({"token_type":"Bearer","access_token":"s","access_secret":"stored","expires_in":0,"user":{"id":"p","name":"Ada","image":null,"created":"epoch"}})
            );
        }
    }
    for detail in [
        None,
        Some(Value::Null),
        Some(Value::try_from(json!({"applicationId":"other"}))?),
    ] {
        let server = host(
            Arc::new(State {
                detail,
                ..State::default()
            }),
            json!({}),
            environment(0),
        )
        .await?;
        let mut req = request("GET", json!({"code":"s"}));
        req.headers.set("authorization", "X a:x")?;
        assert_eq!(
            code(&invoke(&server, "/auth/oauth/token", &mut req).await?),
            Some(401)
        );
    }
    Ok(())
}

#[tokio::test]
async fn token_expiry_strict_boundary_two_clock_reads_and_missing_profile_failure() -> Result<()> {
    for (expiry, expected, remaining) in [
        (None, 200, 0),
        (Some(999), 401, 0),
        (Some(1000), 200, 0),
        (Some(1999), 200, 0),
        (Some(2000), 200, 1),
    ] {
        let mut state = State::default();
        let data = object(state.detail.as_mut().unwrap());
        if let Some(ms) = expiry {
            data.insert(
                "expires".into(),
                Value::DateTime(Timestamp::new(ms / 1000, (ms % 1000) as u32 * 1_000_000)?),
            );
        }
        let server = host(Arc::new(state), json!({}), environment(1000)).await?;
        let mut req = request("GET", json!({"code":"s"}));
        req.headers.set("authorization", "X a:wrong")?;
        let res = invoke(&server, "/auth/oauth/token", &mut req).await?;
        assert_eq!(code(&res), Some(expected));
        if expected == 200 {
            assert_eq!(value(&res)["expires_in"], remaining);
        }
    }
    for bad in [
        json!({"applicationId":"a"}),
        json!({"applicationId":"a","profile":null}),
        json!({"applicationId":"a","profile":{},"expires":"not-date"}),
    ] {
        let server = host(
            Arc::new(State {
                detail: Some(Value::try_from(bad)?),
                ..State::default()
            }),
            json!({}),
            environment(1000),
        )
        .await?;
        let mut req = request("GET", json!({"code":"s"}));
        req.headers.set("authorization", "X a:x")?;
        assert!(
            invoke(&server, "/auth/oauth/token", &mut req)
                .await
                .is_err()
        );
    }
    let mut state = State::default();
    object(state.detail.as_mut().unwrap()).insert("expires".into(), Timestamp::new(1, 0)?.into());
    let reads = Arc::new(AtomicUsize::new(0));
    let counter = reads.clone();
    let env = pages::Environment {
        now: Arc::new(move || Ok(1000 + counter.fetch_add(1, Ordering::SeqCst) as i64)),
        ..Default::default()
    };
    let server = host(Arc::new(state), json!({}), env).await?;
    let mut req = request("GET", json!({"code":"s"}));
    req.headers.set("authorization", "X a:x")?;
    let res = invoke(&server, "/auth/oauth/token", &mut req).await?;
    assert_eq!(value(&res)["expires_in"], -1);
    assert_eq!(reads.load(Ordering::SeqCst), 2);
    Ok(())
}

#[tokio::test]
async fn cancellation_drops_nested_creation_without_detaching_or_mutating_browser_input()
-> Result<()> {
    use std::{
        future::Future,
        task::{Context, Poll, Waker},
    };
    let state = Arc::new(State {
        pending: true,
        ..State::default()
    });
    let server = host(state.clone(), json!({}), environment(0)).await?;
    let mut req = request("POST", input());
    let mut res = Response::new();
    let mut future = Box::pin(server.emit("POST /auth/oauth", &mut req, &mut res));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    drop(future);
    assert_eq!(state.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(
        req.session.get("jwt").and_then(|value| value.as_str()),
        Some("original")
    );
    assert_eq!(value(&res)["id"], "a");
    Ok(())
}

#[tokio::test]
async fn source_mount_gate_all_methods_priorities_fifo_and_outer_abort() -> Result<()> {
    for (config, mounted) in [
        (json!(null), false),
        (json!({}), false),
        (json!({"endpoints":[]}), false),
        (
            json!({"endpoints":[{"method":"GET","route":"/api","type":"public","event":"unused"}]}),
            true,
        ),
    ] {
        let mut app = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers.value("mount", json!({"api":config}))?;
        app.config_layers(layers)?;
        register(&mut app)?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        for method in [
            "GET", "POST", "HEAD", "PUT", "DELETE", "OPTIONS", "PATCH", "CUSTOM",
        ] {
            for path in ["/auth/oauth", "/auth/oauth/token"] {
                let mut res = Response::new();
                let mut req = Request::builder().method(method).url(path).build()?;
                server.handle_route(&mut req, &mut res).await?;
                assert_eq!(code(&res), Some(if mounted { 401 } else { 404 }));
            }
        }
    }
    async fn stop(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        Ok(Flow::Abort)
    }
    async fn tail(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        res.data_mut()?.set("tail", true)?;
        Ok(Flow::Continue)
    }
    for abort in [false, true] {
        let mut app = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers.value("mount",json!({"api":{"endpoints":[{"method":"ALL","route":"/auth/oauth","type":"public","event":"tail"}]}}))?;
        app.config_layers(layers)?;
        if abort {
            app.route_with(
                stackpress_server::RouteSpec::new(
                    stackpress_lib::MethodPattern::Any,
                    "/auth/oauth",
                )
                .priority(1),
                stop,
            )?;
        }
        register(&mut app)?;
        app.on("tail", tail)?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        let mut res = Response::new();
        server
            .handle_route(&mut request("GET", json!({})), &mut res)
            .await?;
        assert_eq!(res.data().get("tail").is_some(), !abort);
        assert_eq!(
            code(&res),
            if abort { Some(204) } else { Some(401) },
            "page Continue permits the configured same-route handler"
        );
    }
    Ok(())
}

#[cfg(feature = "sql")]
mod persisted {
    use super::*;
    use stackpress_framework_api::{
        models::{ApplicationActions, Record, SessionActions},
        sql,
    };
    use stackpress_framework_session::{
        Service as Identity, plugin as identity, storage::ProfileActions,
    };
    use stackpress_sql::Engine;
    use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
    use std::path::PathBuf;

    /// Real external Profile and API stores; no network, fabricated caller or auth substitute.
    struct Database {
        path: PathBuf,
        engine: Arc<Engine>,
    }
    impl Database {
        async fn new() -> Result<Self> {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "slice10-m3-{}-{}.sqlite",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            let engine = Arc::new(Engine::new(
                SqliteDatabase::open(SqliteOptions::file(&path).create_if_missing(true))
                    .await
                    .map_err(Error::from_source)?,
            ));
            let profile = ProfileActions::new(engine.clone(), "");
            let apps = ApplicationActions::new(engine.clone(), "");
            let sessions = SessionActions::new(engine.clone(), "");
            profile.install().await.unwrap();
            apps.install().await.unwrap();
            sessions.install().await.unwrap();
            for id in ["p", "q"] {
                profile
                    .create(&serde_json::from_value::<Record>(
                        json!({"id":id,"name":format!("User {id}"),"roles":["USER"]}),
                    )?)
                    .await
                    .unwrap();
            }
            let mut app: Record = serde_json::from_value(
                json!({"id":"a","profileId":"p","name":"Real app","secret":"app-plain","scopes":["read"]}),
            )?;
            app.insert("expires".into(), stackpress_framework_api::models::date(0));
            apps.create(&app).await.unwrap();
            Ok(Self { path, engine })
        }
        async fn host(&self, barrier: Option<Arc<tokio::sync::Barrier>>) -> Result<Server> {
            let mut app = ServerBuilder::new();
            let engine = self.engine.clone();
            app.on_config(stackpress_lib::queue::task(
                move |start: &mut StartupContext| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        start.register(engine)?;
                        start.register(Arc::new(environment(1000)))?;
                        Ok(Flow::Continue)
                    })
                },
            ))?;
            let mut layers = ConfigLayers::new();
            layers.value("real host",json!({"session":{"key":"identity","seed":"source-seed"},"api":{"expires":60000,"endpoints":[{"method":"GET","route":"/api","type":"public","event":"unused"}]}}))?;
            app.config_layers(layers)?;
            // Public registrars and source event names are the whole identity/storage interface.
            app.plugin("identity", identity::register)?;
            app.plugin("api", register)?;
            app.plugin("api-sql", sql::register)?;
            if let Some(barrier) = barrier {
                app.on_priority(
                    "me",
                    1,
                    stackpress_lib::handler(move |req, res, inv| {
                        let barrier = barrier.clone();
                        Box::pin(async move {
                            // Reenter a real SQL event before waiting, while the original JWT request stays borrowed.
                            let mut data = Nest::new();
                            data.set("id", "a")?;
                            let found = stackpress_framework_api::helpers::resolve(
                                &inv,
                                "application-detail",
                                data,
                            )
                            .await?;
                            assert_eq!(value(&found)["id"], "a");
                            assert!(req.session.get("identity").is_some());
                            assert!(res.result().is_none());
                            barrier.wait().await;
                            Ok(Flow::Continue)
                        })
                    }),
                )?;
            }
            app.build().await.map_err(|failure| failure.error)
        }
        async fn close(self) {
            self.engine.close().await.unwrap();
            std::fs::remove_file(self.path).unwrap();
        }
    }
    async fn jwt(server: &Server, profile: &str) -> String {
        server
            .host()
            .service::<Identity>()
            .unwrap()
            .create(
                json!({"id":profile,"roles":["USER"]})
                    .as_object()
                    .unwrap()
                    .clone(),
            )
            .await
            .unwrap()
    }
    async fn grant(server: &Server, profile: &str, id: &str) -> Result<Response> {
        let mut req = request(
            "POST",
            json!({"client_id":"a","redirect_uri":"/return?existing=1","id":id,"secret":"source-secret","scopes":["unknown","read"],"profileId":"forged","applicationId":"forged","expires":-999}),
        );
        let token = jwt(server, profile).await;
        req.session.set("identity", token.clone(), None)?;
        let mut res = Response::new();
        server.handle_route(&mut req, &mut res).await?;
        assert_eq!(code(&res), Some(302));
        assert_eq!(
            res.headers().get("location"),
            Some(format!("/return?existing=1&code={id}&state=a%26b").as_str())
        );
        assert_eq!(value(&res)["secret"], "app-plain");
        assert_eq!(
            req.session.get("identity").and_then(|value| value.as_str()),
            Some(token.as_str())
        );
        Ok(res)
    }
    #[tokio::test]
    async fn real_jwt_me_application_shared_response_and_session_sql_exchange() -> Result<()> {
        let db = Database::new().await?;
        let server = db.host(None).await?;
        let valid = jwt(&server, "p").await;
        for (cookie, header) in [
            (None, None),
            (Some("broken"), None),
            (None, Some(valid.as_str())),
        ] {
            let mut req = request("GET", input());
            if let Some(cookie) = cookie {
                req.session.set("identity", cookie, None)?;
            }
            if let Some(header) = header {
                req.headers
                    .set("authorization", &format!("Bearer {header}"))?;
            }
            let mut res = Response::new();
            server.handle_route(&mut req, &mut res).await?;
            assert_eq!(code(&res), Some(302));
            assert!(
                res.headers()
                    .get("location")
                    .unwrap()
                    .starts_with("/auth/signin?")
            );
        }
        grant(&server, "p", "actual-code").await?;
        let selected = server
            .resolve("session-detail", json!({"id":"actual-code"}))
            .await?;
        let stored = selected.response.results.unwrap().to_json()?;
        assert_eq!(stored["applicationId"], "a");
        assert_eq!(stored["profileId"], "p");
        assert_eq!(stored["secret"], "source-secret");
        assert_eq!(stored["scopes"], json!(["unknown", "read"]));
        assert_eq!(stored["expires"], "1970-01-01T00:01:01.000Z");
        for _ in 0..2 {
            let mut req = Request::builder()
                .method("GET")
                .url("/auth/oauth/token")
                .data(json!({"code":"actual-code"}))
                .build()?;
            req.headers
                .set("authorization", "Anything a:deliberately-wrong-secret")?;
            let mut res = Response::new();
            server.handle_route(&mut req, &mut res).await?;
            let result = value(&res);
            assert_eq!(result["access_token"], "actual-code");
            assert_eq!(result["access_secret"], "source-secret");
            assert_eq!(result["expires_in"], 60);
            assert_eq!(result["user"]["id"], "p");
            assert_eq!(result["user"]["name"], "User p");
            assert!(result["user"].get("roles").is_none());
            res.to_envelope().try_to_json()?;
        }
        drop(server);
        db.close().await;
        Ok(())
    }
    #[test]
    #[cfg(unix)]
    fn concurrent_real_jwt_and_sql_pages_have_process_deadline() {
        assert!(super::super::proof::watchdog(
            false,
            "oauth::persisted::concurrent_child"
        ));
    }
    #[test]
    #[ignore = "executed by process watchdog parent"]
    fn concurrent_child() -> Result<()> {
        std::fs::write(std::env::var("SLICE10_READY").unwrap(), b"ready")
            .map_err(Error::from_source)?;
        tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .map_err(Error::from_source)?
            .block_on(async {
                let db = Database::new().await?;
                let server = db
                    .host(Some(Arc::new(tokio::sync::Barrier::new(2))))
                    .await?;
                let (a, b) = tokio::join!(
                    grant(&server, "p", "parallel-a"),
                    grant(&server, "q", "parallel-b")
                );
                a?;
                b?;
                for (id, profile) in [("parallel-a", "p"), ("parallel-b", "q")] {
                    let row = server
                        .resolve("session-detail", json!({"id":id}))
                        .await?
                        .response
                        .results
                        .unwrap()
                        .to_json()?;
                    assert_eq!(row["profileId"], profile);
                }
                drop(server);
                db.close().await;
                Ok(())
            })
    }
}

#[tokio::test]
async fn direct_token_needs_no_api_config_and_honors_prior_response_guard() -> Result<()> {
    let state = Arc::new(State::default());
    let mut app = ServerBuilder::new();
    let selected = state.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let selected = selected.clone();
            Box::pin(async move {
                start.register(selected)?;
                Ok(Flow::Continue)
            })
        },
    ))?;
    pages::register(&mut app)?;
    app.on("session-detail", detail)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    for (body, status, skip) in [
        (json!([]), 200, true),
        (json!(false), 200, false),
        (Json::Null, 409, true),
    ] {
        let mut res = Response::new();
        res.set("application/json", Body::Value(Value::try_from(body)?))?;
        res.status_code(status, None)?;
        server
            .emit(
                "GET /auth/oauth/token",
                &mut request("GET", json!({})),
                &mut res,
            )
            .await?;
        assert_eq!(code(&res), Some(if skip { status } else { 401 }));
    }
    let mut req = request("GET", json!({"code":"s"}));
    req.headers.set("authorization", "X a:any")?;
    let res = invoke(&server, "/auth/oauth/token", &mut req).await?;
    assert_eq!(value(&res)["access_token"], "s");
    assert_eq!(state.calls.lock().unwrap().len(), 1);
    Ok(())
}

#[tokio::test]
#[cfg(feature = "native")]
async fn native_profile_date_is_projected_to_source_json_and_clock_errors_propagate() -> Result<()>
{
    let mut state = State::default();
    let profile = object(state.detail.as_mut().unwrap())
        .get_mut("profile")
        .unwrap();
    object(profile).insert("created".into(), Timestamp::new(0, 0)?.into());
    let server = host(Arc::new(state), json!({}), environment(0)).await?;
    let mut req = request("GET", json!({"code":"s"}));
    req.headers.set("authorization", "X a:any")?;
    let res = invoke(&server, "/auth/oauth/token", &mut req).await?;
    assert_eq!(value(&res)["user"]["created"], "1970-01-01T00:00:00.000Z");
    res.to_envelope().try_to_json()?;
    let env = pages::Environment {
        now: Arc::new(|| Err(Error::new("clock fixture"))),
        ..Default::default()
    };
    let server = host(Arc::new(State::default()), json!({}), env).await?;
    assert!(
        invoke(&server, "/auth/oauth", &mut request("POST", input()))
            .await
            .is_err()
    );
    Ok(())
}
