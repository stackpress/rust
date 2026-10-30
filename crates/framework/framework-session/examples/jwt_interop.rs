//! Finite native token producer for the original-jose interoperability proof; no listener.
#[cfg(feature = "native")]
#[tokio::main]
async fn main() {
    use serde_json::json;
    use stackpress_framework_session::Service;
    let service = Service::default();
    let token = service
        .create(
            json!({"id":"interop","roles":["USER"],"custom":{"x":true}})
                .as_object()
                .unwrap()
                .clone(),
        )
        .await
        .unwrap();
    println!("{}", json!({"token":token}));
}
#[cfg(not(feature = "native"))]
fn main() {}
