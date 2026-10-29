//! One real component source for native SSR, hydration and CSR; no host authority.
use leptos::prelude::*;
use stackpress_leptos::{
    components::{
        ADVANCED_STYLES, Dialog, FieldControl, FieldError, Form, FormState, Input, Table,
    },
    resources::cancel_on_cleanup,
    rich::SafeHtml,
};

#[cfg(target_arch = "wasm32")]
mod browser;

/// Fixed hostile public Resource value, used for exact native/WASM replay checks.
pub const RESOURCE_TEXT: &str = "</script><script>globalThis.pwned=1</script>&\"\u{2028}\u{2029}";
pub use safety::{CORPUS, TEXTAREA_PAYLOAD};

mod breadth;
mod core;
mod safety;
mod suggestion_loader;
pub use breadth::BreadthDraft;
pub use collection_proof::parse_presence;
pub mod collection_proof;
/// Typed early nested-field presence shared with the thin client adapter.
pub use stackpress_leptos::components::FieldValue;
mod seed;
pub use core::CoreDraft;
pub use seed::{GallerySeed, decode_initial, decode_preference};

/// Select explicit English/French phrase data without browser-language inference.
fn phrase(locale: &str, english: &'static str, french: &'static str) -> &'static str {
    if locale == "fr" { french } else { english }
}

/// App-owned local form validation; no record/database mutation is implied.
fn validate_name(name: &str) -> FormState {
    if name.trim().is_empty() {
        FormState::Invalid(vec![FieldError {
            field: "profile-name".into(),
            message: "Enter a name".into(),
        }])
    } else {
        FormState::Saved
    }
}

/// Actual asynchronous public loader; its native and browser acquisition differ only here.
async fn load_resource(base: String) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        browser::resource(base)
            .await
            .unwrap_or_else(|_| "Resource unavailable".to_owned())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = base;
        tokio::task::yield_now().await;
        RESOURCE_TEXT.to_owned()
    }
}

