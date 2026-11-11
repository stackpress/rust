//! Construct the production HTTP content client without DNS, a listener or a request send.

/// A standalone email build supplies reqwest's TLS provider for HTTPS request construction.
#[test]
fn native_http_client_constructs_without_an_ambient_tls_provider() {
    let client = super::http_client().unwrap();
    let request = client
        .get("https://content.example.invalid/attachment")
        .build()
        .unwrap();
    assert_eq!(request.method(), reqwest::Method::GET);
    assert_eq!(
        request.url().as_str(),
        "https://content.example.invalid/attachment"
    );
}
