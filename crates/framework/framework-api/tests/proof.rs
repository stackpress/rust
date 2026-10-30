//! Source-mechanics proofs: fake model/session events and fetch, with no network or database.

use std::{
    collections::BTreeMap,
    future::Future,
    pin::pin,
    process::{Child, Command},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use serde_json::{Value as Json, json};
use stackpress_framework_api::{ApiConfig, Service, helpers::*, register};
use stackpress_framework_schema::{
    Documentation, Field, FieldAccess, Input, Model, RecordSchema, Relation, SchemaResult,
    ValueContext, ValueProviders, ValueRule, ValueType,
};
use stackpress_lib::{
    Error, Flow, HandlerFuture, Headers, Invocation, Nest, ParseLimits, Request, Response, Result,
};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, ServerInvocationExt};

/// Build the ordinary host with only the explicitly supplied optional API layer.
async fn configured(layer: Json) -> Result<Server> {
    let mut app = ServerBuilder::new();
    register(&mut app)?;
    let mut layers = ConfigLayers::new();
    layers.value("proof", layer)?;
    app.config_layers(layers)?;
    app.build().await.map_err(|failure| failure.error)
}

#[tokio::test]
async fn optional_config_activation_and_typed_boundary() -> Result<()> {
    // Missing and explicit null deactivate before publishing API resources or callbacks.
    for layer in [json!({}), json!({"api": null})] {
        let server = configured(layer).await?;
        assert!(server.host().service::<Service>().is_err());
        assert_eq!(server.listeners().count(), 0);
    }
    // An empty object activates; unknown source fields and scope names add no new policy.
    let server = configured(json!({"api": {"extra": true, "endpoints": [{
        "method": "ALL", "route": "/same", "type": "app", "event": "custom",
        "priority": -7, "scopes": ["unregistered"], "cors": [], "data": {"active": false}
    }]}}))
    .await?;
    let service = server.host().service::<Service>()?;
    assert_eq!(service.config.endpoints[0].priority, -7);
    assert_eq!(service.config.endpoints[0].scopes, ["unregistered"]);
    assert_eq!(
        server.listeners().count(),
        5 + usize::from(cfg!(feature = "ui-ssr")),
        "Two M3 processes, two CORS callbacks and the endpoint"
    );
    assert!(
        configured(json!({"api": {}}))
            .await?
            .host()
            .service::<Service>()
            .is_ok()
    );
    assert!(
        configured(json!({"api": {"endpoints": [{"type": "typo"}]}}))
            .await
            .is_err()
    );
    let mut duplicate = ServerBuilder::new();
    register(&mut duplicate)?;
    assert!(register(&mut duplicate).is_err());
    Ok(())
}

#[test]
fn source_expiry_omission_null_and_timeclip() -> Result<()> {
    // Source defaulting distinguishes omission from null, zero, fractions and negative TTLs.
    for (wire, expected) in [
        (json!({}), Some(86_400_010)),
        (json!({"expires": null}), Some(10)),
        (json!({"expires": 0}), Some(10)),
        (json!({"expires": -11.9}), Some(-1)),
        (json!({"expires": 0.9}), Some(10)),
        (json!({"expires": 8_640_000_000_000_000_f64}), None),
    ] {
        let config: ApiConfig = serde_json::from_value(wire).map_err(Error::from_source)?;
        assert_eq!(config.expires_at(10), expected);
    }
    let mut config = ApiConfig {
        expires: Input::Value(8_640_000_000_000_000_f64),
        ..ApiConfig::default()
    };
    assert_eq!(config.expires_at(0), Some(8_640_000_000_000_000));
    config.expires = Input::Value(f64::NAN);
    assert_eq!(config.expires_at(0), None);
    config.expires = Input::Value(f64::INFINITY);
    assert_eq!(config.expires_at(0), None);
    Ok(())
}

