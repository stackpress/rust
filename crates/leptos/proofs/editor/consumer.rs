//! Shared real editor and generated source roles, rendered in all three modes.
use leptos::prelude::*;
use stackpress_framework_admin::Draft;
use stackpress_framework_schema::{__private::serde_json, Role};
use stackpress_leptos::components::{TextEditor, TextEditorOptions};
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
#[path = "generated/client/mod.rs"]
mod client;
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
#[path = "generated/source/mod.rs"]
mod source;
#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
#[path = "generated/stores/mod.rs"]
mod stores;
#[allow(dead_code)]
#[path = "generated/ui/mod.rs"]
mod ui;

/// Actual component source, shared by SSR and the WASM entry point.
pub fn app() -> impl IntoView {
    let initial = "<p>Seed <strong>HTML</strong></p>";
    let value = RwSignal::new(initial.to_owned());
    let events = RwSignal::new(Vec::<String>::new());
    let submitted_snapshots = RwSignal::new(Vec::<String>::new());
    let updates = RwSignal::new(Vec::<String>::new());
    let options = TextEditorOptions {
        history: true,
        font: true,
        size: true,
        format: true,
        paragraph: true,
        blockquote: true,
        style: true,
        color: true,
        highlight: true,
        text: true,
        text_style: true,
        remove: true,
        indent: true,
        align: true,
        rule: true,
        list: true,
        lineheight: true,
        table: true,
        link: true,
        image: true,
        image_gallery: true,
        video: true,
        audio: true,
        math: true,
        fullscreen: true,
        showblocks: true,
        code: true,
        preview: true,
        print: true,
        save: true,
        template: true,
    };
    let fields = ui::article::descriptors().unwrap();
    let draft = Draft::new(
        &fields,
        &serde_json::json!({"contents": "<p>Generated <em>HTML</em></p>"}),
        Role::Field,
    )
    .unwrap();
    let generated = ui::article::field(&draft, "generated").unwrap();
    let display = ui::article::view(
        &serde_json::json!({"contents":"<mark data-authored='yes'>Stored HTML</mark>"}),
        "display",
    )
    .unwrap();
    view! {
        <form id="editor-form">
            <TextEditor value options dir="ltr" name="contents" id="hidden"
                on_change=Callback::new(move |html: String| {
                    events.update(|log|log.push(html));
                    #[cfg(target_arch="wasm32")]
                    {
                        use wasm_bindgen::JsCast;
                        let hidden = document().get_element_by_id("hidden").unwrap().unchecked_into::<web_sys::HtmlInputElement>();
                        // Read inside the callback, before reactive rendering can flush.
                        submitted_snapshots.update(|log| log.push(hidden.value()));
                    }
                })
                on_update=Callback::new(move |update: stackpress_leptos::components::TextEditorUpdate| {
                    updates.update(|log|log.push(update.action));
                })/>
        </form>
        <output id="changes" hidden>{move || serde_json::to_string(&events.get()).unwrap()}</output>
        <output id="submitted-snapshots" hidden>{move || serde_json::to_string(&submitted_snapshots.get()).unwrap()}</output>
        <output id="updates" hidden>{move || serde_json::to_string(&updates.get()).unwrap()}</output>
        <button id="external" on:click=move |_|value.set("<p>External</p>".into())>"External value"</button>
        <section id="generated">{generated}</section>
        <section id="generated-display">{display}</section>
    }
}

/// Attach the actual shared component, returning a root disposer.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start() -> js_sys::Function {
    use wasm_bindgen::{JsCast, closure::Closure};
    std::panic::set_hook(Box::new(|info| {
        let global = js_sys::global();
        let console = js_sys::Reflect::get(&global, &"console".into()).unwrap();
        let report = js_sys::Reflect::get(&console, &"error".into())
            .unwrap()
            .unchecked_into::<js_sys::Function>();
        let _ = report.call1(&console, &info.to_string().into());
    }));
    let root = document()
        .get_element_by_id("app")
        .unwrap()
        .unchecked_into::<web_sys::HtmlElement>();
    #[cfg(feature = "hydrate")]
    let handle = leptos::mount::hydrate_from(root, app);
    #[cfg(feature = "csr")]
    let handle = leptos::mount::mount_to(root, app);
    let mut handle = Some(handle);
    Closure::wrap(Box::new(move || {
        handle.take();
    }) as Box<dyn FnMut()>)
    .into_js_value()
    .unchecked_into()
}
