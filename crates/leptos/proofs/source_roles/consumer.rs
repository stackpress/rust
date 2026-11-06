//! All canonical source roles are aggregate-generated and compiled in SSR/hydrate/CSR.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft, source::roles::SourceUploader};
use stackpress_framework_schema::{__private::serde_json, Role};
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

/// The same source tree and deterministic data render on each execution host.
pub fn app() -> impl IntoView {
    let seed: serde_json::Value = serde_json::from_str(include_str!("seed.json")).unwrap();
    let uploads = RwSignal::new(Vec::<String>::new());
    provide_context(SourceUploader(Callback::new(
        move |upload: stackpress_leptos::components::source::SourceUpload| {
            #[cfg(target_arch = "wasm32")]
            {
                use wasm_bindgen::{JsCast, closure::Closure};
                for file in &upload.files {
                    assert_eq!(file.file.name(), file.name);
                }
                if upload
                    .files
                    .first()
                    .is_some_and(|file| file.name == "late.txt")
                {
                    let complete = upload.complete.clone();
                    let finish = Closure::once_into_js(move || {
                        complete.run(vec!["/uploaded/late.txt".into()])
                    });
                    js_sys::Reflect::set(
                        &js_sys::global(),
                        &"finishLateUpload".into(),
                        &finish.unchecked_into::<js_sys::Function>(),
                    )
                    .unwrap();
                    return;
                }
            }
            let urls = upload
                .files
                .iter()
                .map(|file| format!("/uploaded/{}", file.name))
                .collect::<Vec<_>>();
            uploads.update(|values| {
                values.extend(
                    upload
                        .files
                        .iter()
                        .map(|file| format!("{}:{}:{}", file.name, file.size, file.media_type)),
                )
            });
            upload.complete.run(urls);
        },
    )));
    let fields = ui::role_proof::descriptors().unwrap();
    let draft = Draft::new(&fields, &seed, Role::Field).unwrap();
    let snapshot = draft.clone();
    let tracked_fields = fields.clone();
    let filter = Draft::new(&fields, &seed, Role::Filter).unwrap();
    let span = Draft::new(&fields, &seed, Role::Span).unwrap();
    let fields = ui::role_proof::field(&draft, "generated").unwrap();
    let filters = ui::role_proof::filter(&filter, "query").unwrap();
    let spans = ui::role_proof::span(&span, "inline").unwrap();
    let displays = ui::role_proof::view(&seed, "display").unwrap();
    let lists = ui::role_proof::list(&seed, "list").unwrap();
    let check = RwSignal::new(Some("check".to_owned()));
    let square = RwSignal::new(Some("square".to_owned()));
    let circle = RwSignal::new(Some("circle".to_owned()));
    // The same pure arithmetic edge cases run in native SSR and each browser renderer.
    let formulas = [
        "2+3*4", "(2+3)*4", "-7%3", "1/0", "-1/0", "0/0", "1e308*2", "(1 +)",
    ]
    .map(|formula| {
        stackpress_leptos::components::source::source_formula(formula, 0.0, &[]).to_string()
    })
    .join("|");
    view! {<h1>"Source roles proof"</h1><form id="generated-field">{fields}</form><details><summary>"Filters"</summary><form id="generated-filter">{filters}</form></details><details><summary>"Spans"</summary><form id="generated-span">{spans}</form></details>
        <section id="generated-view">{displays}</section><section id="generated-list">{lists}</section>
        <section id="radio-profile" class="sp-actions" aria-label="Source radio styles">
            <stackpress_leptos::components::source::SourceRadio id="profile-check" name="profile-check" value="check" selected=check check=true rounded=true blue=true label="Check" aria_label="Check"/>
            <stackpress_leptos::components::source::SourceRadio id="profile-square" name="profile-square" value="square" selected=square square=true orange=true label="Square" aria_label="Square"/>
            <stackpress_leptos::components::source::SourceRadio id="profile-circle" name="profile-circle" value="circle" selected=circle label="Circle" aria_label="Circle"/>
        </section>
        <output id="formulas" hidden>{formulas}</output>
        <output id="snapshot" hidden>{move||{for field in &tracked_fields{if let Some(signal)=snapshot.get(&field.wire_name){let _=signal.get();}}serde_json::to_string(&snapshot.snapshot().unwrap()).unwrap()}}</output><output id="uploads" hidden>{move||serde_json::to_string(&uploads.get()).unwrap()}</output>
    }
}
/// Attach the actual generated role components, returning a root disposer.
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