/// Compose real table/dialog/form/rich/locale and Resource/Suspense/Head behavior.
#[component]
pub fn Gallery(
    /// The same public initial seed on SSR/hydration/CSR.
    seed: GallerySeed,
    /// Native input edits captured before hydration; no acquisition occurs.
    #[prop(optional)]
    initial_core: Option<CoreDraft>,
    /// Early M2 native drafts, read once without acquiring data.
    #[prop(optional)]
    initial_breadth: Option<BreadthDraft>,
    /// Adopt an early native input edit before hydrating, without a fetch.
    #[prop(optional)]
    initial_draft: Option<String>,
    /// Native safety-sample edit captured before hydration.
    #[prop(optional)]
    initial_safety: Option<String>,
) -> impl IntoView {
    let locale = RwSignal::new(seed.locale.clone());
    let location = RwSignal::new(seed.location.clone());
    let draft = RwSignal::new(initial_draft.unwrap_or_else(|| seed.name.clone()));
    let status = RwSignal::new(FormState::Editing);
    let open = RwSignal::new(false);
    let nested = RwSignal::new(false);
    let locked = RwSignal::new(false);
    let mounted = RwSignal::new(true);
    let trigger = RwSignal::new(true);
    let saved_error = RwSignal::new(String::new());
    let base = seed.base.clone();
    let resource = Resource::new(
        || (),
        move |()| cancel_on_cleanup(load_resource(base.clone())),
    );
    let resource_title = RwSignal::new(String::new());
    let submit = Callback::new(move |()| validate_name(&draft.get()));
    let draft_element = NodeRef::<leptos::html::Input>::new();
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(input) = draft_element.get() {
            let value = draft.get();
            if input.value() != value {
                input.set_value(&value);
            }
        }
        if let Some(html) = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.document_element())
        {
            let _ = html.set_attribute("lang", &locale.get());
        }
    });
    let base = seed.base.clone();
    let choose = move |selected: &'static str| {
        #[cfg(target_arch = "wasm32")]
        browser::preference(
            base.clone(),
            selected.to_owned(),
            location.get(),
            move |ack| match ack {
                Ok((selected, path)) => {
                    locale.set(selected);
                    location.set(path);
                    saved_error.set(String::new());
                }
                Err(()) => saved_error.set("Language could not be saved".into()),
            },
        );
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (&base, selected, location);
    };
    let french = choose.clone();
    let styles = format!(
        "{}{}{}{}",
        include_str!("../../../../crates/leptos/src/components/styles.css"),
        ADVANCED_STYLES,
        stackpress_leptos::components::CORE_STYLES,
        stackpress_leptos::components::BREADTH_STYLES
    );
    view! {
        <stackpress_leptos::leptos_meta::Title text=move || format!("{} · {}", phrase(&locale.get(), "UI gallery", "Galerie UI"), resource_title.get())/>
        <style>{styles}</style>
        <main class="sp-root" data-initial-theme-pending=seed.theme.is_none().then_some("") data-theme=seed.theme.clone().unwrap_or("light".into())><div class="sp-blank">
            <header class="sp-head"><h1>{move || phrase(&locale.get(), "UI gallery", "Galerie UI")}</h1>
                <nav class="sp-actions" aria-label="Language"><button id="english" type="button" on:click=move |_| choose("en")>"English"</button>
                <button id="french" type="button" on:click=move |_| french("fr")>"Français"</button></nav>
            </header>
            <p id="preference-error" role="alert">{move || saved_error.get()}</p>
            <label for="draft">{move || phrase(&locale.get(), "Retained draft", "Brouillon conservé")}</label>
            <input class="sp-input" node_ref=draft_element id="draft" value=seed.name on:input=move |event| { draft.set(event_target_value(&event)); status.set(FormState::Editing); }/>
            <Table caption=move || phrase(&locale.get(), "Profiles", "Profils").to_owned()>
                <thead><tr><th scope="col">"ID"</th><th scope="col">"Name"</th><th scope="col">"Notes"</th></tr></thead>
                <tbody><For each=|| vec![("1", "Ada Lovelace"),("2", "Grace Hopper"),("3", "Yuki Tanaka")] key=|row| row.0 children=|(id,name)| view! {
                    <tr data-row-id=id><th scope="row">{id}</th><td>{name}</td><td>"Owned public row"</td></tr>
                }/></tbody>
            </Table>
            <div class="sp-actions" role="group" aria-label="Gallery actions"><Show when=move || trigger.get()><button id="opener" type="button" on:click=move |_| open.set(true)>"Edit profile"</button></Show>
            <button id="restore" type="button" on:click=move |_| { trigger.set(true); mounted.set(true); }>"Restore dialog"</button>
            </div>
            <Show when=move || mounted.get()>
                <Dialog id="editor" open title=move || phrase(&locale.get(), "Edit profile", "Modifier le profil").to_owned() initial_focus_id="profile-name" fallback_focus_id="draft">
                    <Form id="profile-form" state=status submit>
                        <FieldControl for_id="profile-name" label="Name" error=move || status.get().errors().first().map(|error| error.message.clone()).unwrap_or_default()>
                            <Input id="profile-name" name="name" value=draft described_by="profile-name-help profile-name-error" invalid=move || !status.get().errors().is_empty()/>
                        </FieldControl>
                        <button id="save" type="submit">"Save"</button>
                        <button id="close" type="button" on:click=move |_| open.set(false)>"Close"</button>
                    </Form>
                    <stackpress_leptos::components::ActionGroup label="Dialog actions"><button id="nested-opener" type="button" on:click=move |_| nested.set(true)>"Nested dialog"</button>
                    <button id="locked-opener" type="button" on:click=move |_| locked.set(true)>"Explicit close dialog"</button>
                    <button id="remove-opener" type="button" on:click=move |_| trigger.set(false)>"Remove opener"</button>
                    <button id="unmount" type="button" on:click=move |_| { open.set(false); mounted.set(false); }>"Unmount dialog"</button>
                    </stackpress_leptos::components::ActionGroup><Dialog id="nested" open=nested title="Nested dialog" initial_focus_id="nested-close" fallback_focus_id="profile-name">
                        <button id="nested-close" type="button" on:click=move |_| nested.set(false)>"Close nested"</button>
                    </Dialog>
                    <Dialog id="locked" open=locked title="Explicit close dialog" close_on_escape=false initial_focus_id="locked-close" fallback_focus_id="profile-name">
                        <button id="locked-close" type="button" on:click=move |_| locked.set(false)>"Close dialog"</button>
                    </Dialog>
                </Dialog>
            </Show>
            <safety::SafetySamples initial=initial_safety.unwrap_or_else(|| TEXTAREA_PAYLOAD.into())>
                <button id="refetch" type="button" on:click=move |_| resource.refetch()>"Reload safety resource"</button>
                <section aria-label="Async safety resource"><Suspense fallback=|| view! { <p>"Loading safety resource"</p> }>
                    {move || resource.get().map(move |result| {
                        let value = result.unwrap_or_else(|error| error.to_string());
                        // Keep reactive Head behavior while hostile data stays in this section.
                        resource_title.set("Samples ready".into());
                        view! { <p id="resource">{value}</p> }
                    })}
                </Suspense></section>
            </safety::SafetySamples>
            <core::CoreGallery initial=initial_core.unwrap_or_default() base=seed.base/>
            <breadth::BreadthGallery initial=initial_breadth.unwrap_or_default()/>
        </div></main>
    }
}

