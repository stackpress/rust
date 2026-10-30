//! Hydration/CSR of the same pure source consent and generated management components.
#[path = "../../shared/src/lib.rs"]
pub mod shared;
#[cfg(target_arch = "wasm32")]
mod browser {
    use leptos::prelude::*;
    use stackpress_framework_api::views::{
        ApiOauthPage, Consent,
        management::{Management, ManagementPage},
        roles::{Choice, RelationLoader, RelationRequest},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use wasm_bindgen::{JsCast, prelude::*};
    use web_sys::{HtmlFormElement, HtmlInputElement};
    /// Native form transport preserves source redirects, CSRF and server validation drafts.
    fn submit(data: serde_json::Value) {
        native_form(data, "post");
    }
    /// Submit original aggregate query values through native browser navigation.
    fn search(data: serde_json::Value) {
        native_form(data, "get");
    }
    /// Expand the exact draft into ordinary repeated/bracketed form field names.
    fn native_form(data: serde_json::Value, method: &str) {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Ok(form) = document
            .create_element("form")
            .and_then(|form| form.dyn_into::<HtmlFormElement>().map_err(Into::into))
        else {
            return;
        };
        form.set_method(method);
        /// Preserve arrays and objects without inventing a JSON write endpoint.
        fn append(
            document: &web_sys::Document,
            form: &HtmlFormElement,
            name: &str,
            value: &serde_json::Value,
        ) {
            match value {
                serde_json::Value::Array(rows) => {
                    for row in rows {
                        append(document, form, &format!("{name}[]"), row);
                    }
                }
                serde_json::Value::Object(row) => {
                    for (key, value) in row {
                        append(document, form, &format!("{name}[{key}]"), value);
                    }
                }
                _ => {
                    if let Ok(input) = document
                        .create_element("input")
                        .and_then(|input| input.dyn_into::<HtmlInputElement>().map_err(Into::into))
                    {
                        input.set_type("hidden");
                        input.set_name(name);
                        input.set_value(
                            &value
                                .as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| value.to_string()),
                        );
                        let _ = form.append_child(&input);
                    }
                }
            }
        }
        if let Some(row) = data.as_object() {
            for (key, value) in row {
                append(&document, &form, key, value);
            }
        }
        if let Some(body) = document.body() {
            let _ = body.append_child(&form);
            let _ = form.submit();
        }
    }
    /// Real source search URLs, cancellable on disposal; each field retains only its latest lookup.
    fn relations() {
        let alive = Arc::new(AtomicBool::new(true));
        let active = StoredValue::new_local(std::collections::BTreeMap::<
            String,
            web_sys::AbortController,
        >::new());
        let live = alive.clone();
        on_cleanup(move || {
            live.store(false, Ordering::Relaxed);
            active.with_value(|active| {
                for controller in active.values() {
                    controller.abort();
                }
            });
        });
        provide_context(RelationLoader(Callback::new(
            move |request: RelationRequest| {
                let Ok(controller) = web_sys::AbortController::new() else {
                    return;
                };
                active.update_value(|active| {
                    if let Some(old) = active.insert(request.field.clone(), controller.clone()) {
                        old.abort();
                    }
                });
                let alive = alive.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let Some(window) = web_sys::window() else {
                        return;
                    };
                    let options = web_sys::RequestInit::new();
                    options.set_signal(Some(&controller.signal()));
                    let Ok(response) = wasm_bindgen_futures::JsFuture::from(
                        window.fetch_with_str_and_init(&request.url, &options),
                    )
                    .await
                    else {
                        return;
                    };
                    let Ok(response) = response.dyn_into::<web_sys::Response>() else {
                        return;
                    };
                    // A rejected source search retains the current choices; it is not success.
                    if !response.ok() {
                        return;
                    }
                    let Ok(text) = response.text() else {
                        return;
                    };
                    let Ok(text) = wasm_bindgen_futures::JsFuture::from(text).await else {
                        return;
                    };
                    let Ok(value) = serde_json::from_str::<serde_json::Value>(
                        &text.as_string().unwrap_or_default(),
                    ) else {
                        return;
                    };
                    if controller.signal().aborted() || !alive.load(Ordering::Relaxed) {
                        return;
                    }
                    let rows = value
                        .get("results")
                        .unwrap_or(&value)
                        .as_array()
                        .cloned()
                        .unwrap_or_default();
                    let choices = rows
                        .into_iter()
                        .filter_map(|row| {
                            Some(Choice {
                                id: row.get("id")?.as_str()?.into(),
                                label: row.get("name")?.as_str()?.into(),
                                row: row.clone(),
                            })
                        })
                        .collect();
                    request.complete.run(choices);
                });
            },
        )));
    }
    /// Mount one explicit root; the returned disposer drops resources and all pending lookups.
    #[wasm_bindgen]
    pub fn start() -> Result<js_sys::Function, JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("document missing"))?;
        let app = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("root missing"))?
            .dyn_into::<web_sys::HtmlElement>()?;
        let consent = document
            .get_element_by_id("api-consent-seed")
            .map(|seed| serde_json::from_str::<Consent>(&seed.text_content().unwrap_or_default()))
            .transpose()
            .map_err(|_| JsValue::from_str("invalid consent seed"))?;
        let management = document
            .get_element_by_id("api-management-seed")
            .map(|seed| {
                serde_json::from_str::<Management>(&seed.text_content().unwrap_or_default())
            })
            .transpose()
            .map_err(|_| JsValue::from_str("invalid management seed"))?;
        let root = move || {
            if let Some(page) = consent {
                view! {<ApiOauthPage page/>}.into_any()
            } else if let Some(page) = management {
                relations();
                view!{<ManagementPage page ui=super::shared::management() submit=Callback::new(submit) search=Callback::new(search)/>}.into_any()
            } else {
                view! {<p role="alert">"Page seed missing"</p>}.into_any()
            }
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
