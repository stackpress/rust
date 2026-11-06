//! Local HTML download and image reads. There is no upload endpoint or network API.
use super::{Editor, command, document};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

/// Download an HTML Blob locally, revoking its temporary URL after the click.
pub(super) fn save(editor: Editor) {
    let Some(node) = editor.editable.get() else {
        return;
    };
    let parts = js_sys::Array::of1(&JsValue::from_str(&node.inner_html()));
    let options = web_sys::BlobPropertyBag::new();
    options.set_type("text/html");
    if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options)
        && let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob)
    {
        // A detached native anchor starts the browser download without a request API.
        if let Ok(anchor) = document().create_element("a") {
            let anchor: web_sys::HtmlAnchorElement = anchor.unchecked_into();
            anchor.set_href(&url);
            anchor.set_download("editor-content.html");
            anchor.click();
        }
        // Revoke even when anchor creation failed, so this action retains no Blob URL.
        let _ = web_sys::Url::revoke_object_url(&url);
    }
}

/// Read image bytes with FileReader only; every reader has one terminal callback.
pub(super) fn read_image(editor: Editor, file: web_sys::File) {
    // Preserve Frui's MIME-prefix admission; no extension or file-size policy is added.
    if !file.type_().starts_with("image/") {
        return;
    }
    let Ok(reader) = web_sys::FileReader::new() else {
        return;
    };
    let completed = reader.clone();
    // onloadend runs on success, failure and abort; once_into_js releases its closure
    // after invocation. A disposed root is refused before accessing its node references.
    let callback = Closure::once_into_js(move || {
        if let Ok(result) = completed.result()
            && let Some(data) = result.as_string()
        {
            command(editor, "insertImage", &data);
        }
    });
    reader.set_onloadend(Some(callback.unchecked_ref()));
    if reader.read_as_data_url(&file).is_err() {
        // A synchronous refusal has no terminal event: invoke once to release captures.
        let function: &js_sys::Function = callback.unchecked_ref();
        let _ = function.call0(&JsValue::NULL);
    }
}
