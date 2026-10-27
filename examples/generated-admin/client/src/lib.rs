//! Browser-only mount of exactly the generated SSR component source; no host services.

#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use wasm_bindgen::{JsCast, prelude::*};

    /// Hydrate or mount one caller-owned root, returning its explicit disposer.
    /// # Errors
    /// Reports a missing DOM root or platform API failure to the finite proof harness.
    #[wasm_bindgen]
    pub fn start() -> Result<js_sys::Function, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("missing document"))?;
        let app: web_sys::HtmlElement = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("missing root"))?
            .dyn_into()?;
        let root = || view! { <generated_admin_shared::Admin/> };
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
