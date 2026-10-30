//! Host-free facade consumer for the exact shared source pages, with explicit root disposal.
use stackpress::framework_session::views::{Page, PageProps};
use wasm_bindgen::{JsCast, prelude::*};

#[cfg(any(
    all(feature = "hydrate", feature = "csr"),
    not(any(feature = "hydrate", feature = "csr"))
))]
compile_error!("select exactly one of hydrate or csr for a browser artifact");

/// Hydrate or independently mount a checked public seed without importing native resources.
#[wasm_bindgen]
pub fn start(text: &str) -> Result<js_sys::Function, JsValue> {
    let seed: stackpress_framework_view::PageSnapshot<(Page, PageProps)> =
        stackpress_framework_view::PageSnapshot::decode(text, 1, "session-v1")
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
    // The caller owns unmounting; no detached task or process-global active page is created.
    let mut handle = Some(handle);
    let dispose = Closure::wrap(Box::new(move || {
        handle.take();
    }) as Box<dyn FnMut()>);
    Ok(dispose.into_js_value().unchecked_into())
}
