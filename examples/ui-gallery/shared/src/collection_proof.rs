//! Explicit proof-only collection fixture; never rendered by the owner-facing Gallery.
use leptos::prelude::*;
use stackpress_leptos::components::{
    BoundedList, CollectionFieldset, CollectionRow, ControlError, FieldValue, MetadataEntry,
    MetadataInput, RatingInput, RatingState, TagList, TextList,
};

/// Render adversarial collection states behind the dedicated finite proof entry point.
#[component]
pub fn CollectionProof(
    /// Preserve an early native presence edit in the proof-only collection.
    #[prop(default = FieldValue::Value(false))]
    initial_nested: FieldValue<bool>,
) -> impl IntoView {
    let tags = RwSignal::new(
        BoundedList::new(
            vec![
                CollectionRow {
                    id: 10,
                    value: String::new(),
                },
                CollectionRow {
                    id: 20,
                    value: "0".into(),
                },
                CollectionRow {
                    id: 30,
                    value: "雪".into(),
                },
            ],
            4,
        )
        .unwrap(),
    );
    let text = RwSignal::new(
        BoundedList::new(
            vec![
                CollectionRow {
                    id: 1,
                    value: String::new(),
                },
                CollectionRow {
                    id: 2,
                    value: "0".into(),
                },
            ],
            3,
        )
        .unwrap(),
    );
    let metadata = RwSignal::new(
        BoundedList::new(
            vec![CollectionRow {
                id: 4,
                value: MetadataEntry {
                    key: "count".into(),
                    value: "0".into(),
                },
            }],
            3,
        )
        .unwrap(),
    );
    let disabled = RwSignal::new(false);
    let rating = RatingState::new(5, None).expect("fixed rating bounds");
    let error = RwSignal::new(String::new());
    let submissions = RwSignal::new(0u32);
    // Count publications, including equal values: blank/rejected drafts must not
    // wake a generated adapter and promote Missing/Null into an empty array.
    let changes = RwSignal::new(0u32);
    Effect::watch(
        move || tags.get(),
        move |_, _, _| changes.update(|value| *value += 1),
        false,
    );
    view! {
        <main class="sp-root" data-theme="light">
            <h1>"Collection proof fixture"</h1>
            <button id="proof-disabled" type="button" on:click=move |_|disabled.update(|value| *value = !*value)>"Toggle disabled"</button>
            <button id="proof-error" type="button" on:click=move |_|error.set("Choose a permitted tag".into())>"Show field error"</button>
            <form id="collection-proof-form" on:submit=move |event|{event.prevent_default();submissions.update(|value|*value+=1);}>
                <TextList id="proof-text" label="Text rows" list=text disabled/>
                <MetadataInput id="proof-metadata" label="Metadata rows" list=metadata disabled/>
                <TagList id="proof-tags" label="Tags" list=tags disabled error/>
                <RatingInput id="proof-rating" name="rating-min" label="Rating minimum two" state=rating min=2 disabled error/>
                <button type="submit">"Submit proof"</button>
            </form>
            <NestedProof initial=initial_nested/>
            <output id="collection-proof-state">{move ||serde_json::json!({
                "tags":tags.get().rows().iter().map(|row|(&row.id,&row.value)).collect::<Vec<_>>(),
                "text":text.get().rows().iter().map(|row|(&row.id,&row.value)).collect::<Vec<_>>(),
                "metadata":metadata.get().rows().iter().map(|row|(&row.id,&row.value.key,&row.value.value)).collect::<Vec<_>>(),
                "rating":rating.value(),"submissions":submissions.get(),"disabled":disabled.get(),"changes":changes.get()
            }).to_string()}</output>
        </main>
    }
}

/// Raw presence tokens remain a test fixture until a nullable-boolean control exists.
#[component]
fn NestedProof(initial: FieldValue<bool>) -> impl IntoView {
    // Keep the original stable row identity and typed presence; no Boolean coercion.
    let nested = RwSignal::new(
        BoundedList::new(
            vec![CollectionRow {
                id: 8,
                value: initial,
            }],
            3,
        )
        .unwrap(),
    );
    let nested_render = Callback::new(move |key: u64| {
        let id = format!("nested-{key}-value");
        // Missing-row fallback is presentation only; it never inserts a value.
        let read = move || {
            nested
                .get()
                .rows()
                .iter()
                .find(|row| row.id == key)
                .map(|row| match row.value {
                    FieldValue::Missing => "missing",
                    FieldValue::Null => "null",
                    FieldValue::Value(false) => "false",
                    FieldValue::Value(true) => "true",
                })
                .unwrap_or("missing")
        };
        view!{<label for=id.clone()>"Typed presence"</label><select id=id.clone() name=format!("nested[{key}]") on:change=move |event|{
            // Reject foreign native tokens before acquiring the collection write guard.
            let Ok(next)=parse_presence(&event_target_value(&event)) else{return;};
            nested.update(|list|{let _=list.edit(key,next);});
        }>
            {["missing","null","false","true"].into_iter().map(|value|view!{<option value=value selected=move ||read()==value prop:selected=move ||read()==value>{value}</option>}).collect_view()}
        </select><button type="button" on:click=move |_|nested.update(|list|{let _=list.edit(key,FieldValue::Null);})>"Set null"</button>}.into_any()
    });
    // This factory deliberately reads its own collection: callers never run under its write guard.
    let nested_create = Callback::new(move |()| {
        let _current = nested.get();
        FieldValue::Missing
    });
    view! {
        <form id="nested-proof-form" on:submit=move |event| event.prevent_default()>
            <CollectionFieldset id="nested-rows" label="Typed nested rows (proof only)" list=nested create=nested_create render=nested_render/>
        </form>
        <output id="nested-proof-state">{move || serde_json::json!(nested.get().rows().iter().map(|row| (&row.id, match row.value {
            FieldValue::Missing => "missing", FieldValue::Null => "null", FieldValue::Value(false) => "false", FieldValue::Value(true) => "true"
        })).collect::<Vec<_>>()).to_string()}</output>
    }
}

/// Decode the example's closed presence tokens in shared native-tested code.
/// # Errors
/// Rejects unknown/case/empty tokens; no missing/null/false coercion is performed.
pub fn parse_presence(value: &str) -> Result<FieldValue<bool>, ControlError> {
    match value {
        "missing" => Ok(FieldValue::Missing),
        "null" => Ok(FieldValue::Null),
        "false" => Ok(FieldValue::Value(false)),
        "true" => Ok(FieldValue::Value(true)),
        _ => Err(ControlError),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Presence tokens preserve omitted/null/false/true and reject malformed native values.
    #[test]
    fn presence_codec_boundaries() {
        assert_eq!(parse_presence("missing").unwrap(), FieldValue::Missing);
        assert_eq!(parse_presence("null").unwrap(), FieldValue::Null);
        assert_eq!(parse_presence("false").unwrap(), FieldValue::Value(false));
        assert_eq!(parse_presence("true").unwrap(), FieldValue::Value(true));
        for invalid in ["", "False", "0", "undefined", " null "] {
            assert!(parse_presence(invalid).is_err());
        }
    }
}
