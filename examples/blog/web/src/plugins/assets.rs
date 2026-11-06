//! Source-style buffered asset request adapter for the app's explicit Rust build products.
use serde::{Deserialize, Serialize};
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::path::PathBuf;

/// Original assets root string, relative to the application's working directory.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Assets(pub PathBuf);

/// Select static bytes and the compiled browser products; registration performs no file IO.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("assets", Assets(".build/browser".into()), |_| Ok(()))?;
    app.on_listen(routes)
}
/// Match the original asset plugin's Request-stage contribution.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    start.on("request", asset)?;
    Ok(Flow::Continue)
}
/// Preserve prior body/status skips; native MIME and bytes come from actual source/build assets.
async fn asset(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if !matches!(res.body(), Body::Empty) || res.status().is_some_and(|status| status.code != 404) {
        return Ok(Flow::Continue);
    }
    let path = req.url.path();
    let (mime, bytes) = match path {
        "/styles/global.css" => (
            "text/css",
            format!(
                "{}\n{}\n{}",
                stackpress_leptos::components::BASE_STYLES,
                stackpress_framework_api::views::STYLES,
                include_str!("../../../assets/global.css")
            )
            .into_bytes(),
        ),
        "/styles/article.css" => (
            "text/css",
            include_bytes!("../../../assets/article.css").to_vec(),
        ),
        "/assets/blog.js" | "/assets/session.js" | "/assets/api-checkpoint.js" => (
            "text/javascript",
            include_bytes!("../../../bootstrap.js").to_vec(),
        ),
        "/assets/brand.svg" | "/logo.png" | "/icon.png" | "/favicon.ico" | "/favicon.png" => (
            "image/svg+xml",
            include_bytes!("../../../assets/brand.svg").to_vec(),
        ),
        path if path
            == format!(
                "/assets/{}",
                stackpress_framework_view::theme::BOOTSTRAP_FILE
            ) =>
        {
            (
                "text/javascript",
                stackpress_framework_view::theme::BOOTSTRAP_SOURCE
                    .as_bytes()
                    .to_vec(),
            )
        }
        path if path.starts_with("/assets/") => {
            let file = inv
                .config::<Assets>()?
                .0
                .join(path.trim_start_matches("/assets/"));
            if !file.exists() {
                return Ok(Flow::Continue);
            }
            let mime = match file.extension().and_then(|value| value.to_str()) {
                Some("js") => "text/javascript",
                Some("wasm") => "application/wasm",
                Some("json") => "application/json",
                _ => "application/octet-stream",
            };
            (
                mime,
                std::fs::read(file).map_err(stackpress_lib::Error::from_source)?,
            )
        }
        _ => return Ok(Flow::Continue),
    };
    res.set(mime, Body::Bytes(bytes))?;
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}
