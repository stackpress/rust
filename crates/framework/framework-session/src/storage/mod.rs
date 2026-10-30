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
pub use account::{ActionOptions, PasswordConfig, is_email};
pub use actions::{Actions, AuthActions, ProfileActions};
pub use batch::{BatchMode, BatchStatus};
pub use query::{StoreQuery, StoreSelector};
pub use store::{AuthModel, AuthStore, ModelKind, ProfileModel, ProfileStore, Store};

use crate::columns::Record;
use std::{error::Error, fmt};

/// Fallible source action/storage operation.
pub type Result<T> = std::result::Result<T, ActionError>;

/// Source exception fields with an independently retained native cause.
#[derive(Debug)]
pub struct ActionError {
    /// Source HTTP-style numeric code, independent of transport.
    pub code: u16,
    /// Source exception message, including its original capitalization.
    pub error: String,
    /// Source field errors; absent errors remain an empty map in this native type.
    pub errors: Record,
    /// Original native error, never lost during conversion.
    cause: Option<Box<dyn Error + Send + Sync>>,
}
impl ActionError {
    /// Construct the original field-error response without choosing route behavior.
    pub fn new(code: u16, error: impl Into<String>, errors: Record) -> Self {
        Self {
            code,
            error: error.into(),
            errors,
            cause: None,
        }
    }
    /// Retain a typed failure under an authored action message.
    fn native(cause: impl Error + Send + Sync + 'static) -> Self {
        Self {
            code: 500,
            error: cause.to_string(),
            errors: Record::new(),
            cause: Some(Box::new(cause)),
        }
    }
    /// Source status phrase for the relevant exception codes.
    pub fn status(&self) -> &'static str {
        match self.code {
            400 => "Bad Request",
            401 => "Unauthorized",
            404 => "Not Found",
            _ => "Internal Server Error",
        }
    }
}
impl fmt::Display for ActionError {
    /// Display the exception message once.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.error)
    }
}
impl Error for ActionError {
    /// Preserve the original SQL/schema cause for native callers.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|cause| cause as _)
    }
}
impl From<stackpress_sql::SqlError> for ActionError {
    /// Keep driver failures distinct from preflight 400 errors.
    fn from(cause: stackpress_sql::SqlError) -> Self {
        Self::native(cause)
    }
}
impl From<stackpress_framework_sql::DataError> for ActionError {
    /// Preserve errors from the selected exclusive executor.
    fn from(cause: stackpress_framework_sql::DataError) -> Self {
        Self::native(cause)
    }
}
impl From<stackpress_framework_schema::SchemaError> for ActionError {
    /// Retain source codec/provider failure rather than inventing a validation field.
    fn from(cause: stackpress_framework_schema::SchemaError) -> Self {
        Self::native(cause)
    }
}

#[cfg(feature = "auth-server")]
pub(crate) use account::password_error;
