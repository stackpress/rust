//! Generated fieldset composition: child source adapters remain selected before compilation.
use super::props::form_name;
use crate::{Draft, RoleContext};
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::{Input as Presence, PublicDefault, UiField};
use stackpress_leptos::components::source::{SourceFieldset, SourceFieldsetDisplay, SourceRows};

/// Root-local native form prefix for a generated fieldset row; never a global name override.
#[derive(Clone)]
pub(super) struct SourceFormPrefix(pub String);

/// Compose real generated child input functions inside the shared source fieldset factory.
#[component]
pub fn FieldsetField(
    /// Parent field value and source shape.
    context: RoleContext,
    /// Public descriptors of the authored fieldset, passed by actual generated consumers.
    fields: Vec<UiField>,
    /// Original fieldset singular label used by Add and numbered row headings.
    #[prop(optional, into)]
    fieldset_label: String,
    /// Selected generated child field/filter/span function.
    render: Callback<(Draft, String), AnyView>,
) -> impl IntoView {
    let fieldset_label = if fieldset_label.is_empty() {
        match &context.field.value_type.value {
            stackpress_framework_schema::ValueType::Fieldset(name) => name.clone(),
            _ => String::new(),
        }
    } else {
        fieldset_label
    };
    let add = format!("Add {fieldset_label}").trim().to_owned();
    let row_label = StoredValue::new(fieldset_label);
    let multiple = context.field.value_type.multiple;
    let role = context.role;
    let required = context.field.required && role != stackpress_framework_schema::Role::Filter;
    let defaults = Value::Object(
        fields
            .iter()
            .filter_map(|field| {
                if let PublicDefault::Literal(value) = &field.default {
                    Some((field.wire_name.clone(), value.clone()))
                } else {
                    None
                }
            })
            .collect(),
    );
    let initial = match context.value.get_untracked() {
        Presence::Value(Value::Array(values)) => values
            .into_iter()
            .map(|value| (!value.is_null()).then_some(value))
            .collect(),
        Presence::Value(Value::Object(value)) => vec![Some(Value::Object(value))],
        _ if required && !multiple => vec![Some(defaults.clone())],
        _ => Vec::new(),
    };
    let rows = RwSignal::new(SourceRows { rows: initial });
    let fields = StoredValue::new(fields);
    let prefix = StoredValue::new(form_name(&context));
    let id = StoredValue::new(context.id.clone());
    let parent = context.value;
    let previous = StoredValue::new(rows.get_untracked());
    Effect::new(move |_| {
        let next = rows.get();
        if next != previous.get_value() {
            previous.set_value(next.clone());
            let clean = next.clean();
            parent.set(if multiple {
                Presence::Value(Value::Array(clean))
            } else {
                clean
                    .into_iter()
                    .next()
                    .map(Presence::Value)
                    .unwrap_or(Presence::Missing)
            });
        }
    });
    let row_renderer = Callback::new(move |index: usize| {
        let seed = rows
            .with_untracked(|rows| rows.rows.get(index).cloned().flatten())
            .unwrap_or_else(|| serde_json::json!({}));
        let owner = Owner::new();
        let nested=owner.with(||{
            let row_prefix=if multiple{format!("{}[{index}]",prefix.get_value().trim_end_matches("[]"))}else{prefix.get_value()};
            provide_context(SourceFormPrefix(row_prefix));
            let draft=Draft::new(&fields.get_value(),&seed,role).expect("generated fieldset source object");let child=draft.clone();let initial=StoredValue::new(child.snapshot().expect("generated child draft"));
            Effect::new(move|_|{
                // Track every child signal; snapshot deliberately uses untracked reads itself.
                for field in fields.get_value(){if let Some(value)=child.get(&field.wire_name){let _=value.get();}}
                let Ok(snapshot)=child.snapshot()else{return;};
                if snapshot!=initial.get_value(){initial.set_value(snapshot.clone());rows.update(|rows|{let mut value=rows.rows.get(index).cloned().flatten().unwrap_or_else(||serde_json::json!({}));if let (Some(target),Some(changes))=(value.as_object_mut(),snapshot.as_object()){target.extend(changes.clone());}rows.edit(index,value);});}
            });
            let controls=render.run((draft,format!("{}-{index}",id.get_value())));
            view!{<div class="sp-source-nested-row">
                {multiple.then(||view!{<header class="sp-actions"><h6>{format!("{} {}",row_label.get_value(),index+1)}</h6><button type="button" aria-label=format!("Remove row {}",index+1) on:click=move|_|rows.update(|rows|rows.remove(index))>"Remove"</button></header>})}
                {controls}
            </div>}.into_any()
        });
        leptos::tachys::reactive_graph::OwnedView::new_with_owner(nested, owner).into_any()
    });
    view! {<fieldset class="sp-source-nested"><legend>{context.field.label}</legend>
        <SourceFieldset rows render=row_renderer empty_value=defaults limit=if multiple{0}else{1} add/>
    </fieldset>}
}

/// Compose selected generated child view/list cells into the source fieldset table shape.
#[component]
pub fn FieldsetView(
    /// Parent object or object collection.
    context: RoleContext,
    /// Child source descriptors in authored order.
    fields: Vec<UiField>,
    /// Real compiled generated child cell renderer.
    render: Callback<(Value, String), Vec<AnyView>>,
) -> impl IntoView {
    let multiple = context.field.value_type.multiple;
    let role = context.role;
    let selected = fields
        .into_iter()
        .filter(|field| field.roles.iter().any(|binding| binding.role == role))
        .collect::<Vec<_>>();
    let labels = selected.iter().map(|field| field.label.clone()).collect();
    let values = match context.value.get() {
        Presence::Value(Value::Array(values)) if multiple => values,
        Presence::Value(value) if !multiple => vec![value],
        _ => Vec::new(),
    };
    let rows = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let visible = selected
                .iter()
                .map(|field| {
                    field.required
                        || value
                            .get(&field.wire_name)
                            .is_some_and(super::editor::source_truthy)
                })
                .collect::<Vec<_>>();
            render
                .run((value, format!("{}-{index}", context.id)))
                .into_iter()
                .enumerate()
                .map(|(index, cell)| {
                    if visible.get(index) == Some(&true) {
                        cell
                    } else {
                        ().into_any()
                    }
                })
                .collect()
        })
        .collect();
    view! {<SourceFieldsetDisplay labels rows multiple/>}
}

/// Apply a root-local fieldset form prefix after a generated wrapper has looked up its draft key.
/// Keeping the signal lookup before this name adaptation preserves native schema identities.
pub fn fieldset_context(mut context: RoleContext) -> RoleContext {
    if let Some(prefix) = use_context::<SourceFormPrefix>() {
        context.field.wire_name = format!("{}[{}]", prefix.0, context.field.wire_name);
    }
    context
}
