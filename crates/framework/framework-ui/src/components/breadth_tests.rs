//! Native decision tables for the exact controllers used by hydrated browser widgets.
use super::*;
use leptos::prelude::*;

/// Bounded immutable options include disabled, empty and zero machine values.
fn options() -> SelectOptions {
    SelectOptions::new(vec![
        SelectOption {
            value: "".into(),
            label: "Alpha".into(),
            disabled: false,
        },
        SelectOption {
            value: "blocked".into(),
            label: "Beta".into(),
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
    .unwrap()
}
/// Required/optional/multiple and roving/typeahead choices have explicit decision boundaries.
#[test]
fn disclosure_and_choice_admission() {
    let owner = Owner::new();
    owner.with(|| {
        assert!(ChoiceState::new(options(), Some("missing".into())).is_err());
        assert!(ChoiceState::new(options(), Some("blocked".into())).is_err());
        let choice = ChoiceState::new(options(), None).unwrap();
        assert_eq!(choice.focused(), Some(0));
        assert!(!choice.focus(1));
        assert!(!choice.focus(100));
        assert!(!choice.choose("blocked"));
        assert_eq!(choice.navigate(ChoiceMove::Next, false), Some(2));
        assert_eq!(choice.selected(), None);
        assert_eq!(choice.navigate(ChoiceMove::Last, true), Some(3));
        assert_eq!(choice.selected().as_deref(), Some("gamut"));
        assert_eq!(choice.navigate(ChoiceMove::Next, true), Some(0));
        assert_eq!(choice.selected().as_deref(), Some(""));
        assert_eq!(choice.navigate(ChoiceMove::Previous, false), Some(3));
        assert_eq!(choice.navigate(ChoiceMove::First, false), Some(0));
        assert_eq!(choice.typeahead("g"), Some(2));
        assert_eq!(choice.typeahead("g"), Some(3));
        assert_eq!(choice.typeahead("gam"), Some(3));
        assert_eq!(choice.typeahead("zz"), None);
        assert_eq!(choice.typeahead(""), None);
        assert_eq!(choice.typeahead(&"a".repeat(129)), None);
        choice.clear();
        assert_eq!(choice.selected(), None);
        let empty = ChoiceState::new(SelectOptions::new(vec![]).unwrap(), None).unwrap();
        assert_eq!(empty.navigate(ChoiceMove::Last, false), None);
        assert_eq!(empty.typeahead("g"), None);
        assert!(DisclosureState::new(options(), vec![], false, false).is_err());
        assert!(DisclosureState::new(options(), vec!["".into(), "0".into()], false, true).is_err());
        assert!(DisclosureState::new(options(), vec!["0".into(), "0".into()], true, true).is_err());
        let single = DisclosureState::new(options(), vec!["".into()], false, false).unwrap();
        assert!(!single.toggle(""));
        assert!(single.toggle("0"));
        assert!(!single.is_open(""));
        assert!(single.is_open("0"));
        assert!(!single.toggle("blocked"));
        let multiple = DisclosureState::new(options(), vec![], true, true).unwrap();
        assert!(multiple.toggle(""));
        assert!(multiple.toggle("0"));
        assert!(multiple.is_open(""));
        assert!(multiple.toggle(""));
        assert!(!multiple.is_open(""));
    });
    owner.cleanup();
    let mut buffer = TypeaheadState::default();
    assert_eq!(buffer.push("g", 0.0).as_deref(), Some("g"));
    assert_eq!(buffer.push("a", 20.0).as_deref(), Some("ga"));
    assert_eq!(buffer.push("g", 800.0).as_deref(), Some("g"));
    assert_eq!(buffer.push("g", 810.0).as_deref(), Some("g"));
    assert_eq!(buffer.push(" ", 820.0), None);
    assert_eq!(buffer.push("Enter", 820.0), None);
    assert_eq!(buffer.push("x", f64::NAN), None);
    assert_eq!(buffer.push("x", -1.0), None);
}
/// Paging/URLs distinguish unknown/empty/boundary totals and reject ambiguous encodings.
#[test]
fn paging_and_geometry() {
    let empty = PageWindow::new(80, 10, PageTotal::Known(0)).unwrap();
    assert_eq!(empty.skip(), 0);
    assert!(!empty.has_next());
    assert!(!empty.has_previous());
    assert_eq!(empty.previous(), empty);
    let last = PageWindow::new(100, 2, PageTotal::Known(5)).unwrap();
    assert_eq!(last.skip(), 4);
    assert!(!last.has_next());
    assert_eq!(last.next_page(), None);
    assert_eq!(last.previous().skip(), 2);
    let first = PageWindow::new(0, 2, PageTotal::Known(5)).unwrap();
    assert_eq!(first.next_page().unwrap().skip(), 2);
    assert_eq!(first.query(), "skip=0&limit=2");
    assert_eq!(
        PageWindow::from_query(&first.query(), first.total()).unwrap(),
        first
    );
    assert!(PageWindow::new(0, 0, PageTotal::Known(5)).is_err());
    assert!(PageWindow::new(0, 101, PageTotal::Known(5)).is_err());
    assert!(PageWindow::new(u64::MAX, 1, PageTotal::Unknown { has_more: true }).is_err());
    let near = PageWindow::new(u64::MAX - 1, 1, PageTotal::Unknown { has_more: true }).unwrap();
    assert!(!near.has_next());
    assert_eq!(near.next_page(), None);
    assert!(
        PageWindow::new(0, 100, PageTotal::Unknown { has_more: true })
            .unwrap()
            .has_next()
    );
    assert!(
        !PageWindow::new(0, 100, PageTotal::Unknown { has_more: false })
            .unwrap()
            .has_next()
    );
    for query in [
        "skip=00&limit=2",
        "skip=-1&limit=2",
        "skip=%30&limit=2",
        "skip=0&limit=2&x=3",
        "skip=0&skip=1&limit=2",
        "skip=0",
        "limit=2",
        "skip=18446744073709551616&limit=2",
    ] {
        assert!(
            PageWindow::from_query(query, PageTotal::Known(10)).is_err(),
            "{query}"
        );
    }
    assert_eq!(
        place_popup([90.0, 90.0, 10.0, 10.0], [20.0, 20.0], [100.0, 100.0]).unwrap(),
        [72.0, 66.0]
    );
    assert_eq!(
        place_popup([-50.0, -50.0, 0.0, 0.0], [200.0, 200.0], [100.0, 100.0]).unwrap(),
        [8.0, 8.0]
    );
    assert!(place_popup([f64::NAN, 0.0, 0.0, 0.0], [1.0, 1.0], [100.0, 100.0]).is_err());
    assert!(place_popup([0.0, 0.0, -1.0, 0.0], [1.0, 1.0], [100.0, 100.0]).is_err());
    assert!(place_popup([0.0; 4], [1.0, 1.0], [1.0, 1.0]).is_err());
}
/// Stable identities survive reorder/edit/removal; invalid mutations are atomic.
#[test]
fn typed_collections() {
    let mut list = BoundedList::new(
        vec![
            CollectionRow {
                id: 7,
                value: "".to_owned(),
            },
            CollectionRow {
                id: 9,
                value: "0".into(),
            },
        ],
        3,
    )
    .unwrap();
    assert_eq!(list.add("雪".into()).unwrap(), 10);
    assert!(list.add("excess".into()).is_err());
    list.reorder(10, 0).unwrap();
    assert_eq!(list.rows()[0].id, 10);
    list.edit(9, "false".into()).unwrap();
    assert_eq!(list.rows()[2].value, "false");
    let before = list.clone();
    assert!(list.edit(9, "x".repeat(4097)).is_err());
    assert_eq!(list, before);
    assert!(list.reorder(9, 3).is_err());
    assert!(list.reorder(999, 0).is_err());
    assert!(list.remove(999).is_err());
    assert!(list.edit(999, String::new()).is_err());
    assert_eq!(list.remove(10).unwrap(), "雪");
    assert_eq!(list.add("new".into()).unwrap(), 11);
    assert!(
        BoundedList::new(
            vec![
                CollectionRow {
                    id: 1,
                    value: false
                },
                CollectionRow { id: 1, value: true }
            ],
            2
        )
        .is_err()
    );
    assert!(
        BoundedList::new(
            vec![CollectionRow {
                id: 0,
                value: false
            }],
            2
        )
        .is_err()
    );
    assert!(BoundedList::<String>::new(vec![], 0).is_err());
    assert!(BoundedList::<String>::new(vec![], 101).is_err());
    assert!(
        BoundedList::new(
            vec![CollectionRow {
                id: u64::MAX,
                value: false
            }],
            1
        )
        .is_err()
    );
    assert!(
        !BoundedList::new(
            vec![CollectionRow {
                id: u64::MAX - 1,
                value: false
            }],
            2
        )
        .unwrap()
        .can_add()
    );
    let list = BoundedList::new(
        vec![
            CollectionRow {
                id: 1,
                value: FieldValue::Missing,
            },
            CollectionRow {
                id: 2,
                value: FieldValue::Null,
            },
            CollectionRow {
                id: 3,
                value: FieldValue::Value(false),
            },
        ],
        3,
    )
    .unwrap();
    assert_ne!(list.rows()[0].value, list.rows()[1].value);
    assert_ne!(list.rows()[1].value, list.rows()[2].value);
    let zero = BoundedList::new(
        vec![CollectionRow {
            id: 1,
            value: FieldValue::Value(0_i64),
        }],
        1,
    )
    .unwrap();
    assert_eq!(zero.rows()[0].value, FieldValue::Value(0));
    assert!(
        BoundedList::new(
            vec![CollectionRow {
                id: 1,
                value: f64::NAN
            }],
            1
        )
        .is_err()
    );
    assert!(
        BoundedList::new(
            vec![CollectionRow {
                id: 1,
                value: f32::INFINITY
            }],
            1
        )
        .is_err()
    );
    assert!(
        BoundedList::new(
            vec![CollectionRow {
                id: 1,
                value: MetadataEntry {
                    key: "x".repeat(1025),
                    value: String::new()
                }
            }],
            1
        )
        .is_err()
    );
}
/// Filtering/sorting/replacement never invent record operations or select forged keys.
#[test]
fn local_table_controller() {
    let owner = Owner::new();
    owner.with(|| {
        let data = DisplayTable::new(
            vec!["Name".into(), "Value".into()],
            vec![
                DisplayRow {
                    key: "b".into(),
                    cells: vec!["Beta".into(), "0".into()],
                },
                DisplayRow {
                    key: "a".into(),
                    cells: vec!["Alpha".into(), "".into()],
                },
                DisplayRow {
                    key: "c".into(),
                    cells: vec!["Alpha".into(), "false".into()],
                },
            ],
        )
        .unwrap();
        assert!(TableState::new(data.clone(), 0).is_err());
        let state = TableState::new(data, 1).unwrap();
        assert_eq!(state.column_count(), 2);
        assert_eq!(state.column(0), "Name");
        assert_eq!(state.column(2), "");
        assert!(!state.toggle_sort(2));
        assert!(!state.select("forged", true));
        state.select_page(true);
        assert!(state.selected().contains("b"));
        state
            .page()
            .update(|page| *page = page.next_page().unwrap());
        state.select_page(true);
        assert_eq!(state.selected().len(), 2);
        state.select_page(false);
        assert_eq!(state.selected().len(), 1);
        state.toggle_sort(0);
        assert_eq!(state.rows()[0].key, "a");
        assert_eq!(state.matching()[1].key, "c");
        state.toggle_sort(0);
        assert_eq!(state.rows()[0].key, "b");
        state.toggle_sort(0);
        assert_eq!(state.sort(), None);
        state.adopt_filter("alpha".into()).unwrap();
        assert_eq!(state.filter(), "alpha");
        assert_eq!(
            state.matching().len(),
            3,
            "early draft does not repair the initial SSR row tree"
        );
        assert!(state.adopt_filter("x".repeat(1025)).is_err());
        state.set_filter("alpha".into()).unwrap();
        assert_eq!(state.matching().len(), 2);
        assert_eq!(state.page().get().skip(), 0);
        assert!(state.set_filter("x".repeat(1025)).is_err());
        state.set_filter("missing".into()).unwrap();
        assert!(state.rows().is_empty());
        assert_eq!(state.page().get().total(), PageTotal::Known(0));
        state.set_filter(String::new()).unwrap();
        state.toggle_sort(1);
        state.replace(DisplayTable::new(vec!["One".into()], vec![]).unwrap());
        assert_eq!(state.column_count(), 1);
        assert_eq!(state.column(0), "One");
        assert_eq!(state.sort(), None);
        assert!(state.selected().is_empty());
        assert!(state.rows().is_empty());
    });
    owner.cleanup();
}
/// Range/rating/slugs preserve finite and exact zero/None/Unicode boundaries.
#[test]
fn bounded_input_policies() {
    for (min, max, step) in [
        (0.0, 0.0, 1.0),
        (1.0, 0.0, 1.0),
        (0.0, 1.0, 0.0),
        (0.0, 1.0, -1.0),
        (0.0, 1.0, 0.3),
        (0.0, f64::INFINITY, 1.0),
    ] {
        assert!(RangeScale::new(min, max, step).is_err());
    }
    let scale = RangeScale::new(-1.0, 1.0, 0.25).unwrap();
    assert_eq!(scale.min(), -1.0);
    assert_eq!(scale.max(), 1.0);
    assert_eq!(scale.step(), 0.25);
    assert_eq!(scale.snap(9.0), Some(1.0));
    assert_eq!(scale.snap(-9.0), Some(-1.0));
    assert_eq!(scale.snap(f64::NAN), None);
    let owner = Owner::new();
    owner.with(|| {
        assert!(RangeState::new(scale, RangeValue::Single(0.1)).is_err());
        assert!(
            RangeState::new(
                scale,
                RangeValue::Pair {
                    lower: 1.0,
                    upper: 0.0
                }
            )
            .is_err()
        );
        let pair = RangeState::new(
            scale,
            RangeValue::Pair {
                lower: -0.5,
                upper: 0.5,
            },
        )
        .unwrap();
        assert!(pair.edit(0, "1"));
        assert_eq!(pair.handle(0), Some(0.5));
        assert!(pair.edit(1, "-1"));
        assert_eq!(pair.handle(1), Some(0.5));
        assert!(!pair.edit(2, "0"));
        assert!(!pair.edit(1, "NaN"));
        assert_eq!(pair.handle(3), None);
        let single = RangeState::new(scale, RangeValue::Single(0.0)).unwrap();
        assert!(single.edit(0, ".9"));
        assert_eq!(single.value(), RangeValue::Single(1.0));
        assert!(!single.edit(1, "0"));
        assert_eq!(single.handle(1), None);
        assert!(RatingState::new(0, None).is_err());
        assert!(RatingState::new(101, None).is_err());
        assert!(RatingState::new(5, Some(6)).is_err());
        let rating = RatingState::new(5, None).unwrap();
        assert!(rating.set(Some(0)));
        assert_eq!(rating.value(), Some(0));
        assert!(!rating.set(Some(6)));
        assert!(rating.set(None));
        assert_eq!(rating.value(), None);
    });
    owner.cleanup();
    assert_eq!(
        slugify(" Crème brûlée + 0 ", SlugStyle::Dash, SlugAlphabet::Ascii).unwrap(),
        "cr-me-br-l-e-0"
    );
    assert_eq!(
        slugify(
            " Crème 雪 + 0 ",
            SlugStyle::Underscore,
            SlugAlphabet::Unicode
        )
        .unwrap(),
        "crème_雪_0"
    );
    assert_eq!(
        slugify(" Hello snow_0 ", SlugStyle::Camel, SlugAlphabet::Ascii).unwrap(),
        "HelloSnow0"
    );
    assert_eq!(
        slugify("--", SlugStyle::Dash, SlugAlphabet::Ascii).unwrap(),
        ""
    );
    assert!(slugify(&"x".repeat(4097), SlugStyle::Dash, SlugAlphabet::Ascii).is_err());
    let mut activity = TooltipActivity::default();
    assert!(!activity.requested());
    activity.hover(true);
    assert!(activity.requested());
    activity.focus(true);
    activity.hover(false);
    assert!(activity.requested());
    activity.dismiss();
    assert!(!activity.requested());
    activity.focus(false);
    activity.focus(true);
    assert!(activity.requested());
    assert_eq!(DelayMs::default().get(), 150);
    assert!(DelayMs::new(10_000).is_ok());
    assert!(DelayMs::new(10_001).is_err());
}

/// Search state admits only enabled current values and keeps draft/selection independent.
#[test]
fn searchable_choice_boundaries() {
    let owner = Owner::new();
    owner.with(|| {
        assert!(ComboboxState::new(options(), Some("blocked".into())).is_err());
        let state = ComboboxState::new(options(), None).unwrap();
        state.adopt_query("G".into()).unwrap();
        assert_eq!(state.query(), "G");
        assert!(!state.open().get());
        assert_eq!(state.matching().len(), 2);
        assert!(state.adopt_query("x".repeat(1025)).is_err());
        assert!(state.edit("x".repeat(1025)).is_err());
        state.edit("g".into()).unwrap();
        assert_eq!(state.navigate(true, false), Some(2));
        assert!(state.choose(2));
        assert_eq!(state.selected().as_deref(), Some("0"));
        state.edit("".into()).unwrap();
        assert!(!state.choose(1));
        assert!(!state.choose(999));
        assert_eq!(state.navigate(false, true), Some(3));
        assert_eq!(state.navigate(true, true), Some(0));
        assert!(state.choose(0));
        assert_eq!(state.selected().as_deref(), Some(""));
        assert_eq!(state.admit_native_edit("Ga".into()).unwrap(), "Ga");
        assert_eq!(
            state.selected().as_deref(),
            Some(""),
            "valid admission delegates acquisition to the caller"
        );
        assert!(state.admit_native_edit("雪".repeat(342)).is_err());
        assert_eq!(
            state.selected(),
            None,
            "an oversized native edit cannot submit a stale machine value"
        );
        assert_eq!(state.query(), "Alpha", "the last bounded draft is retained");
        state.edit("missing".into()).unwrap();
        assert_eq!(state.navigate(true, false), None);
        state.cancel();
        assert!(!state.open().get());
        assert_eq!(state.status(), SuggestionStatus::Ready);
        assert!(
            BreadcrumbTrail::new(vec![Crumb {
                label: "unsafe".into(),
                href: Some("javascript:bad()".into())
            }])
            .is_err()
        );
        assert!(BreadcrumbTrail::new(vec![]).is_ok());
    });
    owner.cleanup();
}

/// Equal local generations never permit another controller's completion to publish.
#[test]
fn foreign_suggestion_completion_is_rejected() {
    let owner = Owner::new();
    owner.with(|| {
        let a = ComboboxState::new(options(), None).unwrap();
        let b = ComboboxState::new(options(), None).unwrap();
        let future = || async { Ok(options()) };
        let foreign =
            futures::executor::block_on(a.request("Ga".into(), future()).unwrap()).unwrap();
        let own = futures::executor::block_on(b.request("Ga".into(), future()).unwrap()).unwrap();
        assert!(
            !b.publish(foreign),
            "a foreign live controller has the same local generation"
        );
        assert!(b.publish(own));
    });
    owner.cleanup();
}

/// CSS zoom is measured from computed values, never rounded offset geometry.
#[test]
fn css_zoom_admission() {
    let values = |items: &[&str]| {
        items
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        effective_zoom(&values(&["1", "normal", "200%"])).unwrap(),
        2.0
    );
    assert_eq!(effective_zoom(&values(&["2", "1.25"])).unwrap(), 2.5);
    for items in [
        vec!["NaN"],
        vec!["0"],
        vec!["-1"],
        vec!["1px"],
        vec!["inf"],
        vec!["10000", "10000"],
    ] {
        assert!(effective_zoom(&values(&items)).is_err());
    }
    assert!(effective_zoom(&vec!["1".into(); 65]).is_err());
}

/// Native semantic slots escape public text, expose errors and preserve native span rules.
#[test]
fn breadth_semantic_markup_and_hostile_text() {
    let owner = Owner::new();
    owner.with(||{
        let attack="</textarea><script>globalThis.pwned=1</script>&\"'\u{2028}";
        let error=RwSignal::new(attack.to_owned());
        let choices=ChoiceState::new(options(),Some(String::new())).unwrap();
        let disclosures=DisclosureState::new(options(),vec![],false,true).unwrap();
        let combo=ComboboxState::new(options(),None).unwrap();
        let empty=TableState::new(DisplayTable::new(vec![attack.to_owned()],vec![]).unwrap(),2).unwrap();
        let html=view!{
            <Breadcrumbs label=attack trail=BreadcrumbTrail::new(vec![Crumb{label:attack.into(),href:None}]).unwrap()/>
            <Accordion label=attack><AccordionItem id="native-acc" item="" state=disclosures>{attack}</AccordionItem></Accordion>
            <Details id="native-details-test" label=attack open=RwSignal::new(false)>{attack}</Details>
            <Tabs id="native-tabs" label=attack state=choices/><TabPanel id="native-tabs" index=0 state=choices>{attack}</TabPanel>
            <Menu id="native-menu" state=choices open=RwSignal::new(false) action=Callback::new(|_:String|{})/>
            <Popover id="native-popup" label=attack anchor="native-anchor" open=RwSignal::new(false)>{attack}</Popover>
            <Tooltip id="native-tooltip" label=attack text=attack/>
            <Toast id="native-toast" visible=RwSignal::new(true) message=attack urgent=true/>
            <Pagination label=attack page=RwSignal::new(PageWindow::new(0,2,PageTotal::Unknown{has_more:true}).unwrap())/>
            <Table caption=attack><TableHead><TableRow row_key="head"><TableHeader scope=HeaderScope::ColumnGroup colspan=0 rowspan=0>{attack}</TableHeader><TableHeader scope=HeaderScope::RowGroup rowspan=101>"group"</TableHeader></TableRow></TableHead>
                <TableGroup><TableRow row_key="row"><TableCell colspan=200 rowspan=0><TableRule width=0/>{attack}</TableCell><TableCell><TableRule width=u16::MAX/></TableCell></TableRow></TableGroup><TableFoot><TableRow row_key="foot"><TableCell>"0"</TableCell></TableRow></TableFoot></Table>
            <DataTable id="empty-data" caption="Empty data" state=empty/>
            <RatingInput id="native-rating" name="rating" label=attack state=RatingState::new(5,Some(0)).unwrap() error/>
            <SuggestInput id="native-suggest" name="suggest" label=attack state=combo lookup=Callback::new(|_:String|{}) disabled=true error/>
        }.to_html();
        for expected in ["aria-current=\"page\"","aria-controls=\"native-acc-panel\"","role=\"tablist\"","role=\"tabpanel\"","role=\"menuitem\"","popover=\"auto\"","role=\"tooltip\"","role=\"alert\"","scope=\"colgroup\"","scope=\"rowgroup\"","colspan=\"100\"","rowspan=\"0\"","rowspan=\"100\"","width:1px","width:4096px","No matching rows","aria-describedby=\"native-rating-error\"","aria-invalid=\"true\"","id=\"native-suggest\""]{assert!(html.contains(expected),"missing {expected}");}
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;/textarea&gt;&lt;script&gt;globalThis.pwned=1&lt;/script&gt;"));
        assert!(html.contains("&amp;"));
    });
    owner.cleanup();
}

/// Initial selection is represented once by native checked state, including SSR.
#[test]
fn table_selection_is_native_before_javascript() {
    let owner = Owner::new();
    owner.with(|| {
        let state = TableState::new(
            DisplayTable::new(
                vec!["Name".into()],
                vec![DisplayRow {
                    key: "x".into(),
                    cells: vec!["Zero".into()],
                }],
            )
            .unwrap(),
            2,
        )
        .unwrap();
        assert!(state.select("x", true));
        let html = view! {<DataTable id="seeded" caption="Selected" state/>}.to_html();
        let input = html
            .split("id=\"seeded-row-x\"")
            .nth(1)
            .unwrap()
            .split('>')
            .next()
            .unwrap();
        assert!(
            input.contains("checked"),
            "SSR must expose the initial native selection"
        );
        assert!(
            !html.contains("data-selected="),
            "DataTable must not duplicate checkbox state"
        );
    });
    owner.cleanup();
}

/// Modeless/native close requests preserve external focus without changing modal defaults.
#[test]
fn native_dialog_mode_policies() {
    assert!(DialogMode::Modal.returns_focus(false, false, false));
    assert!(!DialogMode::Nonmodal.returns_focus(false, false, false));
    assert!(DialogMode::Nonmodal.returns_focus(true, false, false));
    assert!(DialogMode::Nonmodal.returns_focus(false, true, false));
    assert!(DialogMode::Nonmodal.returns_focus(false, false, true));
    let owner = Owner::new();
    owner.with(||{
        let open=RwSignal::new(false);
        let modal=view!{<Dialog id="modal-policy" open title="Modal" initial_focus_id="x" fallback_focus_id="y">"Body"</Dialog>}.to_html();
        let nonmodal=view!{<Dialog id="modeless-policy" open title="Modeless" mode=DialogMode::Nonmodal initial_focus_id="x" fallback_focus_id="y">"Body"</Dialog>}.to_html();
        let locked=view!{<Dialog id="modeless-locked" open title="Modeless" mode=DialogMode::Nonmodal close_on_escape=false initial_focus_id="x" fallback_focus_id="y">"Body"</Dialog>}.to_html();
        assert!(!modal.contains("closedby"));
        assert!(nonmodal.contains("closedby=\"closerequest\""));
        assert!(locked.contains("closedby=\"none\""));
    });
    owner.cleanup();
}
