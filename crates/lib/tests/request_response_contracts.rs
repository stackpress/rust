//! M3 values, serialization, one-attempt state, scoped cookies and capability DX.

use serde::{Serialize, Serializer};
use stackpress_lib::session::cookie;
use stackpress_lib::{
    Body, CookieOptions, DeliveryState, DispatchOutcome, EmitterBuilder, Error, ErrorKind, Flow,
    Headers, Invocation, LoadResult, LoadState, Nest, ParseLimits, Priority, PublicError, Request,
    RequestView, Response, ResponseEnvelope, ResponseView, Result, SameSite, Session,
    SessionRevision, SessionValue, SharedNative, SourceLocation, Status, Value,
};
use std::time::{Duration, UNIX_EPOCH};

/// A real typed payload accepted directly by results/rows/json.
#[derive(Serialize)]
struct Profile {
    id: u64,
    name: String,
}

/// Defaults and source precedence preserve false/zero/empty and literal URL helper behavior.
#[test]
fn request_defaults_sources_headers_session_and_redaction() -> Result<()> {
    // Compare independent defaults with overlapping query/post/data and repeated cookie headers.

    let default = Request::new();
    assert_eq!(default.method.as_str(), "GET");
    assert_eq!(default.url.as_str(), "http://unknownhost/");
    assert_eq!(default.mimetype(), "text/plain");
    assert_eq!(default.body().type_name(), "empty");
    assert!(!default.loaded());
    assert!(
        default.query.is_empty()
            && default.post.is_empty()
            && default.headers.is_empty()
            && default.session.is_empty()
    );
    // Repeated Cookie lines must combine without losing first-wins identity; content type is not
    // implicit parser authority.
    let headers = Headers::from_pairs([
        ("COOKIE", "session=first; invalid"),
        ("cookie", "theme=dark; session=second"),
        ("Content-Type", "application/json"),
    ])?;
    let req = Request::builder()
        .url("https://example.test/x?common=query&count=1")
        .method("post")
        .headers(headers)
        .post(serde_json::json!({"common":"post","zero":0}))
        .data(serde_json::json!({"common":"explicit","secret":"DO-NOT-LOG"}))
        .body(Body::Text("PRIVATE-BODY".into()))
        .mimetype("application/json")
        .build()?;
    assert_eq!(req.method.as_str(), "POST");
    assert_eq!(
        req.data.path_as::<String>("common")?,
        Some("explicit".into())
    );
    assert_eq!(req.post.path_as::<String>("common")?, Some("post".into()));
    assert_eq!(req.query.path_as::<String>("common")?, Some("query".into()));
    assert_eq!(req.data.path_as::<u64>("zero")?, Some(0));
    assert_eq!(
        req.session.get("session").and_then(SessionValue::as_str),
        Some("first")
    );
    assert_eq!(
        req.session.get("theme").and_then(SessionValue::as_str),
        Some("dark")
    );
    assert!(req.session.revisions().is_empty());
    assert_eq!(req.headers.get("CONTENT-TYPE"), Some("application/json"));
    // Both original and snapshot wrapper diagnostics must redact buffered secrets.
    for debug in [format!("{req:?}"), format!("{:?}", req.view())] {
        assert!(!debug.contains("DO-NOT-LOG") && !debug.contains("PRIVATE-BODY"));
    }
    // Explicit initial state wins over headers and is not recorded as outgoing revisions.
    let explicit = Session::from_pairs([("session", "explicit")])?;
    let req = Request::builder()
        .url("/x?count=2")
        .query(serde_json::json!({"count":"1"}))
        .session(explicit)
        .build()?;
    assert_eq!(req.url.host_str(), Some("unknownhost"));
    assert_eq!(req.query.path_as::<String>("count")?, Some("1".into()));
    assert_eq!(
        req.session.get("session").and_then(SessionValue::as_str),
        Some("explicit")
    );
    assert_eq!(
        Request::builder()
            .query_string("count=3")
            .build()?
            .data
            .path_as::<u64>("count")?,
        Some(3)
    );
    // A malformed absolute URL cannot be repaired into unknownhost path input.
    assert!(Request::builder().url("https://").build().is_err());
    assert!(Request::builder().method("bad verb").build().is_err());
    assert!(Request::builder().mimetype("broken").build().is_err());
    assert_eq!(
        stackpress_lib::with_unknown_host("https://example.test"),
        "http://unknownhost/https://example.test"
    );
    assert_eq!(
        Request::builder()
            .data_serialized(&serde_json::json!({"x":false}))
            .build()?
            .data
            .path_as::<bool>("x")?,
        Some(false)
    );
    Ok(())
}

