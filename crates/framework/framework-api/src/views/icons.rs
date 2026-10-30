//! Pinned Lucide 1.62.0 24-unit strokes; labels stay with their consuming controls.
//! Geometry follows the existing monorepo Lucide profile (2-unit, round, currentColor).
use leptos::prelude::*;
/// Decorate the source Sign Up heading with the Lucide user-round geometry.
pub(super) fn user() -> AnyView {
    glyph("user")
}
/// Map source icon concepts to the selected Lucide family; unknown names render no icon.
pub(super) fn named(name: &str) -> Option<AnyView> {
    let name = match name {
        "user" | "user-round" => "user",
        "key" => "key",
        "globe" => "globe",
        "laptop" => "laptop",
        "coffee" => "coffee",
        "shield" | "shield-check" => "shield",
        "search" | "plus" | "file-text" | "square-pen" | "copy" | "trash" | "circle-check"
        | "upload" | "download" | "chevron-right" | "blocks" | "menu" => name,
        _ => return None,
    };
    Some(glyph(name))
}
/// Static editable primitives, with no icon font, dynamic import or raw HTML.
fn glyph(name: &str) -> AnyView {
    let geometry=match name {
        "search" => view!{<path d="m21 21-4.34-4.34" />
  <circle cx="11" cy="11" r="8" />}.into_any(),
        "plus" => view!{<path d="M5 12h14" />
  <path d="M12 5v14" />}.into_any(),
        "file-text" => view!{<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" />
  <path d="M14 2v5a1 1 0 0 0 1 1h5" />
  <path d="M10 9H8" />
  <path d="M16 13H8" />
  <path d="M16 17H8" />}.into_any(),
        "square-pen" => view!{<path d="M12 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" />
  <path d="M18.375 2.625a1 1 0 0 1 3 3l-9.013 9.014a2 2 0 0 1-.853.505l-2.873.84a.5.5 0 0 1-.62-.62l.84-2.873a2 2 0 0 1 .506-.852z" />}.into_any(),
        "copy" => view!{<rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
  <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />}.into_any(),
        "trash" => view!{<path d="M10 11v6" />
  <path d="M14 11v6" />
  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" />
  <path d="M3 6h18" />
  <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />}.into_any(),
        "circle-check" => view!{<circle cx="12" cy="12" r="10" />
  <path d="m16 9-5.5 5.5L8 12" />}.into_any(),
        "upload" => view!{<path d="M12 3v12" />
  <path d="m17 8-5-5-5 5" />
  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />}.into_any(),
        "download" => view!{<path d="M12 15V3" />
  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
  <path d="m7 10 5 5 5-5" />}.into_any(),
        "chevron-right" => view!{<path d="m9 18 6-6-6-6" />}.into_any(),
        "blocks" => view!{<path d="M10 22V7a1 1 0 0 0-1-1H4a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-5a1 1 0 0 0-1-1H2" />
  <rect x="14" y="2" width="8" height="8" rx="1" />}.into_any(),
        "menu" => view!{<path d="M4 5h16" />
  <path d="M4 12h16" />
  <path d="M4 19h16" />}.into_any(),
        "key"=>view!{<path d="m2 21 9.6-9.6" /> <path d="m7.5 15.5 2.3 2.3a1 1 0 0 1 0 1.4l-2.1 2.1a1 1 0 0 1-1.4 0L4 19" /> <circle cx="15.5" cy="7.5" r="5.5" />}.into_any(),
        "globe"=>view!{<circle cx="12" cy="12" r="10" /> <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" /> <path d="M2 12h20" />}.into_any(),
        "laptop"=>view!{<path d="M18 5a2 2 0 0 1 2 2v8.526a2 2 0 0 0 .212.897l1.068 2.127a1 1 0 0 1-.9 1.45H3.62a1 1 0 0 1-.9-1.45l1.068-2.127A2 2 0 0 0 4 15.526V7a2 2 0 0 1 2-2z" /> <path d="M20.054 15.987H3.946" />}.into_any(),
        "coffee"=>view!{<path d="M10 2v2" /> <path d="M14 2v2" /> <path d="M16 8a1 1 0 0 1 1 1v8a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4V9a1 1 0 0 1 1-1h14a4 4 0 1 1 0 8h-1" /> <path d="M6 2v2" />}.into_any(),
        "shield"=>view!{<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" /> <path d="m9 12 2 2 4-4" />}.into_any(),
        _=>view!{<circle cx="12" cy="8" r="5" /> <path d="M20 21a8 8 0 0 0-16 0" />}.into_any(),
    };
    view!{<svg class="sp-icon" xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" focusable="false">{geometry}</svg>}.into_any()
}
