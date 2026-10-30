//! Explicit source constituent: publish the default mechanism, without a global guard.

use crate::{CsrfConfig, Service};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

/// Raw source section; missing or nonobject sections have no name/error overrides.
#[derive(Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CsrfSection(
    /// Original section; only object fields contribute typed name/error overrides.
    pub Value,
);

/// Select the CSRF mechanism even when its config section is absent.
/// No request listener or route is contributed; processes call the service explicitly.
///
/// ```
/// use stackpress_framework_csrf::{Service, plugin};
/// use stackpress_server::ServerBuilder;
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> stackpress_server::BuildResult<()> {
/// let mut app = ServerBuilder::new();
/// app.plugin("csrf", plugin::register)?;
/// let host = app.build().await?;
/// assert_eq!(host.host().service::<Service>()?.name(), "csrf");
/// assert_eq!(host.listeners().count(), 0);
/// # Ok(()) }
/// ```
/// # Errors
/// Preserves duplicate typed-section/service and invalid typed override errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("csrf", CsrfSection(Value::Null), |_| Ok(()))?;
    app.on_config(configure)?;
    Ok(())
}

/// Decode string overrides at Config, then publish the stateless mechanism once.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let section = &start.config::<CsrfSection>()?.0;
    // Source path lookups have no overrides on an absent/nonobject section.
    // Typed string fields retain the existing direct API's Rust configuration boundary.
    let config = if section.is_object() {
        serde_json::from_value(section.clone())?
    } else {
        CsrfConfig::default()
    };
    start.register(Arc::new(Service::new(config)))?;
    Ok(Flow::Continue)
}
