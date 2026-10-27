//! Meaningful host contracts over actual generated SQLite operations and port-free HTTP.
use super::{gateway::Gateway, options::Options, panel::actor, pipe, setup};
use base64::{Engine, engine::general_purpose::STANDARD};
use stackpress_server::http::{HttpAdapter, HttpOutput};
use std::{collections::BTreeMap, fs, sync::atomic::Ordering};

/// Build exact launch inputs without opening a socket.
fn options(directory: &std::path::Path, base: &str, csr: bool) -> Options {
    Options::parse(
        &[
            "--port",
            "3001",
            "--directory",
            directory.to_str().unwrap(),
            "--base",
            base,
            "--render",
            if csr { "csr" } else { "hydrate" },
        ]
        .map(String::from),
    )
    .unwrap()
}
/// Preload a minimal real hashed module at every required mount, independent of WASM tool output.
fn assets(directory: &std::path::Path) {
    for (name, mount) in [
        ("gallery", "ui-gallery"),
        ("preferences", "ui-preferences"),
        ("admin", "generated-admin"),
        ("format", "ui-format"),
        ("stream", "ui-stream"),
    ] {
        for mode in ["hydrate", "csr"] {
            let root = directory.join("bundle/disabled").join(name).join(mode);
            fs::create_dir_all(&root).unwrap();
            fs::write(root.join("start.js"), b"").unwrap();
            fs::write(root.join("manifest.json"),serde_json::json!({"entries":[{"route":format!("/{mount}/assets/start.js"),"file":"start.js","size":0,"sha256":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","mime":"text/javascript; charset=utf-8","cache":"revalidate"}]}).to_string()).unwrap();
        }
    }
}
/// Exercise real HttpAdapter admission and keep multivalued headers intact.
async fn request(
    host: &Gateway,
    path: &str,
    method: &str,
    cookie: &str,
    body: &str,
) -> http::Response<Vec<u8>> {
    let request = http::Request::builder()
        .method(method)
        .uri(path)
        .header("cookie", cookie)
        .header("origin", "http://127.0.0.1:3001")
        .header("x-admin-actor", "editor")
        .header("x-admin-csrf", "review-local-csrf")
        .header(
            "content-type",
            if body.is_empty() {
                "text/plain"
            } else {
                "application/json"
            },
        )
        .body(body.as_bytes().to_vec())
        .unwrap();
    let HttpOutput::Response(response) = HttpAdapter::new(Default::default())
        .unwrap()
        .handle(&host.app, request)
        .await
        .unwrap()
        .output
    else {
        panic!("buffered output");
    };
    response
}
/// Decode a public JSON envelope without exposing host-private causes.
fn json(response: &http::Response<Vec<u8>>) -> serde_json::Value {
    serde_json::from_slice(response.body()).unwrap()
}
/// Prepare an owned fresh database and checked assets.
async fn fixture(base: &str, csr: bool) -> (tempfile::TempDir, Gateway, Options) {
    let directory = tempfile::tempdir_in("/private/tmp").unwrap();
    setup::create(directory.path()).await.unwrap();
    assets(directory.path());
    let options = options(directory.path(), base, csr);
    let host = Gateway::new(&options).await.unwrap();
    (directory, host, options)
}
/// Exact flags enforce bounded startup and preserve mode/base/variant selection.
#[test]
fn launch_and_cookie_boundaries() {
    let options = options(std::path::Path::new("/tmp/owned"), "/demo", true);
    assert!(options.csr);
    assert_eq!(options.path("/review"), "/demo/review");
    assert_eq!(options.origin(), "http://127.0.0.1:3001");
    for args in [
        vec![],
        vec!["--port"],
        vec!["--port", "0", "--directory", "x"],
        vec!["--port", "bad", "--directory", "x"],
        vec!["--port", "1", "--port", "2"],
        vec!["--unknown", "x"],
        vec![
            "--port",
            "1",
            "--directory",
            "x",
            "--duration-seconds",
            "7201",
        ],
        vec!["--port", "1", "--directory", "x", "--render", "bad"],
        vec!["--port", "1", "--directory", "x", "--variant", "bad"],
    ] {
        assert!(Options::parse(&args.into_iter().map(String::from).collect::<Vec<_>>()).is_err());
    }
    assert_eq!(actor("slice8-review-role=editor"), "editor");
    assert_eq!(actor("slice8-review-role=reader"), "reader");
    for cookie in [
        "",
        "slice8-review-role=bad",
        "slice8-review-role=reader; slice8-review-role=editor",
        "slice8-review-role; slice8-review-role=editor",
    ] {
        assert_eq!(actor(cookie), "denied");
    }
}
/// Setup is explicit, fresh and related; existing files and symlink targets are preserved.
#[tokio::test]
async fn fresh_setup_and_attachment() {
    let directory = tempfile::tempdir_in("/private/tmp").unwrap();
    assert!(setup::attach(directory.path()).is_err());
    let database = setup::create(directory.path()).await.unwrap();
    assert!(database.is_file());
    assert_eq!(setup::attach(directory.path()).unwrap(), database);
    assert!(setup::create(directory.path()).await.is_err());
    fs::write(
        directory.path().join("state.json"),
        "{\"database\":\"../outside\"}",
    )
    .unwrap();
    assert!(setup::attach(directory.path()).is_err());
    fs::write(
        directory.path().join("state.json"),
        "{\"database\":\"review-missing.sqlite\"}",
    )
    .unwrap();
    assert!(setup::attach(directory.path()).is_err());
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&database, directory.path().join("review-link.sqlite")).unwrap();
        fs::write(
            directory.path().join("state.json"),
            "{\"database\":\"review-link.sqlite\"}",
        )
        .unwrap();
        assert!(setup::attach(directory.path()).is_err());
    }
}
/// One composed host preserves mounts, cookie persistence, roles, generated writes and fault meaning.
#[tokio::test]
async fn composed_host_controls_and_denials() {
    let (_directory, host, options) = fixture("", false).await;
    let denied = request(&host, "/generated-admin/profile/search", "GET", "", "").await;
    assert_eq!(denied.status(), 403);
    for (action, expected) in [
        ("editor", "slice8-review-role=editor"),
        ("reader", "slice8-review-role=reader"),
        ("denied", "slice8-review-role=denied"),
    ] {
        let reply = request(&host, &format!("/review/action/{action}"), "POST", "", "").await;
        assert!(reply.status().is_redirection());
        let cookie = reply.headers()["set-cookie"].to_str().unwrap();
        assert!(cookie.starts_with(expected));
        assert!(cookie.contains("HttpOnly; SameSite=Lax"));
        assert!(!cookie.contains("Secure"));
    }
    let cookie = "slice8-review-role=editor";
    for path in [
        "/review",
        "/ui-gallery/gallery",
        "/ui-preferences/preferences",
        "/generated-admin/profile/search",
        "/generated-admin/roles",
        "/ui-format/",
        "/ui-format/edit/en",
        "/ui-format/edit/fr",
        "/review/valid.csv",
        "/review/invalid.csv",
    ] {
        let response = request(&host, path, "GET", cookie, "").await;
        assert_eq!(
            response.status(),
            200,
            "{path}: {}",
            String::from_utf8_lossy(response.body())
        );
    }
    assert!(
        request(&host, "/", "GET", cookie, "")
            .await
            .status()
            .is_redirection()
    );
    let write = r#"{"input":{"id":"owner-created","name":"Owner record"}}"#;
    assert_eq!(
        request(
            &host,
            "/generated-admin/api/profile/create",
            "POST",
            "slice8-review-role=reader",
            write
        )
        .await
        .status(),
        403
    );
    assert_eq!(
        request(
            &host,
            "/generated-admin/api/profile/create",
            "POST",
            cookie,
            write
        )
        .await
        .status(),
        200
    );
    request(&host, "/review/action/ack", "POST", cookie, "").await;
    let unknown = request(
        &host,
        "/generated-admin/api/profile/create",
        "POST",
        cookie,
        r#"{"input":{"id":"uncertain","name":"Committed once"}}"#,
    )
    .await;
    assert!(!unknown.status().is_success());
    assert_eq!(json(&unknown)["errors"]["outcome"], "unknown");
    assert!(!host.panel.acknowledgement.load(Ordering::SeqCst));
    let records = request(
        &host,
        "/generated-admin/api/profile/get",
        "POST",
        cookie,
        r#"{"key":{"id":"uncertain"}}"#,
    )
    .await;
    assert!(String::from_utf8_lossy(records.body()).contains("uncertain"));
    for action in ["failed", "slow"] {
        request(
            &host,
            &format!("/review/action/{action}"),
            "POST",
            cookie,
            "",
        )
        .await;
        let start = std::time::Instant::now();
        let reply = request(&host, "/generated-admin/profile/search", "GET", cookie, "").await;
        if action == "failed" {
            assert_eq!(reply.status(), 503);
        } else {
            assert_eq!(reply.status(), 200);
            assert!(start.elapsed() >= std::time::Duration::from_millis(750));
        }
        assert_eq!(
            request(&host, "/generated-admin/profile/search", "GET", cookie, "")
                .await
                .status(),
            200
        );
    }
    request(&host, "/review/action/pause", "POST", cookie, "").await;
    assert!(host.panel.paused.load(Ordering::SeqCst));
    let html = request(&host, "/ui-gallery/gallery", "GET", cookie, "").await;
    assert!(String::from_utf8_lossy(html.body()).contains("review-pause\" content=\"true"));
    request(&host, "/review/action/automatic", "POST", cookie, "").await;
    assert!(!host.panel.paused.load(Ordering::SeqCst));
    request(&host, "/review/action/preferences", "POST", cookie, "").await;
    assert_eq!(
        request(
            &host,
            "/ui-preferences/ui-preferences",
            "POST",
            cookie,
            "{}"
        )
        .await
        .status(),
        503
    );
    let flash = request(&host, "/review/action/flash", "POST", cookie, "").await;
    let flash = flash.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let page = request(&host, "/ui-preferences/preferences", "GET", flash, "").await;
    assert!(String::from_utf8_lossy(page.body()).contains("Public review notice"));
    assert!(
        page.headers()["set-cookie"]
            .to_str()
            .unwrap()
            .contains("Max-Age=0")
    );
    let state = request(&host, "/review/state", "GET", cookie, "").await;
    assert_eq!(json(&state)["cleaned_stream_roots"], 0);
    // The same prepared native host delivers exact bytes over HTTP/1 without a socket.
    let response = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "GET".into(),
            path: "/review".into(),
            headers: BTreeMap::new(),
            body: String::new(),
        },
    )
    .await
    .unwrap();
    assert_eq!(response.status, 200);
    assert!(
        String::from_utf8(STANDARD.decode(response.body).unwrap())
            .unwrap()
            .contains("STACKPRESS_REVIEW_PREVIEW")
    );
    let asset = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "GET".into(),
            path: "/ui-gallery/assets/start.js".into(),
            headers: BTreeMap::new(),
            body: String::new(),
        },
    )
    .await
    .unwrap();
    assert_eq!(asset.status, 200);
    host.engine.close().await.unwrap();
}
/// Mounted CSR, startup asset failures and explicit Origin denial remain independent of UI state.
#[tokio::test]
async fn mounted_csr_and_fail_closed_assets() {
    let (directory, host, _) = fixture("/demo", true).await;
    let page = request(
        &host,
        "/demo/generated-admin/profile/search",
        "GET",
        "slice8-review-role=editor",
        "",
    )
    .await;
    assert_eq!(page.status(), 200);
    assert!(String::from_utf8_lossy(page.body()).contains("<div id=\"app\"></div>"));
    assert_ne!(
        request(
            &host,
            "/generated-admin/profile/search",
            "GET",
            "slice8-review-role=editor",
            ""
        )
        .await
        .status(),
        200
    );
    let bad = http::Request::builder()
        .method("POST")
        .uri("/demo/review/action/editor")
        .header("origin", "http://evil.invalid")
        .body(Vec::<u8>::new())
        .unwrap();
    let HttpOutput::Response(bad) = HttpAdapter::new(Default::default())
        .unwrap()
        .handle(&host.app, bad)
        .await
        .unwrap()
        .output
    else {
        panic!()
    };
    assert_eq!(bad.status(), 403);
    host.engine.close().await.unwrap();
    fs::write(
        directory
            .path()
            .join("bundle/disabled/gallery/hydrate/start.js"),
        "tamper",
    )
    .unwrap();
    assert!(
        Gateway::new(&options(directory.path(), "", false))
            .await
            .is_err()
    );
}

