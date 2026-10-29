//! Explicit acknowledged preference values adapted into root-local UI values.
#[cfg(feature = "preferences")]
/// Map a saved explicit choice; absence remains absence until the host's one-shot policy.
pub fn theme(
    value: Option<crate::preferences::ThemePreference>,
) -> Option<stackpress_leptos::state::Theme> {
    value.map(|value| match value {
        crate::preferences::ThemePreference::Light => stackpress_leptos::state::Theme::Light,
        crate::preferences::ThemePreference::Dark => stackpress_leptos::state::Theme::Dark,
    })
}
