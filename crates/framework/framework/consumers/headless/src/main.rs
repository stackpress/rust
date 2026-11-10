//! Explicit model family with no configuration, client catalog, Engine or command provider.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = stackpress_server::ServerBuilder::new();
    // G59 has no stock family. Compose explicit constituents to keep the nullable client.
    stackpress_framework::config::register(&mut app)?;
    stackpress_framework::commands::register_server(&mut app)?;
    stackpress_framework::client::register(&mut app, None)?;
    stackpress_framework::language::register(&mut app)?;
    stackpress_framework_csrf::plugin::register(&mut app)?;
    stackpress_framework::commands::register_sql(&mut app)?;
    stackpress_framework::view::register(&mut app)?;
    stackpress_framework_session::composite::register_with::<models::stores::family::Models>(
        &mut app,
    )?;
    stackpress_framework_api::register_with::<models::stores::family::Models>(&mut app)?;
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

#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "../../generated-models/mod.rs"]
mod models;
