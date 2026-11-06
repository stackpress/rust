//! Original app page operations, lower-priority views, and response-to-error hook.
pub mod article;
pub mod error;
pub mod home;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{RouteSpec, ServerBuilder, StartupContext};

/// Select ordinary functions before compilation; no runtime import or route uniqueness policy.
/// # Errors
/// Preserves registration/startup failures.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(listen)?;
    app.on_route(routes)
}
/// Error and response hooks retain source ordering within the app contribution.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("error", error::page)?;
    start.on("response", response)?;
    Ok(Flow::Continue)
}
/// A response error is offered to the same raw error event; an existing body is its own guard.
async fn response(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if res.error().is_some() {
        inv.emit("error", req, res).await?;
    }
    Ok(Flow::Continue)
}
/// Real model handlers run first; the existing priority/FIFO contract orders view contributions.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    start.get("/", home::page)?;
    start.get("/articles/:slug", article::page)?;
    start.route_with(
        RouteSpec::new(stackpress_lib::MethodPattern::GET, "/").priority(-100),
        home::view,
    )?;
    start.route_with(
        RouteSpec::new(stackpress_lib::MethodPattern::GET, "/articles/:slug").priority(-100),
        article::view,
    )?;
    Ok(Flow::Continue)
}
