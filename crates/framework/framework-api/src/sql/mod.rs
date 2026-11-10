//! Explicit source SQL event integration; never installs tables during startup.
//!
//! Select `sql`, register an app-owned Engine during Config, and select this registrar
//! alongside `api::register`. Without an Engine each event passes through as upstream.
//! Compose SQL alongside the API plugin using the same #14 registration convention.
//! The application supplies an already-open Engine in Config; installation is explicit.
//!
//! ```
//! # #[cfg(feature = "sql")]
//! fn register_data<M: stackpress_framework_sql::source::ApiModels>(
//!     app: &mut stackpress_server::ServerBuilder,
//! ) -> stackpress_lib::Result<()>
//! where
//!     <M::Application as stackpress_framework_sql::source::SourceModel>::Actions:
//!         stackpress_framework_sql::source::SourceEventActions,
//!     <M::Session as stackpress_framework_sql::source::SourceModel>::Actions:
//!         stackpress_framework_sql::source::SourceEventActions,
//! {
//!     stackpress_framework_api::register_with::<M>(app)?;
//!     stackpress_framework_api::sql::register_with::<M>(app)
//! }
//! ```

pub mod defaults;
mod events;
pub use events::{Event, action, register_with};
pub use stackpress_framework_sql::source::SourceStatement;
