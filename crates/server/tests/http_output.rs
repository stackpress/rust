//! Buffered wire conversion, fallback disclosure and independent delivery authority.
#![cfg(feature = "http")]

use http::Request as NativeRequest;
use serde_json::{Value as Json, json};
use stackpress_lib::{
    Body, CookieOptions, DeliveryState, Error, Flow, Invocation, MethodPattern, Nest, ParseLimits,
    Request, Response, Result, SessionValue, SharedNative, Value,
};
use stackpress_server::http::{HttpAdapter, HttpOptions, HttpOutput, HttpReply};
use stackpress_server::{LifecycleStage as Stage, Server, ServerBuilder};
use std::future::{Future, pending};
use std::pin::pin;
use std::task::{Context, Poll, Waker};

/// Produce a chosen response using the real guarded foundation API.
async fn output(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    match req.url.path() {
        "/text" => {
            res.set("text/custom; charset=utf-8", Body::Text("héllo".into()))?;
        }
        "/bytes" => {
            res.set("application/x-binary", Body::Bytes(vec![0, 255, 128]))?;
        }
        "/empty-text" => {
            res.set("text/plain", Body::Text(String::new()))?;
        }
        "/empty-bytes" => {
            res.set("application/octet-stream", Body::Bytes(Vec::new()))?;
        }
        "/explicit-json" => {
            res.json(&json!({"raw":true}))?;
        }
        "/null" => {
            res.results(&Json::Null)?;
        }
        "/false" => {
            res.results(&false)?;
        }
        "/zero" => {
            res.results(&0)?;
        }
        "/array" => {
            res.rows(&json!([false, 0, null]), 0)?;
        }
        "/status" => {
            res.status_code(202, None)?;
        }
        "/rich" => {
            res.results_value(Value::from(SharedNative::new("private native")))?;
        }
        "/transfer" => {
            res.results(&true)?;
            res.headers_mut()?.append("transfer-encoding", "chunked")?;
        }
        "/cookies" => {
            // Explicit repeated headers and scalar revisions must all survive conversion.
            res.headers_mut()?
                .append("set-cookie", "manual=1")?
                .append("set-cookie", "manual=2")?
                .append("x-many", "one")?
                .append("x-many", "two")?
                .append("content-length", "wrong")?
                .append("content-length", "999")?
                .set("content-type", "stale/type")?;
            let session = res.session_mut()?;
            session.set("first", "one", None)?;
            session.set("second", "two", None)?;
            session.set(
                "removed",
                "old",
                Some(CookieOptions {
                    domain: Some("example.test".into()),
                    path: Some("/scope".into()),
                    secure: true,
                    http_only: true,
                    ..Default::default()
                }),
            )?;
            session.remove("removed")?;
            res.results(&true)?;
        }
        "/cookie-list" => {
            res.session_mut()?.set(
                "preferences",
                SessionValue::List(vec!["a".into(), "b".into()]),
                None,
            )?;
            res.headers_mut()?.set("x-secret", "discard-me")?;
            res.results(&"must-not-leak")?;
        }
        "/header-injection" => {
            // Foundation validation prevents malicious headers from ever reaching the adapter.
            assert!(
                res.headers_mut()?
                    .append("x-test", "ok\r\nInjected: yes")
                    .is_err()
            );
            assert!(res.headers_mut()?.append("bad name", "value").is_err());
            assert!(res.headers().is_empty());
            res.results(&true)?;
        }
        "/header-budget" => {
            *res = Response::with_limits(ParseLimits {
                max_bytes: 32,
                ..Default::default()
            });
            res.headers_mut()?.append("x-big", &"x".repeat(33))?;
            res.status_code(200, None)?;
        }
        "/data-budget" => {
            *res = Response::with_limits(ParseLimits {
                max_bytes: 32,
                ..Default::default()
            });
            // Public store replacement must not evade the owning response's final limits.
            *res.data_mut()? = Nest::from_object(json!({"secret": "x".repeat(33)}))?;
            res.status_code(200, None)?;
        }
        "/tiny" => {
            // Even an empty object violates zero-node budgets, requiring the fixed fallback.
            *res = Response::with_limits(ParseLimits {
                max_entries: 0,
                ..Default::default()
            });
            res.status_code(200, None)?;
        }
        "/large" => {
            *res = Response::with_limits(ParseLimits {
                max_bytes: ParseLimits::default().max_bytes + 128,
                ..Default::default()
            });
            res.results(&"x".repeat(ParseLimits::default().max_bytes + 1))?;
        }
        "/stopped" => {
            res.stop()?;
        }
        "/sent" => {
            res.dispatch().await?;
        }
        "/failed" => {
            // A failed callback may already have emitted bytes elsewhere; never retry.
            assert!(
                res.dispatch_with(|_| async { Err(Error::new("partial external output")) })
                    .await
                    .is_err()
            );
        }
        "/attempted" => {
            // Drop a genuinely entered pending delivery, preserving Attempted honestly.
            let mut attempt = pin!(res.dispatch_with(|_| pending::<Result<()>>()));
            let mut context = Context::from_waker(Waker::noop());
            assert!(matches!(attempt.as_mut().poll(&mut context), Poll::Pending));
        }
        "/replace-after-sent" => {
            res.dispatch().await?;
            // No HTTP writer was given to the handler; replacing its value cannot duplicate it.
            *res = Response::new();
            res.results(&"replacement")?;
        }
        "/abort" => {
            return Ok(Flow::Abort);
        }
        "/private-error" => {
            res.results(&"partial secret")?;
            res.session_mut()?.set("password", "secret", None)?;
            res.headers_mut()?.set("x-secret", "secret")?;
            return Err(Error::new("private cause: token=secret"));
        }
        path => {
            // Status routes exercise body-forbidden and illegal completed-status boundaries.
            let code = path.strip_prefix("/code/").unwrap().parse().unwrap();
            res.set("text/plain", Body::Text("representation".into()))?;
            res.headers_mut()?.set("content-length", "999")?;
            res.status_code(code, None)?;
        }
    }
    Ok(Flow::Continue)
}

