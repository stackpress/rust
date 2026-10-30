//! Source-compatible API mechanics over existing Stackpress request/event contracts.
//!
//! M0 exposes optional configuration, header/data/predicate helpers, nested request dispatch
//! and an inline fetch boundary. It does not mount endpoints, OAuth, management pages or SQL
//! operations yet. The later stages retain G42/G43 source behavior, not additional guards.
//! Nothing here binds a listener, opens a database or starts a background task.
//!
//! Select the registration function through the host's normal plugin mechanism:
//!
//! ```
//! use stackpress_server::ServerBuilder;
//! let mut app = ServerBuilder::new();
//! app.plugin("api", stackpress_framework_api::register)?;
//! // M0 claims optional configuration; later milestones supply endpoint integration.
//! # Ok::<(), stackpress_lib::Error>(())
//! ```
//!
//! Direct composition uses the same function, without selecting a nested plugin:
//!
//! ```
//! use stackpress_lib::Result;
//! use stackpress_server::ServerBuilder;
//! fn application(app: &mut ServerBuilder) -> Result<()> {
//!     stackpress_framework_api::register(app)?;
//!     Ok(())
//! }
//! let mut app = ServerBuilder::new();
//! app.plugin("application", application)?;
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

pub mod config;
pub mod helpers;

use std::sync::Arc;

use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};

pub use config::{ApiConfig, ApiEndpoint, ApiScope, ApiType, ApiWebhook, Cors};

/// An explicitly selected, present API configuration; absence publishes no such service.
///
/// This M0 marker makes activation observable without prematurely implementing route phases.
pub struct Service {
    /// The source configuration, unchanged by an invented admission/default policy.
    pub config: ApiConfig,
}

/// Claim an optional API section and queue its configuration-only activation.
///
/// Missing or null `api` remains disabled, with no API service or runtime listener. Present
/// configuration is decoded with the existing host. Endpoint/OAuth/webhook registration is
/// deliberately left to the subsequent released milestones.
///
/// # Errors
/// Returns existing duplicate-section/registration errors. Build retains typed decode/service
/// errors and publishes no partial Server. This function performs no external I/O.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // A null lowest layer permits absence instead of demanding configured resources.
    app.config_section("api", None::<ApiConfig>, config::validate)?;
    app.on_config(configure)?;
    Ok(())
}

/// Publish activation only after the optional typed section has actually been supplied.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    // Clone only the present immutable configuration before mutably borrowing startup.
    if let Some(config) = start.config::<Option<ApiConfig>>()?.clone() {
        start.register(Arc::new(Service { config }))?;
    }
    Ok(Flow::Continue)
}
