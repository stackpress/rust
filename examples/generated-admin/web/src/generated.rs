//! Native generated store graph; the shared/browser crate includes only contracts and UI.
#[path = "../../shared/src/generated/contracts/mod.rs"]
#[allow(
    missing_docs,
    reason = "the same writer-owned contract source compiles with native operations"
)]
pub mod contracts;
#[path = "../../shared/src/generated/ui/pages_host.rs"]
pub mod pages_host;
#[path = "../../shared/src/generated/stores/mod.rs"]
#[allow(
    missing_docs,
    reason = "writer-owned aggregates; generated leaves document their APIs"
)]
pub mod stores;
