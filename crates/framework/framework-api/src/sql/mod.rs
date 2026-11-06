//! Explicit source SQL event integration; never installs tables during startup.
//!
//! Select `sql`, register an app-owned Engine during Config, and select this registrar
//! alongside `api::register`. Without an Engine each event passes through as upstream.
//! Compose SQL alongside the API plugin using the same #14 registration convention.
//! The application supplies an already-open Engine in Config; installation is explicit.
//!
//! ```
//! # #[cfg(feature = "sql")] {
//! use stackpress_framework_api::{register, sql};
//! use stackpress_server::ServerBuilder;
//! let mut app = ServerBuilder::new();
//! app.plugin("api", register)?;
//! app.plugin("api-data", sql::register)?;
//! // Direct composition is `sql::register(&mut app)?` instead.
//! # }
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

pub mod defaults;
mod events;
pub use crate::models::{ApplicationActions, ApplicationStore, SessionActions, SessionStore};
pub use events::{Event, action, register, register_with};
pub use stackpress_framework_sql::source::SourceStatement;
