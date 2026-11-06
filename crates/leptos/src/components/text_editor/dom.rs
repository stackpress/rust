//! Browser-only frui command bridge. No application policy, networking or editor library.
use super::{Editor, toolbar::Action};
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

mod files;
mod selection;

/// Synchronize only the inactive surface, preserving the active browser selection.
pub(super) fn synchronize(editor: Editor, external: Signal<String>, initial: String) {
    // Frui initializes editable.innerHTML on mount. This also removes Leptos's
    // empty child marker in CSR before a user can copy it into the stored value.
    // Track only the node reference, so subsequent edits retain their active caret.
    Effect::new(move |_| {
        if let Some(node) = editor.editable.get() {
            node.set_inner_html(&initial);
        }
    });
    // The original value effect is independent of currentValue: typing does not
    // force a controlled parent to echo every edit before the browser can keep it.
    Effect::new(move |_| editor.value.set(external.get()));
    Effect::new(move |_| {
        let html = editor.value.get();
        if editor.code.get() {
            // Source edits update the visual HTML, never the active textarea caret.
            if let Some(node) = editor.editable.get() {
                node.set_inner_html(&html);
            }
        } else if let Some(node) = editor.textarea.get() {
            // Visual edits update the source surface, never the active DOM selection.
            node.set_value(&html);
        }
    });
}

/// Browser document is the only execution host; native rendering never calls here.
fn document() -> web_sys::Document {
    leptos::prelude::document()
}

/// Execute an original document command, then publish input followed by its action.
fn command(editor: Editor, name: &str, value: &str) {
    // FileReader may complete after disposal or a switch to code mode. Refuse
    // before reading either node reference from an unavailable reactive owner.
    if editor.code.try_get_untracked().unwrap_or(true) {
        return;
    }
    let Some(node) = editor.editable.get() else {
        return;
    };
    let _ = node.focus();
    let doc: web_sys::HtmlDocument = document().unchecked_into();
    // Browser refusal still publishes current HTML, matching ignored execCommand results.
    let _ = if value.is_empty() {
        doc.exec_command_with_show_ui(name, false)
    } else {
        doc.exec_command_with_show_ui_and_value(name, false, value)
    };
    let _ = node.focus();
    editor.input();
    editor.updated(node.inner_html(), name);
}

/// Code switching copies the outgoing surface before changing mode or callbacks.
fn code(editor: Editor) {
    let (Some(visual), Some(source)) = (editor.editable.get(), editor.textarea.get()) else {
        return;
    };
    let was_code = editor.code.get_untracked();
    let html = if was_code {
        source.value()
    } else {
        visual.inner_html()
    };
    // Copy DOM properties first; mode effects must see the outgoing value intact.
    if was_code {
        visual.set_inner_html(&html);
    } else {
        source.set_value(&html);
    }
    // Frui's hidden write keeps authored source at callback time, but its
    // currentValue/onChange read editor.innerHTML after browser normalization.
    let value = if was_code {
        visual.inner_html()
    } else {
        html.clone()
    };
    editor.code.set(!was_code);
    editor.publish_form(value, Some(&html), "codeViewToggle");
}

/// Keep source prompt text, defaults and cancellation behavior.
fn prompt(message: &str, default: &str) -> Option<String> {
    window()
        .prompt_with_message_and_default(message, default)
        .ok()
        .flatten()
}

/// Frui's own prompt check is case-sensitive and requires the literal HTTP(S) prefix.
fn http_prompt(message: &str, default: &str) -> Option<String> {
    prompt(message, default)
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
}

/// Keep source numeric admission distinct from Array.from's truncated length.
fn table_length(value: &str) -> Option<usize> {
    if value.is_empty()
        || value
            .parse::<js_sys::Number>()
            .is_ok_and(|number| number.value_of().is_nan())
    {
        return None;
    }
    // The source omits parseInt's radix: zero preserves hexadecimal prefixes.
    // Negative/NaN lengths become zero, as Array.from's ToLength conversion does.
    Some(js_sys::parse_int(value, 0).max(0.0) as usize)
}

/// Build the original table shape, preserving both prompts even if one is cancelled.
fn table(editor: Editor) {
    let rows = prompt("Enter number of rows", "2");
    let columns = prompt("Enter number of columns", "2");
    let rows = rows.as_deref().and_then(table_length);
    let columns = columns.as_deref().and_then(table_length);
    if let (Some(rows), Some(columns)) = (rows, columns) {
        let row = format!("<tr>{}</tr>", "<td> </td>".repeat(columns));
        command(
            editor,
            "insertHTML",
            &format!("<table border=\"1\">{}</table>", row.repeat(rows)),
        );
    }
}

/// Direction changes affect the editor attribute and emit input, never stored HTML wrappers.
fn direction(editor: Editor, action: Action) {
    let Some(node) = editor.editable.get() else {
        return;
    };
    let current = node
        .get_attribute("dir")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "ltr".into());
    let next = match action {
        Action::Ltr => "ltr",
        Action::Rtl => "rtl",
        // Frui treats every existing direction other than literal ltr as a toggle to ltr.
        _ if current == "ltr" => "rtl",
        _ => "ltr",
    };
    let _ = node.set_attribute("dir", next);
    editor.input();
}

