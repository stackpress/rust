//! Generated default native source schemas.
#![doc = include_str!("../README.md")]

/// Checked-in native output from the public source generator; never edited by hand.
#[cfg(not(target_arch = "wasm32"))]
// Managed partition indexes have no Rustdoc; emitted model/catalog items are documented.
#[allow(missing_docs)]
pub mod generated {
    // Load managed partitions independently; the writer's generated index stays unchanged.
    pub mod source;
    #[cfg(feature = "sql")]
    pub mod stores;
}
#[cfg(not(target_arch = "wasm32"))]
pub use generated::source::{
    application::ApplicationSchema, auth::AuthSchema, profile::ProfileSchema,
    session::SessionSchema,
};

/// Generated native model family and its ordinary stores/actions; importing registers nothing.
#[cfg(all(feature = "sql", not(target_arch = "wasm32")))]
pub use generated::stores::{
    application::{ApplicationActions, ApplicationModel, ApplicationStore},
    auth::{AuthActions, AuthModel, AuthStore},
    family::Models,
    profile::{ProfileActions, ProfileModel, ProfileStore},
    session::{SessionActions, SessionModel, SessionStore},
};
