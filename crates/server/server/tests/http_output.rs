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
    // Request paths choose fixtures only; the adapter still selects each real METHOD/path event.
    match req.url.path() {
        "/text" => {
            // Multibyte text detects character-count framing instead of byte-count framing.
            res.set("text/custom; charset=utf-8", Body::Text("héllo".into()))?;
        }
        "/bytes" => {
            // Invalid UTF-8 must remain a raw buffer throughout native response conversion.
            res.set("application/x-binary", Body::Bytes(vec![0, 255, 128]))?;
        }
        "/empty-text" => {
            // Present empty text is successful output, not the host's missing-result condition.
            res.set("text/plain", Body::Text(String::new()))?;
        }
        "/empty-bytes" => {
            // Empty binary representation retains its explicit MIME and status in the same way.
            res.set("application/octet-stream", Body::Bytes(Vec::new()))?;
        }
        "/explicit-json" => {
            // The lib helper already serialized this body; HTTP must preserve its exact bytes.
            res.json(&json!({"raw":true}))?;
        }
        "/null" => {
            // Explicit null remains a present envelope result instead of disappearing as absence.
            res.results(&Json::Null)?;
        }
        "/false" => {
            // False cannot be mistaken for an empty or unsuccessful operation result.
            res.results(&false)?;
        }
        "/zero" => {
            // Numeric zero exercises the other common truthiness-based omission mistake.
            res.results(&0)?;
        }
        "/array" => {
            // Rows supplies an explicit zero total independently of its nonempty array result.
            res.rows(&json!([false, 0, null]), 0)?;
        }
        "/status" => {
            // A deliberate status-only completion must serialize an envelope without fake results.
            res.status_code(202, None)?;
        }
        "/rich" => {
            // Shared native state is legal in-process data but unsupported by JSON wire output.
            res.results_value(Value::from(SharedNative::new("private native")))?;
        }
        "/transfer" => {
            // A syntactically valid transfer field still requires transport ownership we lack.
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
            // Independent scalar revisions append after the manually supplied Set-Cookie fields.
            let session = res.session_mut()?;
            session.set("first", "one", None)?;
            session.set("second", "two", None)?;
            // Removing a key after setting scoped metadata must retain its original deletion scope.
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
            // List revisions need an explicit codec; guessing a string would change cookie meaning.
            res.session_mut()?.set(
                "preferences",
                SessionValue::List(vec!["a".into(), "b".into()]),
                None,
            )?;
            // These partial values must be discarded by the sanitized conversion fallback.
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
            // Headers accepts its own defaults, so final conversion must recheck Response policy.
            *res = Response::with_limits(ParseLimits {
                max_bytes: 32,
                ..Default::default()
            });
            res.headers_mut()?.append("x-big", &"x".repeat(33))?;
            res.status_code(200, None)?;
        }
        "/data-budget" => {
            // A publicly replaced store can carry looser limits than its enclosing Response.
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
            // One byte above detached defaults is valid under this owning Response's larger policy.
            *res = Response::with_limits(ParseLimits {
                max_bytes: ParseLimits::default().max_bytes + 128,
                ..Default::default()
            });
            res.results(&"x".repeat(ParseLimits::default().max_bytes + 1))?;
        }
        "/pending" => {
            // Cancellation drops this partial call state; the shared adapter owns no mutable slot.
            res.headers_mut()?
                .set("x-pending-secret", "discard-on-drop")?;
            return pending::<Result<Flow>>().await;
        }
        "/stopped" => {
            // Stop consumes framework output deliberately; HTTP should return Suppressed.
            res.stop()?;
        }
        "/sent" => {
            // Foundation finalization supplies no native HTTP buffer, so availability stays
            // explicit.
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
            // One poll enters the writer without blocking on its intentionally permanent pending
            // state.
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
            // Cooperative abort stops later hooks while leaving Open output eligible for
            // normalization.
            return Ok(Flow::Abort);
        }
        "/private-error" => {
            // A later error must replace all partial success/body/session/header state before
            // output.
            res.results(&"partial secret")?;
            res.session_mut()?.set("password", "secret", None)?;
            res.headers_mut()?.set("x-secret", "secret")?;
            // The retained cause is diagnostic only; public errors must not copy its secret text.
            return Err(Error::new("private cause: token=secret"));
        }
        path => {
            // Status routes exercise body-forbidden and illegal completed-status boundaries.
            let code = path.strip_prefix("/code/").unwrap().parse().unwrap();
            // Set establishes a real representation; the later code setter selects the status case.
            res.set("text/plain", Body::Text("representation".into()))?;
            res.headers_mut()?.set("content-length", "999")?;
            res.status_code(code, None)?;
        }
    }
    Ok(Flow::Continue)
}

