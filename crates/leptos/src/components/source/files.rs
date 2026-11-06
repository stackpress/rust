//! Local file selection and caller-supplied upload hooks; no upload transport is built in.
use super::{SourceAppearance, file_state::FileController};
use crate::components::icons::{IconKind, icon};
use leptos::prelude::*;
use std::sync::Arc;

/// A selected browser file, with portable metadata and its browser-only native object.
#[derive(Clone, Debug)]
pub struct SourceFile {
    /// Filename supplied by the browser.
    pub name: String,
    /// MIME type supplied by the browser, without added admission checks.
    pub media_type: String,
    /// Browser-reported byte size.
    pub size: f64,
    /// The original File for the application upload handler.
    #[cfg(target_arch = "wasm32")]
    pub file: leptos::web_sys::File,
}

/// One selected upload batch and its root-local URL completion callback.
#[derive(Clone)]
pub struct SourceUpload {
    /// Single control passes one file; list control passes the entire FileList.
    pub files: Vec<SourceFile>,
    /// Application returns stored URLs. Single controls consume the first URL.
    pub complete: SourceUploadCompletion,
}

/// An application-retainable completion function, independent of the mounted Owner's arena.
/// Calling it after disposal is harmless: only live root-local signals are updated.
#[derive(Clone)]
pub struct SourceUploadCompletion(Arc<dyn Fn(Vec<String>) + Send + Sync>);
impl SourceUploadCompletion {
    /// Retain an application upload continuation without retaining the component root.
    #[cfg(target_arch = "wasm32")]
    fn new(complete: impl Fn(Vec<String>) + Send + Sync + 'static) -> Self {
        Self(Arc::new(complete))
    }
    /// Return uploaded URLs; single-file controls use only the first returned URL.
    pub fn run(&self, urls: Vec<String>) {
        (self.0)(urls);
    }
}

/// Preserve source upload/reset/queue state independently of DOM rendering.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceFileState {
    /// Current uploaded URLs, initialized from defaultValue/value only at mount.
    pub urls: Vec<String>,
    /// Number of list files awaiting completion; single controls use one busy slot.
    pub queued: usize,
}
impl SourceFileState {
    /// Append returned URLs and subtract their count, clamping the source queue at zero.
    pub fn complete(&mut self, urls: Vec<String>, multiple: bool) {
        if multiple {
            self.queued = self.queued.saturating_sub(urls.len());
            self.urls.extend(urls);
        } else {
            self.queued = 0;
            self.urls = urls
                .into_iter()
                .take(1)
                .filter(|url| !url.is_empty())
                .collect();
        }
    }
    /// Removing a list item also decrements a pending count, as Frui does.
    pub fn remove(&mut self, index: usize) {
        self.queued = self.queued.saturating_sub(1);
        if index < self.urls.len() {
            self.urls.remove(index);
        }
    }
}

/// Frui FileInput/ImageInput/FileList/ImageList behavior behind the same app upload hook.
#[component]
pub fn SourceFileInput(
    /// Native selection control ID.
    #[prop(into)]
    id: String,
    /// Hidden successful URL form key, repeated for lists.
    #[prop(into)]
    name: String,
    /// Mount-only URL seed; browser file controls cannot be controlled by URL updates.
    #[prop(optional)]
    initial: Vec<String>,
    /// List mode keeps the chooser visible after uploads.
    #[prop(optional)]
    multiple: bool,
    /// Image mode adds accept=image/* and source URL previews.
    #[prop(optional)]
    image: bool,
    /// Application-owned upload operation. Without it, selection does not start an upload.
    #[prop(optional_no_strip)]
    on_upload: Option<Callback<SourceUpload>>,
    /// Original native selection event, called after the upload handler is invoked.
    #[prop(optional)]
    on_change: Option<Callback<leptos::ev::Event>>,
    /// URLs are published after initialization and each source upload/list removal.
    #[prop(optional)]
    on_update: Option<Callback<Vec<String>>>,
    /// Localizable busy text.
    #[prop(default="Uploading...".into(),into)]
    uploading: String,
    /// Native requirement, forwarded to the unnamed chooser.
    #[prop(optional)]
    required: bool,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let state = FileController::new(initial, multiple);
    // Frui publishes initial list state even when empty; a single publishes nonempty URLs only.
    Effect::new(move |_| {
        let urls = state.urls.get();
        if (multiple || !urls.is_empty())
            && let Some(callback) = on_update
        {
            callback.run(urls);
        }
    });
    let hidden_name = StoredValue::new(name);
    view! {<div class=format!("sp-source-files {}",appearance.class_name) style=appearance.style>
        <Show when=move||multiple ||state.urls.with(Vec::is_empty)&&state.queued.get()==0>
            <input class="sp-input" id=id.clone() type="file" multiple=multiple accept=if image{Some("image/*")}else{None} required=required
                on:change=move|event| {
                    #[cfg(target_arch="wasm32")]
                    if let Some(upload)=on_upload {
                        use wasm_bindgen::JsCast;
                        let files=event.target().and_then(|target|target.dyn_into::<leptos::web_sys::HtmlInputElement>().ok()).and_then(|input|input.files());
                        if let Some(files)=files && (multiple ||files.length()>0) {
                            let files=(0..if multiple{files.length()}else{1}).filter_map(|index|files.get(index)).map(|file|SourceFile{name:file.name(),media_type:file.type_(),size:file.size(),file}).collect::<Vec<_>>();
                            let ticket=state.begin(files.len());
                            upload.run(SourceUpload{files,complete:SourceUploadCompletion::new(move|urls:Vec<String>|{state.complete(&ticket,urls);})});
                        }
                    }
                    #[cfg(not(target_arch="wasm32"))] let _=on_upload;
                    if let Some(callback)=on_change {callback.run(event);}
                }/>
        </Show>
        {move||state.urls.get().into_iter().enumerate().map(move|(index,url)| {
            let src=url.clone();let href=url.clone();
            view!{<div class="sp-source-file">
                {image.then(||view!{<img class="sp-source-file-preview" src=src alt="preview"/>})}
                <a href=href target="_blank" rel="noreferrer">{url.clone()}</a>
                <button type="button" class="sp-icon-action" aria-label=if multiple{"Remove file"}else{"Reset file"}
                    on:click=move|_|state.remove(index)>{icon(IconKind::X)}</button>
                <input type="hidden" name=hidden_name.get_value() value=url/>
            </div>}
        }).collect_view()}
        <Show when=move||{state.queued.get()>0 && (multiple||state.urls.with(Vec::is_empty))}>
            <span role="status">{uploading.clone()}</span>
        </Show>
    </div>}
}