#[test]
fn source_authorization_split_trim_and_method_table() -> Result<()> {
    // Expected outputs are source fixtures, including its positional split and ignored scheme.
    for (method, header, expected) in [
        ("GET", "Anything app", Some(("app", "app", ""))),
        ("get", " app", Some(("app", "app", ""))),
        (
            "POST",
            "Other app:secret:unused",
            Some(("app:secret:unused", "app", "secret")),
        ),
        ("GET", "Bearer app: ignored", Some(("app:", "app", ""))),
        (
            "GET",
            "Bearer \u{feff}app\u{feff}:\u{a0}secret",
            Some(("app\u{feff}:\u{a0}secret", "app", "secret")),
        ),
        ("GET", "Bearer \u{85}", Some(("\u{85}", "\u{85}", ""))),
        ("HEAD", "Bearer app", None),
        ("OPTIONS", "Bearer app:", None),
        ("POST", "Bearer app", None),
        ("GET", "Bearer  app:secret", None),
        ("GET", "Bearer\tapp:secret", None),
        ("GET", "Bearer :secret", None),
        ("GET", "Bearer \u{feff}", None),
        ("GET", "Bearer", None),
    ] {
        let parsed = parse_authorization(method, header);
        assert_eq!(
            parsed.as_ref().map(|value| (
                value.token.as_str(),
                value.id.as_str(),
                value.secret.as_str()
            )),
            expected,
            "{method} {header:?}"
        );
    }
    // Parsing rejection stages 401; a valid GET leaves the response's existing status alone.
    let mut response = Response::new();
    assert!(authorize(&Request::new(), &mut response)?.is_none());
    assert_eq!(response.status().map(|status| status.code), Some(401));
    assert_eq!(response.error(), Some("Unauthorized"));
    let mut headers = Headers::new();
    headers.set("authorization", "Whatever app")?;
    let request = Request::builder().headers(headers).build()?;
    assert!(authorize(&request, &mut response)?.is_some());
    assert_eq!(response.status().map(|status| status.code), Some(401));
    Ok(())
}

#[test]
fn fixed_data_replaces_top_level_and_keeps_literal_keys() -> Result<()> {
    let mut data = Nest::from_object(
        json!({"nested": {"old": 1}, "array": [1], "profileId": "forged", "other": true}),
    )?;
    let fixed = Nest::from_object(
        json!({"nested": {"new": 2}, "array": [], "a.b": 3, "profileId": "fixed"}),
    )?;
    overlay(&mut data, &fixed)?;
    // Session forwarding occurs after the fixed overlay and remains ordinary request data.
    data.set("profileId", "selected-profile")?;
    assert_eq!(
        data.to_json()?,
        json!({"nested": {"new": 2}, "array": [], "a.b": 3, "profileId": "selected-profile", "other": true})
    );
    Ok(())
}

#[test]
fn overlay_reports_inherited_limits_and_retains_prior_keys() -> Result<()> {
    let mut target = Nest::with_limits(ParseLimits {
        max_bytes: 5,
        ..ParseLimits::default()
    });
    let fixed = Nest::from_object(json!({"a": "ok", "b": "too long"}))?;
    assert!(overlay(&mut target, &fixed).is_err());
    assert_eq!(
        target.to_json()?,
        json!({"a": "ok"}),
        "source set is not a transaction"
    );
    Ok(())
}

/// Decode object fixtures independently so includes never accidentally shares containers.
fn predicate(value: Json) -> Result<PredicateObject> {
    let PredicateValue::Object(object) = PredicateValue::from_json(&value)? else {
        panic!("object fixture required");
    };
    Ok(object)
}

#[test]
fn valid_data_subset_scalar_and_missing_table() -> Result<()> {
    for (expected, actual, matches) in [
        (json!({}), json!({"extra": 1}), true),
        (json!({"a": {"b": 2}}), json!({"a": {"b": 2, "c": 3}}), true),
        (
            json!({"a": [1, null, true]}),
            json!({"a": [true, 1, null, "other"]}),
            true,
        ),
        (json!({"a": [1, 1]}), json!({"a": [1]}), true),
        (json!({"a": null}), json!({}), false),
        (json!({"a": null}), json!({"a": null}), true),
        (json!({"a": "same"}), json!({"a": "same"}), true),
        (json!({"a": "left"}), json!({"a": "right"}), false),
        (json!({"a": ["x"]}), json!({"a": ["x", "y"]}), true),
        (json!({"a": "1"}), json!({"a": 1}), false),
        (json!({"a": false}), json!({"a": 0}), false),
        (json!({"a": []}), json!({"a": {}}), false),
        (json!({"a": {}}), json!({"a": []}), false),
        (json!({"a": [2]}), json!({"a": [1]}), false),
        (json!({"a": 1}), json!({"a": 2}), false),
    ] {
        assert_eq!(
            valid_data(&predicate(expected)?, &predicate(actual)?),
            matches
        );
    }
    let undefined = Arc::new(BTreeMap::from([("a".into(), PredicateValue::Undefined)]));
    assert!(
        !valid_data(&undefined, &undefined),
        "actual undefined fails before comparison"
    );
    assert!(PredicateValue::from_json(&Json::Array(vec![Json::Null; 10_001])).is_err());
    Ok(())
}

