//! Finite native token producer for the original-jose interoperability proof; no listener.
#[cfg(feature = "native")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use serde_json::json;
    use stackpress_framework_session::Service;
    let service = Service::default();
    let token = service
        .create(serde_json::Map::from_iter([
            ("id".into(), json!("interop")),
            ("roles".into(), json!(["USER"])),
            ("custom".into(), json!({"x":true})),
        ]))
        .await?;
    println!("{}", json!({"token":token}));
    Ok(())
}
#[cfg(not(feature = "native"))]
fn main() {}
