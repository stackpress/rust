//! Shared toolbar vocabulary and native controls; browser effects stay in dom.rs.
use super::{Editor, TextEditorOptions};
use leptos::prelude::*;
mod icons;

/// Finite source action inventory, shared by button/select rendering and DOM dispatch.
#[derive(Clone, Copy)]
pub(super) enum Action {
    /// Dispatch this original document.execCommand name with the control's value.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    Command(&'static str),
    /// Apply one named span class.
    TextStyle,
    /// Flatten formatting in the selected range.
    RemoveFormat,
    /// Set editable surface line height.
    LineHeight,
    /// Prompt for rows and columns.
    Table,
    /// Prompt for an HTTP(S) link.
    Link,
    /// Open the local image picker.
    Image,
    /// Open the local multiple-image picker.
    Gallery,
    /// Insert a prompted video iframe.
    Video,
    /// Insert a prompted audio element.
    Audio,
    /// Insert a prompted math placeholder.
    Math,
    /// Toggle native fullscreen.
    Fullscreen,
    /// Toggle block guides.
    ShowBlocks,
    /// Switch active editing surface.
    Code,
    /// Open the current visual HTML.
    Preview,
    /// Print the page.
    Print,
    /// Download visual HTML.
    Save,
    /// Toggle the editable direction.
    DirToggle,
    /// Set left-to-right.
    Ltr,
    /// Set right-to-left.
    Rtl,
}
impl Action {
    /// Frui keeps fullscreen, source, preview, print and save available in source mode.
    pub(super) fn edits(self) -> bool {
        !matches!(
            self,
            Self::Fullscreen | Self::Code | Self::Preview | Self::Print | Self::Save
        )
    }
}

/// One icon-only action with native keyboard activation and a control-owned name.
fn button(
    editor: Editor,
    label: &'static str,
    icon: &'static str,
    action: Action,
    value: &'static str,
) -> AnyView {
    view! {
        <button type="button" title=label aria-label=label
            disabled=move || action.edits() && editor.code.get()
            on:click=move |_| editor.act(action, value)>
            {icons::icon(icon)}
        </button>
    }
    .into_any()
}

/// Group only related actions, preserving frui's toolbar order.
fn group(buttons: Vec<AnyView>) -> AnyView {
    view! { <div class="sp-text-editor-group">{buttons}</div> }.into_any()
}

/// One source selector. Font, size, text style and template reset after each action.
fn select(
    editor: Editor,
    label: &'static str,
    placeholder: &'static str,
    action: Action,
    choices: Vec<(&'static str, &'static str)>,
    reset: bool,
) -> AnyView {
    let line_height = matches!(action, Action::LineHeight);
    let choices = choices
        .into_iter()
        .map(|(value, text)| {
            view! { <option value=value selected=line_height && value == "1">{text}</option> }
        })
        .collect_view();
    view! {
        <select title=label aria-label=label disabled=move || editor.code.get()
            on:change=move |event| {
                editor.act(action, &event_target_value(&event));
                #[cfg(target_arch = "wasm32")]
                if reset { event_target::<web_sys::HtmlSelectElement>(&event).set_value(""); }
                #[cfg(not(target_arch = "wasm32"))]
                let _ = reset;
            }>
            {(!line_height).then(|| view! { <option value="" disabled selected>{placeholder}</option> })}
            {choices}
        </select>
    }.into_any()
}

/// The two color controls pass the browser's exact native color value.
fn color(editor: Editor, label: &'static str, command: &'static str) -> AnyView {
    view! {
        <input type="color" title=label aria-label=label disabled=move || editor.code.get()
            on:change=move |event| editor.act(Action::Command(command), &event_target_value(&event))/>
    }.into_any()
}

/// Render the local picker only when its toolbar feature is selected.
fn picker(editor: Editor, gallery: bool) -> AnyView {
    let reference = if gallery { editor.gallery } else { editor.file };
    view! {
        <input type="file" node_ref=reference hidden accept="image/*" multiple=gallery
            aria-hidden="true" tabindex="-1" on:change=move |_event| {
                #[cfg(target_arch = "wasm32")]
                super::dom::upload(editor, &_event, gallery);
            }/>
    }
    .into_any()
}

