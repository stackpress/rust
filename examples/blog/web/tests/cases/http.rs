//! Actual native HTTP sign-in with source cookies, independent of browser instrumentation.
use super::support::App;
use stackpress_server::http::{HttpAdapter, HttpOutput};

/// The exact devmetrics/README Host header reaches the blog through the buffered adapter.
#[tokio::test(flavor = "current_thread")]
async fn native_http_adapter_accepts_documented_localhost_host() {
    let app = App::new().await;
    let options = blog_web::http::network(3000);
    let adapter = HttpAdapter::new(options.http).unwrap();
    let request = http::Request::builder()
        .uri("/")
        .header("host", "localhost:3000")
        .body(Vec::new())
        .unwrap();
    let reply = adapter.handle(&app.server, request).await.unwrap();
    let HttpOutput::Response(response) = reply.output else {
        panic!("response unavailable")
    };
    app.close().await;
    assert_eq!(response.status().as_u16(), 200);
    assert!(String::from_utf8_lossy(response.body()).contains("Knowledge Console"));
}

/// Real native authority checks use supplied duplex IO; no bind, TCP socket or listener.
#[tokio::test(flavor = "current_thread")]
async fn native_host_accepts_localhost_and_explicit_loopback_alias_without_listener() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let app = App::new().await;
    let options = blog_web::http::network(3000);
    assert_eq!(options.address, "127.0.0.1:3000".parse().unwrap());
    assert_eq!(options.http.base_url, "http://localhost:3000/");
    let host =
        stackpress_server_http::HttpServer::from_options(app.server.clone(), options).unwrap();
    let mut replies = Vec::new();
    for authority in ["localhost:3000", "127.0.0.1:3000", "other.example:3000"] {
        let (io, mut client) = tokio::io::duplex(32 * 1024);
        let request = format!("GET / HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\n\r\n");
        let exchange = async move {
            client.write_all(request.as_bytes()).await.unwrap();
            let mut bytes = Vec::new();
            client.read_to_end(&mut bytes).await.unwrap();
            bytes
        };
        // Connection: close bounds the ordinary exchange; the deadline also reaps a defect.
        let served = host.serve_io(
            io,
            "127.0.0.1:43123".parse().unwrap(),
            std::future::pending(),
        );
        let result = tokio::time::timeout(std::time::Duration::from_secs(10), async {
            tokio::join!(served, exchange)
        })
        .await;
        replies.push(result.map(|(_, wire)| String::from_utf8_lossy(&wire).into_owned()));
    }
    drop(host);
    app.close().await;
    for (index, result) in replies.into_iter().enumerate() {
        let wire = result.expect("finite duplex request timed out");
        let code = if index < 2 { 200 } else { 400 };
        assert!(wire.starts_with(&format!("HTTP/1.1 {code}")), "{wire}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn native_asset_adapter_serves_actual_debug_wasm_with_explicit_response_allowance() {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.build/browser");
    let app = App::configured(serde_json::json!({"assets":assets})).await;
    let adapter = HttpAdapter::new(blog_web::http::options("http://blog.test/")).unwrap();
    let request = http::Request::builder()
        .uri("/assets/hydrate/blog_bg.wasm")
        .body(Vec::new())
        .unwrap();
    let reply = adapter.handle(&app.server, request).await.unwrap();
    let HttpOutput::Response(response) = reply.output else {
        panic!("response unavailable")
    };
    app.close().await;
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers()["content-type"], "application/wasm");
    assert_eq!(
        response.body().len() as u64,
        std::fs::metadata(assets.join("hydrate/blog_bg.wasm"))
            .unwrap()
            .len()
    );
    assert_eq!(&response.body()[..4], b"\0asm");
}

#[tokio::test(flavor = "current_thread")]
async fn source_username_signin_redirect_survives_native_http_output() {
    let app = App::new().await;
    let adapter = HttpAdapter::new(Default::default()).unwrap();
    let request = http::Request::builder()
        .method("POST")
        .uri("/auth/signin/username")
        .header("cookie", "csrf=fixture-csrf")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(b"csrf=fixture-csrf&username=developer&secret=developer".to_vec())
        .unwrap();
    let reply = adapter.handle(&app.server, request).await.unwrap();
    let failures = reply
        .report
        .failures
        .iter()
        .map(|failure| format!("{:?}", failure.error))
        .collect::<Vec<_>>();
    let HttpOutput::Response(response) = reply.output else {
        panic!("response unavailable")
    };
    app.close().await;
    assert_eq!(
        response.status().as_u16(),
        302,
        "{failures:?}; body: {}",
        String::from_utf8_lossy(response.body())
    );
    assert_eq!(response.headers().get("location").unwrap(), "/");
}

/// The original native HTTP dispatcher applies its default root cookie scope to sign-in.
/// Without that scope, a browser sends this ADMIN token only under /auth/signin.
#[tokio::test(flavor = "current_thread")]
async fn source_signin_cookie_retains_original_http_root_scope() {
    let app = App::new().await;
    let adapter = HttpAdapter::new(Default::default()).unwrap();
    let request = http::Request::builder()
        .method("POST")
        .uri("/auth/signin/username")
        .header("cookie", "csrf=fixture-csrf")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(b"csrf=fixture-csrf&username=developer&secret=developer".to_vec())
        .unwrap();
    let reply = adapter.handle(&app.server, request).await.unwrap();
    let HttpOutput::Response(response) = reply.output else {
        panic!("response unavailable")
    };
    let cookie = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|header| header.to_str().ok())
        .find(|cookie| cookie.starts_with("session="))
        .map(str::to_owned);
    // Close the fixture even when this framework HTTP parity contract is still defective.
    app.close().await;
    assert_eq!(response.status().as_u16(), 302);
    let cookie = cookie.expect("successful source sign-in must issue its session cookie");
    assert!(
        cookie
            .split(';')
            .skip(1)
            .any(|part| part.trim() == "Path=/"),
        "source HTTP sign-in cookie must retain the default Path=/ (token omitted)"
    );
}
