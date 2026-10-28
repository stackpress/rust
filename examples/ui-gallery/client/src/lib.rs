//! Host-free WASM startup; bootstrap installs inert JSON replay before hydration.
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use stackpress_example_ui_gallery_shared::parse_presence;
    use stackpress_example_ui_gallery_shared::{BreadthDraft, CoreDraft, Gallery, decode_initial};
    use wasm_bindgen::{JsCast, prelude::*};

    /// Execute the same sanitizer in actual WASM for exact corpus comparison.
    #[wasm_bindgen]
    pub fn corpus_json() -> String {
        stackpress_example_ui_gallery_shared::corpus_json()
    }

    /// Mount only the explicit finite collection proof; ordinary startup never calls this.
    #[wasm_bindgen]
    pub fn start_collections() -> Result<js_sys::Function, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("No proof document"))?;
        let app = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("No proof root"))?
            .dyn_into::<web_sys::HtmlElement>()?;
        let initial_nested = selected(&document, "nested-8-value")
            .first()
            .map(|value| parse_presence(value))
            .transpose()
            .map_err(|_| JsValue::from_str("Invalid early nested presence"))?
            .unwrap_or(stackpress_example_ui_gallery_shared::FieldValue::Value(
                false,
            ));
        let root = move || view! {<stackpress_example_ui_gallery_shared::collection_proof::CollectionProof initial_nested/>};
        #[cfg(feature = "hydrate")]
        let handle = leptos::mount::hydrate_from(app.clone(), root);
        #[cfg(feature = "csr")]
        let handle = leptos::mount::mount_to(app.clone(), root);
        let mut handle = Some(handle);
        Ok(Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>)
        .into_js_value()
        .unchecked_into())
    }

    /// Copy a native input property, without rewriting its initial SSR attribute.
    fn input_value(document: &web_sys::Document, id: &str, fallback: &str) -> String {
        document
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_else(|| fallback.to_owned())
    }
    /// Copy an explicit checked property; unchecked never becomes a missing string.
    fn checked(document: &web_sys::Document, id: &str) -> bool {
        document
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .is_some_and(|input| input.checked())
    }
    /// Copy native enabled selected values; the disabled placeholder is not user data.
    fn selected(document: &web_sys::Document, id: &str) -> Vec<String> {
        let Some(select) = document
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlSelectElement>().ok())
        else {
            return Vec::new();
        };
        let options = select.selected_options();
        (0..options.length())
            .filter_map(|index| options.item(index))
            .filter_map(|option| option.dyn_into::<web_sys::HtmlOptionElement>().ok())
            .filter(|option| !option.disabled())
            .map(|option| option.value())
            .collect()
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
        // Adopt native properties before reactive effects can reconcile controlled state.
        // CSR has no preexisting controls and uses the same deterministic defaults.
        let mut core = CoreDraft::default();
        if document.get_element_by_id("core-text").is_some() {
            core.text = input_value(&document, "core-text", &core.text);
            core.password = input_value(&document, "core-password", &core.password);
            if let Some(notes) = document
                .get_element_by_id("core-notes")
                .and_then(|element| element.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
            {
                core.notes = notes.value();
            }
            core.checked = checked(&document, "core-check");
            core.enabled = checked(&document, "core-switch");
            core.radio = if checked(&document, "core-radio-zero") {
                Some("0".into())
            } else if checked(&document, "core-radio-fr") {
                Some("fr".into())
            } else {
                None
            };
            core.single = selected(&document, "core-select").into_iter().next();
            core.multiple = selected(&document, "core-multiple");
        }
        let mut breadth = BreadthDraft::default();
        if let Some(details) = document
            .get_element_by_id("native-details")
            .and_then(|node| node.dyn_into::<web_sys::HtmlDetailsElement>().ok())
        {
            breadth.details = details.open();
        }
        if document.get_element_by_id("text-rows-1-value").is_some() {
            breadth.text[0] = input_value(&document, "text-rows-1-value", &breadth.text[0]);
            breadth.text[1] = input_value(&document, "text-rows-2-value", &breadth.text[1]);
            breadth.slug = input_value(&document, "slug", &breadth.slug);
            breadth.query = input_value(&document, "combo", &breadth.query);
            breadth.suggestion_query = input_value(&document, "suggest", &breadth.suggestion_query);
            // Chips submit exact hidden values; their separate add draft is adopted by
            // TagList before hydration and never becomes a submitted tag implicitly.
            for (index, key) in [10, 20].into_iter().enumerate() {
                breadth.tags[index] = input_value(
                    &document,
                    &format!("tag-rows-{key}-value"),
                    &breadth.tags[index],
                );
            }
            breadth.metadata.key =
                input_value(&document, "metadata-rows-4-key", &breadth.metadata.key);
            breadth.metadata.value =
                input_value(&document, "metadata-rows-4-value", &breadth.metadata.value);
            for (index, id) in ["single-range-0", "pair-range-0", "pair-range-1"]
                .into_iter()
                .enumerate()
            {
                breadth.ranges[index] =
                    input_value(&document, id, &breadth.ranges[index].to_string())
                        .parse()
                        .map_err(|_| JsValue::from_str("Invalid early range value"))?;
            }
            breadth.rating = (0..=5).find(|rating| checked(&document, &format!("rating-{rating}")));
            breadth.filter = input_value(&document, "data-table-filter", &breadth.filter);
            breadth.selected_rows = (0..48)
                .filter(|index| {
                    checked(&document, &format!("data-table-row-r{index}"))
                        || (*index < 12 && checked(&document, "data-table-select-page"))
                })
                .map(|index| format!("r{index}"))
                .collect();
        }
        breadth
            .validate()
            .map_err(|_| JsValue::from_str("Invalid early component draft"))?;
        let safety = document
            .get_element_by_id("safety-notes")
            .and_then(|element| element.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
            .map(|element| element.value())
            .unwrap_or_else(|| stackpress_example_ui_gallery_shared::TEXTAREA_PAYLOAD.into());
        let pending_theme = seed.theme.is_none();
        let root = move || {
            stackpress_example_ui_gallery_shared::provide_meta();
            view! { <Gallery seed initial_draft=early initial_core=core initial_breadth=breadth initial_safety=safety/> }
        };
        #[cfg(feature = "hydrate")]
        let handle = leptos::mount::hydrate_from(app.clone(), root);
        #[cfg(feature = "csr")]
        let handle = leptos::mount::mount_to(app.clone(), root);
        // The native tree was hydrated unchanged; now copy the already-frozen document choice.
        if pending_theme
            && let (Some(html), Ok(Some(root))) = (
                document.document_element(),
                app.query_selector("[data-initial-theme-pending]"),
            )
        {
            let theme = html
                .get_attribute("data-theme")
                .unwrap_or_else(|| "light".into());
            root.set_attribute("data-theme", &theme)?;
            root.remove_attribute("data-initial-theme-pending")?;
        }
        let mut handle = Some(handle);
        let dispose = Closure::wrap(Box::new(move || {
            handle.take();
        }) as Box<dyn FnMut()>);
        Ok(dispose.into_js_value().unchecked_into())
    }
}
