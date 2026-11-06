//! Native behavior contracts; browser interactions are tested through real generated consumers.
use super::*;
use leptos::prelude::*;

/// Scope every render so callbacks and root state are disposed deterministically.
fn render(build: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    let html = owner.with(|| build().to_html());
    owner.cleanup();
    html
}

#[test]
fn source_overflow_retains_javascript_slice_and_split_semantics() {
    for (text, count, words, expected) in [
        ("abc", 0.0, false, "abc"),
        ("abc", f64::NAN, false, "abc"),
        ("abc", -1.0, false, "ab…"),
        ("a  b c", 2.0, true, "a …"),
        ("😀z", 1.0, false, "�…"),
        ("abc", 0.5, false, "…"),
        ("abc", f64::NEG_INFINITY, false, "…"),
        ("abc", f64::INFINITY, false, "abc"),
    ] {
        assert_eq!(source_overflow(text, count, words, true), expected);
    }
    assert_eq!(source_overflow("abcdef", 3.0, false, false), "abc");
}

#[test]
fn source_color_and_mail_links_keep_permissive_source_values() {
    let html = render(|| {
        view!{<SourceColor value="rebeccapurple" size=8/><SourceColor value="var(--brand)" text=false/><SourceColor value="red" box_visible=false text=false/>
        <SourceEmailLink value="any text?subject=source" target="_blank" title="Write"/>}.into_any()
    });
    assert!(html.contains("background-color:rebeccapurple;height:8px"));
    assert!(html.contains("background-color:var(--brand)"));
    assert!(html.contains(">red</span>"));
    assert!(html.contains("href=\"mailto:any text?subject=source\""));
    assert!(html.contains("title=\"Write\""));
}

#[test]
fn source_rating_preserves_unbounded_rounding_and_star_only_output() {
    let html = render(|| view! {<SourceRating value=2.5 max=Some(4.5) remainder=true/>}.into_any());
    assert_eq!(html.matches('★').count(), 3);
    assert_eq!(html.matches('☆').count(), 2);
    assert!(!html.contains(" / "));
    assert_eq!(
        render(|| view! {<SourceRating value=2.9 round="floor"/>}.into_any())
            .matches('★')
            .count(),
        2
    );
    assert_eq!(
        render(|| view! {<SourceRating value=2.1 round="ceil"/>}.into_any())
            .matches('★')
            .count(),
        3
    );
    assert_eq!(
        render(|| view! {<SourceRating value=2.0 max=Some(-1.0)/>}.into_any())
            .matches('★')
            .count(),
        0
    );
    assert_eq!(
        render(|| view! {<SourceRating value=f64::NAN/>}.into_any())
            .matches('★')
            .count(),
        0
    );
}

#[test]
fn source_lists_keep_duplicate_empty_and_zero_children_and_heterogeneous_rows() {
    let html = render(|| {
        view!{<SourceList value=vec!["0".into(),"".into(),"0".into()] ordered=true/>
        <SourceSpread value=vec!["a".into(),"".into(),"b".into()] separator=", "/>
        <SourceSpread value=vec!["x".into(),"y".into()] separator="line"/>
        <SourceTable columns=vec!["first".into()] rows=vec![vec!["0".into()],vec!["x".into(),"y".into()]] stripes=vec!["red".into(),"blue".into(),"green".into()]/ >}.into_any()
    });
    assert!(html.contains("<ol"));
    assert_eq!(html.matches("<li>").count(), 3);
    assert!(html.contains("a, , b"));
    assert!(html.contains("<div>x</div><div>y</div>"));
    assert_eq!(html.matches("<td").count(), 3);
    assert!(html.contains("background-color:red"));
    assert!(html.contains("background-color:green"));
    assert!(
        !render(|| view! {<SourceTable columns=vec![] rows=vec![]/>}.into_any()).contains("<table")
    );
}