#[test]
fn valid_data_reference_identity_and_same_value_zero() -> Result<()> {
    // Q10-REF: structural equality of separately parsed includes elements is insufficient.
    for wire in [json!({"a": [{}]}), json!({"a": [[]]})] {
        assert!(!valid_data(&predicate(wire.clone())?, &predicate(wire)?));
    }
    let shared = Arc::new(vec![
        PredicateValue::Object(predicate(json!({"x": 1}))?),
        PredicateValue::Array(Arc::new(vec![])),
        PredicateValue::Number(f64::NAN),
        PredicateValue::Number(-0.0),
        PredicateValue::Undefined,
    ]);
    let expected = Arc::new(BTreeMap::from([(
        "a".into(),
        PredicateValue::Array(shared.clone()),
    )]));
    let actual = Arc::new(BTreeMap::from([(
        "a".into(),
        PredicateValue::Array(Arc::new(shared.iter().cloned().collect())),
    )]));
    assert!(
        valid_data(&expected, &actual),
        "explicit clones retain container identity"
    );
    let scalar_nan = Arc::new(BTreeMap::from([(
        "a".into(),
        PredicateValue::Number(f64::NAN),
    )]));
    assert!(
        !valid_data(&scalar_nan, &scalar_nan),
        "scalar NaN uses strict equality"
    );
    assert!(valid_data(
        &predicate(json!({"a": 0}))?,
        &predicate(json!({"a": -0.0}))?
    ));
    Ok(())
}

/// Per-host trace, with each guard dropped before any await or nested operation.
#[derive(Default)]
struct Trace(Mutex<Vec<String>>);
impl Trace {
    fn push(&self, message: impl Into<String>) {
        self.0.lock().unwrap().push(message.into());
    }
    fn values(&self) -> Vec<String> {
        self.0.lock().unwrap().clone()
    }
}

async fn trace(invocation: &Invocation<'_>, value: &str) -> Result<()> {
    invocation.service::<Trace>()?.push(value);
    tokio::task::yield_now().await;
    Ok(())
}

async fn before(_: &mut Request, _: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    trace(&invocation, "request").await?;
    Ok(Flow::Continue)
}
async fn after(_: &mut Request, _: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    trace(&invocation, "response").await?;
    Ok(Flow::Continue)
}
async fn child_abort(
    _: &mut Request,
    _: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    trace(&invocation, "child-abort").await?;
    Ok(Flow::Abort)
}
async fn child_error(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Err(Error::new("selected child failure"))
}
async fn tail(_: &mut Request, _: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    trace(&invocation, "tail").await?;
    Ok(Flow::Continue)
}
async fn wrapper(
    req: &mut Request,
    res: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let event = req
        .data
        .get("child")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_owned();
    emit_event(&invocation, &event, req, res).await
}

/// Install fakes under source event names without creating a session or model dependency.
fn traced_builder() -> Result<(ServerBuilder, Arc<Trace>)> {
    let mut app = ServerBuilder::new();
    let trace = Arc::new(Trace::default());
    let owned_trace = trace.clone();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut stackpress_server::StartupContext| {
            let trace = owned_trace.clone();
            Box::pin(async move {
                start.register(trace)?;
                Ok(Flow::Continue)
            })
        },
    ))?;
    app.on("request", before)?;
    app.on("response", after)?;
    app.on("outer", wrapper)?;
    app.on("outer", tail)?;
    app.on_priority("aborted", 1, child_abort)?;
    app.on("aborted", tail)?;
    app.on("failure", child_error)?;
    Ok((app, trace))
}

