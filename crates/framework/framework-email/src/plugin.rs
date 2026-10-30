//! Ordinary email constituent registrar, usable alone or in the session composite.
//!
//! Selection claims only its typed config section. Listen contributes `email-send`
//! only when source config is truthy. No route/request/global guard is installed.

use crate::{
    NativeFactory, TransportFactory,
    events::{EMAIL_SEND, EmailSend},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_lib::{Flow, Invocation, Request, Response, Result, handler, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

/// Lossless raw email section; decode transport variants only when the event needs them.
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EmailSection(
    /// Original section, including missing/falsy modes represented by null.
    pub Value,
);

/// Select the source email constituent with built-in native adapters/provider gap errors.
/// No connection, process or delivery occurs at registration/startup.
///
/// ```
/// use stackpress_framework_email::plugin;
/// use stackpress_server::ServerBuilder;
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> stackpress_server::BuildResult<()> {
/// let mut app = ServerBuilder::new();
/// app.plugin("email", plugin::register)?;
/// let server = app.build().await?;
/// assert_eq!(server.listeners().count(), 0); // Missing config is inactive.
/// # Ok(()) }
/// ```
///
/// # Errors
/// Propagates existing section and listener registration errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    register_with_factory(app, Arc::new(NativeFactory::default()))
}

/// Select the same source constituent with an application-owned provider/fake factory.
/// Empty object config activates it; missing/falsy config contributes no listener.
/// # Errors
/// Propagates existing section and listener registration errors.
pub fn register_with_factory(
    app: &mut ServerBuilder,
    factory: Arc<dyn TransportFactory>,
) -> Result<()> {
    // Server's typed section/default machinery represents absent config with null.
    // It has no registration-time raw config view; deferring the has/truthy checks
    // to Listen is a Rust registration-mechanics adaptation with the same inventory.
    app.config_section("email", EmailSection(Value::Null), |_| Ok(()))?;
    app.on_listen(task(move |start: &mut StartupContext| {
        let factory = factory.clone();
        Box::pin(async move {
            let config = start.config::<EmailSection>()?.0.clone();
            // Test source truthiness only, without eagerly validating unused SMTP options.
            if !crate::options::truthy(&config) {
                return Ok(Flow::Continue);
            }
            let event = EmailSend::new(Some(config), factory);
            start.on(
                EMAIL_SEND,
                handler(
                    move |req: &mut Request, res: &mut Response, _: Invocation<'_>| {
                        let event = event.clone();
                        Box::pin(async move { event.handle(req, res).await })
                    },
                ),
            )?;
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}
