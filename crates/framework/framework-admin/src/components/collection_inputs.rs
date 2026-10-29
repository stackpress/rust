//! Bounded collection editors; wire data has no row IDs or implicit string coercion.
use super::{
    presence,
    sync::{Admission, connect, rejected},
};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input;
use stackpress_leptos::components::{
    BoundedList, CollectionRow, MetadataEntry, MetadataInput, TagList, TextList,
};

/// Admit exact string arrays; Missing/Null have an empty presentation only.
fn strings(value: Input<Value>) -> Admission<Vec<String>> {
    match value {
        Input::Missing | Input::Null => Ok(Vec::new()),
        Input::Value(Value::Array(values)) if values.len() <= 100 => values
            .into_iter()
            .map(|value| match value {
                Value::String(value) if value.len() <= 4096 => Ok(value),
                _ => Err("Expected bounded text rows"),
            })
            .collect(),
        _ => Err("Expected bounded text rows"),
    }
}

/// Assign initial identities once; later user edits/reorders retain the checked row itself.
fn rows<T: stackpress_leptos::components::CollectionValue>(values: Vec<T>) -> BoundedList<T> {
    BoundedList::new(
        values
            .into_iter()
            .enumerate()
            .map(|(index, value)| CollectionRow {
                id: index as u64 + 1,
                value,
            })
            .collect(),
        100,
    )
    .expect("adapter admission checks the complete bounded collection")
}

/// Exact bounded text/tag list. Empty strings, duplicates and explicit empty arrays survive.
#[component]
pub fn TextListField(
    /// Public collection metadata and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let Ok(initial) = strings(context.value.get_untracked()) else {
        return rejected();
    };
    let list = RwSignal::new(rows(initial));
    let error = connect(
        context.value,
        move || {
            list.with(|list| {
                list.rows()
                    .iter()
                    .map(|row| row.value.clone())
                    .collect::<Vec<_>>()
            })
        },
        move |values| list.set(rows(values)),
        strings,
        |values| {
            Ok(Input::Value(Value::Array(
                values.into_iter().map(Value::String).collect(),
            )))
        },
    );
    let state = presence(&context);
    let id = context.id.clone();
    let label = context.text_prop("label", &context.field.label);
    let disabled = context.bool_prop("disabled", false);
    let control = if context.bool_prop("tags", false) {
        view! { <TagList id label list disabled error/> }.into_any()
    } else {
        view! { <TextList id label list disabled error/> }.into_any()
    };
    view! { <div class="sp-role">{control}{state}</div> }.into_any()
}

/// Metadata is a string-valued JSON object, never a lossy conversion of arbitrary JSON.
fn metadata(value: Input<Value>) -> Admission<Vec<MetadataEntry>> {
    match value {
        Input::Missing | Input::Null => Ok(Vec::new()),
        Input::Value(Value::Object(values)) if values.len() <= 100 => values
            .into_iter()
            .map(|(key, value)| match value {
                Value::String(value) if key.len() <= 1024 && value.len() <= 4096 => {
                    Ok(MetadataEntry { key, value })
                }
                _ => Err("Expected bounded string metadata"),
            })
            .collect(),
        _ => Err("Expected bounded string metadata"),
    }
}

/// Refuse duplicate editable keys before constructing an object, preserving the last draft.
fn object(values: Vec<MetadataEntry>) -> Admission<Input<Value>> {
    let mut object = serde_json::Map::new();
    for entry in values {
        if object
            .insert(entry.key, Value::String(entry.value))
            .is_some()
        {
            return Err("Metadata keys must be unique");
        }
    }
    Ok(Input::Value(Value::Object(object)))
}

/// Key/value editor over an exact string-valued object with explicit presence controls.
#[component]
pub fn MetadataField(
    /// Public metadata and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let Ok(initial) = metadata(context.value.get_untracked()) else {
        return rejected();
    };
    let list = RwSignal::new(rows(initial));
    let error = connect(
        context.value,
        move || {
            list.with(|list| {
                list.rows()
                    .iter()
                    .map(|row| row.value.clone())
                    .collect::<Vec<_>>()
            })
        },
        move |values| list.set(rows(values)),
        metadata,
        object,
    );
    let state = presence(&context);
    view! { <div class="sp-role"><MetadataInput id=context.id.clone() label=context.text_prop("label",&context.field.label)
        list disabled=context.bool_prop("disabled",false) error/>{state}</div> }.into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Wire shape admission is exact; invalid/duplicate metadata never discards information.
    #[test]
    fn exact_collection_shapes_and_budgets() {
        assert_eq!(
            strings(Input::Value(json!(["", "0", "", "🦀"]))).unwrap(),
            ["", "0", "", "🦀"]
        );
        for value in [
            json!([false]),
            json!([0]),
            json!("x"),
            json!(vec![""; 101]),
            json!(["x".repeat(4097)]),
        ] {
            assert!(strings(Input::Value(value)).is_err());
        }
        assert!(strings(Input::Missing).unwrap().is_empty());
        assert!(metadata(Input::Null).unwrap().is_empty());
        assert!(metadata(Input::Value(json!({"x":false}))).is_err());
        assert!(metadata(Input::Value(json!({"x".repeat(1025):""}))).is_err());
        let values = metadata(Input::Value(json!({"":"", "zero":"0"}))).unwrap();
        assert_eq!(
            object(values.clone()).unwrap(),
            Input::Value(json!({"":"", "zero":"0"}))
        );
        assert!(object(vec![values[0].clone(), values[0].clone()]).is_err());
    }
}
