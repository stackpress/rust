//! Thin hydration/CSR startup and browser IO glue; data decisions live in blog-shared.
#[cfg(all(feature = "hydrate", feature = "csr"))]
compile_error!("select exactly one browser renderer");
pub use blog_shared::Page;
#[cfg(target_arch = "wasm32")]
mod adapters;
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use wasm_bindgen::{JsCast, prelude::*};
    /// Mount one checked root and return its resource disposer.
    #[wasm_bindgen]
    pub fn start() -> Result<js_sys::Function, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("document missing"))?;
        let app = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("root missing"))?
            .dyn_into::<web_sys::HtmlElement>()?;
        // DOM reads remain here; seed priority, decoding and errors are target-independent.
        let seeds = blog_shared::browser::SEED_IDS.map(|id| {
            document
                .get_element_by_id(id)
                .map(|element| element.text_content())
        });
        let page =
            blog_shared::browser::decode_seed(seeds).map_err(|error| JsValue::from_str(&error))?;
        let root = move || {
            blog_shared::root::meta_context();
            super::adapters::relations();
            page.view(
                Callback::new(super::adapters::submit),
                Callback::new(super::adapters::search),
            )
            .expect("checked source page")
        };
        #[cfg(feature = "hydrate")]
        let handle = leptos::mount::hydrate_from(app, root);
        #[cfg(feature = "csr")]
        app.set_inner_html("");
        #[cfg(feature = "csr")]
        let handle = leptos::mount::mount_to(app, root);
        let mut handle = Some(handle);
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }
}
/// Mount the app using the selected browser renderer.
#[cfg(target_arch = "wasm32")]
pub use browser::start;
