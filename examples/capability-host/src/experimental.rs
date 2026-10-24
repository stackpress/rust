//! Feature-selected preview capability, reusing the original profile-search operation.
//!
//! The module declaration and application selection are both gated by `experimental`.
//! The compile-time guard below makes accidentally compiling this file in a disabled
//! build fail, proving stronger exclusion than merely skipping plugin registration.

#[cfg(not(feature = "experimental"))]
compile_error!("the experimental plugin module must be excluded when its feature is disabled");

use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};

/// Add the preview capability when the application has compiled this plugin.
///
/// # Errors
/// Returns startup-task registration failures; the app declares the profiles prerequisite.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // This module cannot register anything in a build where it is excluded.
    app.on_listen(listen_preview)?;
    Ok(())
}

/// Register an experimental alias without copying the underlying search implementation.
async fn listen_preview(start: &mut StartupContext) -> Result<Flow> {
    // No experimental HTTP exposure is added by this plugin.
    start.on("profile-preview", profile_preview)?;
    Ok(Flow::Continue)
}

/// Reuse the original operation and its errors/Abort while retaining the root lifecycle.
async fn profile_preview(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Nested delegation inherits the root's correlation and current Operation stage.
    inv.delegate("profile-search", req, res).await
}
