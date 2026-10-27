//! A small trusted native query app and a driver-free view of its exact generated contracts.
//!
//! The independent workspace tool owns generation and explicit maintenance commands. This
//! application only reads its selected SQLite schema; importing it never installs a database.

/// Generated source selected at compile time to match the deliberately applied revision.
#[cfg(all(feature = "native", not(feature = "evolved")))]
#[path = "generated/v1/mod.rs"]
#[allow(
    missing_docs,
    reason = "aggregate module declarations; emitted leaves are documented"
)]
pub mod generated;
/// Generated source after the explicit rename and nullable addition.
#[cfg(all(feature = "native", feature = "evolved", not(feature = "expanded")))]
#[path = "generated/v2/mod.rs"]
#[allow(
    missing_docs,
    reason = "aggregate module declarations; emitted leaves are documented"
)]
pub mod generated;
/// Generated source with the additional model, which has no automatic recovery reverse.
#[cfg(all(feature = "native", feature = "expanded"))]
#[path = "generated/v3/mod.rs"]
#[allow(
    missing_docs,
    reason = "aggregate module declarations; emitted leaves are documented"
)]
pub mod generated;
/// Pure source consumer; this feature path has no SQL driver, server or tool dependency.
#[cfg(not(feature = "native"))]
#[path = "pure.rs"]
pub mod generated;

#[cfg(feature = "native")]
pub mod queries;

#[cfg(feature = "native")]
mod statements;
