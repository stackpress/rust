//! Example-only admission and buffered review chrome; never transforms streamed bodies.
use super::{Panel, actor};
use crate::options::Options;
use stackpress_lib::{Body, Error, Flow, Result, handler};
use stackpress_server::ServerBuilder;
use std::sync::{Arc, atomic::Ordering};
/// Install caller/fault admission before dispatch and public chrome after rendering.
/// # Errors
/// Returns lifecycle listener registration failures before host startup.
pub(super) fn configure(
    builder: &mut ServerBuilder,
    options: &Options,
    panel: Arc<Panel>,
) -> Result<()> {
    let hooks = panel.clone();
    let admin = options.path("/generated-admin/");
    let preferences = options.path("/ui-preferences/ui-preferences");
    builder.on_priority(
        "request",
        1000,
        handler(move |req, _, _| {
            let hooks = hooks.clone();
            let admin = admin.clone();
            let preferences = preferences.clone();
            Box::pin(async move {
                // Browser-provided fixture headers are never authority, even on deep links.
                let caller = actor(req.headers.get("cookie").unwrap_or_default());
                req.headers.remove("x-admin-actor");
                req.headers.set("x-admin-actor", caller)?;
                req.headers.remove("x-fixture-caller");
                req.headers.remove("x-fixture-csrf");
                // Preference failure is consumed before the real action can issue cookies.
                let path = req.url.path();
                if path == preferences && hooks.preferences.swap(false, Ordering::SeqCst) {
                    hooks.set_status("007: save refused; draft must remain");
                    return Err(Error::new("fixture preference refusal")
                        .with_code(503)
                        .with_public_message("Preferences unavailable"));
                }
                let data_read = path.starts_with(&admin)
                    && !path.contains("/assets/")
                    && (path.ends_with("/search")
                        || path.ends_with("/get")
                        || path.ends_with("/detail")
                        || path.ends_with("/preview"));
                if data_read {
                    // Take the one-shot token and release its lock before any async delay.
                    let scenario = hooks.read.lock().expect("read lock").take();
                    if scenario == Some("slow") {
                        tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                        hooks.set_status("003: delayed read released");
                    }
                    if scenario == Some("failed") {
                        hooks.set_status("003: read failed safely");
                        return Err(Error::new("fixture read refusal")
                            .with_code(503)
                            .with_public_message("Unable to load records"));
                    }
                }
                Ok(Flow::Continue)
            })
        }),
    )?;
    let paused = panel.clone();
    let home = options.path("/review");
    builder.on_priority("response", -1000, handler(move |_, res, _| {
        let home = home.clone(); let paused = paused.paused.load(Ordering::SeqCst);
        Box::pin(async move {
            res.headers_mut()?.set("cache-control", "no-store")?;
            // Stream bodies remain exclusive and untouched; buffered HTML may gain review chrome.
            if let Body::Text(html) = res.body()
                && html.starts_with("<!") && html.contains("<head>") {
                    let html = html.replacen("<head>", &format!("<head><meta name=\"review-pause\" content=\"{paused}\">"), 1)
                        .replacen("<body>", &format!("<body><nav aria-label=\"Review session\"><a href=\"{home}\" target=\"_blank\" rel=\"noopener\">Review fixtures (opens a second tab)</a></nav>"), 1);
                    // Response::html stages 200; restore the already selected public status.
                    let status = res.status().map(|status| status.code).unwrap_or(200);
                    res.html(html)?; res.status_code(status, None)?;
            }
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}
