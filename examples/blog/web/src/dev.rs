//! Optional development decoration for every HTML document, including auth and admin.

use leptos::prelude::*;
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext};

/// Register after application contributions so even a recovered error page gets the script.
/// # Errors
/// Preserves ordinary lifecycle registration failures.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(listen)
}

/// The response lifecycle has completed page selection; no public data contract is changed.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("response", decorate)?;
    Ok(Flow::Continue)
}

/// Preserve non-HTML bodies and all normal documents byte for byte.
async fn decorate(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if res.mimetype() == Some("text/html")
        && let Body::Text(html) = res.body()
        && let Some(html) = document(html)
    {
        res.html(html)?;
    }
    Ok(Flow::Continue)
}

/// Add Leptos's own AutoReload script only with xtask's explicit runtime port variable.
///
/// xtask sets both this app-specific port and Leptos's runtime watch switch.
/// Normal serving adds no script or placeholder nodes.
pub fn document(html: &str) -> Option<String> {
    std::env::var_os("LEPTOS_WATCH")?;
    let port = std::env::var("BLOG_DEV_RELOAD_PORT")
        .ok()?
        .parse::<u16>()
        .ok()?;
    if port == 0 || !html.contains("</head>") {
        return None;
    }
    let options = leptos::config::LeptosOptions::builder()
        .output_name("blog")
        .reload_port(u32::from(port))
        .build();
    let script = view! { <leptos::hydration::AutoReload options/> }.to_html();
    if script.is_empty() {
        return None;
    }
    // Keep Leptos's complete client protocol and patch engine; add only failure feedback.
    let script = script.replace("patch(msg.view);", include_str!("hot-reload.js"));
    // Hydration pages reload before CSR finishes. The normal Leptos messages stay intact.
    let script = script.replace(
        "if (msg.all) window.location.reload();",
        include_str!("reload-mode.js"),
    );
    // A newly connected tab may have loaded the last-good HTML while Cargo was compiling.
    let build = std::env::var("BLOG_DEV_BUILD_ID")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    let script = script.replace("/live_reload", &format!("/live_reload?build={build}"));
    Some(html.replacen(
        "</head>",
        &format!("<!--blog-dev-reload-->{script}<!--/blog-dev-reload--></head>"),
        1,
    ))
}
