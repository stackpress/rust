//! Source dataset/slot metadata conversion; complete data and behavior live in shared Leptos.
use super::props::{appearance, content, prop, style};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::FieldControl;
use stackpress_leptos::components::source::{
    CountryData, CurrencyData, SourceAppearance, SourceColorInput, SourceCountry,
    SourceCountrySelect, SourceCurrencySelect,
};

/// Original slot strings are class names; CSS objects become native style declarations.
pub(super) fn slot(context: &RoleContext, name: &str) -> SourceAppearance {
    match prop(context, name) {
        Some(Value::String(class_name)) => SourceAppearance {
            class_name: class_name.clone(),
            style: String::new(),
        },
        value => SourceAppearance {
            class_name: String::new(),
            style: style(value),
        },
    }
}

/// Source country/currency input, including searchable string hints and original record callbacks.
#[component]
pub fn CountryField(
    /// Original source selector props and mount-only code.
    context: RoleContext,
) -> impl IntoView {
    let currency = super::props::component(&context).ends_with("currency");
    let initial = match context.value.get_untracked() {
        Presence::Missing | Presence::Null => None,
        _ => Some(content(&context)),
    };
    let draft = context.value;
    let searchable = prop(&context, "searchable").is_some_and(super::editor::source_truthy);
    let search_placeholder = context.text_prop("searchable", "Search...");
    let placeholder = context.text_prop(
        "placeholder",
        if currency {
            "Select a currency"
        } else {
            "Select a country"
        },
    );
    let id = context.id.clone();
    let name = super::props::form_name(&context);
    let display = slot(&context, "display");
    let dropdown = slot(&context, "dropdown");
    let option = slot(&context, "option");
    let appearance = appearance(&context);
    let control = if currency {
        view!{<SourceCurrencySelect id=id.clone() name initial searchable search_placeholder placeholder display dropdown option appearance on_update=Callback::new(move|value:CurrencyData|draft.set(Presence::Value(serde_json::to_value(value).expect("static currency record"))))/>}.into_any()
    } else {
        view!{<SourceCountrySelect id=id.clone() name initial searchable search_placeholder placeholder display dropdown option appearance on_update=Callback::new(move|value:CountryData|draft.set(Presence::Value(serde_json::to_value(value).expect("static country record"))))/>}.into_any()
    };
    view! {<FieldControl for_id=id label=context.field.label>{control}</FieldControl>}
}

/// Source country lookup uses iso3; currency view uses fiat/flagged records only.
#[component]
pub fn CountryView(
    /// Source code and flag/text props; size flags are ignored upstream.
    context: RoleContext,
) -> impl IntoView {
    view! {<SourceCountry value=content(&context) currency=super::props::component(&context).ends_with("currency") flag=context.bool_prop("flag",true) text=context.bool_prop("text",true) appearance=appearance(&context)/ >}
}

/// Source picker/text color editor with named-color conversion and authored slots.
#[component]
pub fn ColorField(
    /// Source text and picker/input appearance slots.
    context: RoleContext,
) -> impl IntoView {
    let value = RwSignal::new(content(&context));
    let draft = context.value;
    view! {<FieldControl for_id=context.id.clone() label=context.field.label.clone()>
        <SourceColorInput id=context.id.clone() name=super::props::form_name(&context) value required=context.input_required() appearance=appearance(&context) input=slot(&context,"input") picker=slot(&context,"picker") on_update=Callback::new(move|value:String|draft.set(Presence::Value(Value::String(value))))/>
    </FieldControl>}
}
