//! Native generated store graph; the shared/browser crate includes only contracts and UI.
#[cfg_attr(
    not(feature = "format"),
    path = "../../shared/src/generated/contracts/mod.rs"
)]
#[cfg_attr(
    feature = "format",
    path = "../../shared/src/generated-format/contracts/mod.rs"
)]
#[allow(
    missing_docs,
    reason = "the same writer-owned contract source compiles with native operations"
)]
pub mod contracts;
#[cfg_attr(
    not(feature = "format"),
    path = "../../shared/src/generated/ui/pages_host.rs"
)]
#[cfg_attr(
    feature = "format",
    path = "../../shared/src/generated-format/ui/pages_host.rs"
)]
pub mod pages_host;
#[cfg_attr(
    not(feature = "format"),
    path = "../../shared/src/generated/stores/mod.rs"
)]
#[cfg_attr(
    feature = "format",
    path = "../../shared/src/generated-format/stores/mod.rs"
)]
#[allow(
    missing_docs,
    reason = "writer-owned aggregates; generated leaves document their APIs"
)]
pub mod stores;