/// Real loopback-HTTP cookie scopes persist/confirm French dark preferences without Secure.
#[tokio::test]
async fn plain_http_preferences_and_stream_cancellation() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let (_directory, host, options) = fixture("", false).await;
    let update = r#"{"locale":"fr","theme":"dark","location":"/ui-preferences/preferences"}"#;
    let saved = request(&host, "/ui-preferences/ui-preferences", "POST", "", update).await;
    assert_eq!(
        saved.status(),
        200,
        "{}",
        String::from_utf8_lossy(saved.body())
    );
    let cookies = saved
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|value| {
            let value = value.to_str().unwrap();
            assert!(value.contains("Path=/ui-preferences"));
            assert!(!value.contains("Secure"));
            value.split(';').next().unwrap().to_owned()
        })
        .collect::<Vec<_>>()
        .join("; ");
    let confirmed = request(
        &host,
        "/ui-preferences/ui-preferences/confirm",
        "POST",
        &cookies,
        update,
    )
    .await;
    assert_eq!(confirmed.status(), 200);
    assert!(confirmed.headers().get("set-cookie").is_none());
    let page = request(&host, "/ui-preferences/preferences", "GET", &cookies, "").await;
    let html = String::from_utf8_lossy(page.body());
    assert!(html.contains("lang=\"fr\" data-theme=\"dark\""));
    // Observe the real native stream before the 800-ms Resource, then disconnect.
    let (server, mut client) = tokio::io::duplex(128);
    let transport = host.http.serve_io(
        server,
        "127.0.0.1:1".parse().unwrap(),
        std::future::pending(),
    );
    let browser = async {
        let start = std::time::Instant::now();
        client.write_all(b"GET /ui-stream/pending HTTP/1.1\r\nHost: 127.0.0.1:3001\r\nConnection: close\r\n\r\n").await.unwrap();
        let mut bytes = Vec::new();
        let mut buffer = [0; 256];
        while !String::from_utf8_lossy(&bytes).contains("id=\"shell\"") {
            let count = client.read(&mut buffer).await.unwrap();
            assert!(count > 0);
            bytes.extend_from_slice(&buffer[..count]);
            assert!(bytes.len() < 8192);
        }
        assert!(start.elapsed() < std::time::Duration::from_millis(700));
        drop(client);
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        tokio::join!(transport, browser)
    })
    .await
    .unwrap();
    assert!(host.panel.probe.cleanups.load(Ordering::SeqCst) > 0);
    assert!(host.panel.probe.released());
    let success = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "GET".into(),
            path: "/ui-stream/stream".into(),
            headers: BTreeMap::new(),
            body: String::new(),
        },
    )
    .await
    .unwrap();
    assert!(!success.interrupted);
    assert!(
        String::from_utf8(STANDARD.decode(success.body).unwrap())
            .unwrap()
            .contains("stackpress-stream-complete")
    );
    let late = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "GET".into(),
            path: "/ui-stream/late".into(),
            headers: BTreeMap::new(),
            body: String::new(),
        },
    )
    .await
    .unwrap();
    let html = String::from_utf8(STANDARD.decode(late.body).unwrap()).unwrap();
    assert!(late.interrupted);
    assert!(html.contains("stackpress-stream-recovery"));
    assert!(!html.contains("stackpress-stream-complete"));
    host.engine.close().await.unwrap();
}