/// A GET-only handler must never be reached through implicit HEAD fallback.
async fn forbidden(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    panic!("unexpected lifecycle/GET callback")
}

/// Assemble a real all-method route, preserving the normal host lifecycle.
async fn application() -> Result<Server> {
    let mut builder = ServerBuilder::new();
    builder.route(MethodPattern::Any, "/*", output)?;
    builder.route(MethodPattern::Any, "/code/:code", output)?;
    builder.build().await.map_err(|error| error.error)
}

/// Invoke the adapter through an ordinary already-buffered native request.
async fn handle(app: &Server, method: &str, path: &str) -> Result<HttpReply> {
    let input = NativeRequest::builder()
        .method(method)
        .uri(path)
        .body(Vec::new())
        .unwrap();
    HttpAdapter::new(HttpOptions::default())?
        .handle(app, input)
        .await
}

/// Require an actual response rather than treating suppression as transport success.
fn native(reply: HttpReply) -> http::Response<Vec<u8>> {
    match reply.output {
        HttpOutput::Response(response) => response,
        other => panic!("expected native response, got {other:?}"),
    }
}

/// Text, byte buffers and explicit serialized JSON keep exact representation/MIME.
#[tokio::test]
async fn exact_text_and_binary_representations() -> Result<()> {
    let app = application().await?;
    for (path, body, mime) in [
        ("/text", "héllo".as_bytes(), "text/custom; charset=utf-8"),
        ("/bytes", &[0, 255, 128], "application/x-binary"),
        ("/empty-text", b"", "text/plain"),
        ("/empty-bytes", b"", "application/octet-stream"),
        (
            "/explicit-json",
            b"{\n  \"raw\": true\n}",
            "application/json",
        ),
    ] {
        let response = native(handle(&app, "GET", path).await?);
        assert_eq!(response.status(), 200);
        assert_eq!(response.body(), body);
        assert_eq!(response.headers()["content-type"], mime);
        assert_eq!(response.headers()["content-length"], body.len().to_string());
    }
    Ok(())
}

/// Presence-sensitive results and status-only output use one consistent public envelope.
#[tokio::test]
async fn public_envelopes_keep_scalar_presence_and_zero_total() -> Result<()> {
    let app = application().await?;
    for (path, expected) in [
        ("/null", Json::Null),
        ("/false", json!(false)),
        ("/zero", json!(0)),
        ("/array", json!([false, 0, null])),
    ] {
        let response = native(handle(&app, "GET", path).await?);
        let envelope: Json = serde_json::from_slice(response.body()).unwrap();
        assert_eq!(envelope["results"], expected);
        assert_eq!(envelope["total"], if path == "/array" { 0 } else { 1 });
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(
            response.headers()["content-length"],
            response.body().len().to_string()
        );
    }
    let response = native(handle(&app, "GET", "/status").await?);
    let envelope: Json = serde_json::from_slice(response.body()).unwrap();
    assert_eq!(response.status(), 202);
    assert_eq!(envelope["code"], 202);
    assert!(envelope.get("results").is_none());
    Ok(())
}