/// Install only the normal scoped metadata context for a browser root.
pub fn provide_meta() {
    stackpress_leptos::leptos_meta::provide_meta_context();
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    use stackpress_framework_view::leptos::{
        async_render::AsyncLeptosRenderer, render::initialize_ssr,
    };
    use stackpress_framework_view::{
        AsyncPageRenderer, BasePath, PageAssets, PageContract, PageId, PageSnapshot,
        PublicLocation, PublicStatus,
    };

    /// Form validation preserves nonempty Unicode/zero text and names invalid controls.
    #[test]
    fn local_form_validation() {
        assert!(!validate_name(" ").errors().is_empty());
        assert_eq!(validate_name("0"), FormState::Saved);
        assert_eq!(validate_name("雪"), FormState::Saved);
        assert_eq!(phrase("fr", "Name", "Nom"), "Nom");
        assert_eq!(phrase("en", "Name", "Nom"), "Name");
    }

    /// Concurrent independent roots retain their own locale/Head and real replay data.
    #[test]
    fn shared_gallery_renders_independent_async_roots() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        runtime.block_on(async {
            initialize_ssr().unwrap();
            let render = |locale: &str| {
                let mut contract = PageContract::new(1, "s8-m2").unwrap();
                contract.locale = locale.into();
                let snapshot = PageSnapshot {
                    revision: 1,
                    build: "s8-m2".into(),
                    page: PageId::try_from("gallery".to_owned()).unwrap(),
                    location: format!("/{locale}/gallery"),
                    locale: locale.into(),
                    theme: Some("light".into()),
                    status: PublicStatus::ok(),
                    data: GallerySeed {
                        locale: locale.into(),
                        theme: Some("light".into()),
                        base: String::new(),
                        location: format!("/{locale}/gallery"),
                        name: "Ada".into(),
                    },
                };
                let renderer = AsyncLeptosRenderer::new(
                    contract,
                    PageAssets {
                        theme_bootstrap: None,
                        build: "s8-m2".into(),
                        base: BasePath::default(),
                        module: PublicLocation::new("/assets/start.js").unwrap(),
                        stylesheet: None,
                    },
                    |snapshot: &PageSnapshot<GallerySeed>| {
                        Ok(view! {<Gallery seed=snapshot.data.clone()/>})
                    },
                )
                .unwrap();
                async move { renderer.render_async(snapshot).await.unwrap().into_html() }
            };
            let (en, fr) = futures::future::join(render("en"), render("fr")).await;
            assert!(en.contains("<title>UI gallery"));
            assert!(fr.contains("<title>Galerie UI"));
            for html in [en, fr] {
                assert!(html.contains("stackpress-resources"));
                assert!(html.contains("<table"));
                assert!(html.contains("id=\"editor\""));
            }
        });
    }
}

/// Return actual compiled sanitizer output for exact native/WASM corpus parity.
pub fn corpus_json() -> String {
    let corpus: Vec<_> = CORPUS
        .iter()
        .map(|input| {
            SafeHtml::html(input)
                .expect("bounded fixed corpus")
                .as_str()
                .to_owned()
        })
        .collect();
    serde_json::to_string(&corpus).expect("owned strings serialize")
}
