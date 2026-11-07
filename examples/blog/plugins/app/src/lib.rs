//! Blog plugin: native pages and portable Leptos views selected before compilation.
//! `ui` contains no native resources; `server` adds page registration and a host render callback.
//! Renderer passthroughs (`ssr`, `hydrate`, `csr`) enable `ui`; browser hosts select exactly
//! one browser renderer and never `server`. This app also owns its generated admin UI selection.

/// Portable generated source-admin views; no native source/store/client partition enters here.
#[cfg(feature = "ui")]
#[allow(missing_docs, reason = "managed module declarations are generated")]
#[path = "../../../.build/app-generated/ui/mod.rs"]
pub mod ui;

#[cfg(feature = "ui")]
pub mod browser;
#[cfg(feature = "ui")]
pub mod form;
#[cfg(feature = "ui")]
pub mod helpers;
#[cfg(feature = "ui")]
pub mod page;
#[cfg(feature = "ui")]
pub mod root;
#[cfg(feature = "ui")]
pub mod views;
#[cfg(feature = "ui")]
pub use page::{Kind, Page};

#[cfg(feature = "server")]
pub mod pages;
#[cfg(feature = "server")]
mod server;
#[cfg(feature = "server")]
pub use server::{RenderPage, register};
