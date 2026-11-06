//! Source Radio visual flags over the existing shared native radio control.
use super::SourceAppearance;
use crate::components::{
    Radio,
    icons::{IconKind, icon},
};
use leptos::prelude::*;

/// Source radio marker/style choices; the existing native Radio owns group and keyboard semantics.
#[component]
pub fn SourceRadio(
    /// Native identity.
    #[prop(into)]
    id: String,
    /// Native group/form key.
    #[prop(into)]
    name: String,
    /// Original submitted value, including an empty string.
    #[prop(into)]
    value: String,
    /// Existing shared group selection.
    selected: RwSignal<Option<String>>,
    /// Optional authored visible label.
    #[prop(optional, into)]
    label: String,
    /// Accessible control name when the source omits a visible label.
    #[prop(optional, into)]
    aria_label: String,
    /// Check takes precedence over square/circle, matching Frui.
    #[prop(optional)]
    check: bool,
    /// Square marker; otherwise the source default marker is a circle.
    #[prop(optional)]
    square: bool,
    /// Round the outer marker border.
    #[prop(optional)]
    rounded: bool,
    /// Source blue accent has precedence over orange.
    #[prop(optional)]
    blue: bool,
    /// Source orange accent.
    #[prop(optional)]
    orange: bool,
    /// Native requirement.
    #[prop(optional)]
    required: bool,
    /// Original callback value and checked flag.
    #[prop(optional)]
    on_update: Option<Callback<(String, bool)>>,
    /// Root label appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let mark = if check {
        "check"
    } else if square {
        "square"
    } else {
        "circle"
    };
    let classes = format!(
        "sp-source-radio sp-source-radio-{mark} {} {} {}",
        if rounded {
            "sp-source-radio-rounded"
        } else {
            ""
        },
        if blue {
            "sp-source-radio-blue"
        } else if orange {
            "sp-source-radio-orange"
        } else {
            ""
        },
        appearance.class_name
    );
    view! {<label class=classes style=appearance.style>
        <Radio id name value selected required attr:aria-label=aria_label
            on:change=move|event|{if let Some(callback)=on_update{callback.run((event_target_value(&event),event_target_checked(&event)));}}/>
        <span class="sp-source-radio-box" aria-hidden="true">{if check{icon(IconKind::Check)}else{view!{<span class="sp-source-radio-dot"></span>}.into_any()}}</span>
        <span>{label}</span>
    </label>}
}