/// Snapshot readers stage post/body updates and successful repeats never call a reader.
async fn successful_loader(mut view: RequestView) -> Result<LoadResult> {
    // Modify the isolated snapshot deliberately; only returned staged body/post may reach the
    // original.

    assert_eq!(
        view.data.path_as::<String>("common")?,
        Some("explicit".into())
    );
    view.data.set("snapshot_only", true)?;
    tokio::task::yield_now().await;
    Ok(LoadResult {
        body: Some(Body::Text("".into())),
        post: Some(Nest::from_object(
            serde_json::json!({"common":false,"total":0}),
        )?),
    })
}
/// Loading remains once-only, atomic on failure, and compatible with borrowed readers.
#[tokio::test]
async fn request_load_success_failure_and_borrowed_native_reader() -> Result<()> {
    // Exercise successful repeat, reader failure, staged validation failure and a caller-borrowed
    // reader independently.

    let mut req = Request::builder()
        .data(serde_json::json!({"common":"explicit"}))
        .build()?;
    // Commit the named loader's staged result, then assert snapshot-only edits never reached
    // original data.
    req.load_with(successful_loader).await?;
    assert_eq!(req.load_state(), LoadState::Loaded);
    assert!(req.loaded());
    assert_eq!(req.data.path_as::<bool>("common")?, Some(false));
    assert!(req.data.path("snapshot_only").is_none());
    assert_eq!(req.post.path_as::<u64>("total")?, Some(0));
    assert!(matches!(req.body(),Body::Text(v) if v.is_empty()));
    // A completed load must skip even a failing replacement callback.
    req.load_with(|_| async { Err(Error::new("must not run twice")) })
        .await?;
    let mut empty = Request::new();
    empty.load().await?;
    assert!(empty.loaded());
    // A real reader failure consumes its attempt and makes implicit retry unsafe.
    let mut failed = Request::new();
    assert!(
        failed
            .load_with(|_| async { Err(Error::new("reader failed")) })
            .await
            .is_err()
    );
    assert_eq!(failed.load_state(), LoadState::Failed);
    assert!(failed.load().await.is_err());
    // Returned oversize body must reject the entire staged update, including otherwise valid post
    // fields.
    let mut limited = Request::builder()
        .limits(ParseLimits {
            max_bytes: 64,
            ..ParseLimits::default()
        })
        .build()?;
    assert!(
        limited
            .load_with(|_| async {
                Ok(LoadResult {
                    body: Some(Body::Bytes(vec![0; 65])),
                    post: Some(Nest::from_object(serde_json::json!({"prefix":true}))?),
                })
            })
            .await
            .is_err()
    );
    assert_eq!(limited.load_state(), LoadState::Failed);
    assert!(limited.data.is_empty() && limited.post.is_empty());
    assert!(matches!(limited.body(), Body::Empty));
    // Borrow a real caller-owned counter to prove no static callback capture is required.
    let mut native_reads = 0usize;
    // The reader borrows its caller; no fake globally owned native resource is needed.
    let mut borrowed = Request::new();
    let reader = &mut native_reads;
    borrowed
        .load_with(move |_| async move {
            *reader += 1;
            Ok(LoadResult::default())
        })
        .await?;
    assert_eq!(native_reads, 1);
    // Replacing a public store does not relax the load result's final budget.
    let mut limited = Request::builder()
        .limits(ParseLimits {
            max_bytes: 64,
            ..Default::default()
        })
        .build()?;
    limited.data = Nest::from_object(serde_json::json!({"large":"x".repeat(65)}))?;
    assert!(limited.load().await.is_err());
    assert_eq!(limited.load_state(), LoadState::Failed);
    Ok(())
}

/// In-process native data remains visibly Shared through source copying and resolve.
#[tokio::test]
async fn request_source_clones_and_resolve_preserve_only_explicit_shared_identity() -> Result<()> {
    // Keep one native identity beside ordinary nested text, then mutate only the separate query
    // collection.

    // Opaque in-process results keep their type and identity; generic JSON conversion must reject
    // them.
    let shared = SharedNative::new(String::from("opaque native"));
    let mut query = Nest::new();
    query
        .set("native", shared.clone())?
        .set_path("profile.name", "Ada")?;
    let mut req = Request::builder().query(query).build()?;
    req.query.set_path("profile.name", "Grace")?;
    assert_eq!(
        req.data.path_as::<String>("profile.name")?,
        Some("Ada".into())
    );
    let native = req
        .data
        .get("native")
        .and_then(Value::as_shared)
        .expect("native");
    assert!(shared.ptr_eq(native));
    req.load().await?;
    async fn native_echo(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // Moving an owned Request into resolve must retain its loaded state and explicit native
        // identity.

        assert!(req.loaded(), "owned request keeps its attempt state");
        res.results_value(req.data.get("native").expect("input").clone())?;
        Ok(Flow::Continue)
    }
    let mut builder = EmitterBuilder::new();
    builder.on("native", native_echo)?;
    // Resolve moves the original request state and transfers only explicit public results into the
    // envelope.
    let response = builder.build().resolve("native", req).await?.response;
    assert!(
        shared.ptr_eq(
            response
                .results
                .as_ref()
                .and_then(Value::as_shared)
                .expect("shared result")
        )
    );
    assert_eq!(
        response
            .try_to_json()
            .expect_err("explicit wire conversion")
            .path(),
        Some("results")
    );
    Ok(())
}

