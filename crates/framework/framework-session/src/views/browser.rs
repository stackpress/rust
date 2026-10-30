//! Browser platform mechanics only; no native handles or persistence acquisition.
use leptos::prelude::*;
#[cfg(all(target_arch = "wasm32", feature = "browser"))]
use stackpress_leptos::state::Theme;
use stackpress_leptos::state::ThemeState;

/// Adopt exactly the host's frozen pre-paint choice; absence stays Light without a sampler.
pub(super) fn adopt_theme(theme: ThemeState, pending: RwSignal<bool>) {
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    Effect::new(move |_| {
        if !pending.get_untracked() {
            return;
        }
        let choice = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element())
            .and_then(|e| e.get_attribute("data-theme"));
        if choice.as_deref() == Some("dark") {
            theme.set(Theme::Dark);
        }
        pending.set(false);
    });
    #[cfg(not(all(target_arch = "wasm32", feature = "browser")))]
    let _ = (theme, pending);
}
/// Source clipboard side effect is deliberately triggered only by a user action.
/// Missing clipboard leaves the value selectable; the browser owns permission/rejection.
pub(super) fn copy_secret(secret: &str, message: RwSignal<String>) {
    #[cfg(all(target_arch = "wasm32", feature = "browser"))]
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        // The WebIDL getter is non-optional, but browsers without Clipboard expose undefined.
        let value: &web_sys::wasm_bindgen::JsValue = clipboard.as_ref();
        if value.is_undefined() || value.is_null() {
            return;
        }
        let promise = clipboard.write_text(secret);
        // Source notifies immediately after submitting the clipboard promise.
        message.set(super::phrase("Secret key copied to clipboard."));
        wasm_bindgen_futures::spawn_local(async move {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
        });
    }
    #[cfg(not(all(target_arch = "wasm32", feature = "browser")))]
    let _ = (secret, message);
}
