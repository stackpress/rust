//! Source-compatible API mechanics over existing Stackpress request/event contracts.
//!
//! Configured public/application/API-session endpoints and source CORS callbacks run over
//! existing raw dispatch, with late inline webhook/fetch integration. Nonempty endpoints also
//! mount the OAuth/token processes; consent rendering remains a later contribution. Native
//! models and SQL actions are selected through features and a separate SQL registrar.
//! G42/G43 source behavior governs each selected process.
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
#[cfg(not(target_arch = "wasm32"))]
pub mod cors;
#[cfg(not(target_arch = "wasm32"))]
pub mod endpoints;
#[cfg(not(target_arch = "wasm32"))]
pub mod helpers;
#[cfg(not(target_arch = "wasm32"))]
pub mod pages;
#[cfg(not(target_arch = "wasm32"))]
pub mod plugin;
#[cfg(not(target_arch = "wasm32"))]
pub mod routes;
#[cfg(not(target_arch = "wasm32"))]
pub mod webhooks;

pub use config::{ApiConfig, ApiEndpoint, ApiScope, ApiType, ApiWebhook, Cors};
#[cfg(not(target_arch = "wasm32"))]
pub use plugin::register;
#[cfg(feature = "ui")]
pub mod views;

/// An explicitly selected, present API configuration; absence publishes no such service.
///
/// Published in Config before the selected endpoint inventory is registered in Route.
pub struct Service {
    /// The source configuration, unchanged by an invented admission/default policy.
    pub config: ApiConfig,
}

#[cfg(all(feature = "codegen", not(target_arch = "wasm32")))]
pub mod codegen;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod models;
#[cfg(all(feature = "sql", not(target_arch = "wasm32")))]
pub mod sql;
