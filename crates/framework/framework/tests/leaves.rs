//! API can reach identity types without selecting the aggregate session registrar.
#![cfg(all(feature = "api", not(feature = "session"), not(target_arch = "wasm32")))]

/// API registration never activates another reachable crate's registrar.
#[tokio::test]
async fn api_alone_does_not_register_reachable_session_crate() {
    let mut app = stackpress_server::ServerBuilder::new();
    let mut layers = stackpress_server::ConfigLayers::new();
    layers.value("configured", serde_json::json!({"api":{"endpoints":[{"method":"GET","route":"/public","event":"unused","type":"public"}]}})).unwrap();
    app.config_layers(layers).unwrap();
    stackpress_framework::register(&mut app).unwrap();
    let host = app.build().await.unwrap();
    let events: Vec<_> = host
        .listeners()
        .map(|listener| listener.pattern.as_str())
        .collect();
    assert!(events.contains(&"GET /public"));
    for absent in [
        "request",
        "me",
        "authorize",
        "email-send",
        "auth-signin",
        "auth-signup",
    ] {
        assert!(
            !events.contains(&absent),
            "unexpected session registrar: {absent}"
        );
    }
}
