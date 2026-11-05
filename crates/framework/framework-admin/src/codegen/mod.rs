//! Native-only catalog registration and escaped role emission into one UI-owned partition.
//!
//! The app constructs one definition registry for schema, SQL, and UI transforms. Runtime
//! imports are compiled adapters accepting `RoleContext`; schema strings are never code.

mod bindings;
mod catalog;
mod emit;
mod pages;
mod source;
mod table;
mod templates;
pub use catalog::{Component, components, register_components};
pub use emit::emit_roles;
pub use pages::emit_pages;
pub use source::{emit_source_pages, emit_source_roles, source_definitions};

/// Register a role-only build transform owning UI after the selected schema transform.
///
/// Applications supply their frozen catalog and actual emitter callback. This alternative
/// to [`register_pages`] does not require SQL and adds no runtime/framework plugin.
///
/// # Errors
/// Preserves registry errors, including selecting both admin transform alternatives.
pub fn register(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()> {
    registry.add(
        stackpress_idea_transformer::TransformSpec::new("framework-admin")
            .owns(&["ui"])
            .requires(&["framework-schema"]),
        transform,
    )
}

/// Register a native-page build transform requiring schema and SQL earlier in the plan.
///
/// The application also checks the finished plan contains `stores/operations.rs` before
/// installation: selecting a SQL transform does not itself promise emitted operations.
/// Pure role generation can instead use [`register`]. Generic shared admin stays plugin-free.
///
/// # Errors
/// Preserves registry errors; planning rejects missing or wrong-order dependencies.
pub fn register_pages(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()> {
    registry.add(
        stackpress_idea_transformer::TransformSpec::new("framework-admin")
            .owns(&["ui"])
            .requires(&["framework-schema", "framework-sql"]),
        transform,
    )
}

#[cfg(feature = "format")]
mod formatted;
