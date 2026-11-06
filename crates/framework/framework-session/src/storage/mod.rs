//! Native source Store/Actions over the existing SQL engine and exclusive DataSession.
//!
//! Direct actions own only an Arc to the application engine. `_in` methods borrow an
//! explicit executor for batch affinity; no pool/transaction is hidden in a store.
//! Generic source errors, active defaults, partial updates and maintenance stay explicit.

mod account;
pub use account::{AccountActions, PasswordConfig, is_email};
pub use stackpress_framework_models::{
    AuthActions, AuthModel, AuthStore, Models as DefaultModels, ProfileActions, ProfileModel,
    ProfileStore,
};
pub use stackpress_framework_sql::source::{
    ActionError, BatchMode, BatchStatus, NativeActions as Actions, NativeStore as Store,
    SourceResult as Result, SourceStatement, StoreQuery, StoreSelector,
};

#[cfg(feature = "auth-server")]
pub(crate) use account::password_error;
