//! Ordinary semantic display components; all text is escaped, with no expression evaluation.
use super::display_values::*;
use leptos::prelude::*;

/// Optional neutral styles for ordinary inputs, action groups and presentation.
pub const CORE_STYLES: &str = include_str!("core.css");

/// Explicit wrapping action group with consistent pointer/keyboard separation.
#[component]
pub fn ActionGroup(
    /// Accessible group name; use a named nav outside for navigation landmarks.
    #[prop(into)]
    label: String,
    /// Native buttons/links; no keyboard behavior is imposed on these actions.
    children: Children,
) -> impl IntoView {
    view! { <div class="sp-actions" role="group" aria-label=label>{children()}</div> }
}

/// Compact escaped status/value presentation, never an authorization signal.
#[component]
pub fn Badge(
    /// Explicit text or semantic child views.
    children: Children,
) -> impl IntoView {
    view! { <span class="sp-badge">{children()}</span> }
}

/// Named section with explicit header/body/footer slots, without child inspection.
#[component]
pub fn Card(
    /// Accessible section name, independent of its visual child composition.
    #[prop(into)]
    label: String,
    /// Explicit CardHead/CardBody/CardFoot children.
    children: Children,
) -> impl IntoView {
    view! { <section class="sp-card" aria-label=label>{children()}</section> }
}

/// Header slot for a card title and description.
#[component]
pub fn CardHead(
    /// Explicit semantic content; arbitrary raw HTML is never implied.
    children: Children,
) -> impl IntoView {
    view! { <header class="sp-card-head">{children()}</header> }
}

/// Body slot for meaningful card content.
#[component]
pub fn CardBody(
    /// Explicit semantic content; arbitrary raw HTML is never implied.
    children: Children,
) -> impl IntoView {
    view! { <div class="sp-card-body">{children()}</div> }
}

/// Footer slot for explicit card actions.
#[component]
pub fn CardFoot(
    /// Explicit semantic content; arbitrary raw HTML is never implied.
    children: Children,
) -> impl IntoView {
    view! { <footer class="sp-card-foot">{children()}</footer> }
}

/// Visible card heading; place cards under the page h1.
#[component]
pub fn CardTitle(
    /// Explicit semantic content; arbitrary raw HTML is never implied.
    children: Children,
) -> impl IntoView {
    view! { <h2 class="sp-card-title">{children()}</h2> }
}

/// Escaped card description with ordinary paragraph semantics.
#[component]
pub fn CardDescription(
    /// Explicit semantic content; arbitrary raw HTML is never implied.
    children: Children,
) -> impl IntoView {
    view! { <p class="sp-card-description">{children()}</p> }
}

/// Labelled native progress; indeterminate and zero are different states.
#[component]
pub fn Progress(
    /// Stable native label target.
    #[prop(into)]
    id: String,
    /// Localizable visible label.
    #[prop(into)]
    label: Signal<String>,
    /// Checked root-owned progress state.
    #[prop(into)]
    value: Signal<ProgressValue>,
) -> impl IntoView {
    let target = id.clone();
    view! { <div class="sp-progress"><label for=target>{move || label.get()}</label>
        <progress id=id max=move || value.get().max() value=move || value.get().value()>{move || value.get().value().map(|number| number.to_string()).unwrap_or_default()}</progress>
    </div> }
}

/// Safe text or checked swatch presentation, with a non-color accessible value.
#[component]
pub fn ColorView(
    /// Checked #RGB/#RRGGBB only.
    value: HexColor,
    /// Show the swatch; text remains visible regardless of color perception.
    #[prop(default = true)]
    swatch: bool,
) -> impl IntoView {
    let style = format!("background-color:{}", value.as_str());
    view! { <span class="sp-color">{swatch.then(|| view! { <span class="sp-swatch" aria-hidden="true" style=style></span> })}<span>{value.as_str().to_owned()}</span></span> }
}

/// Dedicated checked mail link; generic navigation keeps its existing policy.
#[component]
pub fn EmailLink(
    /// Checked mailbox text; no headers or display-name parsing.
    value: EmailAddress,
) -> impl IntoView {
    view! { <a href=value.href()>{value.as_str().to_owned()}</a> }
}

/// Dedicated checked telephone link; region formatting belongs to M3.
#[component]
pub fn PhoneLink(
    /// Checked canonical international token.
    value: PhoneNumber,
) -> impl IntoView {
    view! { <a href=value.href()>{value.as_str().to_owned()}</a> }
}

/// A bounded host-managed same-origin raster image; no automatic remote loading.
#[component]
pub fn Image(
    /// Manifest-declared path/bytes/dimensions checked before construction.
    source: ImageSource,
    /// Meaningful description; an explicitly empty string means decorative.
    #[prop(into)]
    alt: String,
) -> impl IntoView {
    view! { <img class="sp-image" src=source.path width=source.width height=source.height alt=alt loading="lazy" decoding="async" referrerpolicy="no-referrer"/> }
}

/// Noninteractive bounded rating with visible numeric text and decorative stars.
#[component]
pub fn RatingView(
    /// Checked exact value and positive bounded maximum.
    value: RatingValue,
    /// Explicit rounding applies to decorative stars only.
    #[prop(default = RatingRound::Nearest)]
    round: RatingRound,
    /// Include unfilled stars up to max.
    #[prop(default = true)]
    remainder: bool,
) -> impl IntoView {
    view! { <span class="sp-rating"><span aria-hidden="true">{value.stars(round,remainder)}</span><span>{format!("{} / {}",value.value,value.max)}</span></span> }
}