/// A GET-only handler must never be reached through implicit HEAD fallback.
async fn forbidden(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A panic proves the forbidden callback never runs, rather than relying on a private counter.
    panic!("unexpected lifecycle/GET callback")
}

/// Assemble a real all-method route, preserving the normal host lifecycle.
async fn application() -> Result<Server> {
    // The wildcard handles one segment; status fixtures deliberately use a separate nested route.
    let mut builder = ServerBuilder::new();
    builder.route(MethodPattern::Any, "/*", output)?;
    builder.route(MethodPattern::Any, "/code/:code", output)?;
    builder.build().await.map_err(|error| error.error)
}

/// Invoke the adapter through an ordinary already-buffered native request.
async fn handle(app: &Server, method: &str, path: &str) -> Result<HttpReply> {
    // Native request values supply method/path authority without opening a connection or listener.
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
    // Tests expecting bytes must fail if the adapter silently suppresses or loses native output.
    match reply.output {
        HttpOutput::Response(response) => response,
        other => panic!("expected native response, got {other:?}"),
    }
}

/// Text, byte buffers and explicit serialized JSON keep exact representation/MIME.
#[tokio::test]
async fn exact_text_and_binary_representations() -> Result<()> {
    // Reuse one application across explicit body kinds, including present empty representations.
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
        // Invoke the full adapter and compare the selected representation without re-encoding it.
        let response = native(handle(&app, "GET", path).await?);
        // Status/MIME and byte length must describe exactly the returned payload, including UTF-8.
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
    // Use scalar/array values whose presence would be lost by JavaScript-style truthiness tests.
    let app = application().await?;
    for (path, expected) in [
        ("/null", Json::Null),
        ("/false", json!(false)),
        ("/zero", json!(0)),
        ("/array", json!([false, 0, null])),
    ] {
        // Decode native bytes so this test exercises actual envelope serialization and MIME policy.
        let response = native(handle(&app, "GET", path).await?);
        let envelope: Json = serde_json::from_slice(response.body()).unwrap();
        // Results defaults total to one; only the rows fixture deliberately sets zero.
        assert_eq!(envelope["results"], expected);
        assert_eq!(envelope["total"], if path == "/array" { 0 } else { 1 });
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(
            response.headers()["content-length"],
            response.body().len().to_string()
        );
    }
    // Status-only output is valid but has no fabricated result field.
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
    // The fixture combines explicit header repeats, scalar cookie revisions and a scoped tombstone.
    let app = application().await?;
    let response = native(handle(&app, "GET", "/cookies").await?);
    // Inspect separate native values; a combined string could hide cookie replacement/folding.
    let cookies = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .collect::<Vec<_>>();
    // Both manual cookies precede the two scalar revisions and the final scoped removal.
    assert_eq!(cookies.len(), 5);
    assert_eq!(
        &cookies[..4],
        ["manual=1", "manual=2", "first=one", "second=two"]
    );
    assert!(cookies[4].starts_with("removed=;"));
    // A tombstone must preserve domain/path/security and override lifetime to immediate expiry.
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
    // Ordinary repeated values survive, while stale Content-Length/MIME are adapter-owned.
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
    // Invalid field syntax is rejected atomically by the foundation before conversion is reached.
    assert_eq!(
        native(handle(&app, "GET", "/header-injection").await?).status(),
        200
    );
    Ok(())
}

