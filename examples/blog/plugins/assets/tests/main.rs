//! Plugin-local request guard and real source/build asset proofs, without a web host.
use serde_json::json;
use stackpress_lib::{Body, Request, Response};
use stackpress_server::{ConfigLayers, ServerBuilder};

/// Source bytes, owned build products and cooperative request skips use the same registrar.
#[tokio::test(flavor = "current_thread")]
async fn serves_authored_and_selected_build_bytes_with_original_mimes() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    // Build products are disposable fixtures; authored assets remain the shipped source bytes.
    for (file, bytes) in [
        ("built.js", b"compiled".as_slice()),
        ("built.wasm", b"\0asm"),
        ("map.json", b"{}"),
        ("other.bin", b"raw"),
    ] {
        std::fs::write(root.path().join(file), bytes).unwrap();
    }
    let mut builder = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value("owned compiled assets", json!({"assets":root.path()}))
        .unwrap();
    builder.config_layers(layers).unwrap();
    blog_assets::register(&mut builder).unwrap();
    let server = builder.build().await.unwrap();
    for (path, mime, part) in [
        ("/styles/global.css", "text/css", "."),
        ("/styles/article.css", "text/css", "."),
        ("/assets/blog.js", "text/javascript", "__blogDispose"),
        ("/logo.png", "image/svg+xml", "<svg"),
        ("/assets/built.js", "text/javascript", "compiled"),
        ("/assets/built.wasm", "application/wasm", "asm"),
        ("/assets/map.json", "application/json", "{}"),
        ("/assets/other.bin", "application/octet-stream", "raw"),
    ] {
        let mut request = Request::builder().url(path).build().unwrap();
        let mut response = Response::new();
        server
            .emit("request", &mut request, &mut response)
            .await
            .unwrap();
        assert_eq!(response.status().unwrap().code, 200, "{path}");
        assert_eq!(response.mimetype(), Some(mime));
        let Body::Bytes(bytes) = response.body() else {
            panic!("expected native bytes")
        };
        assert!(String::from_utf8_lossy(bytes).contains(part), "{path}");
    }
    // Missing/unclaimed assets and an earlier status/body remain available to later handlers.
    for (path, status, body) in [
        ("/assets/missing.js", None, Body::Empty),
        ("/unclaimed", None, Body::Empty),
        ("/styles/global.css", Some(403), Body::Empty),
        ("/styles/global.css", Some(200), Body::Text("prior".into())),
    ] {
        let mut request = Request::builder().url(path).build().unwrap();
        let mut response = Response::new();
        // An empty untouched response has no status; set() would itself assign 200.
        if !matches!(body, Body::Empty) {
            response.set("text/plain", body.clone()).unwrap();
        }
        if let Some(code) = status {
            response.status_code(code, None).unwrap();
        }
        server
            .emit("request", &mut request, &mut response)
            .await
            .unwrap();
        assert_eq!(response.status().map(|status| status.code), status);
        assert_eq!(format!("{:?}", response.body()), format!("{body:?}"));
    }
}
