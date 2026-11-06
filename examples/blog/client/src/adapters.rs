//! Thin browser DOM/navigation/fetch/AbortController glue; decisions live in blog-shared.
use blog_shared::root::{RelationLoader, RelationRequest};
use leptos::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use wasm_bindgen::JsCast;
use web_sys::{HtmlFormElement, HtmlInputElement};
/// Native form transport preserves source redirects, CSRF and server validation drafts.
pub(super) fn submit(data: serde_json::Value) {
    native_form(data, "post");
}
/// Submit original aggregate query values through native browser navigation.
pub(super) fn search(data: serde_json::Value) {
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
    // Shared Rust owns names, nesting, omission and scalar conversion.
    for (name, value) in blog_shared::form::fields(&data) {
        if let Ok(input) = document
            .create_element("input")
            .and_then(|input| input.dyn_into::<HtmlInputElement>().map_err(Into::into))
        {
            input.set_type("hidden");
            input.set_name(&name);
            input.set_value(&value);
            let _ = form.append_child(&input);
        }
    }
    if let Some(body) = document.body() {
        let _ = body.append_child(&form);
        let _ = form.submit();
    }
}
/// Real source search URLs, cancellable on disposal; each field retains only its latest lookup.
pub(super) fn relations() {
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
                // Only browser IO and its type conversions stay here; shared Rust decides
                // status/JSON/error/cancellation semantics before a completion is published.
                let response = fetch(&request.url, &controller).await.and_then(|text| {
                    blog_shared::browser::relation_response(
                        text.as_deref(),
                        alive.load(Ordering::Relaxed),
                        controller.signal().aborted(),
                    )
                });
                if let Ok(choices) = response {
                    request.complete.run(choices);
                }
            });
        },
    )));
}

/// Convert browser IO into plain response text and shared error variants.
async fn fetch(
    url: &str,
    controller: &web_sys::AbortController,
) -> Result<Option<String>, blog_shared::browser::RelationError> {
    use blog_shared::browser::RelationError;
    let window = web_sys::window().ok_or(RelationError::Fetch)?;
    let options = web_sys::RequestInit::new();
    options.set_signal(Some(&controller.signal()));
    let response =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_str_and_init(url, &options))
            .await
            .map_err(|_| RelationError::Fetch)?
            .dyn_into::<web_sys::Response>()
            .map_err(|_| RelationError::Response)?;
    // The generated RelationField already builds the source search URL in plain Rust;
    // this IO adapter forwards it unchanged, without rebuilding its query or policy.
    blog_shared::browser::response_status(response.status())?;
    let text = response.text().map_err(|_| RelationError::ReadBody)?;
    let text = wasm_bindgen_futures::JsFuture::from(text)
        .await
        .map_err(|_| RelationError::ReadBody)?;
    Ok(text.as_string())
}
