//! In-process HTTP normalization, strict loading and public capability parity.
#![cfg(feature = "http")]

use http::{HeaderValue, Request as NativeRequest};
use serde_json::{Value as Json, json};
use stackpress_lib::{
    Body, Flow, Invocation, LoadState, MethodPattern, Nest, ParseLimits, Request, Response, Result,
};
use stackpress_server::http::{HttpAdapter, HttpOptions, HttpOutput, HttpReply};
use stackpress_server::{LifecycleStage as Stage, Server, ServerBuilder, ServerInvocationExt};

/// Return request construction/loading observations through an ordinary structured result.
async fn inspect(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Payload snapshots demonstrate independent source values and raw body preservation.
    let body = match req.body() {
        Body::Bytes(bytes) => bytes.clone(),
        _ => panic!("HTTP loader must preserve a byte buffer"),
    };
    // Serialize observations only, so separate query/post/data stores can be compared on the wire.
    res.results(&json!({
        "url": req.url.as_str(), "method": req.method.as_str(), "mime": req.mimetype(),
        "data": req.data.value().to_json()?, "post": req.post.value().to_json()?,
        "query": req.query.value().to_json()?, "body": body,
        "headers": req.headers.get_all("x-many"),
        "first": req.session.get("first").and_then(|v| v.as_str()),
        "second": req.session.get("second").and_then(|v| v.as_str()),
        "loaded": req.loaded(),
    }))?;
    Ok(Flow::Continue)
}

/// Fail immediately if a pre-normalization error reaches any lifecycle callback.
async fn forbidden(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A panic makes an otherwise invisible accidental callback fail the owning integration test.
    panic!("malformed input must not enter callbacks")
}

/// Recover a supported-body parse failure while observing the real normalized request.
async fn recover_input(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Failed parsing cannot partially install either post fields or the incoming body.
    assert_eq!(inv.failure()?.unwrap().stage, Stage::Input);
    assert_eq!(req.load_state(), LoadState::Failed);
    assert!(matches!(req.body(), Body::Empty));
    assert!(req.post.is_empty());
    assert_eq!(req.url.path(), "/parse");
    assert_eq!(req.method.as_str(), "POST");
    assert_eq!(req.headers.get("x-id"), Some("normalized"));
    // Aggregate state must still contain only the successful query from normalization.
    assert_eq!(req.data.path_as::<u64>("kept")?, Some(7));
    assert_eq!(req.data.value().to_json()?, json!({"kept": 7}));
    // Deliberate replacement clears public error state; status-only changes would not recover.
    *res = Response::new();
    res.results(&"recovered")?;
    Ok(Flow::Continue)
}

/// Extract a native response, making suppressed/unavailable output a test failure.
fn native(reply: HttpReply) -> http::Response<Vec<u8>> {
    // Availability variants are observable outcomes, so accepting either would hide output bugs.
    match reply.output {
        HttpOutput::Response(response) => response,
        other => panic!("expected native response, got {other:?}"),
    }
}

/// Decode the public envelope without consulting developer diagnostics.
fn envelope(reply: HttpReply) -> Json {
    // Decode only native public bytes; using framework diagnostics could accidentally mask leaks.
    serde_json::from_slice(native(reply).body()).unwrap()
}

/// Build the common request-inspection application without resources or global state.
async fn inspector(method: MethodPattern, path: &str) -> Result<Server> {
    // Each test receives its own immutable application and local correlation sequence.
    let mut app = ServerBuilder::new();
    app.route(method, path, inspect)?;
    app.build().await.map_err(|error| error.error)
}

/// Reject configuration with ambiguous base authority before accepting any input.
#[test]
fn trusted_base_validation() {
    // These values cannot safely supply the origin for a path-only request.
    for base in [
        "/relative",
        "ftp://host/",
        "http://user:pass@host/",
        "http://host/path",
        "http://host/?query=1",
        "http://host/#fragment",
        "http://[bad/",
    ] {
        // Invalid trusted options fail construction, before any request or lifecycle can exist.
        assert!(
            HttpAdapter::new(HttpOptions {
                base_url: base.into(),
                ..Default::default()
            })
            .is_err(),
            "{base}"
        );
    }
    // Retain the positive default case so blanket configuration rejection cannot pass the table.
    assert!(HttpAdapter::new(HttpOptions::default()).is_ok());
}

