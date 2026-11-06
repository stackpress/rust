//! Portable wire values shared by the blog's native and browser entry points.
//!
//! Renderer-specific generated views are selected separately from these JSON page values.

/// Portable generated source-admin views; no native source/store/client partition enters here.
#[allow(missing_docs, reason = "managed module declarations are generated")]
#[path = "../../.build/app-generated/ui/mod.rs"]
pub mod ui;

pub mod browser;
pub mod form;
pub mod helpers;
pub mod page;
pub mod root;
pub mod views;
pub use page::{Kind, Page};
