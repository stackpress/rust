//! Source admission, data forwarding and return/error contracts using current event names.

use std::sync::{Arc, Mutex};

use serde_json::{Value as Json, json};
use stackpress_framework_api::{ApiEndpoint, ApiType, endpoints};
use stackpress_lib::{Error, Flow, Headers, Invocation, MethodPattern, Request, Response, Result};
use stackpress_server::{RouteSpec, Server, ServerBuilder, ServerInvocationExt, StartupContext};

/// An immutable observation made before a fake callback mutates the response.
#[derive(Clone, Debug)]
pub(super) struct Call {
    pub event: String,
    pub method: String,
    pub data: Json,
    pub status: Option<u16>,
}

/// Per-host fakes; no production store or identity service is introduced.
pub(super) struct State {
    pub row: Option<Json>,
    pub lookup_status: u16,
    pub lookup_error: bool,
    pub abort_target: bool,
    pub target_error: bool,
    pub calls: Mutex<Vec<Call>>,
    pub barrier: Option<tokio::sync::Barrier>,
    pub reenter: bool,
    pub profile_from_id: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            row: Some(json!({"id": "row", "secret": "stored", "scopes": ["write"],
            "profileId": "stored-profile", "active": false, "expires": "1970-01-01T00:00:00Z"})),
            lookup_status: 200,
            lookup_error: false,
            abort_target: false,
            target_error: false,
            calls: Mutex::new(Vec::new()),
            barrier: None,
            reenter: false,
            profile_from_id: false,
        }
    }
}

impl State {
    /// Retain one finite snapshot and release its guard before any callback can suspend.
    pub fn record(&self, event: &str, req: &Request, res: &Response) -> Result<()> {
        let call = Call {
            event: event.into(),
            method: req.method.as_str().into(),
            data: req.data.to_json()?,
            status: status(res),
        };
        self.calls.lock().unwrap().push(call);
        Ok(())
    }
    /// Return an owned snapshot for assertions without keeping a lock across dispatch.
    pub fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }
    /// Inspect only callback order when payload details are irrelevant to the assertion.
    pub fn events(&self) -> Vec<String> {
        self.calls().into_iter().map(|call| call.event).collect()
    }
}

/// A representative source endpoint, with explicit route kind and no default CORS.
pub(super) fn endpoint(kind: ApiType, method: &str, path: &str) -> ApiEndpoint {
    ApiEndpoint {
        name: None,
        description: None,
        example: None,
        method: method.into(),
        route: path.into(),
        api_type: kind,
        scopes: vec!["read".into(), "write".into()],
        cors: None,
        event: "target".into(),
        priority: 0,
        data: Default::default(),
    }
}

/// Read staged status without inventing a default status in a raw-dispatch proof.
pub(super) fn status(res: &Response) -> Option<u16> {
    res.status().map(|value| value.code)
}

/// Build a request with deliberately independent browser JWT and API credentials.
pub(super) fn request(
    method: &str,
    url: &str,
    authorization: Option<&str>,
    data: Json,
) -> Result<Request> {
    let mut headers = Headers::new();
    if let Some(authorization) = authorization {
        headers.set("authorization", authorization)?;
    }
    headers.set("cookie", "session=browser-jwt")?;
    let mut request = Request::builder()
        .method(method)
        .url(url)
        .headers(headers)
        .data(data)
        .build()?;
    request.session.set("session", "browser-jwt", None)?;
    Ok(request)
}

/// Install source-named detail fakes and target observers; none perform I/O.
pub(super) fn fixture(state: Arc<State>) -> Result<ServerBuilder> {
    let mut app = ServerBuilder::new();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let state = state.clone();
            Box::pin(async move {
                start.register(state)?;
                Ok(Flow::Continue)
            })
        },
    ))?;
    app.on("application-detail", detail)?;
    app.on("session-detail", detail)?;
    app.on("target", target)?;
    app.on("target", target_tail)?;
    app.on("lookup-observer", observer)?;
    Ok(app)
}

