//! Validate the merged inert seed/replay before mounting the shared component.
use stackpress_example_ui_stream_shared::DemoData;
use stackpress_framework_view::leptos::replay::ResourceReplay;
use stackpress_framework_view::{PageSnapshot, ViewError};

/// Admit the exact seed/build and aggregate replay before browser mounting.
///
/// # Errors
/// Rejects malformed/drifted seed, duplicate/unbounded Resource identities and
/// oversized logical/escaped data. No replay global is written on failure.
pub fn decode(seed: &str, replay: &str) -> Result<(DemoData, ResourceReplay), ViewError> {
    let seed = PageSnapshot::<DemoData>::decode(seed, 1, "ui-stream")?;
    let replay = ResourceReplay::decode(replay, 1024 * 1024)?;
    Ok((seed.data, replay))
}

#[cfg(all(target_arch = "wasm32", feature = "hydrate"))]
mod browser {
    use leptos::prelude::*;
    use stackpress_example_ui_stream_shared::StreamDemo;
    use wasm_bindgen::{JsCast, prelude::*};

    /// The external bootstrap calls once, only after complete document delivery.
    #[wasm_bindgen]
    pub fn start(seed: &str, replay: &str) -> Result<js_sys::Function, JsValue> {
        let (data, replay) =
            super::decode(seed, replay).map_err(|error| JsValue::from_str(&error.to_string()))?;
        let values = js_sys::Array::new();
        for (id, value) in replay.entries() {
            values.set(*id as u32, JsValue::from_str(value));
        }
        let global = js_sys::global();
        js_sys::Reflect::set(&global, &"__RESOLVED_RESOURCES".into(), &values)?;
        for key in ["__SERIALIZED_ERRORS", "__INCOMPLETE_CHUNKS"] {
            js_sys::Reflect::set(&global, &key.into(), &js_sys::Array::new())?;
        }
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("No document"))?;
        let app = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("No app"))?
            .dyn_into::<web_sys::HtmlElement>()?;
        let mut handle = Some(leptos::mount::hydrate_from(
            app,
            move || view! { <StreamDemo data=data/> },
        ));
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// A replayed hostile value remains exact; drift and ambiguous IDs fail admission.
    #[test]
    fn admission() {
        let seed = r#"{"revision":1,"build":"ui-stream","page":"stream","location":"/stream","locale":"en","theme":"light","data":{"delay_ms":800,"fail":false,"pending":false}}"#;
        let replay = ResourceReplay::new(
            vec![(0, "</template><script>bad()</script>&\u{2028}".into())],
            1024,
        )
        .unwrap();
        let (data, decoded) = decode(seed, &replay.script_json().unwrap()).unwrap();
        assert_eq!(data.delay_ms, 800);
        assert_eq!(decoded, replay);
        assert!(decode(&seed.replace("ui-stream", "wrong-build"), "[]").is_err());
        assert!(decode(seed, r#"[[0,"one"],[0,"two"]]"#).is_err());
    }
}
