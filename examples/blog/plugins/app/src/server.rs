//! Original app page operations, lower-priority views, and response-to-error hook.
use crate::Kind;
use crate::pages::{article, error, home};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;
use stackpress_server::{RouteSpec, ServerBuilder, StartupContext};

/// Select app handlers and the host's native projection before compilation.
///
/// `render` owns host configuration/SSR. It is published as an immutable function pointer
/// during configuration and called only by the lower-priority view/error contributions.
/// Registration performs no IO and retains the existing route priorities and FIFO hooks.
/// # Errors
/// Preserves registration/startup failures.
pub fn register(app: &mut ServerBuilder, render: RenderPage) -> Result<()> {
    // Keep the host callback out of browser builds and avoid a reverse crate dependency.
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            Box::pin(async move {
                start.register(std::sync::Arc::new(Renderer(render)))?;
                Ok(Flow::Continue)
            })
        },
    ))?;
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

/// Native page projection supplied by the selecting host, with ordinary invocation lifetimes.
pub type RenderPage = for<'a> fn(
    Kind,
    &'a mut Request,
    &'a mut Response,
    Invocation<'a>,
) -> stackpress_lib::HandlerFuture<'a, Result<Flow>>;

/// Only the app's view callbacks use the selected host projection; never browser code.
struct Renderer(RenderPage);

/// Forward one page to the host's config/snapshot/SSR boundary without changing its result.
pub(crate) async fn render(
    kind: Kind,
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Copy the immutable function pointer before forwarding the original invocation borrow.
    let render = inv.service::<Renderer>()?.0;
    render(kind, req, res, inv).await
}