#[test]
fn source_formula_arithmetic_precedence_templates_and_ieee_results() {
    let eval = |expression| source_formula(expression, 0.0, &[]);
    for (expression, expected) in [
        ("2+3*4", 14.0),
        ("(2+3)*4", 20.0),
        ("20/2/2", 5.0),
        ("10%3*2", 2.0),
        ("-2*-(3+1)", 8.0),
        ("1e3 + .5", 1000.5),
        ("-7%3", -1.0),
        ("1e308*2", f64::INFINITY),
        ("1/0", f64::INFINITY),
        ("-1/0", f64::NEG_INFINITY),
        ("(1 +)", 0.0),
        ("1 2", 0.0),
        ("Math.max(1,2)", 0.0),
        ("2**3", 0.0),
        ("()", 0.0),
        ("", 0.0),
        ("+2", 0.0),
        ("1e+", 0.0),
    ] {
        assert_eq!(eval(expression), expected, "{expression}");
    }
    assert!(eval("0/0").is_nan());
    assert!(eval("1%0").is_nan());
    assert_eq!(
        source_formula("{this}+{x}*2", 4.0, &[("x".into(), 3.0)]),
        10.0
    );
    assert_eq!(source_formula("{this}+{this}", 4.0, &[]), 4.0);
    assert_eq!(source_formula("{this}+{this}+{this}", 4.0, &[]), 0.0);
    assert_eq!(source_formula("{x}+{x}", 0.0, &[("x".into(), 5.0)]), 5.0);
    assert_eq!(source_formula("{missing}+7", 0.0, &[]), 7.0);
    assert_eq!(source_formula("{this}+2", f64::NAN, &[]), 2.0);
    assert_eq!(source_formula("{this}", f64::INFINITY, &[]), f64::INFINITY);
}

#[test]
fn source_common_masks_compile_and_keep_optional_escape_quantifier_behavior() {
    assert_eq!(source_mask_value("999-999", "123456", false), "123-456");
    assert_eq!(source_mask_value("+99 999", "63123", false), "+63 123");
    assert_eq!(
        source_mask_value("(999) 999-9999", "1234567890", false),
        "(123) 456-7890"
    );
    assert_eq!(source_mask_value("aaa-***", "Éab7X9", false), "Éab-7X9");
    assert_eq!(source_mask_value("99[99]", "12", false), "12");
    assert_eq!(source_mask_value("9{2,4}", "123", false), "123");
    assert_eq!(source_mask_value("\\9 99", "42", false), "9 42");
    assert_eq!(source_mask_value("99-99", "", false), "");
    assert_eq!(source_mask_value("99-99", "", true), "__-__");
    assert_eq!(source_mask_value("99-99", "a1b", true), "1_-__");
    for unsupported in ["9{3,2}", "9{+}", "[99", "99]", "99|aa", "\\"] {
        assert!(source_mask(unsupported).is_none(), "{unsupported}");
    }
}

#[test]
fn source_code_tabs_preserve_utf16_selection_and_invalid_json_text() {
    assert_eq!(source_code_tab("a😀b", 1, 3, false), ("a\tb".into(), 2));
    assert_eq!(source_code_tab("a\n\tx", 4, 4, true), ("a\nx".into(), 3));
    assert_eq!(source_code_tab("abc", 2, 2, true), ("abc".into(), 2));
    let html = render(|| {
        let value = RwSignal::new("{ invalid JSON\n</textarea><script>".into());
        view! {<SourceCodeEditor id="code" name="code" value/>}.into_any()
    });
    assert!(html.contains("{ invalid JSON"));
    assert!(!html.contains("</textarea><script>"));
    assert!(html.contains("name=\"code\""));
}

#[test]
fn source_color_conversion_and_small_numeric_specialization_follow_source() {
    for (value, expected) in [
        ("RED", "#FF0000"),
        ("rebeccapurple", "#663399"),
        ("#aB3", "#aabb33"),
        ("abcdef", "#abcdef"),
        ("rgb(300, 2, 3)", "#ff0203"),
        ("rgba(1, 2, 3, .5)", "#010203"),
        ("rgba(1,2,3)", "#010203"),
        ("rgb(1,2,3,.5)", "#010203"),
        ("rgba(1,2,3,.)", "#000000"),
        ("bad", "#bbaadd"),
        ("rgb( 1,2,3)", "#000000"),
    ] {
        assert_eq!(source_color_hex(value), expected, "{value}");
    }
    for (value, expected) in [
        ("12", "9"),
        ("1.2", "9"),
        ("-1", "0"),
        ("0004", "4"),
        ("-", "0"),
        ("", ""),
        ("letters", ""),
        ("1x", "1"),
    ] {
        assert_eq!(source_small(value), expected, "{value}");
    }
}

