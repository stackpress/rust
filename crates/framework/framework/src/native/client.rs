//! Nullable compiled-client service, published at the source Config priority ten.

use super::CompiledClient;
use stackpress_lib::{Error, Result};
#[cfg(feature = "schema-client")]
use stackpress_lib::{Flow, queue::task};
#[cfg(feature = "schema-client")]
use stackpress_server::{ServerBuilder, StartupContext, StartupSpec, StartupStage};
#[cfg(feature = "schema-client")]
use std::sync::Arc;

/// Source client lookup with no executable loader, database or schema-generation effect.
pub struct Client {
    /// Missing generated client remains absent; an empty catalog is still present.
    catalog: Option<&'static CompiledClient>,
}

impl Client {
    /// Return a nullable catalog, or fail a required lookup as the source import would.
    /// # Errors
    /// Returns a native missing-client error only when `nullable` is false and no catalog exists.
    pub fn get(&self, nullable: bool) -> Result<Option<&'static CompiledClient>> {
        // Do not manufacture a successful empty import for a missing generated package.
        if self.catalog.is_none() && !nullable {
            return Err(Error::new("compiled application client is unavailable"));
        }
        Ok(self.catalog)
    }
}

/// Publish the static client before default-priority constituent configuration.
#[cfg(feature = "schema-client")]
pub(crate) fn register(
    app: &mut ServerBuilder,
    catalog: Option<&'static CompiledClient>,
) -> Result<()> {
    // Keep source priority 10; service construction owns no IO or model-descriptor evaluation.
    app.startup(
        StartupSpec::new(StartupStage::Config)
            .priority(10)
            .label("schema-client"),
        task(move |start: &mut StartupContext| {
            Box::pin(async move {
                start.register(Arc::new(Client { catalog }))?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    app.on_listen(super::commands::schema_listen)?;
    Ok(())
}
