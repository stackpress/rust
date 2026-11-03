//! Source Application and persisted API Session records (separate from browser JWTs).
//!
//! Missing fields, explicit null and native Date values remain distinct. The merged
//! schema package owns their common native values/codecs; session retains Profile metadata.
#[cfg(feature = "sql")]
mod actions;
#[cfg(feature = "sql")]
mod batch;
#[cfg(feature = "sql")]
mod membership;
#[cfg(feature = "sql")]
mod query;
mod schema;
#[cfg(feature = "sql")]
mod store;
pub use schema::{
    ApplicationModel, ApplicationSchema, ModelKind, Schema, SessionModel, SessionSchema,
};
pub use stackpress_framework_schema::source::{
    Column, Kind, Record, Value, date, date_with_display,
};
/// Full or projected source Application, without converting missing keys to null.
pub type Application = Record;
/// Source create/update input permits partial records; preparation filters stored fields.
pub type ApplicationInput = Record;
/// Application with explicitly requested Profile fields.
pub type ApplicationExtended = Record;
/// Persisted API Session; never the JWT identity service.
pub type Session = Record;
/// Source partial persisted-session input.
pub type SessionInput = Record;
/// API Session with selected Application/Profile fields.
pub type SessionExtended = Record;
#[cfg(feature = "sql")]
pub use actions::{Actions, ApplicationActions, SessionActions};
#[cfg(feature = "sql")]
pub(crate) use query::source_text;
#[cfg(feature = "sql")]
pub use query::{StoreQuery, StoreSelector};
#[cfg(feature = "sql")]
pub use stackpress_framework_session::storage::{ActionError, BatchMode, BatchStatus, Result};
#[cfg(feature = "sql")]
pub use store::{ApplicationStore, SessionStore, Store};