#[test]
fn source_file_queue_and_tombstoned_collections_preserve_source_transitions() {
    let mut state = SourceFileState {
        urls: vec!["old".into()],
        queued: 3,
    };
    state.complete(vec!["a".into(), "b".into()], true);
    assert_eq!(state.queued, 1);
    assert_eq!(state.urls, ["old", "a", "b"]);
    state.remove(1);
    assert_eq!(state.urls, ["old", "b"]);
    assert_eq!(state.queued, 0);
    state.complete(vec!["new".into(), "ignored".into()], false);
    assert_eq!(state.urls, ["new"]);
    let mut rows = SourceRows {
        rows: vec![Some("a".to_owned()), Some("b".to_owned())],
    };
    rows.remove(0);
    rows.add("c".into());
    rows.edit(1, "B".into());
    rows.edit(0, "ignored".into());
    assert_eq!(rows.rows, [None, Some("B".into()), Some("c".into())]);
    assert_eq!(rows.clean(), ["B", "c"]);
}

#[test]
fn source_relative_thresholds_and_carousel_boundaries_follow_source() {
    for (seconds, short, expected) in [
        (-44.0, false, "a few seconds ago"),
        (-45.0, false, "a minute ago"),
        (-120.0, false, "2 minutes ago"),
        (3600.0, true, "-1h"),
        (-86400.0, false, "a day ago"),
        (-86400.0 * 30.0, true, "1M"),
        (0.0, true, "now"),
        (f64::NAN, false, "Invalid date"),
    ] {
        assert_eq!(source_relative(seconds, short), expected);
    }
    let mut state = SourceCarouselState {
        index: 0.0,
        frames: 3,
        repeat: false,
    };
    state.step(false);
    assert_eq!(state.index, 0.0);
    state.step(true);
    assert_eq!(state.index, 1.0);
    state.index = 2.0;
    state.step(true);
    assert_eq!(state.index, 2.0);
    state.repeat = true;
    state.step(true);
    assert_eq!(state.index, 0.0);
    state.step(false);
    assert_eq!(state.index, 2.0);
}

#[test]
fn source_dataset_lookups_keep_distinct_selector_and_display_codes() {
    assert_eq!(COUNTRIES.len(), 198);
    assert_eq!(CURRENCIES.len(), 116);
    assert!(COLORS.len() > 100);
    let html = render(|| {
        view!{<SourceCountry value="PHL"/><SourceCountry value="PH"/><SourceCountry value="USD" currency=true/><SourceCountry value="PHL" flag=false text=false/>}.into_any()
    });
    assert!(html.contains("Philippines"));
    assert!(html.contains(">PH</span>"));
    assert!(html.contains("US Dollar"));
    let country = COUNTRIES
        .iter()
        .find(|country| country.iso2 == "PH")
        .unwrap();
    assert_eq!(country.iso3, "PHL");
    assert_eq!(country.cur, "PHP");
}

#[cfg(feature = "source")]
#[test]
fn source_markdown_retains_html_and_native_renderer_features() {
    let html =
        source_markdown("# Title\n\n**bold** <span onclick=\"app()\">raw</span>\n\n- [x] done\n");
    assert!(html.contains("<h1>Title</h1>"));
    assert!(html.contains("<strong>bold</strong>"));
    assert!(html.contains("onclick=\"app()\""));
    assert!(html.contains("checked"));
}

#[test]
fn source_formula_deep_nesting_and_radix_numbers_need_no_call_stack_recursion() {
    let expression = format!("{}1{}", "(".repeat(10000), ")".repeat(10000));
    assert_eq!(source_formula(&expression, 0.0, &[]), 1.0);
    assert_eq!(source_formula("0x10+0b10+0o7", 0.0, &[]), 25.0);
    assert_eq!(source_formula("1\u{a0}+\u{feff}2", 0.0, &[]), 3.0);
    assert_eq!(source_formula("0x", 0.0, &[]), 0.0);
}