/// Startup rejects unbounded manifest input and symlink parents before creating caller files.
#[tokio::test]
async fn filesystem_admission_limits() {
    let directory = tempfile::tempdir_in("/private/tmp").unwrap();
    assets(directory.path());
    setup::create(directory.path()).await.unwrap();
    fs::write(
        directory
            .path()
            .join("bundle/disabled/gallery/hydrate/manifest.json"),
        vec![b' '; 65537],
    )
    .unwrap();
    assert!(
        Gateway::new(&options(directory.path(), "", false))
            .await
            .is_err()
    );
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir_in("/private/tmp").unwrap();
        let link = directory.path().join("linked");
        std::os::unix::fs::symlink(outside.path(), &link).unwrap();
        assert!(setup::create(&link.join("must-not-exist")).await.is_err());
        assert!(!outside.path().join("must-not-exist").exists());
    }
    let mut wrong = options(directory.path(), "", false);
    wrong.variant = "plugin".into();
    assert!(Gateway::new(&wrong).await.is_err());
}

/// The composed native HTTP path previews without writes, confirms explicitly and guards CSV export.
#[tokio::test]
async fn native_csv_preview_confirm_and_export() {
    let (_directory, host, options) = fixture("", false).await;
    let csv = "id,name\ncsv-native,=5+5\n";
    let mut headers = BTreeMap::from([
        ("cookie".into(), "slice8-review-role=editor".into()),
        ("origin".into(), options.origin()),
        ("content-type".into(), "text/csv".into()),
        ("x-admin-csrf".into(), "review-local-csrf".into()),
    ]);
    let preview = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "POST".into(),
            path: "/generated-admin/api/profile/preview".into(),
            headers: headers.clone(),
            body: STANDARD.encode(csv),
        },
    )
    .await
    .unwrap();
    assert_eq!(preview.status, 200);
    let preview: serde_json::Value =
        serde_json::from_slice(&STANDARD.decode(preview.body).unwrap()).unwrap();
    assert_eq!(preview["results"]["rows"].as_array().unwrap().len(), 1);
    let before = request(
        &host,
        "/generated-admin/api/profile/get",
        "POST",
        "slice8-review-role=editor",
        r#"{"key":{"id":"csv-native"}}"#,
    )
    .await;
    assert_eq!(before.status(), 404, "preview must not persist a row");
    headers.insert("x-admin-confirmed".into(), "true".into());
    let confirm = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "POST".into(),
            path: "/generated-admin/api/profile/confirm".into(),
            headers: headers.clone(),
            body: STANDARD.encode(csv),
        },
    )
    .await
    .unwrap();
    assert_eq!(confirm.status, 200);
    let after = request(
        &host,
        "/generated-admin/api/profile/get",
        "POST",
        "slice8-review-role=editor",
        r#"{"key":{"id":"csv-native"}}"#,
    )
    .await;
    assert_eq!(json(&after)["results"]["name"], "=5+5");
    let export = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "GET".into(),
            path: "/generated-admin/profile/export".into(),
            headers: headers.clone(),
            body: String::new(),
        },
    )
    .await
    .unwrap();
    assert_eq!(export.status, 200);
    let csv = String::from_utf8(STANDARD.decode(export.body).unwrap()).unwrap();
    assert!(csv.contains("'=5+5"));
    assert!(!csv.contains("ADMIN_PRIVATE"));
    // A forged authority header cannot upgrade the selected reader at the native edge.
    headers.insert("cookie".into(), "slice8-review-role=reader".into());
    headers.insert("x-admin-actor".into(), "editor".into());
    let denied = pipe::exchange(
        &host,
        &options.origin(),
        pipe::Input {
            method: "POST".into(),
            path: "/generated-admin/api/profile/confirm".into(),
            headers,
            body: STANDARD.encode("id,name\nforged,No\n"),
        },
    )
    .await
    .unwrap();
    assert_eq!(denied.status, 403);
    host.engine.close().await.unwrap();
}