/// Source lookup input contains only the header ID, never the original browser request.
async fn detail(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(req.method.as_str(), "GET");
    assert_eq!(req.data.entries().len(), 1);
    assert!(req.headers.get("authorization").is_none());
    assert!(req.headers.get("cookie").is_none());
    assert!(req.session.is_empty());
    let state = inv.service::<State>()?;
    state.record(inv.event(), req, res)?;
    if let Some(barrier) = &state.barrier {
        barrier.wait().await;
    }
    if state.reenter {
        // Reenter the same immutable host after record() has released its synchronization guard.
        inv.emit("lookup-observer", req, &mut Response::new())
            .await?;
    }
    tokio::task::yield_now().await;
    if state.lookup_error {
        return Err(Error::new("selected detail failure").with_code(503));
    }
    if let Some(mut row) = state.row.clone() {
        if state.profile_from_id {
            row["profileId"] = json!(format!(
                "profile-{}",
                req.data.get("id").unwrap().as_str().unwrap()
            ));
        }
        res.results(&row)?;
    }
    res.status_code(state.lookup_status, None)?;
    Ok(Flow::Abort) // Detail Abort must not become the endpoint's return.
}

/// Observe target entry before writing results, including pre-existing error/status fields.
async fn target(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.service::<State>()?;
    state.record("target", req, res)?;
    if state.target_error {
        return Err(Error::new("selected target failure").with_code(502));
    }
    res.results(&req.data)?;
    Ok(if state.abort_target {
        Flow::Abort
    } else {
        Flow::Continue
    })
}

/// A second child listener reveals whether Abort stayed inside the target event.
async fn target_tail(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<State>()?.record("target-tail", req, res)?;
    Ok(Flow::Continue)
}

/// A later outer route reveals whether admission failure or child Abort stopped its chain.
pub(super) async fn tail(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    inv.service::<State>()?.record("route-tail", req, res)?;
    Ok(Flow::Continue)
}

/// An ordinary nested observer used by the finite reentrancy/concurrency scenario.
async fn observer(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<State>()?
        .record("lookup-observer", req, res)?;
    Ok(Flow::Continue)
}

/// Register the direct endpoint inventory plus a lower-priority outer observer.
pub(super) async fn direct(config: &[ApiEndpoint], state: Arc<State>) -> Result<Server> {
    let mut app = fixture(state)?;
    endpoints::routes(&mut app, config)?;
    if let Some(first) = config.first() {
        app.route_with(
            RouteSpec::new(MethodPattern::Any, &first.route).priority(-100),
            tail,
        )?;
    }
    app.build().await.map_err(|failure| failure.error)
}

#[tokio::test]
async fn protected_get_and_non_get_credential_decision_table() -> Result<()> {
    for kind in [ApiType::App, ApiType::Session] {
        for (method, header, allowed) in [
            ("GET", Some("Ignored id"), true),
            ("GET", Some("Ignored id:wrong"), true),
            ("HEAD", Some("Ignored id"), false),
            ("HEAD", Some("Ignored id:stored"), true),
            ("POST", Some("Ignored id:wrong"), false),
            ("POST", Some("Ignored id:stored"), true),
            ("OPTIONS", Some("Ignored id:"), false),
            ("OPTIONS", Some("Ignored id:stored"), true),
            ("GET", None, false),
            ("GET", Some("Ignored  id:stored"), false),
        ] {
            let state = Arc::new(State::default());
            let server = direct(&[endpoint(kind, method, "/api")], state.clone()).await?;
            let mut req = request(method, "/api", header, json!({"caller": true}))?;
            let mut res = Response::new();
            server
                .emit(&format!("{method} /api"), &mut req, &mut res)
                .await?;
            assert_eq!(status(&res), Some(if allowed { 200 } else { 401 }));
            assert_eq!(
                state.events().iter().any(|value| value == "target"),
                allowed
            );
            assert_eq!(
                state.events().last().unwrap(),
                "route-tail",
                "rejection never adds Abort"
            );
            assert_eq!(
                req.session.get("session").unwrap().as_str(),
                Some("browser-jwt")
            );
            if let Some(lookup) = state
                .calls()
                .iter()
                .find(|call| call.event.ends_with("-detail"))
            {
                assert_eq!(
                    lookup.event,
                    if kind == ApiType::App {
                        "application-detail"
                    } else {
                        "session-detail"
                    }
                );
                assert_eq!(lookup.method, "GET");
                assert_eq!(lookup.data, json!({"id": "id"}));
            }
        }
    }
    Ok(())
}