#[tokio::test]
async fn raw_emit_isolates_child_abort_and_no_listener() -> Result<()> {
    for (child, expected) in [
        (
            "aborted",
            vec!["request", "child-abort", "tail", "response"],
        ),
        ("missing", vec!["request", "tail", "response"]),
    ] {
        let (app, trace) = traced_builder()?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        let mut request = Request::builder().data(json!({"child": child})).build()?;
        server
            .handle("outer", &mut request, &mut Response::new())
            .await?;
        assert_eq!(
            trace.values(),
            expected,
            "child returns do not control the outer chain"
        );
    }
    let (app, trace) = traced_builder()?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let mut request = Request::builder()
        .data(json!({"child": "failure"}))
        .build()?;
    assert!(
        server
            .emit("outer", &mut request, &mut Response::new())
            .await
            .is_err()
    );
    assert!(
        trace.values().is_empty(),
        "real child error propagates before outer tail"
    );
    Ok(())
}

/// Compile-time check of borrowing futures, not a detached spawn that changes request ownership.
fn send<T: Send>(value: T) -> T {
    value
}

async fn me(req: &mut Request, res: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    let jwt = req
        .session
        .get("session")
        .and_then(|value| value.as_str())
        .unwrap();
    trace(&invocation, &format!("me:{jwt}")).await?;
    res.results(&json!({"id": format!("profile-{jwt}")}))?;
    Ok(Flow::Continue)
}
async fn detail(req: &mut Request, res: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    // A source ctx.resolve(object) starts a new request and only forwards explicit object data.
    assert_eq!(req.method.as_str(), "GET");
    assert!(req.headers.get("authorization").is_none());
    assert!(req.headers.get("cookie").is_none());
    assert!(req.session.is_empty());
    let id = req
        .data
        .get("id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_owned();
    assert_eq!(req.data.to_json()?, json!({"id": id}));
    trace(&invocation, &format!("detail:{id}")).await?;
    res.results(&json!({"id": id, "profileId": format!("profile-{id}")}))?;
    Ok(Flow::Abort)
}
async fn lookup(req: &mut Request, res: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    // Browser me receives the original request, while app/session lookups receive fresh objects.
    let mut browser = Response::new();
    send(emit_event(&invocation, "me", req, &mut browser)).await?;
    let id = req
        .data
        .get("id")
        .and_then(|value| value.as_str())
        .unwrap()
        .to_owned();
    let data = Nest::from_object(json!({"id": id}))?;
    let separate = send(resolve(&invocation, "session-detail", data.clone())).await?;
    assert_eq!(
        separate.result().unwrap().to_json()?,
        json!({"id": id, "profileId": format!("profile-{id}")})
    );
    assert!(
        res.result().is_none(),
        "fresh resolve owns an independent response"
    );
    send(resolve_into(&invocation, "application-detail", data, res)).await?;
    req.data.set("profileId", format!("profile-{id}"))?;
    assert_eq!(
        req.session.get("session").and_then(|value| value.as_str()),
        Some(id.as_str())
    );
    Ok(Flow::Continue)
}

#[tokio::test]
async fn nested_requests_are_send_fresh_and_concurrently_isolated() -> Result<()> {
    let (mut app, trace) = traced_builder()?;
    app.on("me", me)?;
    app.on("session-detail", detail)?;
    app.on("application-detail", detail)?;
    app.on("lookup", lookup)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    let server = &server;
    let execute = |id: &'static str| async move {
        let mut headers = Headers::new();
        headers
            .set("authorization", "Custom browser")?
            .set("cookie", "session=browser")?;
        let mut request = Request::builder()
            .method("POST")
            .headers(headers)
            .data(json!({"id": id, "private": true}))
            .build()?;
        request.session.set("session", id, None)?;
        let mut response = Response::new();
        server.handle("lookup", &mut request, &mut response).await?;
        assert_eq!(
            request
                .data
                .get("profileId")
                .and_then(|value| value.as_str()),
            Some(format!("profile-{id}").as_str())
        );
        assert_eq!(response.result().unwrap().to_json()?["id"], id);
        Ok::<_, Error>(())
    };
    let (first, second) = tokio::join!(send(execute("A")), send(execute("B")));
    first?;
    second?;
    let values = trace.values();
    assert_eq!(values.iter().filter(|value| *value == "request").count(), 2);
    assert_eq!(
        values.iter().filter(|value| *value == "response").count(),
        2
    );
    for id in ["A", "B"] {
        assert_eq!(
            values
                .iter()
                .filter(|value| **value == format!("detail:{id}"))
                .count(),
            2
        );
        assert!(values.contains(&format!("me:{id}")));
    }
    Ok(())
}

