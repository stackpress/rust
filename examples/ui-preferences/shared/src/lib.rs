//! One deterministic shared preferences component; transport and policy stay in adapters.
use leptos::prelude::*;
use stackpress_framework_view::access::ActionId;
use stackpress_framework_view::flash::FlashRoot;
use stackpress_framework_view::preferences::{PreferenceState, SaveState, ThemePreference};
#[cfg(test)]
use stackpress_framework_view::{access::AllowedActions, flash::FlashNotice};
use stackpress_leptos::language::LanguageContext;

mod model;
pub use model::{STYLES, Seed, action, action_at_origin, decode, language, registry};
#[cfg(target_arch = "wasm32")]
mod browser;

/// Dirty draft ownership is independent of settings and cannot be replaced by a save.
#[component]
pub fn PreferencesPage(
    /// Identical public values on SSR/hydration/CSR.
    seed: Seed,
    /// Adopt the native input value typed before JS without replacing its DOM node.
    #[prop(optional)]
    initial_draft: Option<String>,
) -> impl IntoView {
    // Acknowledged settings and editable draft live in distinct root-owned signals.
    let state = RwSignal::new(PreferenceState::new(
        seed.preferences.clone(),
        seed.location,
    ));
    let phrases = LanguageContext::new(language(&seed.preferences.locale));
    let draft = RwSignal::new(initial_draft.unwrap_or_else(|| "Ada".into()));
    // Public notices are admitted once per root; dismissal never touches persistence authority.
    let mut dedupe = FlashRoot::default();
    let flash = RwSignal::new(seed.flash.and_then(|notice| dedupe.admit(notice)));
    let input = NodeRef::<leptos::html::Input>::new();
    // DOM effects adapt acknowledged pure state only; they never acquire preferences.
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        let current = state.get();
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            // Mirror acknowledged locale/theme only; unset preserves the already frozen head choice.
            if let Some(html) = document.document_element() {
                let _ = html.set_attribute("lang", &current.current().locale);
                // Unset preserves the pre-paint choice; locale saves never sample the OS again.
                if let Some(theme) = current.current().theme {
                    let _ = html.set_attribute("data-theme", theme.as_str());
                }
            }
            document.set_title(&phrases.text("Preferences", &[]));
        }
        // Keep the same native input/focus, including edits captured before hydration.
        if let Some(input) = input.get() {
            let value = draft.get();
            // Avoid redundant DOM writes that would disturb selection while typing.
            if input.value() != value {
                input.set_value(&value);
            }
        }
    });
    let base = seed.base.clone();
    let choose = move |locale: Option<&str>, theme: Option<ThemePreference>| {
        // Start from acknowledged values so a locale-only action preserves saved theme absence.
        let mut desired = state.get_untracked().current().clone();
        // Only the explicitly selected setting changes in this intent.
        if let Some(locale) = locale {
            desired.locale = locale.into();
        }
        if let Some(theme) = theme {
            desired.theme = Some(theme);
        }
        // Reserve the single write slot before the optional browser adapter starts acquisition.
        let mut update = None;
        state.update(|state| update = state.begin(desired));
        #[cfg(target_arch = "wasm32")]
        if let Some(update) = update {
            browser::save(base.clone(), update, state, phrases);
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (&base, update);
    };
    let en = choose.clone();
    let fr = choose.clone();
    let light = choose.clone();
    let dark = choose.clone();
    let link_base = seed.base.as_str().to_owned();
    // Public hints control rendering only; the host rechecks every protected operation.
    let allowed = seed
        .allowed
        .contains(&ActionId::new("private.page").expect("fixed action"));
    view! {
        <main>
            <h1>{move || phrases.text("Preferences", &[])}</h1>
            <p>"Finite preference proof · host policy required for protected actions"</p>
            <nav aria-label="Language">
                <button id="english" disabled=move || state.get().status() == SaveState::Pending on:click=move |_| en(Some("en"), None)>"English"</button>
                <button id="french" disabled=move || state.get().status() == SaveState::Pending on:click=move |_| fr(Some("fr"), None)>"Français"</button>
            </nav>
            <nav aria-label="Theme">
                <button id="light" disabled=move || state.get().status() == SaveState::Pending on:click=move |_| light(None, Some(ThemePreference::Light))>"Light"</button>
                <button id="dark" disabled=move || state.get().status() == SaveState::Pending on:click=move |_| dark(None, Some(ThemePreference::Dark))>"Dark"</button>
            </nav>
            <label for="draft">{move || phrases.text("Draft", &[])}</label>
            <input id="draft" node_ref=input value="Ada" on:input=move |event| draft.set(event_target_value(&event))/>
            <p id="samples">{move || format!("{} · {} · {}", phrases.text("January 2, 2026", &[]), phrases.text("1,234.50", &[]), phrases.plural_text(0, "%s item", "%s items", &["0".into()]))}</p>
            <a id="home" href=move || format!("{link_base}/{}/preferences", state.get().current().locale)>{move || phrases.text("Home", &[])}</a>
            <p id="save-error" role="alert">{move || if state.get().status() == SaveState::Failed { "Preferences could not be saved. Your draft is preserved." } else { "" }}</p>
            <section aria-label="Public notice">{move || flash.get().map(|notice| view! { <aside id="flash" role="status"><span>{notice.message().to_owned()}</span><button id="dismiss" on:click=move |_| flash.set(None)>"Dismiss"</button></aside> })}</section>
            <div class="actions">
            <a id="private-link" aria-disabled=(!allowed).to_string() href=allowed.then(|| format!("{}/private", seed.base.as_str()))>"Protected page"</a>
            <button id="private-mutation" disabled=!seed.allowed.contains(&ActionId::new("private.mutate").expect("fixed action"))>"Protected mutation"</button>
            </div>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stackpress_framework_view::BasePath;
    use stackpress_framework_view::preferences::Preferences;
    /// Pure admission, translation and SSR share the same host-selected settings.
    #[test]
    fn seed_phrases_and_ssr() {
        let seed = Seed {
            build: "s8-b-m1".into(),
            preferences: Preferences {
                locale: "fr".into(),
                theme: None,
            },
            base: BasePath::new("/demo").unwrap(),
            location: "/demo/fr/preferences?q=%252F".into(),
            flash: Some(FlashNotice::new("id".into(), "<script>inert</script>".into()).unwrap()),
            allowed: AllowedActions::default(),
        };
        let text = serde_json::to_string(&seed).unwrap();
        assert_eq!(decode(&text).unwrap().preferences, seed.preferences);
        for bad in [
            "{}".to_owned(),
            "x".repeat(16385),
            text.replace("s8-b-m1", "old"),
            text.replace("\"fr\"", "\"xx\""),
        ] {
            assert!(decode(&bad).is_err());
        }
        let english = language("en");
        let french = language("fr");
        assert_eq!(
            english.plural_text(0, "%s item", "%s items", &["0".into()]),
            "0 items"
        );
        assert_eq!(
            french.plural_text(0, "%s item", "%s items", &["0".into()]),
            "0 article"
        );
        assert_eq!(
            french.text(" %s / %s / %s ", &["0".into(), "false".into(), "".into()]),
            " 0 / false /  "
        );
        #[cfg(feature = "ssr")]
        {
            let owner = Owner::new();
            let html = owner.with(|| view! { <PreferencesPage seed/> }.to_html());
            assert!(html.contains("Préférences"));
            assert!(html.contains("&lt;script&gt;inert&lt;/script&gt;"));
            owner.cleanup();
        }
    }
}
