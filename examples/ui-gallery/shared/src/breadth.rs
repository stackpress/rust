//! Actual M2 component composition, shared by SSR, hydrated web and CSR.
use leptos::prelude::*;
use stackpress_framework_ui::components::*;

/// Native drafts copied before hydration; no persistence/environment lookup is performed.
#[derive(Clone)]
pub struct BreadthDraft {
    /// Native Details can toggle before JavaScript.
    pub details: bool,
    /// Existing keyed collection inputs.
    pub text: [String; 2],
    /// Native slug draft; normalization remains an explicit action.
    pub slug: String,
    /// Search text before the combobox adapter mounts; it triggers no acquisition.
    pub query: String,
    /// Early explicit suggestion text; adoption performs no lookup.
    pub suggestion_query: String,
    /// Existing tag rows retain their stable identities.
    pub tags: [String; 2],
    /// Existing typed metadata row.
    pub metadata: MetadataEntry,
    /// Existing typed nested presence value.
    pub nested: FieldValue<bool>,
    /// Native single and ordered-pair range edits.
    pub ranges: [f64; 3],
    /// Exact selected native radio; None remains unrated.
    pub rating: Option<u8>,
    /// Native filter draft; initial SSR rows remain until an explicit filter edit.
    pub filter: String,
    /// Initially visible row selections.
    pub selected_rows: Vec<String>,
}
impl Default for BreadthDraft {
    /// The exact deterministic SSR/CSR defaults.
    fn default() -> Self {
        Self {
            details: false,
            text: ["One".into(), "0".into()],
            slug: "Hello 雪 0".into(),
            query: String::new(),
            suggestion_query: String::new(),
            tags: ["rust".into(), "雪".into()],
            metadata: MetadataEntry {
                key: "count".into(),
                value: "0".into(),
            },
            nested: FieldValue::Value(false),
            ranges: [0.0, 2.0, 8.0],
            rating: None,
            filter: String::new(),
            selected_rows: Vec::new(),
        }
    }
}
impl BreadthDraft {
    /// Native-tested admission before hydration allocates a root or starts acquisition.
    /// # Errors
    /// Rejects oversized early drafts without discarding the original native DOM.
    pub fn validate(&self) -> Result<(), ControlError> {
        if self.text.iter().any(|value| value.len() > 4096)
            || self.slug.len() > 4096
            || self.query.len() > 1024
            || self.suggestion_query.len() > 1024
            || self.tags.iter().any(|value| value.len() > 4096)
            || !self.metadata.valid_collection_value()
            || self.filter.len() > 1024
            || self.rating.is_some_and(|value| value > 5)
            || self.ranges.iter().any(|value| {
                !value.is_finite() || !(0.0..=10.0).contains(value) || value.fract() != 0.0
            })
            || self.ranges[1] > self.ranges[2]
            || self.selected_rows.len() > 7
            || self.selected_rows.iter().enumerate().any(|(index, key)| {
                !(0..7).any(|row| key == &format!("r{row}"))
                    || self.selected_rows[..index].contains(key)
            })
        {
            Err(ControlError)
        } else {
            Ok(())
        }
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
/// Small checked immutable option dataset with empty/zero/disabled cases.
fn options() -> SelectOptions {
    SelectOptions::new(vec![
        SelectOption {
            value: "".into(),
            label: "Alpha".into(),
            disabled: false,
        },
        SelectOption {
            value: "blocked".into(),
            label: "Blocked".into(),
            disabled: true,
        },
        SelectOption {
            value: "0".into(),
            label: "Gamma".into(),
            disabled: false,
        },
        SelectOption {
            value: "gamut".into(),
            label: "Gamut".into(),
            disabled: false,
        },
    ])
    .expect("fixed bounded options")
}
/// Compose every M2 family with explicit controller ownership and public data only.
#[component]
pub fn BreadthGallery(
    /// Early native drafts or deterministic defaults.
    initial: BreadthDraft,
) -> impl IntoView {
    let accordion = DisclosureState::new(options(), vec!["".into()], true, true).unwrap();
    let tabs = ChoiceState::new(options(), Some("".into())).unwrap();
    let automatic = ChoiceState::new(options(), Some("".into())).unwrap();
    let details = RwSignal::new(initial.details);
    let popover = RwSignal::new(false);
    let child = RwSignal::new(false);
    let drawer = RwSignal::new(false);
    let drawer_child = RwSignal::new(false);
    let nonmodal = RwSignal::new(false);
    let nonmodal_child = RwSignal::new(false);
    let mounted = RwSignal::new(true);
    let selected = RwSignal::new(String::new());
    let menu = ChoiceState::new(options(), None).unwrap();
    let menu_open = RwSignal::new(false);
    let action = Callback::new(move |value| selected.set(value));
    let menubar = ChoiceState::new(
        SelectOptions::new(vec![
            SelectOption {
                value: "file".into(),
                label: "File".into(),
                disabled: false,
            },
            SelectOption {
                value: "edit".into(),
                label: "Edit".into(),
                disabled: false,
            },
        ])
        .unwrap(),
        None,
    )
    .unwrap();
    let entries = vec![
        MenuEntry {
            choices: ChoiceState::new(options(), None).unwrap(),
        },
        MenuEntry {
            choices: ChoiceState::new(options(), None).unwrap(),
        },
    ];
    let combo = ComboboxState::new(options(), None).unwrap();
    combo
        .adopt_query(initial.query)
        .expect("bounded native draft");
    let suggestions = ComboboxState::new(SelectOptions::new(vec![]).unwrap(), None).unwrap();
    suggestions
        .adopt_query(initial.suggestion_query)
        .expect("bounded early query");
    let calls = RwSignal::new(0u32);
    let lookup = Callback::new(move |query: String| {
        calls.update(|value| *value += 1);
        let future = super::suggestion_loader::load(query.clone());
        if let Ok(work) = suggestions.request(query, future) {
            #[cfg(target_arch = "wasm32")]
            leptos::task::spawn_local(async move {
                if let Some(completion) = work.await {
                    suggestions.publish(completion);
                }
            });
            #[cfg(not(target_arch = "wasm32"))]
            drop(work);
        }
    });
    let list = RwSignal::new(
        BoundedList::new(
            initial
                .text
                .into_iter()
                .enumerate()
                .map(|(index, value)| CollectionRow {
                    id: index as u64 + 1,
                    value,
                })
                .collect(),
            3,
        )
        .unwrap(),
    );
    let tags = RwSignal::new(
        BoundedList::new(
            vec![
                CollectionRow {
                    id: 10,
                    value: initial.tags[0].clone(),
                },
                CollectionRow {
                    id: 20,
                    value: initial.tags[1].clone(),
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
                value: initial.metadata,
            }],
            3,
        )
        .unwrap(),
    );
    let nested = RwSignal::new(
        BoundedList::new(
            vec![CollectionRow {
                id: 8,
                value: initial.nested,
            }],
            3,
        )
        .unwrap(),
    );
    let nested_render = Callback::new(move |key: u64| {
        let id = format!("nested-{key}-value");
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
    let table = TableState::new(
        DisplayTable::new(
            vec!["Name".into(), "Value".into()],
            (0..7)
                .map(|i| DisplayRow {
                    key: format!("r{i}"),
                    cells: vec![format!("Person {i}"), i.to_string()],
                })
                .collect(),
        )
        .unwrap(),
        2,
    )
    .unwrap();
    table
        .adopt_filter(initial.filter)
        .expect("bounded early filter draft");
    for key in initial.selected_rows {
        table.select(&key, true);
    }
    let single = RangeState::new(
        RangeScale::new(0.0, 10.0, 1.0).unwrap(),
        RangeValue::Single(initial.ranges[0]),
    )
    .unwrap();
    let range = RangeState::new(
        RangeScale::new(0.0, 10.0, 1.0).unwrap(),
        RangeValue::Pair {
            lower: initial.ranges[1],
            upper: initial.ranges[2],
        },
    )
    .unwrap();
    let rating = RatingState::new(5, initial.rating).unwrap();
    let slug = RwSignal::new(initial.slug);
    let unmount = Callback::new(move |()| {
        popover.set(false);
        child.set(false);
        drawer.set(false);
        drawer_child.set(false);
        nonmodal.set(false);
        nonmodal_child.set(false);
        mounted.set(false);
    });
    let toast = RwSignal::new(false);
    let timer_toast = RwSignal::new(false);
    let feedback = RwSignal::new(String::new());
    view! {<section id="breadth-core" aria-label="Component breadth"><h2>"Disclosure and navigation"</h2>
        <Breadcrumbs label="Breadcrumb" trail=BreadcrumbTrail::new(vec![Crumb{label:"Home".into(),href:Some("/".into())},Crumb{label:"Gallery".into(),href:Some("/gallery".into())}]).unwrap()/>
        <Details id="native-details" label="Native details" open=details><p>"Retained details content"</p></Details>
        <Accordion label="Sections"><AccordionItem id="acc-alpha" item="" state=accordion><p>"Alpha content"</p></AccordionItem><AccordionItem id="acc-gamma" item="0" state=accordion><p>"Gamma content"</p></AccordionItem><AccordionItem id="acc-disabled" item="blocked" state=accordion><p>"Disabled content"</p></AccordionItem></Accordion>
        <Tabs id="manual-tabs" label="Manual tabs" state=tabs/><TabPanel id="manual-tabs" index=0 state=tabs><input id="retained-tab-input" aria-label="Retained tab draft" value="Keep me"/></TabPanel><TabPanel id="manual-tabs" index=1 state=tabs><p>"Disabled panel"</p></TabPanel><TabPanel id="manual-tabs" index=2 state=tabs><p>"Gamma panel"</p></TabPanel><TabPanel id="manual-tabs" index=3 state=tabs><p>"Gamut panel"</p></TabPanel>
        <Tabs id="auto-tabs" label="Automatic vertical tabs" state=automatic activation=TabActivation::Automatic vertical=true/><TabPanel id="auto-tabs" index=0 state=automatic><p>"Automatic Alpha"</p></TabPanel><TabPanel id="auto-tabs" index=1 state=automatic><p>"Disabled panel"</p></TabPanel><TabPanel id="auto-tabs" index=2 state=automatic><p>"Automatic Gamma"</p></TabPanel><TabPanel id="auto-tabs" index=3 state=automatic><p>"Automatic Gamut"</p></TabPanel>
        <Menubar id="bar" label="Editor commands" state=menubar entries action/>
        <button id="menu-trigger" type="button" aria-haspopup="menu" aria-controls="command-menu" aria-expanded=move ||menu_open.get().to_string() on:click=move |_|menu_open.update(|value|*value= !*value)>"Commands"</button>
        <Popover id="command-menu" anchor="menu-trigger" label="Commands" open=menu_open role=PopupRole::Menu initial_focus="commands-item-0"><Menu id="commands" state=menu open=menu_open action/></Popover><output id="menu-result">{move ||selected.get()}</output>
        <h2>"Overlays"</h2><ActionGroup label="Overlay actions">
            <button id="popup-trigger" type="button" aria-controls="breadth-popup" aria-expanded=move ||popover.get().to_string() on:click=move |_|popover.update(|v|*v= !*v)>"Open popover"</button>
            <button id="drawer-trigger" type="button" aria-controls="breadth-drawer" aria-expanded=move ||drawer.get().to_string() on:click=move |_|drawer.set(true)>"Open drawer"</button>
            <button id="nonmodal-trigger" type="button" on:click=move |_|nonmodal.set(true)>"Open nonmodal"</button>
            <button id="restore-overlays" type="button" on:click=move |_|mounted.set(true)>"Restore overlays"</button>
        </ActionGroup><input id="outside-overlay" aria-label="Outside overlay focus" value="Outside"/>
        <Show when=move ||mounted.get()>
            <Popover id="breadth-popup" anchor="popup-trigger" label="Popover example" open=popover initial_focus="popup-close"><ActionGroup label="Popover actions"><button id="popup-close" type="button" on:click=move |_|popover.set(false)>"Close popover"</button><button id="child-popup-trigger" type="button" on:click=move |_|child.set(true)>"Nested popover"</button><button id="popup-unmount" type="button" on:click=move |_|unmount.run(())>"Unmount overlays"</button></ActionGroup>
                <Popover id="child-popup" anchor="child-popup-trigger" label="Nested popover" open=child initial_focus="child-popup-close"><button id="child-popup-close" type="button" on:click=move |_|child.set(false)>"Close nested popover"</button></Popover>
            </Popover>
            <Drawer id="breadth-drawer" open=drawer title="Drawer example" initial_focus_id="drawer-close" fallback_focus_id="drawer-trigger"><ActionGroup label="Drawer actions"><button id="drawer-close" type="button" on:click=move |_|drawer.set(false)>"Close drawer"</button><button id="drawer-child-trigger" type="button" on:click=move |_|drawer_child.set(true)>"Nested drawer"</button><button id="drawer-unmount" type="button" on:click=move |_|unmount.run(())>"Unmount drawer"</button></ActionGroup><Drawer id="drawer-child" open=drawer_child title="Nested drawer" initial_focus_id="drawer-child-close" fallback_focus_id="drawer-close" leading=true><button id="drawer-child-close" type="button" on:click=move |_|drawer_child.set(false)>"Close child drawer"</button></Drawer></Drawer>
            <Dialog id="nonmodal" open=nonmodal title="Nonmodal example" mode=DialogMode::Nonmodal initial_focus_id="nonmodal-close" fallback_focus_id="nonmodal-trigger"><ActionGroup label="Modeless actions"><button id="nonmodal-close" type="button" on:click=move |_|nonmodal.set(false)>"Close nonmodal"</button><button id="nonmodal-child-trigger" type="button" on:click=move |_|nonmodal_child.set(true)>"Nested modeless"</button></ActionGroup><Tooltip id="nested-tip" label="Explain modeless" text="Nested tooltip owns Escape" delay=DelayMs::new(40).unwrap()/><Dialog id="nonmodal-child" open=nonmodal_child title="Nested modeless" mode=DialogMode::Nonmodal initial_focus_id="nonmodal-child-close" fallback_focus_id="nonmodal-child-trigger"><button id="nonmodal-child-close" type="button" on:click=move |_|nonmodal_child.set(false)>"Close modeless child"</button></Dialog></Dialog>
            <Tooltip id="tip" label="Explain controls" text="A plain text description" delay=DelayMs::new(40).unwrap()/>
        </Show>
        <ActionGroup label="Notifications"><button id="show-toast" type="button" on:click=move |_|toast.set(true)>"Show notice"</button><button id="show-timer-toast" type="button" on:click=move |_|timer_toast.set(true)>"Show timed notice"</button></ActionGroup>
        <Toast id="manual-toast" visible=toast message="Saved public message"/><Toast id="timer-toast" visible=timer_toast message="Finite message" duration=DelayMs::new(250).unwrap()/>
        <h2>"Search and collections"</h2><form id="collection-form" on:submit=move |event|event.prevent_default()><Combobox id="combo" label="Search choices" name="combo-value" state=combo/><SuggestInput id="suggest" label="Find person" name="suggest-value" state=suggestions lookup/><output id="suggest-calls">{move ||calls.get()}</output>
        <TextList id="text-rows" label="Text rows" list/><TagList id="tag-rows" label="Editable tags" list=tags/><MetadataInput id="metadata-rows" label="Metadata rows" list=metadata/>
        <CollectionFieldset id="nested-rows" label="Typed nested rows" list=nested create=nested_create render=nested_render/>
        </form><h2>"Controlled table"</h2><DataTable id="data-table" caption="Paged people" state=table striped=true sticky=true/>
        <ActionGroup label="Table snapshot actions"><button id="update-table-row" type="button" on:click=move |_|table.replace(DisplayTable::new(vec!["Name".into(),"Value".into()],(0..7).map(|i|DisplayRow{key:format!("r{i}"),cells:vec![format!("Updated {i}"),i.to_string()]}).collect()).unwrap())>"Update table rows"</button>
        <button id="shrink-table" type="button" on:click=move |_|table.replace(DisplayTable::new(vec!["Name".into(),"Value".into()],vec![DisplayRow{key:"r0".into(),cells:vec!["Person 0".into(),"0".into()]}]).unwrap())>"Shrink table"</button></ActionGroup>
        <Table caption="Explicit native table slots"><TableHead><TableRow row_key="head"><TableHeader colspan=2 sticky=true>"Grouped columns"</TableHeader></TableRow><TableRow row_key="columns"><TableHeader>"Key"</TableHeader><TableHeader>"Value"</TableHeader></TableRow></TableHead><TableGroup><TableRow row_key="body"><TableHeader scope=HeaderScope::Row>"Zero"</TableHeader><TableCell>"0"<TableRule width=120/></TableCell></TableRow></TableGroup><TableFoot><TableRow row_key="foot"><TableCell colspan=2>"Summary"</TableCell></TableRow></TableFoot></Table>
        <h2>"Bounded editing"</h2><form id="bounded-form" on:submit=move |event|event.prevent_default()><Slider id="single-range" label="Single slider" name="single" state=single/><Slider id="pair-range" label="Ordered pair" name="pair" state=range/><RatingInput id="rating" label="Rating" name="rating" state=rating/><SlugInput id="slug" label="Slug draft" name="slug" value=slug style=SlugStyle::Dash alphabet=SlugAlphabet::Unicode/>
        </form><button id="breadth-inspect" type="button" on:click=move |_|feedback.set(serde_json::json!({"text":list.get().rows().iter().map(|row|&row.value).collect::<Vec<_>>(),"combo":combo.query(),"selected":combo.selected(),"rating":rating.value(),"slug":slug.get(),"ranges":[single.handle(0),range.handle(0),range.handle(1)],"tags":tags.get().rows().iter().map(|row|&row.value).collect::<Vec<_>>(),"metadata":metadata.get().rows().iter().map(|row|(&row.value.key,&row.value.value)).collect::<Vec<_>>(),"nested":nested.get().rows().iter().map(|row|match row.value{FieldValue::Missing=>"missing",FieldValue::Null=>"null",FieldValue::Value(false)=>"false",FieldValue::Value(true)=>"true"}).collect::<Vec<_>>(),"filter":table.filter(),"selectedRows":table.selected(),"suggestQuery":suggestions.query(),"table":table.rows().iter().map(|row|&row.key).collect::<Vec<_>>()}).to_string())>"Inspect breadth state"</button><output id="breadth-state" class="sp-core-state" aria-label="Last inspected breadth state">{move ||feedback.get()}</output>
    </section>}
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
    /// Early native edits are admitted before creating a root, with exact byte budgets.
    #[test]
    fn early_draft_boundaries() {
        assert!(BreadthDraft::default().validate().is_ok());
        for field in 0..12 {
            let mut draft = BreadthDraft::default();
            match field {
                0 => draft.text[0] = "雪".repeat(1366),
                1 => draft.slug = "x".repeat(4097),
                2 => draft.query = "x".repeat(1025),
                3 => draft.suggestion_query = "x".repeat(1025),
                4 => draft.tags[0] = "x".repeat(4097),
                5 => draft.metadata.key = "x".repeat(1025),
                6 => draft.filter = "x".repeat(1025),
                7 => draft.rating = Some(6),
                8 => draft.ranges[0] = f64::NAN,
                9 => draft.ranges = [0.0, 9.0, 8.0],
                10 => draft.selected_rows = vec!["forged".into()],
                _ => draft.selected_rows = vec!["r0".into(), "r0".into()],
            };
            assert!(draft.validate().is_err());
        }
        let valid = BreadthDraft {
            details: true,
            text: [String::new(), "0".into()],
            slug: "x".repeat(4096),
            query: "x".repeat(1024),
            ..BreadthDraft::default()
        };
        assert!(valid.validate().is_ok());
    }
}