/// Test-only provider; no real CUID implementation is claimed before Lane A delivers it.
struct Provider {
    calls: AtomicUsize,
    output: Json,
}
impl ValueProviders for Provider {
    fn generate(&self, name: &str, args: &[Json]) -> SchemaResult<Json> {
        assert_eq!(name, "cuid");
        assert!(args.is_empty());
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.output.clone())
    }
    fn validate(&self, _: &str, _: &Json, _: &[Json]) -> SchemaResult<bool> {
        Ok(true)
    }
}

/// A minimal descriptor fixture: API owns its FK, while Profile remains an external model.
fn field(name: &str, kind: ValueType) -> Field {
    Field {
        name: name.into(),
        rust_name: name.into(),
        column: name.into(),
        value_type: kind,
        storage_length: None,
        required: true,
        multiple: false,
        key: name == "id",
        default: None,
        nonempty: false,
        active: false,
        relation: None,
        unique: false,
        searchable: false,
        sortable: false,
        timestamp: false,
        generated: false,
        input_required: true,
        value_rule: None,
        rules: vec![],
        documentation: Documentation::default(),
        roles: vec![],
        access: FieldAccess::default(),
        attributes: vec![],
    }
}

#[test]
fn sql_descriptor_provider_and_external_profile_boundary() -> Result<()> {
    let rule = ValueRule {
        provider: "cuid".into(),
        args: vec![],
        revision: "M0-fake".into(),
    };
    let provider = Provider {
        calls: AtomicUsize::new(0),
        output: json!("fixture-id"),
    };
    let context = ValueContext {
        providers: Some(&provider),
        ..ValueContext::default()
    };
    let mut id = field("id", ValueType::String);
    id.value_rule = Some(rule.clone());
    id.input_required = false;
    let mut profile = field("profile", ValueType::Model("Profile".into()));
    profile.relation = Some(Relation {
        local: "profileId".into(),
        foreign: "id".into(),
        inverse: None,
    });
    let schema = RecordSchema {
        model: Model {
            name: "Session".into(),
            rust_name: "ApiSession".into(),
            module: "api_session".into(),
            fields: vec![id, field("profileId", ValueType::String), profile],
            documentation: Documentation::default(),
            query: vec![],
            attributes: vec![],
        },
        fieldsets: vec![],
        enums: vec![],
    };
    // Explicit provider resolution is the existing storage-preparation seam, not a new generator.
    let generated = rule
        .resolve_for(&ValueType::String, &context)
        .map_err(Error::from_source)?;
    assert_eq!(provider.calls.load(Ordering::SeqCst), 1);
    let record = json!({"id": generated, "profileId": "external-profile"});
    assert!(
        schema
            .validate(&record, &context, false, true)
            .map_err(Error::from_source)?
            .is_empty()
    );
    let roundtrip: RecordSchema =
        serde_json::from_value(serde_json::to_value(&schema).map_err(Error::from_source)?)
            .map_err(Error::from_source)?;
    assert_eq!(
        roundtrip.model.fields[2].relation.as_ref().unwrap().local,
        "profileId"
    );
    assert_eq!(
        roundtrip.model.fields[2].value_type,
        ValueType::Model("Profile".into())
    );
    assert!(
        rule.resolve_for(&ValueType::String, &ValueContext::default())
            .is_err()
    );
    let wrong = Provider {
        calls: AtomicUsize::new(0),
        output: json!(7),
    };
    assert!(
        rule.resolve_for(
            &ValueType::String,
            &ValueContext {
                providers: Some(&wrong),
                now: None
            }
        )
        .is_err()
    );
    Ok(())
}

