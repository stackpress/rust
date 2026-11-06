//! G55 native monospaced code editing/display, with shared UTF-16 tab edits.
use super::SourceAppearance;
use leptos::prelude::*;

/// Plain escaped monospaced source display replacing Shiki under G55.
#[component]
pub fn SourceCode(
    /// Source text; invalid JSON remains ordinary text.
    #[prop(into)]
    value: String,
    /// Authored language label, with no syntax highlighting or dynamic grammar load.
    #[prop(optional, into)]
    language: String,
    /// Explicit source language-label toggle.
    #[prop(optional)]
    show_language: bool,
    /// Source numbers/showLineNumbers merged by the metadata adapter.
    #[prop(optional)]
    numbers: bool,
    /// First displayed line number.
    #[prop(default = 1.0)]
    starting_line_number: f64,
    /// Original default-style switch.
    #[prop(default = true)]
    add_default_styles: bool,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Language-label appearance.
    #[prop(optional)]
    language_appearance: SourceAppearance,
) -> impl IntoView {
    let text = if numbers {
        let lines = value.split('\n').collect::<Vec<_>>();
        let count = lines.len();
        lines.into_iter().enumerate().map(|(index,line)|view!{<span class="sp-source-code-line"><span class="sp-source-line-number" aria-hidden="true">{format!("{} ",starting_line_number+index as f64)}</span>{line.to_owned()}</span>{if index+1<count{"\n"}else{""}}}).collect_view().into_any()
    } else {
        value.into_any()
    };
    view! {<div class=format!("{} {}",if add_default_styles{"sp-source-code"}else{""},appearance.class_name) style=appearance.style>
        {show_language.then(||view!{<span class=language_appearance.class_name style=language_appearance.style>{language}</span>})}
        <pre><code>{text}</code></pre>
    </div>}
}

/// Monospaced code editor with native selection, tab insertion and no JSON validity enforcement.
#[component]
pub fn SourceCodeEditor(
    /// Native textarea ID.
    #[prop(into)]
    id: String,
    /// Successful native textarea key.
    #[prop(into)]
    name: String,
    /// Shared root-owned content; no parsing/formatting mutation is imposed.
    value: RwSignal<String>,
    /// Original onUpdate callback.
    #[prop(optional)]
    on_update: Option<Callback<String>>,
    /// Native requirement.
    #[prop(optional)]
    required: bool,
    /// Native visible rows.
    #[prop(default = 10)]
    rows: u32,
    /// Authored appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let node = NodeRef::<leptos::html::Textarea>::new();
    let initial = super::super::inputs::textarea_seed(&value.get_untracked());
    let publish = Callback::new(move |text: String| {
        value.set(text.clone());
        if let Some(callback) = on_update {
            callback.run(text);
        }
    });
    view! {<textarea class=format!("sp-input sp-source-code-editor {}",appearance.class_name) style=appearance.style id=id name=name rows=rows required=required
    node_ref=node inner_html=initial prop:value=move||value.get() on:input=move|event|publish.run(event_target_value(&event))
    on:keydown=move|event:leptos::ev::KeyboardEvent|{
        if event.key()!="Tab"||event.is_composing(){return;}event.prevent_default();
        #[cfg(target_arch="wasm32")]
        if let Some(node)=node.get(){
            let text=node.value();let start=node.selection_start().ok().flatten().unwrap_or(0) as usize;let end=node.selection_end().ok().flatten().unwrap_or(start as u32) as usize;
            let (next,cursor)=source_code_tab(&text,start,end,event.shift_key());node.set_value(&next);let _=node.set_selection_range(cursor as u32,cursor as u32);publish.run(next);
        }
    }></textarea>}
}

/// UTF-16 tab edit shared by native tests and the browser textarea adapter.
/// Tab replaces selection; Shift+Tab removes one leading tab from the caret's line.
pub fn source_code_tab(text: &str, start: usize, end: usize, backward: bool) -> (String, usize) {
    let mut units = text.encode_utf16().collect::<Vec<_>>();
    let start = start.min(units.len());
    let end = end.max(start).min(units.len());
    if backward {
        let line = units[..start]
            .iter()
            .rposition(|c| *c == 10)
            .map_or(0, |index| index + 1);
        if units.get(line) == Some(&9) {
            units.remove(line);
            (String::from_utf16_lossy(&units), start.saturating_sub(1))
        } else {
            (text.into(), start)
        }
    } else {
        units.splice(start..end, [9]);
        (String::from_utf16_lossy(&units), start + 1)
    }
}
