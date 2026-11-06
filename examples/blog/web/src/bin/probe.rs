//! Finite stdin/stdout HTTP proof host. No socket is opened; files belong to its caller.
use serde::Deserialize;
use serde_json::json;
use stackpress_server::{
    ConfigLayers,
    http::{HttpAdapter, HttpOutput},
};
use std::io::{BufRead, Write};

/// One caller-buffered request, retaining repeated response headers on the wire.
#[derive(Deserialize)]
struct Input {
    method: String,
    path: String,
    #[serde(default)]
    headers: Vec<(String, String)>,
    #[serde(default)]
    body: Vec<u8>,
}

/// Explicit fixture setup/population precedes the ordinary complete application startup.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("owned fixture directory required")?,
    );
    std::fs::create_dir_all(&root)?;
    let path = root.join("blog.sqlite");
    // The copied-out nickname proof upgrades a real seeded file before opening this host.
    let existing = match std::env::args().nth(2).as_deref() {
        None => false,
        Some("--existing") => true,
        _ => return Err("only --existing is accepted after the fixture directory".into()),
    };
    if !existing {
        blog_web::plugins::store::setup(&path).await?;
    }
    let mut layers = ConfigLayers::new();
    layers.value("owned finite HTTP fixture",json!({"sqlite":{"path":path},"client":{"revisions":root.join("revisions")},"api":{"webhooks":[]}}))?;
    let app = blog_web::client::build(layers, "abc123", Default::default()).await?;
    let mut request = stackpress_lib::Request::new();
    let mut response = stackpress_lib::Response::new();
    if !existing {
        app.emit("populate", &mut request, &mut response).await?;
    }
    let adapter = HttpAdapter::new(blog_web::http::options("http://blog.test/"))?;
    let stdin = std::io::stdin();
    // Only this finite child is kept alive; EOF closes hooks/SQLite before process exit.
    for line in stdin.lock().lines() {
        let input: Input = serde_json::from_str(&line?)?;
        let mut request = http::Request::builder()
            .method(input.method.as_str())
            .uri(input.path);
        for (key, value) in input.headers {
            request = request.header(key, value);
        }
        let reply = adapter.handle(&app, request.body(input.body)?).await?;
        let HttpOutput::Response(response) = reply.output else {
            return Err("HTTP output unavailable".into());
        };
        let headers = response
            .headers()
            .iter()
            .map(|(key, value)| (key.as_str(), value.to_str().unwrap_or_default()))
            .collect::<Vec<_>>();
        println!(
            "{}",
            json!({"status":response.status().as_u16(),"headers":headers,"body":response.body()})
        );
        std::io::stdout().flush()?;
    }
    blog_web::client::shutdown(&app).await?;
    Ok(())
}
