//! Example application replacement, referenced only when its generator plugin is selected.
use leptos::prelude::*;
use stackpress_framework_admin::{
    RoleContext,
    components::{TextField, TextListField},
};

/// A compiled custom component with the same typed role interface as framework definitions.
#[component]
pub fn AccentField(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    view! { <div data-admin-plugin="accent"><TextField context/></div> }
}

/// Compiled collection replacement selected before normalization, retaining exact row behavior.
#[component]
pub fn AccentTags(
    /// Approved collection descriptor and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    view! { <div data-admin-plugin="tags"><TextListField context/></div> }
}
