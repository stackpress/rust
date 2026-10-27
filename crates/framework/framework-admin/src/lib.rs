//! Public role drafts and Leptos adapters. No database operations or routes are registered.
//!
//! `codegen` is a build-host capability; browser consumers select only `hydrate` or `csr`.
//! Generated callers pass only projected fields to [`Draft`]. Components never evaluate props
//! as Rust, HTML, imports, templates, SQL, or event names.
//!
//! ```
//! use stackpress_framework_admin::Draft;
//! let draft = Draft::default();
//! assert_eq!(draft.snapshot()?, serde_json::json!({}));
//! assert!(draft.get("unapproved").is_none());
//! # Ok::<(), stackpress_framework_schema::SchemaError>(())
//! ```

mod draft;
pub use draft::{Draft, RoleContext};
#[cfg(feature = "codegen")]
pub mod codegen;
pub mod components;
