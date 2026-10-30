//! Browser artifact consumes only the deliberate public session seed, never native services.
#[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))]
mod browser {
    use stackpress::{
        framework_session::views::{Page, PageProps},
        framework_view::PageSnapshot,
    };
    use wasm_bindgen::{JsCast, prelude::*};

    #[cfg(all(feature = "hydrate", feature = "csr"))]
    compile_error!("select exactly one browser renderer");

    /// Mount the shared source view and return an explicit disposal action to its owner.
    #[wasm_bindgen]
    pub fn start(text: &str) -> Result<js_sys::Function, JsValue> {
        let seed: PageSnapshot<(Page, PageProps)> = PageSnapshot::decode(text, 1, "session-v1")
            .map_err(|_| JsValue::from_str("invalid public seed"))?;
        let app: web_sys::HtmlElement = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .ok_or_else(|| JsValue::from_str("missing app"))?
            .dyn_into()?;
        let root = move || {
            leptos_meta::provide_meta_context();
            stackpress::framework_session::views::page(seed.data.0, seed.data.1.clone())
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