/// Toolbar projection contains no global state, upload endpoint or app-owned styling.
pub(super) fn render(editor: Editor, options: TextEditorOptions, direction: bool) -> AnyView {
    use self::Action::*;
    let mut groups = Vec::new();
    if options.history {
        groups.push(group(vec![
            button(editor, "Undo", "undo-2", Command("undo"), ""),
            button(editor, "Redo", "redo-2", Command("redo"), ""),
        ]));
    }
    if options.font {
        groups.push(select(
            editor,
            "Font",
            "Font",
            Command("fontName"),
            [
                "Arial",
                "Comic Sans MS",
                "Courier New",
                "Impact",
                "Georgia",
                "Tahoma",
                "Trebuchet MS",
                "Verdana",
            ]
            .into_iter()
            .map(|v| (v, v))
            .collect(),
            true,
        ));
    }
    if options.size {
        // Keep frui's px strings rather than replacing them with execCommand's numeric scale.
        groups.push(select(
            editor,
            "Font Size",
            "Size",
            Command("fontSize"),
            [
                "8px", "9px", "10px", "11px", "12px", "14px", "16px", "18px", "20px", "22px",
                "24px", "26px", "28px", "36px", "48px", "72px",
            ]
            .into_iter()
            .map(|v| (v, v))
            .collect(),
            true,
        ));
    }
    if options.format {
        groups.push(select(
            editor,
            "Block Format",
            "Format",
            Command("formatBlock"),
            vec![
                ("h1", "Heading 1"),
                ("h2", "Heading 2"),
                ("h3", "Heading 3"),
                ("h4", "Heading 4"),
                ("h5", "Heading 5"),
                ("h6", "Heading 6"),
                ("p", "Paragraph"),
                ("pre", "Code"),
            ],
            false,
        ));
    }
    if options.paragraph {
        groups.push(group(vec![button(
            editor,
            "Paragraph",
            "pilcrow",
            Command("formatBlock"),
            "p",
        )]));
    }
    if options.blockquote {
        groups.push(group(vec![button(
            editor,
            "Blockquote",
            "quote",
            Command("formatBlock"),
            "blockquote",
        )]));
    }
    if options.style {
        groups.push(group(vec![
            button(editor, "Bold", "bold", Command("bold"), ""),
            button(editor, "Italic", "italic", Command("italic"), ""),
            button(editor, "Underline", "underline", Command("underline"), ""),
        ]));
    }
    if options.color {
        groups.push(color(editor, "Text Color", "foreColor"));
    }
    if options.highlight {
        groups.push(color(editor, "Highlight", "hiliteColor"));
    }
    if options.text {
        groups.push(group(vec![
            button(
                editor,
                "Strikethrough",
                "strikethrough",
                Command("strikeThrough"),
                "",
            ),
            button(editor, "Subscript", "subscript", Command("subscript"), ""),
            button(
                editor,
                "Superscript",
                "superscript",
                Command("superscript"),
                "",
            ),
        ]));
    }
    if options.text_style {
        groups.push(select(
            editor,
            "Text Style",
            "T Style",
            TextStyle,
            vec![
                ("code", "Code"),
                ("translucent", "Translucent"),
                ("shadow", "Shadow"),
                ("spaced", "Spaced"),
                ("bordered", "Bordered"),
                ("neon", "NEON"),
            ],
            true,
        ));
    }
    if options.remove {
        groups.push(group(vec![button(
            editor,
            "Remove Format",
            "eraser",
            RemoveFormat,
            "",
        )]));
    }
    if options.indent {
        groups.push(group(vec![
            button(
                editor,
                "Indent",
                "list-indent-increase",
                Command("indent"),
                "",
            ),
            button(
                editor,
                "Outdent",
                "list-indent-decrease",
                Command("outdent"),
                "",
            ),
        ]));
    }
    if options.align {
        groups.push(group(vec![
            button(
                editor,
                "Align Left",
                "text-align-start",
                Command("justifyLeft"),
                "",
            ),
            button(
                editor,
                "Align Center",
                "text-align-center",
                Command("justifyCenter"),
                "",
            ),
            button(
                editor,
                "Align Right",
                "text-align-end",
                Command("justifyRight"),
                "",
            ),
        ]));
    }
    if options.rule {
        groups.push(group(vec![button(
            editor,
            "Horizontal Rule",
            "minus",
            Command("insertHorizontalRule"),
            "",
        )]));
    }
    if options.list {
        groups.push(group(vec![
            button(
                editor,
                "Ordered List",
                "list-ordered",
                Command("insertOrderedList"),
                "",
            ),
            button(
                editor,
                "Unordered List",
                "list",
                Command("insertUnorderedList"),
                "",
            ),
        ]));
    }
    if options.lineheight {
        groups.push(select(
            editor,
            "Line Height",
            "",
            LineHeight,
            ["1", "1.15", "1.5", "2"]
                .into_iter()
                .map(|v| (v, v))
                .collect(),
            false,
        ));
    }
    if options.table {
        groups.push(group(vec![button(
            editor,
            "Insert Table",
            "table",
            Table,
            "",
        )]));
    }
    if options.link {
        groups.push(group(vec![button(editor, "Insert Link", "link", Link, "")]));
    }
    if options.image {
        groups.push(group(vec![
            button(editor, "Insert Image", "image", Image, ""),
            picker(editor, false),
        ]));
    }
    if options.image_gallery {
        groups.push(group(vec![
            button(editor, "Insert from Gallery", "images", Gallery, ""),
            picker(editor, true),
        ]));
    }
    if options.video {
        groups.push(group(vec![button(
            editor,
            "Insert Video",
            "video",
            Video,
            "",
        )]));
    }
    if options.audio {
        groups.push(group(vec![button(
            editor,
            "Insert Audio",
            "volume-2",
            Audio,
            "",
        )]));
    }
    if options.math {
        groups.push(group(vec![button(
            editor,
            "Insert Math",
            "radical",
            Math,
            "",
        )]));
    }
    if options.fullscreen {
        groups.push(group(vec![button(
            editor,
            "Toggle Fullscreen",
            "maximize",
            Fullscreen,
            "",
        )]));
    }
    if options.showblocks {
        groups.push(group(vec![button(
            editor,
            "Toggle Block Visibility",
            "grid-2x2",
            ShowBlocks,
            "",
        )]));
    }
    if options.code {
        groups.push(view! {
            <button type="button" title=move || if editor.code.get() { "TextEditor View" } else { "Code View" }
                aria-label=move || if editor.code.get() { "Switch to TextEditor View" } else { "Switch to Code View" }
                on:click=move |_| editor.act(Code, "")>
                {move || icons::icon(if editor.code.get() { "eye" } else { "code" })}
            </button>
        }.into_any());
    }
    if options.preview {
        groups.push(group(vec![button(
            editor,
            "Preview Content",
            "eye",
            Preview,
            "",
        )]));
    }
    if options.print {
        groups.push(group(vec![button(editor, "Print", "printer", Print, "")]));
    }
    if options.save {
        groups.push(group(vec![button(
            editor,
            "Save as HTML",
            "save",
            Save,
            "",
        )]));
    }
    if options.template {
        groups.push(select(
            editor,
            "Insert Template",
            "Template",
            Command("insertHTML"),
            vec![
                ("<p><strong>Header</strong><br>Content</p>", "Simple Block"),
                (
                    "<table border='1'><tr><td>Cell</td></tr></table>",
                    "Table Block",
                ),
            ],
            true,
        ));
    }
    if direction {
        groups.push(group(vec![
            button(
                editor,
                "Toggle Text Direction",
                "arrow-right-left",
                DirToggle,
                "",
            ),
            button(editor, "Left to Right", "arrow-left", Ltr, ""),
            button(editor, "Right to Left", "arrow-right", Rtl, ""),
        ]));
    }
    view! { <div class="sp-text-editor-toolbar" role="group" aria-label="Text formatting">{groups}</div> }.into_any()
}
