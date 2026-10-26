//! Host-free WASM startup; bootstrap installs inert JSON replay before hydration.
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use stackpress_example_ui_gallery_shared::{Gallery, decode_initial};
    use wasm_bindgen::{JsCast, prelude::*};

    /// Execute the same sanitizer in actual WASM for exact corpus comparison.
    #[wasm_bindgen]
    pub fn corpus_json() -> String {
        stackpress_example_ui_gallery_shared::corpus_json()
    }

    /// Mount the same source, with an optional native edit made before hydration.
    #[wasm_bindgen]
    pub fn start(page: &str, replay: &str) -> Result<js_sys::Function, JsValue> {
        let (seed, replay) =
            decode_initial(page, replay).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let values = js_sys::Array::new();
        for (id, value) in replay.entries() {
            values.set(*id as u32, JsValue::from_str(value));
        }
        let global = js_sys::global();
        js_sys::Reflect::set(&global, &JsValue::from_str("__RESOLVED_RESOURCES"), &values)?;
        for key in ["__SERIALIZED_ERRORS", "__INCOMPLETE_CHUNKS"] {
            js_sys::Reflect::set(&global, &JsValue::from_str(key), &js_sys::Array::new())?;
        }
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("No document"))?;
        let app: web_sys::HtmlElement = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("No root"))?
            .dyn_into()?;
        let early = document
            .get_element_by_id("draft")
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|element| element.value())
            .unwrap_or_else(|| seed.name.clone());
        let root = move || {
            stackpress_example_ui_gallery_shared::provide_meta();
            view! { <Gallery seed initial_draft=early/> }
        };
        #[cfg(feature = "hydrate")]
        let handle = leptos::mount::hydrate_from(app, root);
        #[cfg(feature = "csr")]
        let handle = leptos::mount::mount_to(app, root);
        let mut handle = Some(handle);
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }
}