/// Repeated headers and scoped cookie tombstones survive; framing and MIME are authoritative.
#[tokio::test]
async fn cookies_and_multivalue_headers_are_never_folded() -> Result<()> {
    let app = application().await?;
    let response = native(handle(&app, "GET", "/cookies").await?);
    let cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(cookies.len(), 5);
    assert_eq!(
        &cookies[..4],
        ["manual=1", "manual=2", "first=one", "second=two"]
    );
    assert!(cookies[4].starts_with("removed=;"));
    for scope in [
        "Domain=example.test",
        "Path=/scope",
        "Max-Age=0",
        "1970",
        "Secure",
        "HttpOnly",
    ] {
        assert!(
            cookies[4].contains(scope),
            "missing {scope}: {}",
            cookies[4]
        );
    }
    assert_eq!(response.headers().get_all("x-many").iter().count(), 2);
    assert_eq!(
        response.headers().get_all("content-length").iter().count(),
        1
    );
    assert_eq!(
        response.headers()["content-length"],
        response.body().len().to_string()
    );
    assert_eq!(response.headers()["content-type"], "application/json");
    assert_eq!(
        native(handle(&app, "GET", "/header-injection").await?).status(),
        200
    );
    Ok(())
}

/// HEAD selects HEAD/Any only and suppresses bytes while retaining representation length/MIME.
#[tokio::test]
async fn head_never_falls_back_to_get() -> Result<()> {
    let app = application().await?;
    let get = native(handle(&app, "GET", "/text").await?);
    let head = native(handle(&app, "HEAD", "/text").await?);
    assert!(head.body().is_empty());
    assert_eq!(head.status(), get.status());
    assert_eq!(head.headers(), get.headers());
    let mut builder = ServerBuilder::new();
    builder.get("/get-only", forbidden)?;
    builder.head("/head-only", output)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    assert_eq!(
        native(handle(&app, "HEAD", "/get-only").await?).status(),
        404
    );
    // An explicit HEAD route also runs without a GET contribution.
    let mut builder = ServerBuilder::new();
    builder.head("/text", output)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    assert_eq!(
        native(handle(&app, "HEAD", "/text").await?).headers()["content-length"],
        "6"
    );
    Ok(())
}

/// Forbidden-body statuses remove inconsistent lengths and never serialize their body.
#[tokio::test]
async fn body_forbidden_statuses_and_completed_status_range() -> Result<()> {
    let app = application().await?;
    for method in ["GET", "HEAD"] {
        for code in [204, 205, 304] {
            let response = native(handle(&app, method, &format!("/code/{code}")).await?);
            assert_eq!(response.status().as_u16(), code);
            assert!(response.body().is_empty());
            assert_eq!(
                response
                    .headers()
                    .get("content-length")
                    .map(|v| v.to_str().unwrap()),
                if code == 205 { Some("0") } else { None }
            );
        }
    }
    for code in [200, 599] {
        assert_eq!(
            native(handle(&app, "GET", &format!("/code/{code}")).await?)
                .status()
                .as_u16(),
            code
        );
    }
    for code in [0, 100, 199, 600, 999] {
        let reply = handle(&app, "GET", &format!("/code/{code}")).await?;
        assert_eq!(reply.report.failures[0].stage, Stage::Output);
        assert_eq!(native(reply).status(), 500);
    }
    Ok(())
}

/// Every fallible conversion occurs before the one publication, without lifecycle retries.
#[tokio::test]
async fn conversion_failure_is_one_sanitized_empty_500() -> Result<()> {
    let mut builder = ServerBuilder::new();
    builder.route(MethodPattern::Any, "/*", output)?;
    // Output failures cannot re-enter an error event after the lifecycle has finished.
    builder.on("error", forbidden)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    for path in [
        "/rich",
        "/transfer",
        "/cookie-list",
        "/tiny",
        "/data-budget",
        "/header-budget",
    ] {
        let reply = handle(&app, "GET", path).await?;
        assert!(reply.report.terminal_failure);
        assert_eq!(reply.report.failures.len(), 1);
        assert_eq!(reply.report.failures[0].stage, Stage::Output);
        assert!(reply.report.error_offers.is_empty());
        let output_stages = reply
            .report
            .stages
            .iter()
            .filter(|s| s.stage == Stage::Output)
            .collect::<Vec<_>>();
        // Observe conversion failure followed by the sole successful fallback publication.
        assert_eq!(output_stages.len(), 2);
        assert_eq!(output_stages[1].delivery_state, DeliveryState::Sent);
        if path == "/cookie-list" {
            let message = reply.report.failures[0].error.message();
            assert!(message.contains("preferences"));
            assert!(message.contains("session list value requires an explicit codec"));
        }
        let response = native(reply);
        assert_eq!(response.status(), 500, "{path}");
        assert!(response.body().is_empty());
        assert_eq!(response.headers().len(), 1);
        assert_eq!(response.headers()["content-length"], "0");
    }
    Ok(())
}