/// Multi-value headers remain separate and reject injection before modifying originals.
#[test]
fn headers_validate_case_repeat_values_and_atomic_rejection() -> Result<()> {
    // Combine case variants and repeated Set-Cookie before attempting an invalid framing update.

    let mut headers = Headers::new();
    headers
        .set("X-Test", "one")?
        .append("x-test", "two")?
        .append("Set-Cookie", "a=1")?
        .append("set-cookie", "b=2")?;
    assert_eq!(headers.len(), 2);
    assert_eq!(headers.get_all("X-TEST").expect("values"), ["one", "two"]);
    assert_eq!(headers.entries().count(), 2);
    // Name/value injection must fail before replacing the existing repeated header values.
    assert!(headers.set("bad name", "v").is_err());
    assert!(
        headers
            .append("x-test", "injected\r\nHeader: value")
            .is_err()
    );
    assert_eq!(headers.get_all("x-test").expect("unchanged").len(), 2);
    assert_eq!(
        headers.remove("SET-COOKIE").expect("removed"),
        ["a=1", "b=2"]
    );
    assert!(!headers.has("set-cookie"));
    assert!(!format!("{headers:?}").contains("one"));
    Ok(())
}

/// Typed serialization failures are returned before any output/status/total mutation.
struct Broken;
impl Serialize for Broken {
    /// An intentional custom serializer failure.
    fn serialize<S: Serializer>(&self, _: S) -> std::result::Result<S::Ok, S::Error> {
        // Return a genuine serializer error so response atomicity is proved before any body/status
        // replacement.

        Err(serde::ser::Error::custom("broken serialization"))
    }
}
/// Results retain objects, rows retain arrays, while json explicitly stores text.
#[test]
fn typed_response_helpers_and_serialization_are_atomic() -> Result<()> {
    // Use a typed struct rather than only serde_json::Value to prove the public Serialize contract.

    let mut response = Response::new();
    assert!(
        response.status().is_none() && response.result().is_none() && response.mimetype().is_none()
    );
    assert_eq!(response.total(), 0);
    let profile = Profile {
        id: 7,
        name: "Ada".into(),
    };
    response.results(&profile)?;
    assert_eq!(response.status(), Some(&Status::OK));
    assert_eq!(response.total(), 1);
    assert_eq!(
        response.result().expect("result").to_json()?,
        serde_json::json!({"id":7,"name":"Ada"})
    );
    // Snapshot the public output before a custom serializer failure and invalid row shape.
    let before = response.to_envelope().try_to_json()?;
    assert!(response.results(&Broken).is_err());
    assert!(response.rows(&profile, 2).is_err());
    assert_eq!(response.to_envelope().try_to_json()?, before);
    // A supplied zero total is meaningful even when one typed row is buffered.
    response.rows(&[profile], 0)?;
    assert_eq!(response.total(), 0);
    assert!(response.result().and_then(Value::as_array).is_some());
    response.json(&serde_json::json!({"name":"Ada"}))?;
    assert!(matches!(response.body(),Body::Text(v) if v.contains("Ada")));
    assert_eq!(response.mimetype(), Some("application/json"));
    response
        .html("<h1>Ada</h1>")?
        .status_code(201, None)?
        .set_total(3)?;
    assert_eq!(response.mimetype(), Some("text/html"));
    assert_eq!(response.status(), Some(&Status::CREATED));
    response.xml("<profile/>")?;
    assert_eq!(response.mimetype(), Some("text/xml"));
    // Redirect selection changes Location and status together only after header validation.
    response.redirect("/profile")?;
    assert!(response.redirected());
    assert_eq!(response.headers().get("LOCATION"), Some("/profile"));
    assert_eq!(response.status(), Some(&Status::FOUND));
    assert!(response.redirect("bad\r\nInjected: yes").is_err());
    // JSON text conversion must honor an explicitly larger response budget through serialization
    // and final buffering.
    let policy = ParseLimits {
        max_bytes: ParseLimits::default().max_bytes + 128,
        ..Default::default()
    };
    let text = "x".repeat(ParseLimits::default().max_bytes + 1);
    let mut large = Response::with_limits(policy);
    large.json(&text)?;
    assert!(matches!(large.body(),Body::Text(body) if body.len()>ParseLimits::default().max_bytes));
    response.set("application/octet-stream", Body::Bytes(vec![0, 255]))?;
    assert_eq!(response.body().type_name(), "bytes");
    assert!(response.to_envelope().try_to_json().is_err());
    Ok(())
}