#[tokio::test]
async fn scopes_are_any_exact_overlap_without_registry_or_role_checks() -> Result<()> {
    for (permits, held, allowed) in [
        (json!(["read", "write"]), json!(["write"]), true),
        (json!([]), json!(["write"]), false),
        (json!(["write"]), json!([]), false),
        (json!(["unregistered"]), json!(["unregistered"]), true),
        (json!(["WRITE"]), json!(["write"]), false),
        (json!(["article:*"]), json!(["article:read"]), false),
        (json!(["*"]), json!(["*"]), true),
        (json!([""]), json!([""]), true),
        (
            json!(["write"]),
            json!([false, {"scope": "write"}, "write"]),
            true,
        ),
    ] {
        let mut config = endpoint(ApiType::App, "GET", "/api");
        config.scopes = serde_json::from_value(permits).map_err(Error::from_source)?;
        let mut state = State::default();
        state.row.as_mut().unwrap()["scopes"] = held;
        let state = Arc::new(state);
        let server = direct(&[config], state.clone()).await?;
        server
            .emit(
                "GET /api",
                &mut request("GET", "/api", Some("Any id"), json!({}))?,
                &mut Response::new(),
            )
            .await?;
        assert_eq!(state.events().contains(&"target".into()), allowed);
    }
    Ok(())
}

