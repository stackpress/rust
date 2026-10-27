//! Native-only catalog registration and escaped role emission into one UI-owned partition.
//!
//! The app constructs one definition registry for schema, SQL, and UI transforms. Runtime
//! imports are compiled adapters accepting `RoleContext`; schema strings are never code.

mod bindings;
mod catalog;
mod emit;
mod pages;
mod table;
pub use catalog::{Component, components, register_components};
pub use emit::emit_roles;
pub use pages::emit_pages;