/// Response-selected limits must remain usable when exporting its public envelope as JSON.
#[test]
fn response_envelope_json_respects_selected_limits() -> Result<()> {
    // Put a nested text value just above the default budget under a deliberately larger policy.
    let defaults = ParseLimits::default();
    let limits = ParseLimits {
        max_bytes: defaults.max_bytes + 256,
        ..defaults
    };
    let text = "x".repeat(defaults.max_bytes + 1);
    let mut results = Nest::with_limits(limits);
    results.set_path("profile.bio", text.clone())?;
    let value = results.into_value();

    // The default response must still reject this value rather than silently relaxing its policy.
    assert!(Response::new().results_value(value.clone()).is_err());
    let mut response = Response::with_limits(limits);
    response.results_value(value.clone())?;
    response
        .errors_mut()?
        .set_path("profile.bio", text.clone())?;

    // Standalone envelopes retain a defined default wire budget when no policy is supplied.
    let standalone = ResponseEnvelope {
        results: Some(value),
        ..Default::default()
    };
    assert!(standalone.try_to_json().is_err());

    // Direct response conversion retains its own policy rather than applying unrelated defaults.
    let json = response.try_to_json()?;
    assert_eq!(
        json["results"]["profile"]["bio"].as_str(),
        Some(text.as_str())
    );

    assert_eq!(
        json["errors"]["profile"]["bio"].as_str(),
        Some(text.as_str())
    );
    assert_eq!(response.delivery_state(), DeliveryState::Open);

    // Cloned and moved data-only envelopes can use an explicit copy of their originating budget.
    let selected = response.limits();
    assert_eq!(selected, limits);
    assert_eq!(response.to_envelope().try_to_json_with(&selected)?, json);
    assert_eq!(response.into_envelope().try_to_json_with(&selected)?, json);
    Ok(())
}

/// A replaced field-error store cannot replace the owning response's tighter conversion policy.
#[test]
fn response_json_retains_limits_after_public_store_replacement() -> Result<()> {
    // Mutable store access can replace a Nest with one retaining a larger independent policy.
    let limits = ParseLimits {
        max_bytes: 64,
        ..Default::default()
    };
    let mut response = Response::with_limits(limits);
    *response.errors_mut()? = Nest::from_object(serde_json::json!({"detail": "x".repeat(65)}))?;
    let failure = response.try_to_json().expect_err("owning response policy");
    assert_eq!(failure.kind(), ErrorKind::Limit);
    assert_eq!(failure.path(), Some("errors.detail"));

    // A copied policy can select a different wire boundary without changing the response itself.
    let mut wire_limits = response.limits();
    wire_limits.max_bytes = 128;
    assert!(
        response
            .to_envelope()
            .try_to_json_with(&wire_limits)
            .is_ok()
    );
    assert_eq!(response.limits(), limits);
    assert!(response.try_to_json().is_err());
    assert_eq!(response.delivery_state(), DeliveryState::Open);
    Ok(())
}

/// A resolver caller can retain an explicit wire policy without adding state to public envelopes.
#[tokio::test]
async fn resolved_envelope_json_accepts_an_explicit_large_budget() -> Result<()> {
    /// This operation deliberately selects a response policy matching its prepared input data.
    async fn echo(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // Replacing output is an explicit operation choice; the resulting envelope remains data.
        *res = Response::with_limits(req.data.limits());
        res.results_value(req.data.value().clone())?;
        Ok(Flow::Continue)
    }

    // Prepare owned input under the same explicit policy that will be used at the wire boundary.
    let defaults = ParseLimits::default();
    let limits = ParseLimits {
        max_bytes: defaults.max_bytes + 256,
        ..defaults
    };
    let text = "x".repeat(defaults.max_bytes + 1);
    let mut input = Nest::with_limits(limits);
    input.set_path("profile.bio", text.clone())?;
    let request = Request::builder().limits(limits).data(input).build()?;
    let mut builder = EmitterBuilder::new();
    builder.on("echo", echo)?;
    let call = builder.build().resolve("echo", request).await?;

    // Resolution still separates execution outcome from the public response data; the caller
    // explicitly chooses a large wire policy rather than relying on hidden origin tracking.
    assert_eq!(call.outcome, DispatchOutcome::Completed);
    assert!(call.response.try_to_json().is_err());
    let json = call.response.try_to_json_with(&limits)?;
    assert_eq!(
        json["results"]["profile"]["bio"].as_str(),
        Some(text.as_str())
    );
    Ok(())
}

