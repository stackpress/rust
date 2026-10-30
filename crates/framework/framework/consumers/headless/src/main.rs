//! No configuration, client, Engine, provider or process binding is needed at startup.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = stackpress_server::ServerBuilder::new();
    app.plugin("framework", stackpress_framework::register)?;
    let server = app.build().await?;
    assert!(server.host().service::<stackpress_sql::Engine>().is_err());
    assert!(
        server
            .host()
            .service::<stackpress_framework::commands::Bindings>()
            .is_err()
    );
    assert!(
        server
            .host()
            .service::<stackpress_framework_session::Service>()
            .is_err()
    );
    assert!(
        !server
            .listeners()
            .any(|listener| listener.pattern == "request")
    );
    assert!(
        server
            .host()
            .service::<stackpress_framework::client::Client>()?
            .get(true)?
            .is_none()
    );
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    println!("slice11 headless consumer passed");
    Ok(())
}
