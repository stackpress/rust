//! Frui-compatible HTML editing, independent of application policy or storage.
//!
//! Browser mutations live in the wasm-only DOM adapter. The shared toolbar and
//! form surfaces render identically for SSR, hydration and CSR. HTML is intentionally
//! unsanitized; callers own content security, including URLs and inserted markup.
use leptos::tachys::html::attribute::any_attribute::AnyAttribute;
use leptos::{html, prelude::*};

#[cfg(target_arch = "wasm32")]
mod dom;
#[cfg(all(test, feature = "ssr"))]
mod tests;
mod toolbar;

/// Opt-in editor presentation. Every flag defaults to false, exactly as frui.
/// Direction is a separate TextEditor prop because its string also seeds the document.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TextEditorOptions {
    /// Undo and redo.
    pub history: bool,
    /// Font-family selector.
    pub font: bool,
    /// Font-size selector (original execCommand values).
    pub size: bool,
    /// Block-format selector.
    pub format: bool,
    /// Paragraph action.
    pub paragraph: bool,
    /// Blockquote action.
    pub blockquote: bool,
    /// Bold, italic and underline.
    pub style: bool,
    /// Text color picker.
    pub color: bool,
    /// Highlight color picker.
    pub highlight: bool,
    /// Strikethrough, subscript and superscript.
    pub text: bool,
    /// Named text-style selector.
    pub text_style: bool,
    /// Remove-format action.
    pub remove: bool,
    /// Indent and outdent.
    pub indent: bool,
    /// Left, center and right alignment.
    pub align: bool,
    /// Horizontal rule insertion.
    pub rule: bool,
    /// Ordered and unordered lists.
    pub list: bool,
    /// Line-height selector.
    pub lineheight: bool,
    /// Prompt-based table insertion.
    pub table: bool,
    /// Prompt-based HTTP(S) link insertion.
    pub link: bool,
    /// Local image-file insertion.
    pub image: bool,
    /// Local multiple-image insertion.
    pub image_gallery: bool,
    /// Prompt-based video iframe insertion.
    pub video: bool,
    /// Prompt-based audio insertion.
    pub audio: bool,
    /// Prompt-based math placeholder insertion.
    pub math: bool,
    /// Fullscreen action.
    pub fullscreen: bool,
    /// Block-boundary visibility action.
    pub showblocks: bool,
    /// HTML source toggle.
    pub code: bool,
    /// Preview in a new window.
    pub preview: bool,
    /// Browser print action.
    pub print: bool,
    /// HTML file download.
    pub save: bool,
    /// Original two template choices.
    pub template: bool,
}

/// A source-compatible update notification. Commands publish an input update first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextEditorUpdate {
    /// Exact stored HTML, never trimmed or sanitized.
    pub value: String,
    /// "input", "codeViewToggle", or the original browser command name.
    pub action: String,
}

/// Root-local form value, mode and callbacks; node references never cross a root.
#[derive(Clone, Copy)]
struct Editor {
    /// Current exact hidden-input value; browser input is authoritative during edits.
    value: RwSignal<String>,
    /// Source mode disables formatting and shows the textarea.
    code: RwSignal<bool>,
    /// Both native and browser renders retain the same editable node slot.
    editable: NodeRef<html::Div>,
    /// Source surface exists in both modes so switching does not discard its value.
    textarea: NodeRef<html::Textarea>,
    /// Native form field is written synchronously before application callbacks.
    hidden: NodeRef<html::Input>,
    /// Hidden local image picker; never submits file bytes.
    file: NodeRef<html::Input>,
    /// Hidden local gallery picker, independent of the form field.
    gallery: NodeRef<html::Input>,
    /// Application callback receives HTML after every source change.
    on_change: Option<Callback<String>>,
    /// Source action callback follows the change callback.
    on_update: Option<Callback<TextEditorUpdate>>,
}
impl Editor {
    /// Publish without coercion; a command can follow this with its second update.
    fn publish(self, value: String, action: &str) {
        self.publish_form(value, None, action);
    }