/// Conversion must use the owning response's enlarged budget, rather than detached defaults.
#[tokio::test]
async fn structured_conversion_retains_response_limits() -> Result<()> {
    let app = application().await?;
    let reply = handle(&app, "GET", "/large").await?;
    assert!(reply.report.failures.is_empty());
    let response = native(reply);
    assert_eq!(response.status(), 200);
    let payload: Json = serde_json::from_slice(response.body()).unwrap();
    assert_eq!(
        payload["results"].as_str().unwrap().len(),
        ParseLimits::default().max_bytes + 1
    );
    Ok(())
}

/// Suppression and foreign finalization return honest variants without manufacturing output.
#[tokio::test]
async fn finalized_states_are_not_retried() -> Result<()> {
    let app = application().await?;
    assert!(matches!(
        handle(&app, "GET", "/stopped").await?.output,
        HttpOutput::Suppressed
    ));
    for (path, expected) in [
        ("/sent", DeliveryState::Sent),
        ("/failed", DeliveryState::Failed),
        ("/attempted", DeliveryState::Attempted),
    ] {
        let reply = handle(&app, "GET", path).await?;
        assert!(reply.report.terminal_failure);
        assert!(
            reply
                .report
                .failures
                .iter()
                .all(|f| f.stage == Stage::Output)
        );
        assert!(matches!(reply.output, HttpOutput::Unavailable { state } if state == expected));
    }
    let replacement = handle(&app, "GET", "/replace-after-sent").await?;
    assert!(replacement.report.failures.is_empty());
    let response = native(replacement);
    assert_eq!(
        serde_json::from_slice::<Json>(response.body()).unwrap()["results"],
        "replacement"
    );
    let aborted = native(handle(&app, "GET", "/abort").await?);
    assert_eq!(aborted.status(), 204);
    assert!(aborted.body().is_empty());
    Ok(())
}

/// Developer causes and discarded partial response/session state never enter public output.
#[tokio::test]
async fn application_errors_keep_private_diagnostics_off_wire() -> Result<()> {
    let app = application().await?;
    let reply = handle(&app, "GET", "/private-error").await?;
    assert!(
        reply.report.failures[0]
            .error
            .message()
            .contains("token=secret")
    );
    assert!(!reply.report.to_string().contains("token=secret"));
    let response = native(reply);
    assert_eq!(response.status(), 500);
    assert!(response.headers().get("set-cookie").is_none());
    assert!(response.headers().get("x-secret").is_none());
    let wire = String::from_utf8_lossy(response.body());
    for secret in [
        "private cause",
        "token=secret",
        "partial secret",
        "password",
        "stack",
    ] {
        assert!(!wire.contains(secret));
    }
    Ok(())
}

/// HTTP's complete future remains Send and reusable for independent concurrent inputs.
#[tokio::test]
async fn handling_futures_are_send_and_independent() -> Result<()> {
    let app = application().await?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    let first = NativeRequest::builder()
        .uri("/text")
        .body(Vec::new())
        .unwrap();
    let second = NativeRequest::builder()
        .uri("/bytes")
        .body(Vec::new())
        .unwrap();
    // The helper's bound is a compiler proof, while join proves no shared native state exists.
    let (one, two) = tokio::join!(
        send(adapter.handle(&app, first)),
        send(adapter.handle(&app, second))
    );
    assert_ne!(
        one.as_ref().unwrap().report.correlation_id,
        two.as_ref().unwrap().report.correlation_id
    );
    assert_eq!(native(one?).body(), "héllo".as_bytes());
    assert_eq!(native(two?).body(), &[0, 255, 128]);
    Ok(())
}

/// Require Send statically without moving the borrowed server/adapter into background tasks.
fn send<T: Future + Send>(future: T) -> T {
    future
}