/// Hand-built envelopes use defaults unless an explicit policy is supplied for both value fields.
#[test]
fn standalone_envelope_json_uses_explicit_limits_for_each_value() -> Result<()> {
    // Exercise errors/results separately so a missed policy forwarding call cannot hide behind
    // the other field's earlier failure.
    let defaults = ParseLimits::default();
    let limits = ParseLimits {
        max_bytes: defaults.max_bytes + 256,
        ..defaults
    };
    for field in ["errors", "results"] {
        let text = "x".repeat(defaults.max_bytes + 1);
        let mut data = Nest::with_limits(limits);
        data.set_path("profile.bio", text.clone())?;
        let value = data.into_value();
        let mut envelope = ResponseEnvelope {
            code: Some(0),
            status: Some(String::new()),
            error: Some(String::new()),
            total: Some(0),
            stack: Some(vec![]),
            ..Default::default()
        };
        if field == "errors" {
            // A public field assignment remains available without new required/private fields.
            envelope.errors = Some(value);
        } else {
            envelope.results = Some(value);
        }

        // Legacy/default conversion still rejects the same value with full envelope context.
        let failure = envelope
            .try_to_json()
            .expect_err("default budget must reject");
        assert_eq!(failure.kind(), ErrorKind::Limit);
        assert_eq!(
            failure.path(),
            Some(format!("{field}.profile.bio").as_str())
        );
        let json = envelope.try_to_json_with(&limits)?;
        assert_eq!(json[field]["profile"]["bio"].as_str(), Some(text.as_str()));
        assert_eq!(json["code"], 0);
        assert_eq!(json["status"], "");
        assert_eq!(json["error"], "");
        assert_eq!(json["total"], 0);
        assert_eq!(json["stack"], serde_json::json!([]));
        // The absent sibling field stays absent; successful explicit conversion does not change
        // the policy subsequently used by the no-argument operation.
        let absent = if field == "errors" {
            "results"
        } else {
            "errors"
        };
        assert!(json.get(absent).is_none());
        assert!(envelope.try_to_json().is_err());
    }
    Ok(())
}

/// Explicit policies can be stricter too, without losing native paths or changing field presence.
#[test]
fn envelope_json_limits_preserve_paths_presence_and_tighter_budgets() -> Result<()> {
    // A three-node tree succeeds exactly at its depth/entry/byte boundaries.
    let value = Value::try_from(serde_json::json!({"profile": {"active": false}}))?;
    let envelope = ResponseEnvelope {
        results: Some(value),
        errors: Some(Value::Null),
        total: Some(0),
        ..Default::default()
    };
    let exact = ParseLimits {
        max_bytes: "profileactive".len(),
        max_depth: 2,
        max_entries: 3,
        ..Default::default()
    };
    let before = envelope.try_to_json()?;
    assert_eq!(envelope.try_to_json_with(&exact)?, before);
    assert!(before["errors"].is_null());
    assert_eq!(before["results"]["profile"]["active"], false);
    assert!(before.get("code").is_none());
    for tighter in [
        ParseLimits {
            max_bytes: exact.max_bytes - 1,
            ..exact
        },
        ParseLimits {
            max_depth: exact.max_depth - 1,
            ..exact
        },
        ParseLimits {
            max_entries: exact.max_entries - 1,
            ..exact
        },
    ] {
        // Each independent budget failure is qualified by the envelope field and leaves input
        // data intact for another conversion under an adequate policy.
        let failure = envelope
            .try_to_json_with(&tighter)
            .expect_err("tighter budget");
        assert_eq!(failure.kind(), ErrorKind::Limit);
        assert!(
            failure
                .path()
                .is_some_and(|path| path.starts_with("results"))
        );
        assert_eq!(envelope.try_to_json()?, before);
    }

    // Non-JSON native values are rejected under every adequate budget; keep the exact child
    // suffix for both result and field-error paths rather than collapsing it to the envelope root.
    let mut data = Nest::new();
    data.set_path("profile.native", SharedNative::new(7u32))?;
    for field in ["errors", "results"] {
        let mut envelope = ResponseEnvelope::default();
        let mut response = Response::new();
        if field == "errors" {
            // Field errors remain object-root data even when a nested native child is present.
            envelope.errors = Some(data.value().clone());
            *response.errors_mut()? = data.clone();
        } else {
            // Structured results retain the same native child without choosing a wire encoding.
            envelope.results = Some(data.value().clone());
            response.results_value(data.value().clone())?;
        }
        let failure = envelope
            .try_to_json_with(&ParseLimits::default())
            .expect_err("native");
        assert_eq!(
            failure.path(),
            Some(format!("{field}.profile.native").as_str())
        );
        // Response-owned export preserves the same failure path and does not attempt delivery.
        let failure = response.try_to_json().expect_err("native response value");
        assert_eq!(
            failure.path(),
            Some(format!("{field}.profile.native").as_str())
        );
        assert_eq!(response.delivery_state(), DeliveryState::Open);
    }
    Ok(())
}

