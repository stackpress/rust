//! Source-compatible API mechanics over existing Stackpress request/event contracts.
//!
//! Configured public/application/API-session endpoints and source CORS callbacks run over
//! existing raw dispatch, with late inline webhook/fetch integration. OAuth, management and SQL retain their
//! later milestones. G42/G43 source behavior governs each selected process.
//! Nothing here binds a listener, opens a database or starts a background task.
//!
//! Select the registration function through the host's normal plugin mechanism:
//!
//! ```
//! use stackpress_server::ServerBuilder;
//! let mut app = ServerBuilder::new();
//! app.plugin("api", stackpress_framework_api::register)?;
//! // Missing API configuration leaves this selection inert.
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
pub mod cors;
pub mod endpoints;
pub mod helpers;
pub mod plugin;
pub mod routes;
pub mod webhooks;

pub use config::{ApiConfig, ApiEndpoint, ApiScope, ApiType, ApiWebhook, Cors};
pub use plugin::register;

/// An explicitly selected, present API configuration; absence publishes no such service.
///
/// Published in Config before the selected endpoint inventory is registered in Route.
pub struct Service {
    /// The source configuration, unchanged by an invented admission/default policy.
    pub config: ApiConfig,
}