/// Preserve pathname bytes/case and trusted authority independently of query/header data.
#[tokio::test]
async fn exact_path_and_trusted_authority() -> Result<()> {
    // Use a non-default TLS origin/port so blind Host or forwarded trust changes visible output.
    let adapter = HttpAdapter::new(HttpOptions {
        base_url: "https://trusted.test:8443/".into(),
        ..Default::default()
    })?;
    // Distinct literal registrations detect collapsing slashes or decoding percent escapes.
    for path in ["/A%2Fb", "//A///B/", "/A%3Fb%23c", "/Case", "/a%2fb"] {
        let app = inspector(MethodPattern::GET, path).await?;
        // The query/header event suggestions remain untrusted data beside an exact literal route.
        let input = NativeRequest::builder()
            .uri(format!("{path}?event=secret&n=2"))
            .header("host", "attacker.test")
            .header("x-forwarded-proto", "ftp, https")
            .body(Vec::new())
            .unwrap();
        // Run complete normalization, loading, dispatch and output through the public adapter.
        let reply = adapter.handle(&app, input).await?;
        // Route metadata preserves native spelling and never copies query secrets into diagnostics.
        assert_eq!(reply.report.event, format!("GET {path}"));
        assert!(!reply.report.to_string().contains("event=secret"));
        // The informational URL must agree with matching while query values are parsed separately.
        let result = envelope(reply)["results"].clone();
        assert_eq!(
            result["url"],
            format!("https://trusted.test:8443{path}?event=secret&n=2")
        );
        assert_eq!(result["query"]["n"], 2);
    }
    // Absolute HTTP(S) targets retain their checked authority instead of the configured base.
    let app = inspector(MethodPattern::GET, "/absolute/").await?;
    let input = NativeRequest::builder()
        .uri("http://absolute.test:8080/absolute/")
        .body(Vec::new())
        .unwrap();
    assert_eq!(
        envelope(adapter.handle(&app, input).await?)["results"]["url"],
        "http://absolute.test:8080/absolute/"
    );
    Ok(())
}

/// Refuse unsafe normalization/non-path targets without synthesizing callback input.
#[tokio::test]
async fn malformed_targets_have_only_input_and_output() -> Result<()> {
    // Every hook panics: the test proves malformed input cannot acquire a fabricated Request.
    let mut builder = ServerBuilder::new();
    for event in ["request", "error", "response"] {
        builder.on(event, forbidden)?;
    }
    let app = builder.build().await.map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    for target in [
        "example.test:443",
        "*",
        "/a/../b",
        "/a/%2e%2e/b",
        "/a\\b",
        "ftp://host/path",
        "http://user:pass@host/path",
    ] {
        // Each target is a native URI value but violates this buffered adapter's bounded contract.
        let input = NativeRequest::builder()
            .uri(target)
            .body(Vec::new())
            .unwrap();
        let reply = adapter.handle(&app, input).await?;
        // Neither recovery hooks nor operation dispatch may appear in this failure's history.
        assert_eq!(
            reply
                .report
                .stages
                .iter()
                .map(|s| s.stage)
                .collect::<Vec<_>>(),
            [Stage::Input, Stage::Output],
            "{target}"
        );
        assert!(reply.report.error_offers.is_empty());
        assert_eq!(reply.report.failures[0].stage, Stage::Input);
        // Public failure remains a native 400 without credential text from the rejected target.
        let response = native(reply);
        assert_eq!(response.status(), 400, "{target}");
        assert!(!String::from_utf8_lossy(response.body()).contains("user:pass"));
    }
    Ok(())
}

