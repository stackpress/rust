//! Explicit proof-only collection fixture; never rendered by the owner-facing Gallery.
use leptos::prelude::*;
use stackpress_framework_ui::components::{
    BoundedList, CollectionRow, MetadataEntry, MetadataInput, TagList, TextList,
};

/// Render adversarial collection states behind the dedicated finite proof entry point.
#[component]
pub fn CollectionProof() -> impl IntoView {
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
                <button type="submit">"Submit proof"</button>
            </form>
            <output id="collection-proof-state">{move ||serde_json::json!({
                "tags":tags.get().rows().iter().map(|row|(&row.id,&row.value)).collect::<Vec<_>>(),
                "text":text.get().rows().iter().map(|row|(&row.id,&row.value)).collect::<Vec<_>>(),
                "metadata":metadata.get().rows().iter().map(|row|(&row.id,&row.value.key,&row.value.value)).collect::<Vec<_>>(),
                "submissions":submissions.get(),"disabled":disabled.get(),"changes":changes.get()
            }).to_string()}</output>
        </main>
    }
}