/// Transport fake records one borrowed call and yields so its Send future crosses a suspension.
#[derive(Default)]
struct FetchFake {
    calls: Mutex<Vec<FetchCall>>,
    reject: bool,
}
/// Recorded URI, method, content type and exact borrowed bytes.
type FetchCall = (String, String, String, Vec<u8>);
impl Fetch for FetchFake {
    fn send<'a>(&'a self, request: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>> {
        Box::pin(async move {
            self.calls.lock().unwrap().push((
                request.uri.into(),
                request.method.into(),
                request.content_type.into(),
                request.body.into(),
            ));
            tokio::task::yield_now().await;
            if self.reject {
                Err(Error::new("transport rejected"))
            } else {
                Ok(500)
            }
        })
    }
}

#[tokio::test]
async fn inline_fetch_borrows_awaits_once_and_ignores_resolved_status() -> Result<()> {
    let body = br#"{"data":{},"params":{"secret":"intentional"},"results":{}}"#;
    for reject in [false, true] {
        let fetch = FetchFake {
            reject,
            ..FetchFake::default()
        };
        assert_eq!(
            send(fetch_once(
                &fetch,
                "http://127.0.0.1/configured",
                "POST",
                body
            ))
            .await
            .is_err(),
            reject
        );
        assert_eq!(
            *fetch.calls.lock().unwrap(),
            vec![(
                "http://127.0.0.1/configured".into(),
                "POST".into(),
                "application/json".into(),
                body.to_vec()
            )]
        );
    }
    Ok(())
}

/// A borrowed host service can observe nested-future release without a detached owner.
struct LookupDrop<'a>(&'a Trace);
impl Drop for LookupDrop<'_> {
    fn drop(&mut self) {
        self.0.push("lookup-dropped");
    }
}
async fn pending_lookup(
    _: &mut Request,
    _: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let trace = invocation.service::<Trace>()?;
    trace.push("lookup-entered");
    let _notice = LookupDrop(trace);
    std::future::pending().await
}
async fn cancel_lookup(
    _: &mut Request,
    _: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let waker = Waker::noop();
    {
        let mut future = pin!(send(resolve(&invocation, "pending-lookup", Nest::new())));
        // Poll the real dispatch until its selected handler suspends, then drop it explicitly.
        assert!(matches!(
            future.as_mut().poll(&mut Context::from_waker(waker)),
            Poll::Pending
        ));
        assert_eq!(
            invocation.service::<Trace>()?.values(),
            ["request", "lookup-entered"]
        );
    }
    assert_eq!(
        invocation.service::<Trace>()?.values(),
        ["request", "lookup-entered", "lookup-dropped"]
    );
    Ok(Flow::Continue)
}

#[tokio::test]
async fn cancelling_nested_lookup_drops_borrowed_child() -> Result<()> {
    let (mut app, trace) = traced_builder()?;
    app.on("pending-lookup", pending_lookup)?;
    app.on("cancel-lookup", cancel_lookup)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    server
        .handle("cancel-lookup", &mut Request::new(), &mut Response::new())
        .await?;
    assert_eq!(
        trace.values(),
        ["request", "lookup-entered", "lookup-dropped", "response"]
    );
    Ok(())
}