/// Checked extension verbs normalize to uppercase; hidden operation selectors have no authority.
#[tokio::test]
async fn method_and_event_selection() -> Result<()> {
    // Lowercase extension methods use foundation token normalization before method/path matching.
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    let app = inspector(MethodPattern::exact("CUSTOM")?, "/inspect").await?;
    let input = NativeRequest::builder()
        .method("custom")
        .uri("/inspect")
        .body(Vec::new())
        .unwrap();
    assert_eq!(
        envelope(adapter.handle(&app, input).await?)["results"]["method"],
        "CUSTOM"
    );
    // A private named capability must remain unreachable without a deliberately registered route.
    let mut builder = ServerBuilder::new();
    builder.on("profile-search", forbidden)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    // Try all three untrusted selector locations in one request: query, header and structured body.
    let input = NativeRequest::builder()
        .method("POST")
        .uri("/emit?event=profile-search")
        .header("x-event", "profile-search")
        .header("content-type", "application/json")
        .body(br#"{"event":"profile-search"}"#.to_vec())
        .unwrap();
    // A missing METHOD/path route stays missing even though the suggested capability exists.
    assert_eq!(native(adapter.handle(&app, input).await?).status(), 404);
    Ok(())
}

/// Repeated header/cookie lines remain separate and raw bodies never invent POST fields.
#[tokio::test]
async fn repeated_headers_cookies_and_raw_mime() -> Result<()> {
    // Reuse one immutable adapter across raw MIME choices; no call may retain another's state.
    let app = inspector(MethodPattern::POST, "/inspect").await?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    for mime in [
        None,
        Some("text/plain"),
        Some("application/octet-stream"),
        Some("application/x-custom"),
    ] {
        // Duplicate Cookie lines test both multi-value preservation and foundation first-value
        // wins.
        let mut input = NativeRequest::builder()
            .method("POST")
            .uri("/inspect")
            .header("x-many", "one")
            .header("x-many", "two")
            .header("cookie", "first=one%20two")
            .header("cookie", "second=three; first=ignored");
        if let Some(mime) = mime {
            // Leaving the header absent is a separate case from an explicit text/binary/custom
            // type.
            input = input.header("content-type", mime);
        }
        // Invalid UTF-8/form-looking bytes must remain opaque when MIME does not select a parser.
        let input = input.body(vec![0, 255, 37, 61]).unwrap();
        let result = envelope(adapter.handle(&app, input).await?)["results"].clone();
        // Header and cookie order survive conversion without comma folding or double decoding.
        assert_eq!(result["headers"], json!(["one", "two"]));
        assert_eq!(result["first"], "one two");
        assert_eq!(result["second"], "three");
        // Raw loading finalizes the body but invents no POST fields from arbitrary bytes.
        assert_eq!(result["post"], json!({}));
        assert_eq!(result["body"], json!([0, 255, 37, 61]));
        assert_eq!(result["loaded"], true);
    }
    Ok(())
}

/// Successful structured loads preserve raw bytes while post overrides aggregate query values.
#[tokio::test]
async fn structured_body_loading() -> Result<()> {
    // Route capture injection follows successful loading and overrides only aggregate data.
    let app = inspector(MethodPattern::POST, "/parse/:id").await?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    // Known structured formats share one load contract while retaining their original byte buffers.
    for (mime, body) in [
        (
            "application/json",
            br#"{"id":"post","active":false,"n":0}"#.as_slice(),
        ),
        (
            "application/problem+json",
            br#"{"id":"post","active":false,"n":0}"#.as_slice(),
        ),
        (
            "application/x-www-form-urlencoded",
            b"id=post&active=false&n=0".as_slice(),
        ),
        (
            "multipart/form-data; boundary=B",
            b"--B\r\nContent-Disposition: form-data; name=\"id\"\r\n\r\npost\r\n--B--\r\n"
                .as_slice(),
        ),
    ] {
        // Collide query, parsed post and route capture IDs to make their precedence observable.
        let input = NativeRequest::builder()
            .method("POST")
            .uri("/parse/route?id=query")
            .header("content-type", mime)
            .body(body.to_vec())
            .unwrap();
        // Complete the lifecycle before comparing independent source stores and the raw buffer.
        let result = envelope(adapter.handle(&app, input).await?)["results"].clone();
        assert_eq!(result["query"]["id"], "query");
        assert_eq!(result["post"]["id"], "post");
        assert_eq!(result["data"]["id"], "route");
        assert_eq!(result["body"], json!(body));
    }
    Ok(())
}

/// Malformed known media use one Input error offer and never partially mutate loaded state.
#[tokio::test]
async fn strict_parse_failure_is_atomic_and_recoverable() -> Result<()> {
    // Only the error handler may run; it inspects failed load state and performs explicit recovery.
    let mut builder = ServerBuilder::new();
    builder.on("request", forbidden)?;
    builder.post("/parse", forbidden)?;
    builder.on("error", recover_input)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    // Syntax, root-shape, encoding, percent-decoding and multipart-boundary failures are distinct.
    for (mime, body) in [
        ("application/json", b"{bad".as_slice()),
        ("application/json", b"[]"),
        ("application/json", b"null"),
        ("application/json", &[255]),
        ("application/x-www-form-urlencoded", b"id=%ZZ"),
        ("multipart/form-data", b"missing boundary"),
    ] {
        // Keep a valid query/header alongside the bad body so normalization can genuinely succeed.
        let input = NativeRequest::builder()
            .method("POST")
            .uri("/parse?kept=7")
            .header("x-id", "normalized")
            .header("content-type", mime)
            .body(body.to_vec())
            .unwrap();
        let reply = adapter.handle(&app, input).await?;
        // Exactly one retained Input failure is offered; successful recovery supplies public
        // output.
        assert_eq!(reply.report.error_offers.len(), 1);
        assert_eq!(reply.report.failures.len(), 1);
        assert_eq!(envelope(reply)["results"], "recovered");
    }
    Ok(())
}

/// Normalization rejects ambiguous/binary headers before callbacks rather than guessing.
#[tokio::test]
async fn invalid_headers_and_method_are_no_request_failures() -> Result<()> {
    // No normalized request means even an error hook must remain unreachable.
    let mut builder = ServerBuilder::new();
    builder.on("request", forbidden)?;
    builder.on("error", forbidden)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    let adapter = HttpAdapter::new(HttpOptions::default())?;
    // Native header values can contain opaque octets that foundation text headers must reject.
    let mut binary = NativeRequest::new(Vec::new());
    binary
        .headers_mut()
        .insert("x-opaque", HeaderValue::from_bytes(&[255]).unwrap());
    // Multiple types are ambiguous even when both occurrences are individually valid MIME strings.
    let repeated = NativeRequest::builder()
        .header("content-type", "application/json")
        .header("content-type", "text/plain")
        .body(Vec::new())
        .unwrap();
    // The native token parser accepts this length; foundation method policy supplies its own bound.
    let long_method = NativeRequest::builder()
        .method("X".repeat(129).as_str())
        .body(Vec::new())
        .unwrap();
    for (index, input) in [binary, repeated, long_method].into_iter().enumerate() {
        let reply = adapter.handle(&app, input).await?;
        if index == 0 {
            let error = reply.report.failures[0].error.as_ref();
            assert_eq!(error.message(), "HTTP request header decoding failed");
            assert!(
                std::error::Error::source(error)
                    .unwrap()
                    .is::<http::header::ToStrError>()
            );
            assert_eq!(error.kind(), stackpress_lib::ErrorKind::External);
            assert_eq!(error.public_view().error, "Internal Server Error");
        }
        // G17 returns a fixed public 400 and offers no callback an invented normalized request.
        assert!(reply.report.error_offers.is_empty());
        assert_eq!(native(reply).status(), 400);
    }
    Ok(())
}

/// Depth, node, merge and multipart-part budgets fail atomically inside the Input offer.
#[tokio::test]
async fn parsed_and_merged_limits_use_the_shared_input_stage() -> Result<()> {
    // Recovery inspects unchanged query/body/post state for both parse and install-time failures.
    let mut builder = ServerBuilder::new();
    builder.on("request", forbidden)?;
    builder.on("error", recover_input)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    // The second JSON fails parsing; the third fits post alone but exceeds the aggregate merge.
    for (limits, mime, body) in [
        (
            ParseLimits {
                max_depth: 2,
                ..Default::default()
            },
            "application/json",
            br#"{"a":{"b":{"c":1}}}"#.as_slice(),
        ),
        (
            ParseLimits {
                max_entries: 3,
                ..Default::default()
            },
            "application/json",
            br#"{"a":1,"b":2,"c":3}"#,
        ),
        (
            ParseLimits {
                max_entries: 3,
                ..Default::default()
            },
            "application/json",
            br#"{"a":1,"b":2}"#,
        ),
        (
            ParseLimits {
                max_part_headers: 8,
                ..Default::default()
            },
            "multipart/form-data; boundary=B",
            b"--B\r\nContent-Disposition: form-data; name=\"a\"\r\n\r\n1\r\n--B--\r\n",
        ),
    ] {
        // Only the selected parsing budget changes; request method/path/headers remain valid.
        let adapter = HttpAdapter::new(HttpOptions {
            limits,
            ..Default::default()
        })?;
        let input = NativeRequest::builder()
            .method("POST")
            .uri("/parse?kept=7")
            .header("x-id", "normalized")
            .header("content-type", mime)
            .body(body.to_vec())
            .unwrap();
        let reply = adapter.handle(&app, input).await?;
        // All budget failure classes belong to one Input offer rather than a second lifecycle.
        assert_eq!(reply.report.failures[0].stage, Stage::Input);
        assert_eq!(reply.report.error_offers.len(), 1);
        assert_eq!(envelope(reply)["results"], "recovered");
    }
    Ok(())
}

/// Binary multipart files retain exact bytes and metadata without implicit wire serialization.
async fn inspect_file(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // A file is typed rich input, not a lossy UTF-8 string or an implicit filesystem path.
    let Some(stackpress_lib::Value::File(file)) = req.post.path("upload") else {
        panic!("expected a typed multipart file");
    };
    // Validate exact untrusted metadata and bytes before producing a simple JSON-safe result.
    assert_eq!(file.name, "input.bin");
    assert_eq!(file.mimetype, "application/octet-stream");
    assert_eq!(file.data, [0, 255]);
    assert!(matches!(req.body(), Body::Bytes(_)));
    res.results(&true)?;
    Ok(Flow::Continue)
}

/// Adapter loading uses the existing byte-safe multipart parser, including binary file parts.
#[tokio::test]
async fn multipart_binary_file_input() -> Result<()> {
    // The callback inspects typed file input while all parsing stays inside the adapter loader.
    let mut builder = ServerBuilder::new();
    builder.post("/upload", inspect_file)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    // Include NUL/invalid UTF-8 inside the file part to catch accidental whole-body text decoding.
    let body = [
        b"--B\r\nContent-Disposition: form-data; name=\"upload\"; ".as_slice(),
        b"filename=\"input.bin\"\r\nContent-Type: application/octet-stream\r\n",
        b"\r\n\x00\xff\r\n--B--\r\n",
    ]
    .concat();
    // Supply the same explicit boundary in the MIME header and buffered multipart body.
    let input = NativeRequest::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "multipart/form-data; boundary=B")
        .body(body.to_vec())
        .unwrap();
    let reply = HttpAdapter::new(HttpOptions::default())?
        .handle(&app, input)
        .await?;
    // Successful parsing reaches the named callback exactly through the ordinary host lifecycle.
    assert!(reply.report.failures.is_empty());
    assert_eq!(envelope(reply)["results"], true);
    Ok(())
}

