//! Source Selection/Range transformations; HTML and selected nodes remain browser-owned.
use super::{Editor, document};
use leptos::prelude::*;

/// Borrow the current browser range without introducing a saved-selection policy.
fn selection() -> Option<(web_sys::Selection, web_sys::Range)> {
    let selection = document().get_selection().ok().flatten()?;
    let range = selection.get_range_at(0).ok()?;
    Some((selection, range))
}

/// Unwrap prior source text-style spans, retaining their children in order.
fn unwrap_styles(fragment: &web_sys::DocumentFragment) {
    let Ok(spans) = fragment.query_selector_all("span[class*=\"__frui-form-text-editor-t-\"]")
    else {
        return;
    };
    for index in 0..spans.length() {
        let Some(span) = spans.item(index) else {
            continue;
        };
        let Some(parent) = span.parent_node() else {
            continue;
        };
        while let Some(child) = span.first_child() {
            // Move, rather than clone, so nested text keeps its original order.
            let _ = parent.insert_before(&child, Some(&span));
        }
        let _ = parent.remove_child(&span);
    }
}

/// Source text styles wrap the selected fragment and select the resulting span.
pub(super) fn text_style(editor: Editor, style: &str) {
    if let Some((selection, range)) = selection()
        && let Ok(fragment) = range.extract_contents()
        && let Ok(span) = document().create_element("span")
    {
        // Only prior Frui text-style classes are unwrapped; other authored markup stays.
        unwrap_styles(&fragment);
        span.set_class_name(&format!("__frui-form-text-editor-t-{style}"));
        let _ = span.append_child(&fragment);
        let _ = range.insert_node(&span);
        let _ = selection.remove_all_ranges();
        // Selecting the new span enables the next toolbar transform on that same text.
        if let Ok(new_range) = document().create_range() {
            let _ = new_range.select_node_contents(&span);
            let _ = selection.add_range(&new_range);
        }
    }
    // Even an absent browser range emits the source input callback.
    editor.input();
}

/// Remove the source's precise inline elements and replace the selection with a paragraph.
pub(super) fn remove_format(editor: Editor) {
    if let Some((selection, range)) = selection()
        && let Ok(fragment) = range.clone_contents()
        && let Ok(container) = document().create_element("div")
    {
        let _ = container.append_child(&fragment);
        if let Ok(elements) = container.query_selector_all(
            "b, i, u, strike, sub, sup, font, span[style], span[class*=\"__frui-form-text-editor-t-\"]",
        ) {
            for index in 0..elements.length() {
                let Some(element) = elements.item(index) else { continue };
                if let Some(parent) = element.parent_node() {
                    // Frui deliberately flattens each matched element to textContent.
                    let text = document().create_text_node(&element.text_content().unwrap_or_default());
                    let _ = parent.replace_child(&text, &element);
                }
            }
        }
        container.normalize();
        // A paragraph (including a single-space empty fallback) is the source replacement.
        if let Ok(paragraph) = document().create_element("p") {
            let html = container.inner_html();
            paragraph.set_inner_html(if html.is_empty() { " " } else { &html });
            let _ = range.delete_contents();
            let _ = range.insert_node(&paragraph);
            let _ = selection.remove_all_ranges();
            let _ = selection.add_range(&range);
        }
    }
    editor.input();
    if let Some(node) = editor.editable.get() {
        editor.updated(node.inner_html(), "removeFormat");
    }
}