/// Presence-based envelope import retains false/zero/null/empty and stages failures.
#[test]
fn envelope_presence_error_roundtrip_and_shared_results() -> Result<()> {
    // Apply present falsy/empty fields separately from absent fields so overlay behavior is
    // observable.

    let mut response = Response::new();
    response
        .results(&serde_json::json!({"old":true}))?
        .set_total(8)?;
    response.errors_mut()?.set("old", "old error")?;
    response.apply_envelope(ResponseEnvelope {
        code: Some(9999),
        status: Some("Custom".into()),
        error: Some("".into()),
        errors: Some(Value::Object(Default::default())),
        results: Some(Value::Bool(false)),
        total: Some(0),
        stack: Some(vec![SourceLocation::caller()]),
    })?;
    assert_eq!(response.status().expect("status").code, 9999);
    assert_eq!(response.status().expect("status").message, "Custom");
    assert_eq!(response.error(), Some(""));
    assert_eq!(response.total(), 0);
    assert!(response.errors().is_empty());
    assert_eq!(response.stack().expect("stack").len(), 1);
    assert_eq!(response.result().and_then(Value::as_bool), Some(false));
    let before = response.to_envelope().try_to_json()?;
    assert!(
        response
            .apply_envelope(ResponseEnvelope {
                code: Some(400),
                errors: Some(Value::Null),
                ..Default::default()
            })
            .is_err()
    );
    assert_eq!(response.to_envelope().try_to_json()?, before);
    response.apply_envelope(ResponseEnvelope {
        results: Some(Value::Null),
        ..Default::default()
    })?;
    assert!(response.to_envelope().try_to_json()?["results"].is_null());
    let shared = SharedNative::new(String::from("private resource"));
    response.results_value(shared.clone().into())?;
    assert!(
        response
            .to_envelope()
            .results
            .as_ref()
            .and_then(Value::as_shared)
            .expect("native")
            .ptr_eq(&shared)
    );
    let error = response
        .to_envelope()
        .try_to_json()
        .expect_err("explicit wire conversion");
    assert_eq!(error.path(), Some("results"));
    assert!(
        response
            .into_envelope()
            .results
            .and_then(|v| match v {
                Value::Shared(v) => Some(v),
                _ => None,
            })
            .expect("moved shared")
            .ptr_eq(&shared)
    );
    Ok(())
}

/// Original causes/frames are retained without automatic public stack/path disclosure.
#[test]
fn response_error_disclosure_and_owned_cause_are_preserved() -> Result<()> {
    // Keep a private source error and public message distinct, then transfer ownership back through
    // into_error.

    let cause = "invalid".parse::<u32>().expect_err("cause");
    let error = Error::from_source(cause)
        .with_code(400)
        .with_public_message("Invalid count")
        .with_errors(serde_json::json!({"count":["must be an integer"]}));
    let mut response = Response::new();
    response.apply_error(error)?;
    assert_eq!(response.error(), Some("Invalid count"));
    assert!(response.diagnostic().is_some());
    assert!(response.to_envelope().stack.is_none());
    let json = response.to_envelope().try_to_json()?;
    assert_eq!(
        json["errors"]["count"],
        serde_json::json!(["must be an integer"])
    );
    assert!(!json.to_string().contains("registered"));
    let error = response.into_error();
    assert!(std::error::Error::source(&error).is_some_and(|e| e.is::<std::num::ParseIntError>()));
    let mut response = Response::new();
    response.set_error(PublicError {
        code: 400,
        status: "Bad Request".into(),
        error: "Explicit".into(),
        errors: serde_json::json!({}),
    })?;
    response.set_stack(vec![SourceLocation::caller()])?;
    assert!(response.to_envelope().try_to_json()?["stack"].is_array());
    assert_eq!(response.into_error().message(), "Explicit");
    Ok(())
}

/// A named native writer observes an isolated snapshot and does not own the framework response.
async fn send(view: ResponseView) -> Result<()> {
    // Inspect a data-only snapshot after suspension; the callback cannot mutate the original output
    // owner.

    tokio::task::yield_now().await;
    assert_eq!(view.status.as_ref().map(|s| s.code), Some(200));
    assert!(view.envelope.results.is_some());
    Ok(())
}
/// Finalization prevents later mutation and repeated/failed writer attempts.
#[tokio::test]
async fn response_delivery_success_failure_stop_and_borrowed_writer() -> Result<()> {
    // Use separate responses for each native attempt outcome so failed/stopped states cannot mask
    // retries.

    let mut response = Response::new();
    response.results(&false)?;
    response.dispatch_with(send).await?;
    assert!(response.sent());
    assert_eq!(response.delivery_state(), DeliveryState::Sent);
    response
        .dispatch_with(|_| async { Err(Error::new("must not run twice")) })
        .await?;
    assert!(
        response.results(&true).is_err()
            && response.headers_mut().is_err()
            && response.session_mut().is_err()
            && response.data_mut().is_err()
            && response.errors_mut().is_err()
            && response.stop().is_err()
    );
    // Explicit stop suppresses writer entry and still freezes every output mutation accessor.
    let mut stopped = Response::new();
    stopped.stop()?;
    assert!(stopped.sent());
    assert_eq!(stopped.delivery_state(), DeliveryState::Stopped);
    stopped
        .dispatch_with(|_| async { Err(Error::new("suppressed")) })
        .await?;
    let mut failed = Response::new();
    assert!(
        failed
            .dispatch_with(|_| async { Err(Error::new("partly sent")) })
            .await
            .is_err()
    );
    assert_eq!(failed.delivery_state(), DeliveryState::Failed);
    assert!(failed.dispatch().await.is_err());
    let mut empty = Response::new();
    empty.dispatch().await?;
    assert!(empty.sent());
    // Borrow a caller-owned writer receipt so Send does not become an unnecessary static ownership
    // demand.
    let mut written = Vec::new();
    let writer = &mut written;
    let mut borrowed = Response::new();
    borrowed.html("native bytes")?;
    borrowed
        .dispatch_with(move |view| async move {
            if let Body::Text(text) = view.body {
                writer.extend_from_slice(text.as_bytes());
            }
            Ok(())
        })
        .await?;
    assert_eq!(written, b"native bytes");
    Ok(())
}