/// Each raw budget accepts its boundary and rejects boundary+1 before callbacks.
#[tokio::test]
async fn input_limit_boundaries() -> Result<()> {
    let app = inspector(MethodPattern::POST, "/").await?;
    // Method=4, path=1, header name/value=3, body=32: total=40 bytes.
    let options = HttpOptions {
        max_request_bytes: 40,
        max_header_bytes: 3,
        max_headers: 1,
        limits: ParseLimits {
            max_bytes: 32,
            ..Default::default()
        },
        ..Default::default()
    };
    for (field, delta) in [
        ("total", 0),
        ("total", 1),
        ("headers", 1),
        ("count", 1),
        ("body", 1),
    ] {
        // Lower one ceiling at a time; the same input is accepted exactly at every baseline limit.
        let mut policy = options.clone();
        match field {
            "total" => policy.max_request_bytes -= delta,
            "headers" => policy.max_header_bytes -= delta,
            "count" => policy.max_headers -= delta,
            "body" => policy.limits.max_bytes -= delta,
            _ => unreachable!(),
        }
        // Keep native bytes identical so failures are attributable to policy rather than parsing.
        let input = NativeRequest::builder()
            .method("POST")
            .header("x", "12")
            .body(vec![0; 32])
            .unwrap();
        let reply = HttpAdapter::new(policy)?.handle(&app, input).await?;
        // Crossing a raw ceiling is pre-normalization failure, so public status is fixed at 400.
        assert_eq!(
            native(reply).status(),
            if delta == 0 { 200 } else { 400 },
            "{field}"
        );
    }
    Ok(())
}

