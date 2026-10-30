//! Auth constituent selection, separate from the identity and Lane B email registrars.

use super::{AuthSection, Hooks, events, pages};
use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

/// Select reusable auth events and config-gated auth process routes.
///
/// Missing auth config still installs the three source events. No Engine, email,
/// session or CSRF service is required during startup; each process reads its actual
/// dependencies at the source access point. Select the CSRF/identity constituents
/// independently. M3 supplies views; M4 wires the reviewed source composite.
///
/// ```
/// use std::sync::Arc;
/// use stackpress_framework_session::auth::{self, Hooks};
/// use stackpress_server::ServerBuilder;
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> stackpress_lib::Result<()> {
/// let mut app = ServerBuilder::new();
/// app.plugin("auth", auth::register)?;
/// let server = Arc::new(app.build().await.map_err(|failure| failure.error)?);
/// let hooks = server.host().service::<Hooks>()?;
/// hooks.bind(&server, tokio::runtime::Handle::current());
/// let result = server.resolve("auth-signout", serde_json::json!({})).await?;
/// assert_eq!(result.response.code, Some(200));
/// hooks.shutdown().await;
/// # Ok(())
/// # }
/// ```
/// # Errors
/// Preserves existing duplicate typed-section/service and registration errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("auth", AuthSection::default(), |_| Ok(()))?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    app.on_route(routes)?;
    Ok(())
}
/// Publish only the hook owner; the application binds its finished server after build.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(Hooks::default()))?;
    Ok(Flow::Continue)
}
/// Source auth listener order is independent of the auth route config gate.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("auth-signup", events::signup)?;
    start.on("auth-signin", events::signin)?;
    start.on("auth-signout", events::signout)?;
    Ok(Flow::Continue)
}
/// Install source GETs first, then POSTs, retaining multiple matching route handlers.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    let Some(config) = &start.config::<AuthSection>()?.0 else {
        return Ok(Flow::Continue);
    };
    let base = config.base.as_deref().unwrap_or("/auth").to_owned();
    start.get(&format!("{base}/signup"), pages::signup)?;
    start.get(&format!("{base}/signin"), pages::signin)?;
    start.get(
        &format!("{base}/signin/2fa/:profile/:auth/:challenge"),
        pages::two_factor,
    )?;
    start.get(
        &format!("{base}/signin/link/:auth/:challenge"),
        pages::magic_link,
    )?;
    start.get(&format!("{base}/signin/email"), pages::email)?;
    start.get(&format!("{base}/signin/otp/:auth/:challenge"), pages::otp)?;
    start.get(&format!("{base}/signin/phone"), pages::phone)?;
    start.get(&format!("{base}/signin/username"), pages::username)?;
    start.get(&format!("{base}/signout"), pages::signout)?;
    start.post(&format!("{base}/signup"), pages::signup)?;
    start.post(
        &format!("{base}/signin/2fa/:profile/:auth/:challenge"),
        pages::two_factor,
    )?;
    start.post(
        &format!("{base}/signin/link/:auth/:challenge"),
        pages::magic_link,
    )?;
    start.post(&format!("{base}/signin/email"), pages::email)?;
    start.post(&format!("{base}/signin/otp/:auth/:challenge"), pages::otp)?;
    start.post(&format!("{base}/signin/phone"), pages::phone)?;
    start.post(&format!("{base}/signin/username"), pages::username)?;
    #[cfg(feature = "leptos-ssr")]
    crate::presentation::auth_routes(start, &base)?;
    Ok(Flow::Continue)
}
