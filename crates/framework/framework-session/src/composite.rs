//! Original session package grouping, without changing the standalone constituent registrars.
//!
//! The main aggregate selects CSRF earlier, then views, this session composite, API and admin.
//! Missing configuration retains each constituent's own activation rules. Registration performs
//! no database, network or listener setup and grants no ownership of any route or event.

use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

/// Register auth, email, then identity, matching upstream `stackpress-session/src/plugin.ts`.
///
/// This calls the ordinary functions directly. The caller may select the whole group with
/// `app.plugin("session", composite::register)`, or call the same constituent functions itself.
/// `plugin::register` remains the existing identity-only entry point used by API consumers.
/// CSRF selection and native SQL/hook ownership remain explicit application responsibilities.
///
/// ```
/// use stackpress_framework_session::composite;
/// use stackpress_server::ServerBuilder;
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> stackpress_lib::Result<()> {
/// let mut app = ServerBuilder::new();
/// app.plugin("session", composite::register)?;
/// let server = app.build().await.map_err(|failure| failure.error)?;
/// // Source auth Listen events exist even when no auth Route config is present.
/// assert!(server.listeners().any(|listener| listener.pattern == "auth-signin"));
/// assert!(!server.listeners().any(|listener| listener.pattern == "me"));
/// # Ok(())
/// # }
/// ```
///
/// # Errors
/// Propagates the constituents' existing config/service/registration failures.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Keep source contribution order through Config, Listen and Route startup phases.
    crate::auth::register(app)?;
    stackpress_framework_email::plugin::register(app)?;
    crate::plugin::register(app)?;
    Ok(())
}
