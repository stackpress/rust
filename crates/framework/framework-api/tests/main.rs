//! Source mechanics, endpoints, webhooks, SQL and OAuth contracts in one test executable.
mod config;
mod cors;
mod endpoints;
mod plugins;
#[path = "proof.rs"]
mod proof;
mod webhooks;

#[cfg(feature = "sql")]
mod generated_family;
#[cfg(feature = "sql")]
mod sql;

mod oauth;

#[cfg(all(feature = "codegen", feature = "ui-ssr"))]
mod url_roles;

#[cfg(all(feature = "ui-ssr", feature = "sql"))]
mod composite_admin;
#[cfg(feature = "ui-ssr")]
mod consent;
#[cfg(all(feature = "ui-ssr", feature = "sql"))]
mod management;
#[cfg(all(feature = "ui-ssr", feature = "sql"))]
#[path = "fixtures/management.rs"]
#[allow(
    dead_code,
    reason = "the selected management proof compiles the complete generated public catalog"
)]
mod shared;

#[path = "fixtures/generation.rs"]
mod fixture_generation;
#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "fixtures/models/mod.rs"]
mod models;
/// A stale fixture fails without modifying output; the example regenerates it explicitly.
#[test]
fn generated_models_are_fresh() {
    fixture_generation::fixtures(true).unwrap();
}
