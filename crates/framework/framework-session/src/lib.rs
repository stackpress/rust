//! Configured identity mechanisms following the original Stackpress Session contract.
//!
//! Default builds expose portable configuration and permission data only. `native` adds
//! JWT instances and storage codec seams; `server` adds ordinary identity registration.
//! No configuration installs default protection: hosts and individual processes decide.

mod types;
pub use types::{
    SessionConfig, SessionData, SessionPermission, SessionPermissionList, SessionRoute,
    SessionTokenData,
};
pub mod permissions;

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod jwt;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod session;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use session::{Clock, Service, Session, SessionError, SessionResult};
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod codecs;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod plugin;
