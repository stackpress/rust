//! Native source Store/Actions over the existing SQL engine and exclusive DataSession.
//!
//! Direct actions own only an Arc to the application engine. `_in` methods borrow an
//! explicit executor for batch affinity; no pool/transaction is hidden in a store.
//! Generic source errors, active defaults, partial updates and maintenance stay explicit.

mod query;
mod statement;
pub use statement::SourceStatement;
mod account;
mod actions;
mod batch;
mod store;
pub use account::{AccountActions, ActionOptions, PasswordConfig, is_email};
pub use actions::{Actions, AuthActions, ProfileActions};
pub use batch::{BatchMode, BatchStatus};
pub use query::{StoreQuery, StoreSelector};
pub use store::{AuthModel, AuthStore, ModelKind, ProfileModel, ProfileStore, Store};

pub use stackpress_framework_sql::source::{ActionError, SourceResult as Result};

#[cfg(feature = "auth-server")]
pub(crate) use account::password_error;
