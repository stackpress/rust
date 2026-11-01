//! Source mechanics, endpoints, webhooks, SQL and OAuth contracts in one test executable.
mod config;
mod cors;
mod endpoints;
mod plugins;
#[path = "proof.rs"]
mod proof;
mod webhooks;

#[cfg(feature = "sql")]
mod sql;

mod oauth;

#[cfg(feature = "ui-ssr")]
mod consent;
#[cfg(all(feature = "ui-ssr", feature = "sql"))]
mod management;
#[cfg(all(feature = "ui-ssr", feature = "sql"))]
#[path = "../../../../examples/api-checkpoint/shared/src/lib.rs"]
#[allow(
    dead_code,
    reason = "the selected management proof compiles the complete generated public catalog"
)]
mod shared;