#[tokio::test]
async fn fixed_data_and_session_profile_follow_source_order() -> Result<()> {
    for kind in [ApiType::Public, ApiType::App, ApiType::Session] {
        let mut config = endpoint(kind, "POST", "/api/:id");
        config.data = json!({"nested": {"fixed": 2}, "array": [], "id": "fixed-id",
            "profileId": "fixed-profile", "active": false, "literal.key": 7})
        .as_object()
        .unwrap()
        .clone();
        let state = Arc::new(State {
            lookup_status: 500,
            ..State::default()
        });
        let server = direct(&[config], state.clone()).await?;
        let mut req = request(
            "POST",
            "/api/path-id",
            Some("Other credential-id:stored"),
            json!({"nested": {"caller": 1}, "array": [1], "profileId": "forged", "id": "caller-id", "other": true}),
        )?;
        let mut res = Response::new();
        res.status_code(419, None)?;
        server.emit("POST /api/path-id", &mut req, &mut res).await?;
        let target = state
            .calls()
            .into_iter()
            .find(|call| call.event == "target")
            .unwrap();
        assert_eq!(
            target.status,
            Some(419),
            "detail status ignored; no prior-response guard added"
        );
        assert_eq!(
            target.data,
            json!({"nested": {"fixed": 2}, "array": [], "id": "fixed-id",
            "profileId": if kind == ApiType::Session {"stored-profile"} else {"fixed-profile"},
            "active": false, "literal.key": 7, "other": true})
        );
        assert_eq!(
            req.headers.get("authorization"),
            Some("Other credential-id:stored")
        );
        assert_eq!(
            req.session.get("session").unwrap().as_str(),
            Some("browser-jwt")
        );
        if kind == ApiType::Public {
            assert!(
                !state
                    .events()
                    .iter()
                    .any(|event| event.ends_with("-detail"))
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn source_result_truthiness_and_property_error_precedence() -> Result<()> {
    for row in [
        None,
        Some(Json::Null),
        Some(json!(false)),
        Some(json!(0)),
        Some(json!("")),
    ] {
        let state = Arc::new(State {
            row,
            ..State::default()
        });
        let server = direct(&[endpoint(ApiType::App, "GET", "/api")], state).await?;
        let mut res = Response::new();
        server
            .emit(
                "GET /api",
                &mut request("GET", "/api", Some("X id"), json!({}))?,
                &mut res,
            )
            .await?;
        assert_eq!(status(&res), Some(401));
    }
    // Truthy malformed rows reach the source property access: GET fails on scopes, while
    // non-GET rejects a missing/non-string secret before touching malformed scopes.
    for row in [json!({}), json!([]), json!(true), json!(1), json!("text")] {
        for method in ["GET", "POST"] {
            let state = Arc::new(State {
                row: Some(row.clone()),
                ..State::default()
            });
            let server = direct(&[endpoint(ApiType::App, method, "/api")], state).await?;
            let mut res = Response::new();
            let result = server
                .emit(
                    &format!("{method} /api"),
                    &mut request(method, "/api", Some("X id:stored"), json!({}))?,
                    &mut res,
                )
                .await;
            assert_eq!(result.is_err(), method == "GET");
            if method == "GET" {
                assert_eq!(
                    result.unwrap_err().code(),
                    500,
                    "source property failure is a runtime error"
                );
            }
            if method == "POST" {
                assert_eq!(status(&res), Some(401));
            }
        }
    }
    // A matching non-GET secret also reaches malformed scopes as a real error, not 401.
    let state = Arc::new(State {
        row: Some(json!({"secret": "stored", "scopes": {}})),
        ..State::default()
    });
    let server = direct(&[endpoint(ApiType::Session, "POST", "/api")], state).await?;
    let failure = server
        .emit(
            "POST /api",
            &mut request("POST", "/api", Some("X id:stored"), json!({}))?,
            &mut Response::new(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        failure.code(),
        500,
        "malformed lookup scopes remain a runtime failure"
    );
    Ok(())
}

#[tokio::test]
async fn get_does_not_require_stored_secret_and_profile_values_are_ordinary_data() -> Result<()> {
    for profile in [
        None,
        Some(Json::Null),
        Some(json!(7)),
        Some(json!({"ordinary": true})),
    ] {
        let mut row = json!({"scopes": ["write"]});
        if let Some(profile) = &profile {
            row["profileId"] = profile.clone();
        }
        let state = Arc::new(State {
            row: Some(row),
            ..State::default()
        });
        let server = direct(&[endpoint(ApiType::Session, "GET", "/api")], state.clone()).await?;
        let mut req = request("GET", "/api", Some("X id"), json!({"profileId": "forged"}))?;
        server
            .emit("GET /api", &mut req, &mut Response::new())
            .await?;
        assert_eq!(req.data.to_json()?.get("profileId"), profile.as_ref());
        assert!(state.events().contains(&"target".into()));
    }
    Ok(())
}

#[tokio::test]
async fn current_lookup_errors_propagate_and_unknown_lookup_denies() -> Result<()> {
    let state = Arc::new(State {
        lookup_error: true,
        ..State::default()
    });
    let server = direct(&[endpoint(ApiType::Session, "GET", "/api")], state.clone()).await?;
    let failure = server
        .emit(
            "GET /api",
            &mut request("GET", "/api", Some("X id"), json!({}))?,
            &mut Response::new(),
        )
        .await
        .unwrap_err();
    assert_eq!(
        failure.code(),
        503,
        "the selected detail error is not remapped"
    );
    assert_eq!(state.events(), ["session-detail"]);
    let mut app = ServerBuilder::new();
    endpoints::routes(&mut app, &[endpoint(ApiType::App, "GET", "/api")])?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let mut res = Response::new();
    server
        .emit(
            "GET /api",
            &mut request("GET", "/api", Some("X id"), json!({}))?,
            &mut res,
        )
        .await?;
    assert_eq!(status(&res), Some(401));
    Ok(())
}

#[tokio::test]
async fn fixed_empty_key_uses_source_member_count_including_collisions() -> Result<()> {
    for (input, expected) in [
        (
            json!({"0": "zero", "x": true}),
            json!({"0": "zero", "x": true, "2": "appended", "a.b": "literal"}),
        ),
        (
            json!({"2": "old", "x": true}),
            json!({"2": "appended", "x": true, "a.b": "literal"}),
        ),
    ] {
        let mut config = endpoint(ApiType::Public, "GET", "/api");
        config.data = json!({"": "appended", "a.b": "literal"})
            .as_object()
            .unwrap()
            .clone();
        let state = Arc::new(State::default());
        let server = direct(&[config], state.clone()).await?;
        server
            .emit(
                "GET /api",
                &mut request("GET", "/api", None, input)?,
                &mut Response::new(),
            )
            .await?;
        let target = state
            .calls()
            .into_iter()
            .find(|call| call.event == "target")
            .unwrap();
        assert_eq!(target.data, expected);
    }
    Ok(())
}

#[tokio::test]
async fn target_error_propagates_after_completed_overlay_and_prevents_outer_tail() -> Result<()> {
    let state = Arc::new(State {
        target_error: true,
        ..State::default()
    });
    let mut config = endpoint(ApiType::Public, "GET", "/api");
    config.data.insert("completed".into(), json!(true));
    let server = direct(&[config], state.clone()).await?;
    let mut req = request("GET", "/api", None, json!({}))?;
    let mut res = Response::new();
    res.status_code(409, None)?;
    let error = server
        .emit("GET /api", &mut req, &mut res)
        .await
        .unwrap_err();
    assert_eq!(error.code(), 502);
    assert_eq!(req.data.to_json()?, json!({"completed": true}));
    assert_eq!(status(&res), Some(409));
    assert_eq!(state.events(), ["target"]);
    Ok(())
}
