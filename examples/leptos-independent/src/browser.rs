//! Thin WASM adapter for explicit fixture data and native draft adoption.
use crate::{IndependentRoot, Initial};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, prelude::*};

/// Mount once from checked explicit inputs and return ownership of root disposal.
/// # Errors
/// Rejects unknown choices, excessive drafts/replay, missing root or invalid DOM types.
#[cfg(any(feature = "hydrate", feature = "csr"))]
#[wasm_bindgen]
pub fn start(
    theme: &str,
    locale: &str,
    draft: String,
    replay: &str,
) -> Result<js_sys::Function, JsValue> {
    // The fixture exposes trusted panic diagnostics to its proof console, not its UI.
    std::panic::set_hook(Box::new(|info| {
        let global = js_sys::global();
        if let Ok(console) = js_sys::Reflect::get(&global, &"console".into())
            && let Ok(value) = js_sys::Reflect::get(&console, &"error".into())
            && let Ok(report) = value.dyn_into::<js_sys::Function>()
        {
            let _ = report.call1(&console, &info.to_string().into());
        }
    }));
    let initial = Initial::new(theme, locale, draft).map_err(JsValue::from_str)?;
    #[cfg(feature = "async")]
    {
        let checked = stackpress_leptos::replay::ResourceReplay::decode(replay, 1024 * 1024)
            .map_err(|_| JsValue::from_str("Invalid independent replay"))?;
        let values = js_sys::Array::new();
        for (id, value) in checked.entries() {
            values.set(*id as u32, JsValue::from_str(value));
        }
        let global = js_sys::global();
        // These are upstream Leptos Resource globals, not application page globals.
        js_sys::Reflect::set(&global, &"__RESOLVED_RESOURCES".into(), &values)?;
        for key in ["__SERIALIZED_ERRORS", "__INCOMPLETE_CHUNKS"] {
            js_sys::Reflect::set(&global, &key.into(), &js_sys::Array::new())?;
        }
    }
    #[cfg(not(feature = "async"))]
    if replay != "[]" {
        return Err(JsValue::from_str("Resource feature is disabled"));
    }
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| JsValue::from_str("No document"))?;
    let element = document
        .get_element_by_id("independent-root")
        .ok_or_else(|| JsValue::from_str("No independent root"))?
        .dyn_into::<web_sys::HtmlElement>()?;
    let root = move || {
        stackpress_leptos::leptos_meta::provide_meta_context();
        view! {<IndependentRoot initial/>}
    };
    #[cfg(feature = "hydrate")]
    let handle = leptos::mount::hydrate_from(element, root);
    #[cfg(feature = "csr")]
    let handle = leptos::mount::mount_to(element, root);
    // The JS caller owns the returned disposer. Repeating it is intentionally harmless.
    let mut handle = Some(handle);
    let dispose = Closure::wrap(Box::new(move || {
        handle.take();
    }) as Box<dyn FnMut()>);
    Ok(dispose.into_js_value().unchecked_into())
}
