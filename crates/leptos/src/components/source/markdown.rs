//! G55 pulldown-cmark rendering and edit/preview composition, without a sanitizer.
use super::{SourceAppearance, SourceCodeEditor};
use leptos::prelude::*;

/// G55 Markdown rendering with raw HTML retained, using the already locked pulldown-cmark.
#[cfg(feature = "source")]
pub fn source_markdown(value: &str) -> String {
    use pulldown_cmark::{Options, Parser, html};
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let mut html = String::new();
    html::push_html(&mut html, Parser::new_ext(value, options));
    html
}

/// G55 Markdown view; no added sanitizer is applied to the original content path.
#[cfg(feature = "source")]
#[component]
pub fn SourceMarkdown(
    /// Source Markdown and embedded HTML.
    #[prop(into)]
    value: Signal<String>,
) -> impl IntoView {
    view! {<div class="sp-source-markdown" inner_html=move||source_markdown(&value.get())></div>}
}

/// Source edit/preview modes with the approved native textarea and pulldown-cmark renderer.
#[cfg(feature = "source")]
#[component]
pub fn SourceMarkdownEditor(
    /// Native textarea identity.
    #[prop(into)]
    id: String,
    /// Native form key.
    #[prop(into)]
    name: String,
    /// Caller-owned content, used for the original controlled preview.
    value: RwSignal<String>,
    /// Original onUpdate callback.
    #[prop(optional)]
    on_update: Option<Callback<String>>,
    /// Original default nine rows.
    #[prop(default = 9)]
    rows: u32,
    /// Native requirement.
    #[prop(optional)]
    required: bool,
    /// Textarea appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let preview = RwSignal::new(false);
    view! {<div class="sp-source-markdown-editor">
        <div class="sp-actions"><button type="button" aria-label="Edit Markdown" aria-pressed=move||(!preview.get()).to_string() on:click=move|_|preview.set(false)>"Edit"</button>
            <button type="button" aria-label="Preview Markdown" aria-pressed=move||preview.get().to_string() on:click=move|_|preview.set(true)>"Preview"</button></div>
        <div hidden=move||preview.get()><SourceCodeEditor id name value rows required appearance on_update=Callback::new(move|text|{if let Some(callback)=on_update{callback.run(text);}})/ ></div>
        <iframe title="Markdown preview" hidden=move||!preview.get() srcdoc=move||source_markdown(&value.get())></iframe>
    </div>}
}