/// Cookie parsing preserves first values and raw malformed encodings; custom codecs are deliberate.
#[test]
fn cookies_parse_serialize_options_and_reject_injection() -> Result<()> {
    // Round-trip percent encoding and typed attributes while pinning first-wins parsing and
    // injection failures.

    let parsed = cookie::parse(
        " bad; foo = first ; foo=second; empty= ; encoded=%20value%20; raw=%E0%A4%A",
    )?;
    assert_eq!(parsed["foo"], "first");
    assert_eq!(parsed["empty"], "");
    assert_eq!(parsed["encoded"], " value ");
    assert_eq!(parsed["raw"], "%E0%A4%A");
    assert_eq!(
        cookie::parse_with("token=abc", |_| Some("decoded".into()))?["token"],
        "decoded"
    );
    // Select all relevant typed cookie attributes so output order and exact scope are visible.
    let options = CookieOptions {
        domain: Some(".example.com".into()),
        path: Some("/docs".into()),
        expires: Some(UNIX_EPOCH + Duration::from_secs(1_735_787_045)),
        max_age: Some(60),
        http_only: true,
        secure: true,
        partitioned: true,
        priority: Some(Priority::High),
        same_site: Some(SameSite::Strict),
    };
    let serialized = cookie::serialize("foo", "bar baz", &options)?;
    assert_eq!(
        serialized,
        concat!(
            "foo=bar%20baz; Max-Age=60; Domain=.example.com; Path=/docs; ",
            "Expires=Thu, 02 Jan 2025 03:04:05 GMT; HttpOnly; Secure; Partitioned; ",
            "Priority=High; SameSite=Strict"
        )
    );
    for priority in [Priority::Low, Priority::Medium, Priority::High] {
        assert!(
            cookie::serialize(
                "x",
                "y",
                &CookieOptions {
                    priority: Some(priority),
                    ..Default::default()
                }
            )?
            .contains("Priority=")
        );
    }
    for site in [SameSite::Lax, SameSite::None] {
        assert!(
            cookie::serialize(
                "x",
                "y",
                &CookieOptions {
                    same_site: Some(site),
                    ..Default::default()
                }
            )?
            .contains("SameSite=")
        );
    }
    assert_eq!(
        cookie::serialize_with("foo", "bar", &Default::default(), |v| v.to_uppercase())?,
        "foo=BAR"
    );
    assert!(cookie::serialize("bad name", "v", &Default::default()).is_err());
    assert!(cookie::serialize_with("x", "y", &Default::default(), |_| "bad;value".into()).is_err());
    assert!(
        cookie::serialize(
            "x",
            "y",
            &CookieOptions {
                domain: Some("bad_domain".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        cookie::serialize(
            "x",
            "y",
            &CookieOptions {
                path: Some("/bad\n".into()),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        cookie::serialize(
            "x",
            "y",
            &CookieOptions {
                expires: Some(UNIX_EPOCH - Duration::from_secs(1)),
                ..Default::default()
            }
        )
        .is_err()
    );
    assert!(
        cookie::serialize(
            "x",
            "y",
            &CookieOptions {
                expires: Some(UNIX_EPOCH + Duration::from_secs(253_402_300_800)),
                ..Default::default()
            }
        )
        .is_err()
    );
    Ok(())
}

/// Revision scopes survive delete/clear/repeated delete, while live options are removed.
#[test]
fn session_values_revisions_scope_and_explicit_list_codec() -> Result<()> {
    // Set a nondefault scope, update its value, then delete repeatedly to detect scope loss in
    // tombstones.

    let mut session = Session::from_pairs([("initial", "value")])?;
    assert!(session.revisions().is_empty());
    let options = CookieOptions {
        domain: Some("example.com".into()),
        path: Some("/preferences".into()),
        http_only: true,
        ..Default::default()
    };
    session
        .set("theme", "dark", Some(options.clone()))?
        .set("theme", "light", None)?;
    assert_eq!(session.options("theme"), Some(&options));
    assert!(matches!(
        session.revisions().get("theme"),
        Some(SessionRevision::Set { options: Some(v), .. })
            if v.path.as_deref() == Some("/preferences")
    ));
    assert!(session.remove("theme")?);
    assert!(!session.remove("theme")?);
    assert!(session.options("theme").is_none());
    let headers = session.cookie_headers()?;
    assert!(
        headers[0].contains("Max-Age=0")
            && headers[0].contains("Domain=example.com")
            && headers[0].contains("Path=/preferences")
    );
    session.set("roles", vec!["ADMIN".into(), "USER".into()], None)?;
    assert!(session.cookie_headers().is_err());
    let headers = session.cookie_headers_with(|value| match value {
        SessionValue::Text(value) => Ok(value.clone()),
        SessionValue::List(values) => Ok(serde_json::to_string(values)?),
    })?;
    assert!(headers.iter().any(|value| value.starts_with("roles=%5B")));
    // Session values/options/revisions own ordinary strings; editing a Clone cannot change the
    // original.
    let mut cloned = session.clone();
    cloned.set("initial", "changed", None)?;
    assert_eq!(
        session.get("initial").and_then(SessionValue::as_str),
        Some("value")
    );
    assert!(!format!("{session:?}").contains("ADMIN"));
    // Clearing live state must record removals for every key while retaining prior nondefault
    // scope.
    session.clear()?;
    assert!(session.is_empty() && session.entries().count() == 0);
    assert!(session.revisions().contains_key("initial"));
    assert!(!session.remove("missing")?);
    assert!(session.set("bad name", "x", None).is_err());
    Ok(())
}

/// Primary named capability returns a typed struct directly through Serialize.
async fn create(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Read typed operation data and return a real serializable struct through the capability
    // response.

    assert_eq!(inv.event(), "article-create");
    res.results(&Profile {
        id: 7,
        name: req.data.path_as::<String>("title")?.unwrap_or_default(),
    })?;
    Ok(Flow::Continue)
}
/// A named operation resolves a child without recursively entering general lifecycle hooks.
async fn publish(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Concurrent child resolves each own their request/response; only immutable ancestry is shared.

    let (first, second) = tokio::join!(
        inv.resolve("article-create", serde_json::json!({"title":"first"})),
        inv.resolve("article-create", serde_json::json!({"title":"second"})),
    );
    assert_eq!(first?.response.code, Some(200));
    res.apply_envelope(second?.response)?;
    Ok(Flow::Continue)
}
/// Plain data/owned requests/nested resolve retain the familiar capability-call shape.
#[tokio::test]
async fn resolve_plain_data_owned_request_and_isolated_children() -> Result<()> {
    // Register lifecycle sentinels so any accidental request/response/error redispatch fails
    // visibly.

    let mut builder = EmitterBuilder::new();
    builder.on_priority("article-create", 10, create)?;
    builder.on("publish", publish)?;
    async fn unexpected(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // This callback must be reached only by the explicit failure capability, never automatic
        // lifecycle behavior.

        Err(Error::new("general lifecycle must not run"))
    }
    builder.on("request", unexpected)?;
    builder.on("response", unexpected)?;
    builder.on("error", unexpected)?;
    async fn abort(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // A normal 200/false result can coexist with cooperative Abort; outcome is not a response
        // status.

        res.results(&false)?;
        Ok(Flow::Abort)
    }
    builder.on("abort", abort)?;
    builder.on("failure", unexpected)?;
    let emitter = builder.build();
    // Plain object input must become a request without entering any lifecycle sentinel.
    let resolution = emitter
        .resolve("article-create", serde_json::json!({"title":"First Post"}))
        .await?;
    assert_eq!(resolution.outcome, DispatchOutcome::Completed);
    assert_eq!(
        resolution.response.try_to_json()?["results"],
        serde_json::json!({"id":7,"name":"First Post"})
    );
    let req = Request::builder()
        .data(serde_json::json!({"title":"owned"}))
        .build()?;
    assert_eq!(
        emitter
            .resolve("article-create", req)
            .await?
            .response
            .try_to_json()?["results"]["name"],
        "owned"
    );
    assert_eq!(
        emitter
            .resolve("publish", Nest::new())
            .await?
            .response
            .try_to_json()?["results"]["name"],
        "second"
    );
    assert_eq!(
        emitter.resolve("missing", Nest::new()).await?.outcome,
        DispatchOutcome::NoListener
    );
    assert!(
        emitter
            .resolve("missing", Nest::new())
            .await?
            .response
            .code
            .is_none()
    );
    assert!(
        emitter
            .resolve("article-create", serde_json::json!([]))
            .await
            .is_err()
    );
    // Assert an aborted capability keeps its normal result/status rather than being converted to a
    // legacy abort code.
    let aborted = emitter.resolve("abort", Nest::new()).await?;
    assert_eq!(aborted.outcome, DispatchOutcome::Aborted);
    assert_eq!(aborted.response.code, Some(200));
    assert_eq!(
        aborted.response.results.as_ref().and_then(Value::as_bool),
        Some(false)
    );
    let error = emitter
        .resolve("failure", Nest::new())
        .await
        .expect_err("ordinary error is not an envelope");
    assert_eq!(error.frames()[0].event, "failure");
    assert_eq!(error.message(), "general lifecycle must not run");
    Ok(())
}
