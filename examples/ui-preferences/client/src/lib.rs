//! Host-free startup; validate the exact public seed before creating a root.
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use stackpress_example_ui_preferences_shared::{PreferencesPage, decode};
    use wasm_bindgen::{JsCast, prelude::*};

    /// Hydrate/mount the same component, adopting any early native draft edit.
    #[wasm_bindgen]
    pub fn start(text: &str) -> Result<js_sys::Function, JsValue> {
        let seed = decode(text).map_err(|_| JsValue::from_str("Invalid page seed"))?;
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("Missing document"))?;
        let app: web_sys::HtmlElement = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("Missing root"))?
            .dyn_into()?;
        let early = document
            .get_element_by_id("draft")
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_else(|| "Ada".into());
        let root = move || view! { <PreferencesPage seed initial_draft=early/> };
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
