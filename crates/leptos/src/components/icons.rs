//! Small private Lucide set; exact pinned masters and licences live in third-party/lucide.
use leptos::prelude::*;

/// Closed geometry inventory, independent of the consuming control's semantics.
#[derive(Clone, Copy)]
pub(super) enum IconKind {
    /// Upward arrow: move an existing row earlier.
    ArrowUp,
    /// Downward arrow: move an existing row later.
    ArrowDown,
    /// Opposing arrows: a column without an active sort direction.
    ArrowUpDown,
    /// Trash can: remove an editable row.
    Trash,
    /// Crossing strokes: remove a tag chip.
    X,
    /// Addition strokes: append the explicit tag draft.
    Plus,
    /// Five-point star: a positive rating option.
    Star,
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
        IconKind::ArrowUpDown => view! {
            <path d="m21 16-4 4-4-4"/><path d="M17 20V4"/>
            <path d="m3 8 4-4 4 4"/><path d="M7 4v16"/>
        }.into_any(),
        IconKind::Trash => view! {
            <path d="M10 11v6"/><path d="M14 11v6"/>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
            <path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        }
        .into_any(),
        IconKind::X => view! { <path d="M18 6 6 18"/><path d="m6 6 12 12"/> }.into_any(),
        IconKind::Plus => view! { <path d="M5 12h14"/><path d="M12 5v14"/> }.into_any(),
        IconKind::Star => view! {
            <path d="M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z"/>
        }.into_any(),
    };
    // Stars keep the same viewBox/stroke at their larger, approved rating size.
    let size = if matches!(kind, IconKind::Star) {
        24
    } else {
        16
    };
    // Erase the SVG's attribute tuple at this private boundary so nested generated
    // field adapters do not multiply its type depth during async hydration.
    view! {
        <svg class="sp-icon" xmlns="http://www.w3.org/2000/svg" width=size height=size
            viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
            stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">
            {geometry}
        </svg>
    }
    .into_any()
}
