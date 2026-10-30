//! Frui-style country selector and national draft; no new authoritative phone rule.
use super::{PageProps, countries::COUNTRIES, types::text};
use leptos::prelude::*;

/// Parse with approved pinned metadata, retaining formatting separately from server cleanup.
fn phone_data(value: &str) -> Option<(String, String)> {
    let parsed = phonenumber::parse(None, value).ok()?;
    let country = parsed.country().id()?.as_ref().to_owned();
    let full = parsed
        .format()
        .mode(phonenumber::Mode::International)
        .to_string();
    Some((country, full))
}
/// Country defaults PH; only the source-formatted full value has the native submission name.
pub(super) fn field(props: &PageProps, label: &str, value: String, required: bool) -> AnyView {
    let country = phone_data(&value)
        .map(|data| data.0)
        .unwrap_or_else(|| "PH".into());
    let dial = COUNTRIES
        .iter()
        .find(|item| item.0 == country)
        .map(|item| item.2)
        .unwrap_or("+63");
    let initial = phone_data(&value)
        .or_else(|| phone_data(&format!("{dial} {value}")))
        .map(|data| data.1)
        .unwrap_or_default();
    let national = initial
        .strip_prefix(dial)
        .unwrap_or(&initial)
        .trim()
        .to_owned();
    let selected = RwSignal::new(country);
    let draft = RwSignal::new(national);
    let full = RwSignal::new(initial);
    let sync = move || {
        let dial = COUNTRIES
            .iter()
            .find(|item| item.0 == selected.get())
            .map(|item| item.2)
            .unwrap_or("+63");
        full.set(
            phone_data(&format!("{dial} {}", draft.get()))
                .map(|data| data.1)
                .unwrap_or_default(),
        );
    };
    let error = text(props.errors.get("phone")).unwrap_or_default();
    let invalid = !error.is_empty();
    let options=COUNTRIES.iter().map(|(iso,name,dial)|{let iso=iso.to_string();view!{<option value=iso.clone() selected=move ||selected.get()==iso>{format!("{iso} {dial} — {name}")}</option>}}).collect_view();
    view!{<div class="session-field"><label for="session-phone">{super::view_phrase(label)}</label><div class="session-phone-row">
        <select aria-label="Phone country" prop:value=move ||selected.get() on:change=move |event|{selected.set(event_target_value(&event));sync();}>{options}</select>
        <input id="session-phone" type="tel" required=required value=move ||draft.get() prop:value=move ||draft.get() aria-describedby="session-phone-error" aria-invalid=invalid.to_string() on:input=move |event|{draft.set(event_target_value(&event));sync();}/>
        <input type="hidden" name="phone" value=move ||full.get()/>
    </div><p id="session-phone-error" class="session-field-error" role="alert">{error}</p></div>}.into_any()
}
