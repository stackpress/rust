//! Complete theme-first HTML assembly for this finite example only.
use super::failure;
use leptos::prelude::*;
use stackpress_example_ui_preferences_shared::{PreferencesPage, STYLES, Seed, language};
use stackpress_lib::Result;

/// Render one deterministic shared source under a request-local reactive Owner.
pub(super) fn render(seed: Seed, csr: bool) -> Result<String> {
    // The public seed is escaped for inert script text before any HTML assembly.
    let json = serde_json::to_string(&seed)
        .map_err(|cause| failure("serialize preferences seed", cause))?;
    let json = stackpress_framework_view::escape_script_json(&json);
    // CSR deliberately has no SSR body; native rendering disposes all request-local signals.
    let body = if csr {
        String::new()
    } else {
        let owner = Owner::new();
        let body = owner.with(|| view! { <PreferencesPage seed=seed.clone()/> }.to_html());
        owner.cleanup();
        body
    };
    // Explicit choices are complete SSR output. Only unset uses the classic head asset.
    let unset = seed.preferences.theme.is_none();
    let bootstrap = if unset {
        stackpress_framework_view::theme::bootstrap_tag(
            &seed.base,
            Some(
                &stackpress_framework_view::theme::bootstrap_location()
                    .map_err(|e| failure("theme asset", e))?,
            ),
            None,
        )
        .map_err(|e| failure("theme head", e))?
    } else {
        String::new()
    };
    // Mark only unset choices; without the asset this same document stays readable Light.
    let marker = if unset { " data-theme-unset" } else { "" };
    Ok(format!(
        "<!doctype html><html lang=\"{}\" data-theme=\"{}\"{marker}><head>{bootstrap}<meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{}</title><style>{STYLES}</style></head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"preferences-seed\">{json}</script><script type=\"module\" src=\"{}/assets/start.js\"></script></body></html>",
        seed.preferences.locale,
        seed.preferences
            .theme
            .map_or("light", |theme| theme.as_str()),
        language(&seed.preferences.locale).phrase("Preferences"),
        seed.base.as_str()
    ))
}