    /// Preserve Frui's immediate form write separately from its next reactive value.
    /// Exiting source mode publishes normalized DOM HTML while callbacks can still
    /// observe the authored source in the hidden input until the reactive update.
    fn publish_form(self, value: String, form_value: Option<&str>, action: &str) {
        // FormData/submission inside onChange must see Frui's synchronous hidden
        // write, without waiting for the hidden input's reactive value property.
        #[cfg(target_arch = "wasm32")]
        if let Some(hidden) = self.hidden.get_untracked() {
            hidden.set_value(form_value.unwrap_or(&value));
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = form_value;
        // Release the signal write before calling application code, which may echo
        // a controlled value or synchronously inspect this same form draft.
        self.value.set(value.clone());
        if let Some(callback) = self.on_change {
            callback.run(value.clone());
        }
        self.updated(value, action);
    }

    /// Keep onUpdate distinct so execCommand does not duplicate onChange.
    fn updated(self, value: String, action: &str) {
        // Keep the source action literal independent of the caller's HTML ownership.
        if let Some(callback) = self.on_update {
            callback.run(TextEditorUpdate {
                value,
                action: action.into(),
            });
        }
    }

    /// Dispatch only on the browser; SSR has markup and no ambient DOM side effects.
    fn act(self, action: toolbar::Action, value: &str) {
        #[cfg(target_arch = "wasm32")]
        dom::act(self, action, value);
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (self, action, value);
    }

    /// Copy HTML from contentEditable after an actual browser input.
    fn input(self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(editor) = self.editable.get() {
            self.publish(editor.inner_html(), "input");
        }
    }
}

/// Compact HTML editor ported from frui/form/TextEditor, without an editor library.
///
/// Every toolbar flag is off by default. Individual flags override the options
/// carrier. Extra attributes are applied to the hidden input, like frui's spread;
/// name/class_name/id/disabled/required therefore describe that form field.
/// The visible editing surface has its own editor_id and accessible name.
///
/// HTML is inserted without sanitization and emitted from the active editing
/// surface. Closing source view returns the browser's normalized innerHTML.
/// This component adds no sanitization or
/// URL/content policy beyond frui's own prompt checks; applications secure their
/// stored/displayed HTML. Use the separate rich feature for its explicit sanitizer.
///
/// The source component defaults value to an empty string even with defaultValue.
/// default_value seeds the first render; the value effect subsequently reconciles
/// to value (empty when omitted). External changes update the hidden value and
/// inactive surface, preserving the active caret, just as the source effects do.
///
/// Native SSR emits initial HTML without running browser actions. Build ssr,
/// hydrate and csr separately, and provide the same initial props when hydrating.
///
/// # Examples
///
/// ```
/// use stackpress_leptos::leptos::prelude::*;
/// use stackpress_leptos::components::{HTML, TextEditor};
///
/// fn article_editor() -> impl IntoView {
///     let contents = RwSignal::new("<p>Article</p>".to_owned());
///     view! {
///         <TextEditor name="contents" value=contents style=true code=true
///             on_change=Callback::new(move |html: String| contents.set(html))/>
///         <HTML value=contents/>
///     }
/// }
/// ```
#[component]
pub fn TextEditor(
    /// External value; defaults to empty, matching the original component.
    #[prop(optional, into)]
    value: Signal<String>,
    /// Initial fallback before the source's value effect runs.
    #[prop(optional, into)]
    default_value: String,
    /// Native hidden-input form key.
    #[prop(optional, into)]
    name: String,
    /// Native hidden-input identity.
    #[prop(optional, into)]
    id: String,
    /// Class for the hidden input, matching frui className.
    #[prop(optional, into)]
    class_name: String,
    /// Optional label target for the visible contentEditable surface.
    #[prop(optional, into)]
    editor_id: String,
    /// Hidden-input disabled attribute; like frui, does not disable the editor.
    #[prop(optional)]
    disabled: bool,
    /// Hidden-input required attribute; authoritative validation belongs to the host.
    #[prop(optional)]
    required: bool,
    /// Source error prop is reserved and currently has no rendering effect.
    #[prop(optional, into)]
    error: String,
    /// Arbitrary native hidden-input attributes/events, including data and ARIA.
    #[prop(optional)]
    attributes: Vec<AnyAttribute>,
    /// All toolbar options can be passed together by a metadata adapter.
    #[prop(optional)]
    options: TextEditorOptions,
    /// Undo and redo; overrides the corresponding options flag.
    #[prop(optional)]
    history: Option<bool>,
    /// Font-family selector; overrides the corresponding options flag.
    #[prop(optional)]
    font: Option<bool>,
    /// Font-size selector (original execCommand values); overrides the corresponding options flag.
    #[prop(optional)]
    size: Option<bool>,
    /// Block-format selector; overrides the corresponding options flag.
    #[prop(optional)]
    format: Option<bool>,
    /// Paragraph action; overrides the corresponding options flag.
    #[prop(optional)]
    paragraph: Option<bool>,
    /// Blockquote action; overrides the corresponding options flag.
    #[prop(optional)]
    blockquote: Option<bool>,
    /// Bold, italic and underline; overrides the corresponding options flag.
    #[prop(optional)]
    style: Option<bool>,
    /// Text color picker; overrides the corresponding options flag.
    #[prop(optional)]
    color: Option<bool>,
    /// Highlight color picker; overrides the corresponding options flag.
    #[prop(optional)]
    highlight: Option<bool>,
    /// Strikethrough, subscript and superscript; overrides the corresponding options flag.
    #[prop(optional)]
    text: Option<bool>,
    /// Named text-style selector; overrides the corresponding options flag.
    #[prop(optional)]
    text_style: Option<bool>,
    /// Remove-format action; overrides the corresponding options flag.
    #[prop(optional)]
    remove: Option<bool>,
    /// Indent and outdent; overrides the corresponding options flag.
    #[prop(optional)]
    indent: Option<bool>,
    /// Left, center and right alignment; overrides the corresponding options flag.
    #[prop(optional)]
    align: Option<bool>,
    /// Horizontal rule insertion; overrides the corresponding options flag.
    #[prop(optional)]
    rule: Option<bool>,
    /// Ordered and unordered lists; overrides the corresponding options flag.
    #[prop(optional)]
    list: Option<bool>,
    /// Line-height selector; overrides the corresponding options flag.
    #[prop(optional)]
    lineheight: Option<bool>,
    /// Prompt-based table insertion; overrides the corresponding options flag.
    #[prop(optional)]
    table: Option<bool>,
    /// Prompt-based HTTP(S) link insertion; overrides the corresponding options flag.
    #[prop(optional)]
    link: Option<bool>,
    /// Local image-file insertion; overrides the corresponding options flag.
    #[prop(optional)]
    image: Option<bool>,
    /// Local multiple-image insertion; overrides the corresponding options flag.
    #[prop(optional)]
    image_gallery: Option<bool>,
    /// Prompt-based video iframe insertion; overrides the corresponding options flag.
    #[prop(optional)]
    video: Option<bool>,
    /// Prompt-based audio insertion; overrides the corresponding options flag.
    #[prop(optional)]
    audio: Option<bool>,
    /// Prompt-based math placeholder insertion; overrides the corresponding options flag.
    #[prop(optional)]
    math: Option<bool>,
    /// Fullscreen action; overrides the corresponding options flag.
    #[prop(optional)]
    fullscreen: Option<bool>,
    /// Block-boundary visibility action; overrides the corresponding options flag.
    #[prop(optional)]
    showblocks: Option<bool>,
    /// HTML source toggle; overrides the corresponding options flag.
    #[prop(optional)]
    code: Option<bool>,
    /// Preview in a new window; overrides the corresponding options flag.
    #[prop(optional)]
    preview: Option<bool>,
    /// Browser print action; overrides the corresponding options flag.
    #[prop(optional)]
    print: Option<bool>,
    /// HTML file download; overrides the corresponding options flag.
    #[prop(optional)]
    save: Option<bool>,
    /// Original two template choices; overrides the corresponding options flag.
    #[prop(optional)]
    template: Option<bool>,
    /// Any nonempty direction shows direction controls; rtl also seeds the RTL class.
    #[prop(optional, into)]
    dir: String,
    /// Override direction-control visibility for source metadata's legacy Boolean dir.
    #[prop(optional)]
    direction_controls: Option<bool>,
    /// Receives exact HTML after source input or a toolbar operation.
    #[prop(optional)]
    on_change: Option<Callback<String>>,
    /// Receives input then command updates, or a single codeViewToggle update.
    #[prop(optional)]
    on_update: Option<Callback<TextEditorUpdate>>,
) -> impl IntoView {
    // Source initial state prefers nonempty defaultValue to value, then its effect
    // reconciles value separately. Static inner_html avoids resetting the active caret.
    let initial = if default_value.is_empty() {
        value.get_untracked()
    } else {
        default_value
    };
    let editor = Editor {
        value: RwSignal::new(initial.clone()),
        code: RwSignal::new(false),
        editable: NodeRef::new(),
        textarea: NodeRef::new(),
        hidden: NodeRef::new(),
        file: NodeRef::new(),
        gallery: NodeRef::new(),
        on_change,
        on_update,
    };
    let options = TextEditorOptions {
        history: history.unwrap_or(options.history),
        font: font.unwrap_or(options.font),
        size: size.unwrap_or(options.size),
        format: format.unwrap_or(options.format),
        paragraph: paragraph.unwrap_or(options.paragraph),
        blockquote: blockquote.unwrap_or(options.blockquote),
        style: style.unwrap_or(options.style),
        color: color.unwrap_or(options.color),
        highlight: highlight.unwrap_or(options.highlight),
        text: text.unwrap_or(options.text),
        text_style: text_style.unwrap_or(options.text_style),
        remove: remove.unwrap_or(options.remove),
        indent: indent.unwrap_or(options.indent),
        align: align.unwrap_or(options.align),
        rule: rule.unwrap_or(options.rule),
        list: list.unwrap_or(options.list),
        lineheight: lineheight.unwrap_or(options.lineheight),
        table: table.unwrap_or(options.table),
        link: link.unwrap_or(options.link),
        image: image.unwrap_or(options.image),
        image_gallery: image_gallery.unwrap_or(options.image_gallery),
        video: video.unwrap_or(options.video),
        audio: audio.unwrap_or(options.audio),
        math: math.unwrap_or(options.math),
        fullscreen: fullscreen.unwrap_or(options.fullscreen),
        showblocks: showblocks.unwrap_or(options.showblocks),
        code: code.unwrap_or(options.code),
        preview: preview.unwrap_or(options.preview),
        print: print.unwrap_or(options.print),
        save: save.unwrap_or(options.save),
        template: template.unwrap_or(options.template),
    };
    let _ = error;
    // Effects do not run during SSR. Keeping them DOM-only also leaves default-only
    // consumers independent of a renderer mode and browser global.
    #[cfg(target_arch = "wasm32")]
    dom::synchronize(editor, value, initial.clone());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = value;
    let toolbar = toolbar::render(
        editor,
        options,
        direction_controls.unwrap_or(!dir.is_empty()),
    );
    let class = if dir == "rtl" {
        "sp-text-editor sp-text-editor-rtl"
    } else {
        "sp-text-editor"
    };
    view! {
        <div class=class>
            {toolbar}
            <input node_ref=editor.hidden type="hidden" id=id name=name class=class_name disabled=disabled required=required
                value=move || editor.value.get() prop:value=move || editor.value.get() {..attributes}/>
            <div node_ref=editor.editable id=editor_id class="sp-text-editor-editable"
                role="textbox" aria-multiline="true" aria-label="Rich Text Editor" dir=(!dir.is_empty()).then_some(dir)
                contenteditable=move || (!editor.code.get()).to_string()
                style:display=move || if editor.code.get() { "none" } else { "block" }
                inner_html=initial.clone()
                on:input=move |_| editor.input()
                on:click=move |_event| {
                    #[cfg(target_arch = "wasm32")]
                    dom::link_click(editor, &_event);
                }></div>
            <textarea node_ref=editor.textarea class="sp-text-editor-source"
                aria-label="HTML Source" style:display=move || if editor.code.get() { "block" } else { "none" }
                on:input=move |event| {
                    // The source textarea publishes only while code mode owns the active value.
                    if editor.code.get_untracked() {
                        editor.publish(event_target_value(&event), "input");
                    }
                }>{initial}</textarea>
        </div>
    }
}

/// Source-faithful frui/view/HTML: render the supplied value directly as HTML.
///
/// No sanitization, escaping or URL policy is added. Applications must secure
/// stored HTML before rendering it when their own trust model requires that.
/// SSR, hydration and CSR all use this same insertion boundary.
#[component]
pub fn HTML(
    /// Exact stored HTML, including application-owned attributes and markup.
    #[prop(into)]
    value: Signal<String>,
) -> impl IntoView {
    view! { <div inner_html=move || value.get()></div> }
}
