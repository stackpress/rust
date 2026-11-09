//! Finite SQLite/event/CLI proofs; the outer proof runner supplies an external deadline.

#[path = "cases/storage.rs"]
mod storage;

#[path = "cases/admin.rs"]
mod admin;
#[path = "cases/auth_api.rs"]
mod auth_api;
#[path = "cases/commands.rs"]
mod commands;
#[path = "cases/config.rs"]
mod config;
#[path = "cases/extension.rs"]
#[cfg(feature = "extension-proof")]
mod extension;
#[path = "cases/history.rs"]
mod history;
#[path = "cases/http.rs"]
mod http;
#[path = "cases/pages.rs"]
mod pages;
#[path = "cases/render.rs"]
mod render;
#[path = "cases/support.rs"]
mod support;