/// HEAD selects HEAD/Any only and suppresses bytes while retaining representation length/MIME.
#[tokio::test]
async fn head_never_falls_back_to_get() -> Result<()> {
    // An Any route supplies the same representation to each method without implicit dispatch
    // fallback.
    let app = application().await?;
    let get = native(handle(&app, "GET", "/text").await?);
    let head = native(handle(&app, "HEAD", "/text").await?);
    // HEAD suppresses only wire bytes and keeps representation status/MIME/byte length.
    assert!(head.body().is_empty());
    assert_eq!(head.status(), get.status());
    assert_eq!(head.headers(), get.headers());
    // A GET-only route panics if reached, making a silent HEAD-to-GET fallback visible.
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
    // Cross status body rules with HEAD so neither branch accidentally restores framing or bytes.
    let app = application().await?;
    for method in ["GET", "HEAD"] {
        for code in [204, 205, 304] {
            let response = native(handle(&app, method, &format!("/code/{code}")).await?);
            // 205 uses a zero length; 204/304 remove the fixture's stale length entirely.
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
    // The completed-status range includes both endpoints without clamping or default substitution.
    for code in [200, 599] {
        assert_eq!(
            native(handle(&app, "GET", &format!("/code/{code}")).await?)
                .status()
                .as_u16(),
            code
        );
    }
    // Interim and application-only codes fail at Output after the normal lifecycle has completed.
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
    // Register normal fixture routes beside a panic error hook, proving output never re-enters
    // hooks.
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
        // Each independent conversion failure must choose its fallback before any guarded dispatch.
        let reply = handle(&app, "GET", path).await?;
        // Preserve one original Output error while allowing a single sanitized native response.
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
            // Cookie identity and codec reason stay available to developers, never the empty wire
            // body.
            let message = reply.report.failures[0].error.message();
            assert!(message.contains("preferences"));
            assert!(message.contains("session list value requires an explicit codec"));
        }
        // No body, cookie or secret header from the partial handler response may survive fallback.
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
    // The fixture exceeds default string budgets by one byte under an explicit larger response
    // policy.
    let app = application().await?;
    let reply = handle(&app, "GET", "/large").await?;
    assert!(reply.report.failures.is_empty());
    let response = native(reply);
    assert_eq!(response.status(), 200);
    // Decode real native output; using a detached envelope's defaults would have yielded fallback
    // 500.
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
    // Suppression is intentional and distinct from prior/incomplete/failed foreign delivery.
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
        // These states can represent partial external output, so manufacturing another reply is
        // unsafe.
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
    // Whole-value replacement can reopen framework state but never gave the handler an HTTP writer.
    let replacement = handle(&app, "GET", "/replace-after-sent").await?;
    assert!(replacement.report.failures.is_empty());
    let response = native(replacement);
    assert_eq!(
        serde_json::from_slice::<Json>(response.body()).unwrap()["results"],
        "replacement"
    );
    // Abort alone still permits one empty normalized native completion instead of Suppressed.
    let aborted = native(handle(&app, "GET", "/abort").await?);
    assert_eq!(aborted.status(), 204);
    assert!(aborted.body().is_empty());
    Ok(())
}

/// Developer causes and discarded partial response/session state never enter public output.
#[tokio::test]
async fn application_errors_keep_private_diagnostics_off_wire() -> Result<()> {
    // The fixture fails after writing secret body/header/session values into a partial response.
    let app = application().await?;
    let reply = handle(&app, "GET", "/private-error").await?;
    // The original cause is retained explicitly; default formatting must keep its message private.
    assert!(
        reply.report.failures[0]
            .error
            .message()
            .contains("token=secret")
    );
    assert!(!reply.report.to_string().contains("token=secret"));
    // Native error output is fresh and sanitized, including absence of partial cookies/headers.
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
    // Two requests borrow the same immutable host/adapter while owning separate buffers/reports.
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
    // Independent correlations and payloads rule out a shared mutable native result slot.
    assert_ne!(
        one.as_ref().unwrap().report.correlation_id,
        two.as_ref().unwrap().report.correlation_id
    );
    assert_eq!(native(one?).body(), "héllo".as_bytes());
    assert_eq!(native(two?).body(), &[0, 255, 128]);
    Ok(())
}

/// Dropping a pending HTTP lifecycle cannot poison or reuse the next call's output authority.
#[tokio::test]
async fn cancelled_http_call_leaves_adapter_and_host_usable() -> Result<()> {
    // Reuse the same immutable adapter/host after cancelling a handler with partial response state.
    let app = application().await?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    {
        let input = NativeRequest::builder()
            .uri("/pending")
            .body(Vec::new())
            .unwrap();
        let mut call = pin!(adapter.handle(&app, input));
        let mut context = Context::from_waker(Waker::noop());
        // One poll enters the pending operation; leaving this block drops the complete owned call.
        assert!(matches!(call.as_mut().poll(&mut context), Poll::Pending));
    }
    // A new request must allocate a new report and successfully publish its own response once.
    let input = NativeRequest::builder()
        .uri("/text")
        .body(Vec::new())
        .unwrap();
    let reply = adapter.handle(&app, input).await?;
    assert_eq!(reply.report.correlation_id.as_str(), "2");
    assert!(reply.report.failures.is_empty());
    let response = native(reply);
    // Neither the pending handler's secret header nor its unfinished delivery state survives.
    assert_eq!(response.body(), "héllo".as_bytes());
    assert!(response.headers().get("x-pending-secret").is_none());
    Ok(())
}

/// HEAD suppresses both pre-normalization error envelopes and conversion-failure fallback bytes.
#[tokio::test]
async fn head_failure_paths_preserve_output_suppression() -> Result<()> {
    // Malformed target failure happens before any normalized request can reach lifecycle callbacks.
    let app = application().await?;
    let reply = handle(&app, "HEAD", "*").await?;
    assert_eq!(reply.report.failures[0].stage, Stage::Input);
    assert!(reply.report.error_offers.is_empty());
    let response = native(reply);
    // The error representation is described by its headers while HEAD exposes zero payload bytes.
    assert_eq!(response.status(), 400);
    assert!(response.body().is_empty());
    assert!(
        response.headers()["content-length"]
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            > 0
    );
    // A later rich-value conversion failure chooses the fixed empty fallback before publication.
    let reply = handle(&app, "HEAD", "/rich").await?;
    assert_eq!(reply.report.failures[0].stage, Stage::Output);
    assert!(reply.report.terminal_failure);
    let response = native(reply);
    assert_eq!(response.status(), 500);
    assert!(response.body().is_empty());
    assert_eq!(response.headers()["content-length"], "0");
    Ok(())
}

/// Keep unsupported rich values in process while deliberately choosing a no-payload status.
async fn rich_without_payload(
    req: &mut Request,
    res: &mut Response,
    _: Invocation<'_>,
) -> Result<Flow> {
    // The test registers only numeric-code paths, so parsing the fixture suffix is an invariant.
    let code = req.url.path().rsplit('/').next().unwrap().parse().unwrap();
    res.results_value(Value::from(SharedNative::new("private native body")))?;
    res.status_code(code, None)?;
    // Stale application framing must be removed even when representation conversion is skipped.
    res.headers_mut()?.set("content-length", "999")?;
    Ok(Flow::Continue)
}

/// Body-forbidden status rules skip unsupported representation serialization entirely.
#[tokio::test]
async fn forbidden_body_statuses_skip_rich_value_conversion() -> Result<()> {
    // A rich value would fail JSON conversion if a refactor serialized before checking status.
    let mut builder = ServerBuilder::new();
    builder.route(MethodPattern::Any, "/no-body/:code", rich_without_payload)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    for method in ["GET", "HEAD"] {
        for code in [204, 205, 304] {
            let reply = handle(&app, method, &format!("/no-body/{code}")).await?;
            // No conversion failure or fallback is appropriate when no payload can be sent.
            assert!(reply.report.failures.is_empty());
            let response = native(reply);
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
    Ok(())
}

/// Require Send statically without moving the borrowed server/adapter into background tasks.
fn send<T: Future + Send>(future: T) -> T {
    // Returning the same future preserves borrowing while enforcing Send at the call site.
    future
}