struct DropNotice(Arc<AtomicUsize>);
impl Drop for DropNotice {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
struct PendingFetch(Arc<AtomicUsize>);
impl Fetch for PendingFetch {
    fn send<'a>(&'a self, _: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>> {
        Box::pin(async move {
            let _notice = DropNotice(self.0.clone());
            std::future::pending().await
        })
    }
}

#[test]
fn cancelling_inline_fetch_drops_pending_send() {
    let drops = Arc::new(AtomicUsize::new(0));
    let fetch = PendingFetch(drops.clone());
    let waker = Waker::noop();
    {
        let mut future = pin!(fetch_once(&fetch, "configured", "POST", b"{}"));
        assert!(matches!(
            future.as_mut().poll(&mut Context::from_waker(waker)),
            Poll::Pending
        ));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
    }
    assert_eq!(
        drops.load(Ordering::SeqCst),
        1,
        "no detached delivery survives cancellation"
    );
}

/// Reenter the host from fetch without retaining a synchronization guard across dispatch.
struct ReentrantFetch<'a, 'host>(&'a Invocation<'host>);
impl Fetch for ReentrantFetch<'_, '_> {
    fn send<'a>(&'a self, _: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>> {
        Box::pin(async move {
            let response = resolve(
                self.0,
                "application-detail",
                Nest::from_object(json!({"id": "reentrant"}))?,
            )
            .await?;
            assert!(response.result().is_some());
            Ok(200)
        })
    }
}
async fn reentrant(_: &mut Request, _: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    send(fetch_once(
        &ReentrantFetch(&invocation),
        "configured",
        "POST",
        b"{}",
    ))
    .await?;
    Ok(Flow::Continue)
}

/// Parent owns the process group until wait reaps it, including assertion/panic paths.
struct OwnedChild {
    child: Child,
    reaped: bool,
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        // Never signal a reaped PID, which could already have been reused by another process.
        if !self.reaped {
            // The child can be stuck outside Tokio; kill its group, then kill/reap the child.
            let _ = Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", self.child.id())])
                .status();
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

/// Readiness and completion have separate hard wall-clock deadlines; no runtime timer is trusted.
#[cfg(unix)]
pub(super) fn watchdog(block: bool, test: &str) -> bool {
    use std::os::unix::process::CommandExt;
    let ready = std::env::temp_dir().join(format!(
        "slice10-{}-{test}-{block}.ready",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&ready);
    let mut child = OwnedChild {
        child: Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--ignored", "--nocapture"])
            .env("SLICE10_READY", &ready)
            .env("SLICE10_BLOCK", if block { "1" } else { "0" })
            .process_group(0)
            .spawn()
            .unwrap(),
        reaped: false,
    };
    let start = Instant::now();
    while !ready.exists() {
        child.reaped = child.child.try_wait().unwrap().is_some();
        assert!(!child.reaped, "child exited before readiness");
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "readiness deadline exceeded"
        );
        std::thread::sleep(Duration::from_millis(10)); // Watchdog polling, not scenario ordering.
    }
    let start = Instant::now();
    let deadline = if block {
        Duration::from_millis(100)
    } else {
        Duration::from_secs(10)
    };
    let completed = loop {
        if let Some(status) = child.child.try_wait().unwrap() {
            child.reaped = true;
            assert!(status.success());
            break true;
        }
        if start.elapsed() >= deadline {
            break false;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Drop always kills/reaps before returning, including the deliberately blocked child.
    drop(child);
    std::fs::remove_file(ready).unwrap();
    completed
}

#[test]
#[cfg(unix)]
fn reentrant_lookup_fetch_has_process_watchdog() {
    assert!(
        watchdog(false, "proof::watchdog_child"),
        "reentrant case deadline exceeded"
    );
}

#[test]
#[cfg(unix)]
fn process_watchdog_kills_and_reaps_blocked_child() {
    assert!(
        !watchdog(true, "proof::watchdog_child"),
        "blocked child unexpectedly completed"
    );
}

#[test]
#[ignore = "run only inside the parent process watchdog"]
fn watchdog_child() -> Result<()> {
    // Observable readiness precedes the selected case; the blocked case proves parent kill/reap.
    std::fs::write(std::env::var("SLICE10_READY").unwrap(), b"ready")
        .map_err(Error::from_source)?;
    if std::env::var("SLICE10_BLOCK").unwrap() == "1" {
        loop {
            std::thread::park();
        }
    }
    tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(Error::from_source)?
        .block_on(async {
            let (mut app, trace) = traced_builder()?;
            app.on("application-detail", detail)?;
            app.on("reentrant", reentrant)?;
            let server = app.build().await.map_err(|failure| failure.error)?;
            server
                .handle("reentrant", &mut Request::new(), &mut Response::new())
                .await?;
            assert_eq!(trace.values(), ["request", "detail:reentrant", "response"]);
            Ok(())
        })
}
