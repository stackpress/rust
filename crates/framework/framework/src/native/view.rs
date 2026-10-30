//! Source view-presence bridge; renderer/development host binding remains application-owned.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

/// Optional source view section, distinct from preferences/flash admission configuration.
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Section(pub Option<Value>);

/// Present native view configuration; not a renderer or a browser snapshot.
pub struct Settings {
    /// Original section, kept as configuration rather than implicitly projected public data.
    pub config: Value,
}

/// Queue only the present-view service and command; do not select a renderer or open assets.
pub(crate) fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("view", Section::default(), |_| Ok(()))?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    Ok(())
}

/// Publish real settings only when source configuration exists.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    if let Some(config) = start.config::<Section>()?.0.clone() {
        start.register(Arc::new(Settings { config }))?;
    }
    Ok(Flow::Continue)
}

/// Source build command remains explicit; provider setup and dev routes belong to M1.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    if start.config::<Section>()?.0.is_some() {
        super::commands::view_listen(start)?;
    }
    Ok(Flow::Continue)
}