/// Typed boolean display, retaining explicit false and caller-supplied phrases.
#[component]
pub fn YesNo(
    /// No truthiness coercion from strings/numbers/JSON occurs.
    #[prop(into)]
    value: Signal<bool>,
    /// Localizable true phrase.
    #[prop(default = "Yes".to_owned())]
    yes: String,
    /// Localizable false phrase.
    #[prop(default = "No".to_owned())]
    no: String,
) -> impl IntoView {
    view! { <span>{move || if value.get() { yes.clone() } else { no.clone() }}</span> }
}

/// Typed text presentation using fixed CSS values; source text remains intact.
#[derive(Clone, Copy, Default)]
pub enum TextCase {
    /// Preserve source presentation.
    #[default]
    None,
    /// CSS uppercase presentation.
    Uppercase,
    /// CSS lowercase presentation.
    Lowercase,
    /// CSS word-capitalization presentation.
    Capitalize,
}
impl TextCase {
    /// No arbitrary style string can enter this boundary.
    fn style(self) -> &'static str {
        match self {
            Self::None => "text-transform:none",
            Self::Uppercase => "text-transform:uppercase",
            Self::Lowercase => "text-transform:lowercase",
            Self::Capitalize => "text-transform:capitalize",
        }
    }
}
/// Fixed text transform with the same source DOM on every renderer.
#[component]
pub fn TextTransform(
    /// Escaped source text, never HTML.
    #[prop(into)]
    value: Signal<String>,
    /// Explicit presentation transform; document language controls native CSS shaping.
    #[prop(default = TextCase::None)]
    format: TextCase,
) -> impl IntoView {
    view! { <span style=format.style()>{move || value.get()}</span> }
}

/// UTF-8-safe shortened text with the full value available as an accessible label.
#[component]
pub fn TextOverflow(
    /// Exact source text; Unicode scalar boundaries never split UTF-8.
    #[prop(into)]
    value: Signal<String>,
    /// Explicit character/word count; zero means no prefix.
    limit: TextLimit,
    /// Append one ellipsis only when content was actually removed.
    #[prop(default = true)]
    ellipsis: bool,
) -> impl IntoView {
    // The short visual text is decorative; assistive readers receive the full
    // value as real text rather than an unsupported label on a generic span.
    view! { <span title=move || value.get()><span aria-hidden="true">{move || overflow_text(&value.get(),limit,ellipsis)}</span><span class="sp-sr-only">{move || value.get()}</span></span> }
}

/// Named semantic definition list; empty/zero text values are retained.
#[component]
pub fn MetadataView(
    /// Unique bounded label/value data in explicit caller order.
    value: MetadataValues,
) -> impl IntoView {
    view! { <dl class="sp-metadata">{value.0.into_iter().map(|(label,value)| view! { <div><dt>{label}</dt><dd>{value}</dd></div> }).collect_view()}</dl> }
}

/// Explicit inline separator or separate display lines; no magic string mode.
#[derive(Clone, Debug)]
pub enum SpreadSeparator {
    /// Escaped inline separator, with no HTML interpretation.
    Inline(String),
    /// One native line per value.
    Lines,
}
/// Present bounded exact values, preserving empty strings and zero text.
#[component]
pub fn Spread(
    /// Static display values; this is not a keyed editable collection.
    value: DisplayValues,
    /// Selected separator behavior.
    separator: SpreadSeparator,
) -> impl IntoView {
    match separator {
        SpreadSeparator::Inline(separator) => view! { <span>{value.entries().join(&separator)}</span> }.into_any(),
        SpreadSeparator::Lines => view! { <div>{value.entries().iter().cloned().map(|value| view! { <div>{value}</div> }).collect_view()}</div> }.into_any(),
    }
}
/// Semantic list of escaped badges; duplicate/empty display values are retained.
#[component]
pub fn Tags(
    /// Bounded read-only data; no add/edit/remove behavior is implied.
    value: DisplayValues,
) -> impl IntoView {
    view! { <ul class="sp-tags">{value.entries().iter().cloned().map(|value| view! { <li><Badge>{value}</Badge></li> }).collect_view()}</ul> }
}

/// Bounded native read-only table with explicit schema, unique identities and empty state.
#[component]
pub fn Tabular(
    /// Checked ordered columns and exact-width row cells.
    value: DisplayTable,
    /// Native caption and overflow-region label.
    #[prop(into)]
    caption: String,
    /// Localizable direction when the admitted data is empty.
    #[prop(default = "No rows".to_owned())]
    empty: String,
) -> impl IntoView {
    let width = value.columns.len();
    let is_empty = value.rows.is_empty();
    view! { <super::Table caption=caption>
        <thead><tr>{value.columns.into_iter().map(|label| view! { <th scope="col">{label}</th> }).collect_view()}</tr></thead>
        <tbody>{is_empty.then(|| view! { <tr><td colspan=width>{empty}</td></tr> })}
        {value.rows.into_iter().map(|row| view! { <tr data-row-id=row.key>{row.cells.into_iter().map(|cell| view! { <td>{cell}</td> }).collect_view()}</tr> }).collect_view()}</tbody>
    </super::Table> }
}
