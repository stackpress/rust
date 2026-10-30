//! Source metadata, JSON normalization and MIME assembly through native library mechanics.
//!
//! Unsupported compiler capabilities are routed to a complete native provider before IO.

mod headers;
mod json;
mod metadata;
mod mime;
mod node;
mod part;

pub(crate) use json::normalize_json;
pub use metadata::{MessageMetadata, addresses, metadata};
pub(crate) use mime::mime;
