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
