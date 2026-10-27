//! Complete theme-first HTML assembly for this finite example only.
use super::failure;
use leptos::prelude::*;
use stackpress_example_ui_preferences_shared::{PreferencesPage, STYLES, Seed, language};
use stackpress_lib::Result;

/// Render one deterministic shared source under a request-local reactive Owner.
pub(super) fn render(seed: Seed, csr: bool) -> Result<String> {
    let json = serde_json::to_string(&seed)
        .map_err(|cause| failure("serialize preferences seed", cause))?;
    let json = stackpress_framework_view::escape_script_json(&json);
    let body = if csr {
        String::new()
    } else {
        let owner = Owner::new();
        let body = owner.with(|| view! { <PreferencesPage seed=seed.clone()/> }.to_html());
        owner.cleanup();
        body
    };
    // This finite example retains its own shell and CSS palette. PageContract now
    // admits system; adopting the shared style projector is a coordinator decision.
    // All shell attributes/title are fixed registry/enum text, not arbitrary input.
    Ok(format!(
        "<!doctype html><html lang=\"{}\" data-theme=\"{}\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>{STYLES}</style></head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"preferences-seed\">{json}</script><script type=\"module\" src=\"{}/assets/start.js\"></script></body></html>",
        seed.preferences.locale,
        seed.preferences.theme.as_str(),
        language(&seed.preferences.locale).phrase("Preferences"),
        seed.base.as_str()
    ))
}