#[test]
fn source_upload_tickets_retain_captured_array_and_queue_on_out_of_order_completion() {
    let owner = Owner::new();
    owner.with(|| {
        let control = super::file_state::FileController::new(vec!["old".into()], true);
        let first = control.begin(1);
        let second = control.begin(2);
        control.complete(&second, vec!["second".into()]);
        assert_eq!(control.queued.get_untracked(), 2);
        control.remove(0);
        assert_eq!(control.urls.get_untracked(), ["second"]);
        // Frui's late closure mutates its originally captured array, including the removed old value.
        control.complete(&first, vec!["first".into()]);
        assert_eq!(control.urls.get_untracked(), ["old", "second", "first"]);
        assert_eq!(control.queued.get_untracked(), 0);
    });
    owner.cleanup();
}

#[test]
fn source_mask_keyboard_deletion_skips_literals_and_placeholder_tails() {
    assert_eq!(
        source_mask_delete("99-99", "12-3_", 5, 5, true),
        Some(("12-__".into(), 3))
    );
    assert_eq!(
        source_mask_delete("99-99", "12-34", 2, 2, true),
        Some(("13-4_".into(), 1))
    );
    assert_eq!(
        source_mask_delete("99-99", "12-34", 0, 2, false),
        Some(("34-__".into(), 0))
    );
    assert_eq!(
        source_mask_delete("99-99", "12-34", 2, 2, false),
        Some(("12-4_".into(), 3))
    );
}

#[test]
fn source_radio_visual_flags_reuse_native_group_and_pinned_check_geometry() {
    let html = render(|| {
        let selected = RwSignal::new(Some("a".into()));
        view!{<SourceRadio id="a" name="choice" value="a" selected check=true square=true rounded=true blue=true orange=true aria_label="Choice"/>}.into_any()
    });
    assert!(html.contains("sp-source-radio-check"));
    assert!(html.contains("sp-source-radio-rounded"));
    assert!(html.contains("sp-source-radio-blue"));
    assert!(!html.contains("sp-source-radio-orange"));
    assert!(html.contains("M20 6 9 17l-5-5"));
    assert!(html.contains("type=\"radio\""));
    assert!(html.contains("aria-label=\"Choice\""));
}

#[test]
fn source_input_appearance_uses_single_native_class_and_style_attributes() {
    let html = render(|| {
        let value = RwSignal::new("person@example.test".into());
        view!{<crate::components::Input input_type="email" id="email" name="email" value class_name="authored" style=Some("margin-top:2px".into())/>}.into_any()
    });
    assert!(html.contains("class=\"sp-input authored\""));
    assert_eq!(html.matches("class=").count(), 1);
    assert_eq!(html.matches("style=").count(), 1);
    assert!(html.contains("type=\"email\""));
}

#[test]
fn source_country_missing_selection_omits_hidden_form_value_but_empty_string_is_present() {
    let absent = render(|| view! {<SourceCountrySelect id="country" name="country"/>}.into_any());
    assert!(!absent.contains("type=\"hidden\""));
    let empty = render(|| {
        view! {<SourceCountrySelect id="country" name="country" initial=Some(String::new())/>}
            .into_any()
    });
    assert!(empty.contains("type=\"hidden\""));
    assert!(empty.contains("value=\"\""));
}

#[test]
fn source_native_input_omits_empty_associations_and_keeps_actual_associations() {
    use crate::components::Input;
    let absent = render(|| {
        view! {<Input input_type="url" id="url" name="url" value=RwSignal::new(String::new()) omit_empty_associations=true/>}.into_any()
    });
    assert!(!absent.contains("aria-describedby="));
    assert!(!absent.contains("aria-invalid="));
    let associated = render(|| {
        view! {<Input id="url" name="url" value=RwSignal::new(String::new()) omit_empty_associations=true described_by="url-help" invalid=true/>}.into_any()
    });
    assert!(associated.contains("aria-describedby=\"url-help\""));
    assert!(associated.contains("aria-invalid=\"true\""));
}
