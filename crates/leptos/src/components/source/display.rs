//! Frui display behavior without the stricter Typed data constructors.
use super::SourceAppearance;
use leptos::prelude::*;

/// JavaScript UTF-16/split-space clipping, including negative slice endpoints.
/// Zero and NaN leave the original text intact; ellipsis is explicitly opt-in.
pub fn source_overflow(text: &str, count: f64, words: bool, hellip: bool) -> String {
    let size = if words {
        text.split(' ').count()
    } else {
        text.encode_utf16().count()
    };
    // The original uses a truthy count and tests the untruncated numeric limit first.
    if count == 0.0 || count.is_nan() || size as f64 <= count {
        return text.into();
    }
    let end = if count < 0.0 {
        size as f64 + count.trunc()
    } else {
        count.trunc()
    }
    .clamp(0.0, size as f64) as usize;
    let mut visible = if words {
        text.split(' ').take(end).collect::<Vec<_>>().join(" ")
    } else {
        String::from_utf16_lossy(&text.encode_utf16().take(end).collect::<Vec<_>>())
    };
    if hellip {
        visible.push('…');
    }
    visible
}

/// Native CSS swatch and/or text, matching frui/view/Color rather than HexColor admission.
#[component]
pub fn SourceColor(
    /// Any CSS color string; native style parsing owns its interpretation.
    #[prop(into)]
    value: String,
    /// Display the swatch (default true).
    #[prop(default = true)]
    box_visible: bool,
    /// Display text when the swatch is shown (default true).
    #[prop(default = true)]
    text: bool,
    /// Swatch size; Frui's sm/md/lg precedence is chosen by the adapter.
    #[prop(default = 12)]
    size: u32,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let swatch = format!("background-color:{value};height:{size}px;width:{size}px");
    // When box is false, Frui always renders text, even when text=false too.
    if box_visible && text {
        view! { <span class=format!("sp-source-color {}",appearance.class_name) style=appearance.style>
            <span aria-hidden="true" class="sp-source-swatch" style=swatch></span><span>{value}</span>
        </span> }.into_any()
    } else if box_visible {
        view! { <span class=format!("sp-source-swatch {}",appearance.class_name) style=format!("{};{swatch}",appearance.style)></span> }.into_any()
    } else {
        view! { <span class=appearance.class_name style=appearance.style>{value}</span> }.into_any()
    }
}

/// Unrestricted mailto presentation matching Frui's EmailLink/Link composition.
#[component]
pub fn SourceEmailLink(
    /// Exact source mailbox text; no additional mailbox or URL admission.
    #[prop(into)]
    value: String,
    /// Native link target.
    #[prop(optional, into)]
    target: String,
    /// Native title.
    #[prop(optional, into)]
    title: String,
    /// Native appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let href = format!("mailto:{value}");
    view! { <a href=href target=target title=title class=appearance.class_name style=appearance.style>{value}</a> }
}

/// Stars only, with the original max-or-rating and rounding semantics.
#[component]
pub fn SourceRating(
    /// Numeric value after JavaScript Number conversion by the source adapter.
    value: f64,
    /// Zero/absent max falls back to the rounded rating; negative max renders none.
    #[prop(optional_no_strip)]
    max: Option<f64>,
    /// Show unfilled stars up to the selected max.
    #[prop(optional)]
    remainder: bool,
    /// round, ceil or floor; other values use JavaScript Math.round behavior.
    #[prop(optional, into)]
    round: String,
) -> impl IntoView {
    let rating = match round.as_str() {
        "ceil" => value.ceil(),
        "floor" => value.floor(),
        _ => (value + 0.5).floor(),
    };
    let limit = max
        .filter(|max| *max != 0.0 && !max.is_nan())
        .unwrap_or(rating);
    // Frui's for loop compares integer indices to a possibly fractional limit.
    // Do not introduce a finite cap or reinterpret Infinity; the original loop is unbounded too.
    (0usize..)
        .take_while(move |index| (*index as f64) < limit)
        .filter_map(|index| {
            if (index as f64) < rating {
                Some("★")
            } else if remainder {
                Some("☆")
            } else {
                None
            }
        })
        .map(|star| view! { <span>{star}</span> })
        .collect_view()
}

/// Escaped unbounded primitive values joined by an exact separator, or native lines.
#[component]
pub fn SourceSpread(
    /// Source primitive/array String conversions; empty values remain present.
    value: Vec<String>,
    /// The special string line creates one div per value; default is one space.
    #[prop(default = " ".into(), into)]
    separator: String,
    /// Native appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    if separator == "line" {
        view! { <div class=appearance.class_name style=appearance.style>{value.into_iter().map(|value|view!{<div>{value}</div>}).collect_view()}</div> }.into_any()
    } else {
        view! { <span class=appearance.class_name style=appearance.style>{value.join(&separator)}</span> }.into_any()
    }
}

/// Ordered/unordered source list without imposing uniqueness or a row budget.
#[component]
pub fn SourceList(
    /// React-renderable primitive children in source order.
    value: Vec<String>,
    /// Select ol rather than ul.
    #[prop(optional)]
    ordered: bool,
    /// Native appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let rows = value
        .into_iter()
        .map(|value| view! {<li>{value}</li>})
        .collect_view();
    if ordered {
        view! {<ol class=appearance.class_name style=appearance.style>{rows}</ol>}.into_any()
    } else {
        view! {<ul class=appearance.class_name style=appearance.style>{rows}</ul>}.into_any()
    }
}

/// Frui tabular: first-row keys form headings; each row retains its own value order/width.
#[component]
pub fn SourceTable(
    /// First-row object keys. Empty source data renders no table.
    columns: Vec<String>,
    /// Per-row ordered primitive children, including heterogeneous widths.
    rows: Vec<Vec<String>>,
    /// Optional three stripe colors: headings, even rows, odd rows.
    #[prop(optional)]
    stripes: Vec<String>,
    /// Appearance applies to each cell, as in Frui.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    if rows.is_empty() {
        return ().into_any();
    }
    let head_style = format!(
        "{};text-align:left;background-color:{}",
        appearance.style,
        stripes.first().map(String::as_str).unwrap_or("")
    );
    let head_class = appearance.class_name.clone();
    view! { <table class="sp-source-table"><thead><tr>{columns.into_iter().map(|key|view!{<th class=head_class.clone() style=head_style.clone()>{key}</th>}).collect_view()}</tr></thead>
        <tbody>{rows.into_iter().enumerate().map(|(index,row)| {
            let style=format!("{};background-color:{}",appearance.style,stripes.get(1+index%2).map(String::as_str).unwrap_or(""));
            row_view(row,appearance.class_name.clone(),style)
        }).collect_view()}</tbody>
    </table> }.into_any()
}

/// Keep cell style and order owned by their source row.
fn row_view(row: Vec<String>, class_name: String, style: String) -> impl IntoView {
    view! { <tr>{row.into_iter().map(|value|view!{<td class=class_name.clone() style=style.clone()>{value}</td>}).collect_view()}</tr> }
}
