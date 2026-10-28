//! Document-only error representation; API/download and core lifecycle policy stay unchanged.
use stackpress_framework_admin::pages::Action;
use stackpress_lib::{Flow, Method, Request, Result, handler};
use stackpress_server::ServerBuilder;

/// Match the compiled document vocabulary using trusted mount and normalized method/path.
fn document_model(request: &Request, mount: &str) -> Option<&'static str> {
    if request.method != Method::GET && request.method != Method::HEAD {
        return None;
    }
    // Reuse the exact host parser: API prefixes, unknown models and extra segments fail.
    let (module, name) = super::route(request, mount, false).ok()?;
    let action = super::action(name).ok()?;
    if action == Action::Export {
        return None;
    }
    // The generator does not register remove/restore pages for models without soft removal.
    if matches!(action, Action::Remove | Action::Restore) && !super::spec(module).ok()?.soft_remove
    {
        return None;
    }
    Some(module)
}

/// Install after ordinary fresh-error staging, retaining diagnostics and original status.
/// # Errors
/// Returns error-listener registration failures before host startup.
pub(super) fn register(builder: &mut ServerBuilder, mount: String) -> Result<()> {
    builder.on_priority(
        "error",
        -1000,
        handler(move |request, response, _| {
            let mount = mount.clone();
            Box::pin(async move {
                let Some(module) = document_model(request, &mount) else {
                    return Ok(Flow::Continue);
                };
                let status = response.status().map(|status| status.code).unwrap_or(500);
                if status < 400 {
                    return Ok(Flow::Continue);
                }
                // Body selection preserves the original diagnostic but stages a 200 status.
                // Restore the error code explicitly; never mark a denial as successful.
                response.html(crate::render::failure_document(
                    status,
                    &format!("{mount}{module}/search"),
                ))?;
                response.status_code(status, None)?;
                response.headers_mut()?.set("cache-control", "no-store")?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    Ok(())
}