/// Use the editor's visual HTML, including when source mode is currently active.
fn preview(editor: Editor) {
    if let Some(node) = editor.editable.get()
        && let Ok(Some(preview)) = window().open_with_url_and_target("", "_blank")
        && let Some(doc) = preview.document()
    {
        let doc: web_sys::HtmlDocument = doc.unchecked_into();
        let chunks = js_sys::Array::of1(&JsValue::from_str(&format!(
            "<html><body>{}</body></html>",
            node.inner_html()
        )));
        let _ = doc.write(&chunks);
        let _ = doc.close();
    }
}

/// Browser-only dispatch; formatting actions become inert during source editing.
pub(super) fn act(editor: Editor, action: Action, value: &str) {
    // The same admission rule drives native disabled controls and browser actions.
    if action.edits() && editor.code.get_untracked() {
        return;
    }
    use super::toolbar::Action::*;
    match action {
        Command(name) => command(editor, name, value),
        Code => code(editor),
        TextStyle => selection::text_style(editor, value),
        RemoveFormat => selection::remove_format(editor),
        Table => table(editor),
        Link => {
            // Only the source's own literal scheme check applies to link prompts.
            if let Some(url) = http_prompt("Enter URL", "https://") {
                command(editor, "createLink", &url);
            }
        }
        Video => {
            // Preserve the source iframe attributes and unsanitized URL interpolation.
            if let Some(url) = http_prompt("Enter video URL (e.g., YouTube embed)", "") {
                command(
                    editor,
                    "insertHTML",
                    &format!(
                        "<iframe \n            src=\"{url}\" \n            width=\"560\" \n            height=\"315\" \n            frameborder=\"0\" \n            allowfullscreen\n          >\n          </iframe>"
                    ),
                );
            }
        }
        Audio => {
            // Audio and math are authored HTML inserts, not upload/network operations.
            if let Some(url) = http_prompt("Enter audio URL", "") {
                command(
                    editor,
                    "insertHTML",
                    &format!("<audio controls src=\"{url}\"></audio>"),
                );
            }
        }
        Math => {
            if let Some(expression) =
                prompt("Enter math expression (e.g., x^2)", "").filter(|v| !v.is_empty())
            {
                command(
                    editor,
                    "insertHTML",
                    &format!(
                        "<span class=\"frui-form-text-editor-math\">[Math: {expression}]</span>"
                    ),
                );
            }
        }
        Image | Gallery => {
            // Native file pickers stay separate from the hidden HTML form field.
            let picker = if matches!(action, Gallery) {
                editor.gallery
            } else {
                editor.file
            };
            if let Some(node) = picker.get() {
                node.click();
            }
        }
        Fullscreen => {
            // Like Frui, fullscreen owns the editable div rather than the toolbar.
            if document().fullscreen_element().is_some() {
                document().exit_fullscreen();
            } else if let Some(node) = editor.editable.get() {
                let _ = node.request_fullscreen();
            }
        }
        ShowBlocks => {
            if let Some(node) = editor.editable.get() {
                let _ = node.class_list().toggle("frui-form-text-editor-show-block");
            }
        }
        LineHeight => {
            // This style belongs to the editing surface, outside the stored innerHTML.
            if let Some(node) = editor.editable.get() {
                let _ = web_sys::HtmlElement::style(&node).set_property("line-height", value);
                editor.input();
            }
        }
        DirToggle | Ltr | Rtl => direction(editor, action),
        Preview => preview(editor),
        Print => {
            let _ = window().print();
        }
        Save => files::save(editor),
    }
}

/// A single picker reads only its first file; the gallery starts one reader per image.
pub(super) fn upload(editor: Editor, event: &leptos::ev::Event, gallery: bool) {
    if editor.code.get_untracked() {
        return;
    }
    let input = event_target::<web_sys::HtmlInputElement>(event);
    let Some(files) = input.files() else { return };
    let count = if gallery {
        files.length()
    } else {
        files.length().min(1)
    };
    for index in 0..count {
        if let Some(file) = files.item(index) {
            files::read_image(editor, file);
        }
    }
}

/// Ctrl/Meta-click opens the authored href as in frui, without changing its policy.
pub(super) fn link_click(editor: Editor, event: &leptos::ev::MouseEvent) {
    if editor.code.get_untracked() || !(event.ctrl_key() || event.meta_key()) {
        return;
    }
    if let Some(target) = event
        .target()
        .and_then(|node| node.dyn_into::<web_sys::Element>().ok())
        && let Ok(Some(link)) = target.closest("a[href]")
    {
        event.prevent_default();
        if let Some(href) = link.get_attribute("href").filter(|v| !v.is_empty()) {
            let _ = window().open_with_url_and_target_and_features(
                &href,
                "_blank",
                "noopener,noreferrer",
            );
        }
    }
}