/// A shared capability returns exactly the same public envelope through call and HTTP.
async fn profile(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Explicit zero totals and the route/call ID must survive both public envelope encoders.
    res.rows(&json!({"id": req.data.path_as::<String>("id")?}), 0)?;
    Ok(Flow::Continue)
}

/// The application explicitly grants one route access to its named capability.
async fn delegate(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Only this application route grants HTTP access to the otherwise named capability.
    inv.delegate("profile-search", req, res).await
}

/// Native input never exposes a generic event endpoint; route delegation remains explicit.
#[tokio::test]
async fn call_and_http_public_envelope_parity() -> Result<()> {
    // Build once so both entry surfaces share identical capability code and lifecycle policy.
    let mut builder = ServerBuilder::new();
    builder.on("profile-search", profile)?;
    builder.get("/profiles/:id", delegate)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    // The direct call supplies the same ID that the HTTP route captures from its pathname.
    let called = app
        .call("profile-search", Nest::from_object(json!({"id":"abc"}))?)
        .await?;
    let input = NativeRequest::builder()
        .uri("/profiles/abc")
        .body(Vec::new())
        .unwrap();
    let http = HttpAdapter::new(HttpOptions::default())?
        .handle(&app, input)
        .await?;
    // Compare only public envelopes; native framing and developer reports are separate contracts.
    assert_eq!(called.response.try_to_json()?, envelope(http));
    Ok(())
}
