//! Finite service-API bridge for real browser proofs; never binds or opens a socket.

use http_body_util::{BodyExt, Full};
use hyper::{Request, body::Bytes};
use stackpress_example_profile_ui_gateway::{Gateway, GatewayLimits, assets::AssetBundle};
use stackpress_framework_view::BasePath;
use std::{io::Write, path::PathBuf};

/// Emit one JSON status/header line then exact response bytes, and exit.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let method = args.first().ok_or("method required")?;
    let uri = args.get(1).ok_or("URI required")?;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist/hydrate");
    let config = std::env::var_os("PROFILE_UI_CONFIG").map(PathBuf::from);
    let layers = stackpress_example_profile_ui_web::backend::configuration(config.as_deref())?;
    let host = Gateway::with_config(
        BasePath::new(std::env::var("PROFILE_UI_BASE").unwrap_or_default())?,
        AssetBundle::load(&path)?,
        GatewayLimits::default(),
        layers,
    )
    .await?;
    let request = Request::builder()
        .method(method.as_str())
        .uri(uri)
        .header(
            "content-type",
            if method == "POST" {
                "application/json"
            } else {
                "text/plain"
            },
        )
        .body(Full::new(Bytes::from(
            args.get(2).cloned().unwrap_or_default(),
        )))?;
    let response = host.handle(request).await;
    let headers = response
        .headers()
        .iter()
        .map(|(k, v)| {
            (
                k.as_str().to_owned(),
                v.to_str().unwrap_or_default().to_owned(),
            )
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    let meta = serde_json::json!({"status":response.status().as_u16(),"headers":headers});
    let body = response.into_body().collect().await?.to_bytes();
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{meta}")?;
    stdout.write_all(&body)?;
    Ok(())
}
