//! Exact vendored Lucide masters; private static markup, never application HTML.
use leptos::prelude::*;

/// Decorative geometry at the same 16px size as the existing inline icon family.
pub(super) fn icon(name: &str) -> AnyView {
    // The closed match prevents dynamic SVG, external resources or arbitrary markup.
    let svg = match name {
        "arrow-left" => include_str!("../../../../third-party/lucide/arrow-left.svg"),
        "arrow-right" => include_str!("../../../../third-party/lucide/arrow-right.svg"),
        "arrow-right-left" => include_str!("../../../../third-party/lucide/arrow-right-left.svg"),
        "bold" => include_str!("../../../../third-party/lucide/bold.svg"),
        "code" => include_str!("../../../../third-party/lucide/code.svg"),
        "eraser" => include_str!("../../../../third-party/lucide/eraser.svg"),
        "eye" => include_str!("../../../../third-party/lucide/eye.svg"),
        "grid-2x2" => include_str!("../../../../third-party/lucide/grid-2x2.svg"),
        "image" => include_str!("../../../../third-party/lucide/image.svg"),
        "images" => include_str!("../../../../third-party/lucide/images.svg"),
        "list-indent-decrease" => {
            include_str!("../../../../third-party/lucide/list-indent-decrease.svg")
        }
        "list-indent-increase" => {
            include_str!("../../../../third-party/lucide/list-indent-increase.svg")
        }
        "italic" => include_str!("../../../../third-party/lucide/italic.svg"),
        "link" => include_str!("../../../../third-party/lucide/link.svg"),
        "list" => include_str!("../../../../third-party/lucide/list.svg"),
        "list-ordered" => include_str!("../../../../third-party/lucide/list-ordered.svg"),
        "maximize" => include_str!("../../../../third-party/lucide/maximize.svg"),
        "minus" => include_str!("../../../../third-party/lucide/minus.svg"),
        "pilcrow" => include_str!("../../../../third-party/lucide/pilcrow.svg"),
        "printer" => include_str!("../../../../third-party/lucide/printer.svg"),
        "quote" => include_str!("../../../../third-party/lucide/quote.svg"),
        "redo-2" => include_str!("../../../../third-party/lucide/redo-2.svg"),
        "save" => include_str!("../../../../third-party/lucide/save.svg"),
        "radical" => include_str!("../../../../third-party/lucide/radical.svg"),
        "strikethrough" => include_str!("../../../../third-party/lucide/strikethrough.svg"),
        "subscript" => include_str!("../../../../third-party/lucide/subscript.svg"),
        "superscript" => include_str!("../../../../third-party/lucide/superscript.svg"),
        "table" => include_str!("../../../../third-party/lucide/table.svg"),
        "text-align-center" => include_str!("../../../../third-party/lucide/text-align-center.svg"),
        "text-align-end" => include_str!("../../../../third-party/lucide/text-align-end.svg"),
        "text-align-start" => include_str!("../../../../third-party/lucide/text-align-start.svg"),
        "underline" => include_str!("../../../../third-party/lucide/underline.svg"),
        "undo-2" => include_str!("../../../../third-party/lucide/undo-2.svg"),
        "video" => include_str!("../../../../third-party/lucide/video.svg"),
        "volume-2" => include_str!("../../../../third-party/lucide/volume-2.svg"),
        _ => unreachable!("closed editor icon inventory"),
    };
    view! { <span class="sp-text-editor-icon" aria-hidden="true" inner_html=svg></span> }.into_any()
}
