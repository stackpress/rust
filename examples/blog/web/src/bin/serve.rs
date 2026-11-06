//! Explicit native web adapter. Development launches belong to devmetrics, never proof scripts.
use stackpress_server::ConfigLayers;
use stackpress_server_http::HttpServer;

/// Open only the configured existing SQLite file, then drain HTTP before closing its services.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let port = args
        .next()
        .ok_or("devmetrics-assigned port required")?
        .parse::<u16>()?;
    let path = args.next().unwrap_or_else(|| "config/develop.json".into());
    let mut layers = ConfigLayers::new();
    layers.json(&path, &std::fs::read_to_string(&path)?)?;
    // Auth link generation and the native HTTP origin agree with the managed assigned port.
    layers.value(
        "native web address",
        serde_json::json!({"server":{"port":port}}),
    )?;
    let seed = std::env::var("DATABASE_SEED").unwrap_or_else(|_| "abc123".into());
    let app = blog_web::client::build(layers, &seed, Default::default()).await?;
    let options = blog_web::http::network(port);
    let public_origin = options.http.base_url.clone();
    let bound = HttpServer::bind(app.clone(), options).await?;
    println!("ready {public_origin}");
    let mut signal_error = None;
    let report = bound
        .serve_until(async {
            signal_error = tokio::signal::ctrl_c().await.err();
        })
        .await;
    // The native host has drained all requests before the application closes hooks/database.
    blog_web::client::shutdown(&app).await?;
    if let Some(error) = signal_error {
        return Err(error.into());
    }
    if report.failure.is_some() {
        return Err("HTTP serving failed after drain".into());
    }
    Ok(())
}
