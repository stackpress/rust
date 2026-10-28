//! Small private Lucide set; exact pinned masters and licences live in third-party/lucide.
use leptos::prelude::*;

/// Closed geometry inventory for collection actions, independent of button semantics.
#[derive(Clone, Copy)]
pub(super) enum IconKind {
    /// Upward arrow: move an existing row earlier.
    ArrowUp,
    /// Downward arrow: move an existing row later.
    ArrowDown,
    /// Trash can: remove an editable row.
    Trash2,
    /// Crossing strokes: remove a tag chip.
    X,
    /// Addition strokes: append the explicit tag draft.
    Plus,
}

/// Render decorative inline geometry; the consuming control owns its accessible name.
pub(super) fn icon(kind: IconKind) -> AnyView {
    // Keep primitive attributes identical to the pinned Lucide masters. No raw HTML,
    // dynamic paths, resource references or external icon font enters this renderer.
    let geometry = match kind {
        IconKind::ArrowUp => view! {
            <path d="m5 12 7-7 7 7"/><path d="M12 19V5"/>
        }
        .into_any(),
        IconKind::ArrowDown => view! {
            <path d="M12 5v14"/><path d="m19 12-7 7-7-7"/>
        }
        .into_any(),
        IconKind::Trash2 => view! {
            <path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/>
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/>
            <line x1="10" x2="10" y1="11" y2="17"/>
            <line x1="14" x2="14" y1="11" y2="17"/>
        }
        .into_any(),
        IconKind::X => view! { <path d="M18 6 6 18"/><path d="m6 6 12 12"/> }.into_any(),
        IconKind::Plus => view! { <path d="M5 12h14"/><path d="M12 5v14"/> }.into_any(),
    };
    // Erase the SVG's attribute tuple at this private boundary so nested generated
    // field adapters do not multiply its type depth during async hydration.
    view! {
        <svg class="sp-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16"
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
            {geometry}
        </svg>
    }
    .into_any()
}
