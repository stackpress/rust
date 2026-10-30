//! Compact labeled form controls; native form submission remains usable without JavaScript.
use super::{PageProps, PasswordRules, evaluate_password, icons::icon, password_checklist};
use leptos::prelude::*;

/// Source hidden CSRF field; generation/validation belong to native explicit call sites.
pub fn account_csrf_fields(props: &PageProps) -> AnyView {
    hidden(&props.config.csrf_name, &props.config.csrf_token)
}
/// Shared hidden value; Leptos escapes attribute text on every target.
pub(super) fn hidden(name: &str, value: &str) -> AnyView {
    view! {<input type="hidden" name=name.to_owned() value=value.to_owned()/>}.into_any()
}
/// Back link, optional title icon and a single focusable main heading.
pub fn account_back_header(title: &str, back: Option<String>, glyph: Option<&str>) -> AnyView {
    view!{<header class="session-page-header">{back.map(|back|view!{<a href=back aria-label=super::phrase("Back") class="session-icon-button">{icon("chevron-left")}</a>})}{glyph.map(icon)}<h1 tabindex="-1">{super::view_phrase(title)}</h1></header>}.into_any()
}
/// Root-local theme action reuses the shared theme context; no implicit cookie/storage policy.
pub(super) fn theme_toggle() -> AnyView {
    let theme = stackpress_leptos::state::use_theme().ok();
    // A direct form may have no theme provider; its button remains a harmless local control.
    view! {
        <button type="button" class="session-icon-button" aria-label="Toggle theme"
            on:click=move |_| {
                if let Some(theme) = theme {
                    theme.toggle();
                }
            }>
            {move || icon(if theme.is_some_and(|theme| {
                theme.get() == stackpress_leptos::state::Theme::Dark
            }) { "moon" } else { "sun" })}
        </button>
    }
    .into_any()
}
/// Source field binding with explicit label/help/error associations.
pub(super) fn field(
    props: &PageProps,
    name: &str,
    label: &str,
    kind: &str,
    value: String,
    required: bool,
) -> AnyView {
    if kind == "tel" {
        return super::phone_input::field(props, label, value, required);
    }
    let error = super::types::text(props.errors.get(name)).unwrap_or_default();
    input_field(
        name,
        label,
        kind,
        RwSignal::new(value),
        required,
        error,
        false,
    )
}
/// Controlled inline input, with an accessible Lucide reveal toggle for password fields.
pub(super) fn input_field(
    name: &str,
    label: &str,
    kind: &str,
    value: RwSignal<String>,
    required: bool,
    error: String,
    readonly: bool,
) -> AnyView {
    let id = format!("session-{name}");
    let error_id = format!("{id}-error");
    let shown = RwSignal::new(false);
    let password = kind == "password";
    let kind = kind.to_owned();
    let target = id.clone();
    let invalid = !error.is_empty();
    // The same signal supplies the SSR value attribute and the live DOM property.
    // Revealing a password changes its type without replacing its submitted value.
    view! {
        <div class="session-field">
            <label for=id.clone()>{super::view_phrase(label)}</label>
            <div class="session-input-row">
                <input id=id name=name.to_owned()
                    type=move || if password && shown.get() { "text".to_owned() } else { kind.clone() }
                    required=required readonly=readonly
                    value=move || value.get() prop:value=move || value.get()
                    autocomplete=if password { "off" } else { "on" }
                    aria-invalid=invalid.to_string() aria-describedby=error_id.clone()
                    maxlength=if name == "code" { Some(6) } else { None }
                    on:input=move |event| value.set(event_target_value(&event))/>
                {password.then(|| view! {
                    <button type="button" class="session-icon-button" aria-controls=target
                        aria-label=move || if shown.get() { "Hide password" } else { "Show password" }
                        aria-pressed=move || shown.get().to_string()
                        on:click=move |_| shown.update(|shown| *shown = !*shown)>
                        {move || icon(if shown.get() { "eye-off" } else { "eye" })}
                    </button>
                })}
            </div>
            <p class="session-field-error" id=error_id role="alert">{error}</p>
        </div>
    }
    .into_any()
}
/// Submit control keeps source text and native keyboard activation.
pub(super) fn submit(label: &str, danger: bool) -> AnyView {
    view!{<button type="submit" class=if danger{"session-button danger"}else{"session-button primary"}>{super::view_phrase(label)}</button>}.into_any()
}
/// Source weighted strength feedback, updated from the same password signal as the input.
pub fn password_strength(secret: RwSignal<String>, rules: PasswordRules) -> AnyView {
    let checks = password_checklist(&rules);
    // Re-evaluate the existing checklist on input; an empty password keeps feedback hidden.
    view! {
        <div class="session-strength" aria-live="polite">{move || {
            let value = secret.get();
            let result = evaluate_password(&value, &checks);
            (!value.is_empty()).then(|| view! {
                <progress max="100" value=result.score aria-label="Password strength"></progress>
                <ul>{result.failed.into_iter().map(|(_, label)| view! {
                    <li>{icon("x")}{super::view_phrase(&label)}</li>
                }).collect_view()}</ul>
            })
        }}</div>
    }
    .into_any()
}

/// Source copy-on-click secret field plus a named keyboard-operable copy button.
pub(super) fn secret_field(secret: String) -> AnyView {
    let message = RwSignal::new(String::new());
    let click = secret.clone();
    let button = secret.clone();
    view! {
        <div class="session-field">
            <label for="session-secret">{super::view_phrase("Or Copy the Secret Key")}</label>
            <div class="session-input-row">
                <input id="session-secret" type="text" name="secret" value=secret readonly
                    on:click=move |_| super::browser::copy_secret(&click, message)/>
                <button class="session-button" type="button"
                    on:click=move |_| super::browser::copy_secret(&button, message)>
                    {super::view_phrase("Copy")}
                </button>
            </div>
            <p role="status" class="session-note">{move || message.get()}</p>
        </div>
    }
    .into_any()
}
