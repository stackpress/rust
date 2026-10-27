//! Transition contracts, including failure atomicity and event-order permutations.
use super::*;
use crate::format::{DecimalValue, DisplayLocale, E164Phone};
use std::ops::Bound::{Excluded, Included, Unbounded};

/// A roomy deterministic decimal fixture, without application-specific bounds.
fn decimal() -> DecimalPolicy {
    DecimalPolicy::new(2, 40, Unbounded, Unbounded).unwrap()
}
/// Replace a draft as a native input event with a collapsed end selection.
fn input<P: EditPolicy>(state: &mut EditState<P>, text: &str) {
    state.input(text, EditSelection::end(text)).unwrap();
}

/// Localized grammar, padding, half-even ties and signed zero do not use floats.
#[test]
fn decimal_grammar_and_rounding() {
    for (locale, text, wire, shown) in [
        (DisplayLocale::English, "1,234.125", "1234.12", "1,234.12"),
        (
            DisplayLocale::French,
            "1\u{202f}234,135",
            "1234.14",
            "1\u{202f}234,14",
        ),
        (
            DisplayLocale::French,
            "1\u{00a0}234,5",
            "1234.50",
            "1\u{202f}234,50",
        ),
        (DisplayLocale::English, "+0001.5", "1.50", "1.50"),
        (DisplayLocale::French, "-0,004", "-0.00", "-0,00"),
        (DisplayLocale::English, ".5", "0.50", "0.50"),
        (
            DisplayLocale::English,
            "9007199254740993",
            "9007199254740993.00",
            "9,007,199,254,740,993.00",
        ),
    ] {
        let mut state = EditState::new(decimal(), None, locale).unwrap();
        input(&mut state, text);
        assert!(state.commit(false).unwrap());
        assert_eq!(state.canonical().as_deref(), Some(wire));
        assert_eq!(state.draft(), shown);
        assert!(!state.commit(false).unwrap());
    }
    for (locale, text) in [
        (DisplayLocale::English, "1.234,56"),
        (DisplayLocale::French, "1,234.56"),
        (DisplayLocale::French, "1\u{202f}234\u{00a0}567"),
        (DisplayLocale::English, "1e3"),
        (DisplayLocale::English, "1 234"),
        (DisplayLocale::French, "١٢"),
        (DisplayLocale::English, "--1"),
    ] {
        assert!(decimal().admit(text, locale).is_err(), "{text}");
    }
    for partial in ["-", "+", "1."] {
        let mut state = EditState::new(decimal(), None, DisplayLocale::English).unwrap();
        input(&mut state, partial);
        assert!(state.commit(false).is_err());
        assert_eq!(state.draft(), partial);
        assert_eq!(state.value(), None);
    }
}
/// Required/nullable failures preserve the previous canonical value and invalid draft.
#[test]
fn clear_fail_and_recover() {
    let mut state = EditState::new(
        decimal(),
        Some(DecimalValue::parse("12.50").unwrap()),
        DisplayLocale::English,
    )
    .unwrap();
    input(&mut state, "");
    assert_eq!(state.commit(false), Err(EditError::Required));
    assert_eq!(state.canonical().as_deref(), Some("12.50"));
    assert_eq!(state.draft(), "");
    assert!(state.commit(true).unwrap());
    assert_eq!(state.canonical(), None);
    input(&mut state, "bad");
    assert_eq!(state.error(), Some(EditError::Grammar));
    assert!(state.commit(true).is_err());
    state.cancel();
    assert_eq!(state.draft(), "");
    assert_eq!(state.error(), None);
    assert!(!state.dirty());
    input(&mut state, "0");
    assert!(state.commit(false).unwrap());
    input(&mut state, "-0");
    assert!(state.commit(false).unwrap());
    assert_eq!(state.canonical().as_deref(), Some("-0.00"));
}
/// Exact inclusive/exclusive endpoints, negative magnitudes and configuration checks.
#[test]
fn exact_bounds_and_budgets() {
    let lower = DecimalValue::parse("-10").unwrap();
    let upper = DecimalValue::parse("0").unwrap();
    let policy =
        DecimalPolicy::new(2, 8, Included(lower.clone()), Excluded(upper.clone())).unwrap();
    for (text, ok) in [
        ("-10", true),
        ("-10.01", false),
        ("-0.01", true),
        ("-0", false),
        ("1", false),
    ] {
        assert_eq!(
            policy.parse(text, DisplayLocale::English).is_ok(),
            ok,
            "{text}"
        );
    }
    assert!(DecimalPolicy::new(19, 40, Unbounded, Unbounded).is_err());
    assert!(DecimalPolicy::new(2, 2, Unbounded, Unbounded).is_err());
    assert!(DecimalPolicy::new(0, 97, Unbounded, Unbounded).is_err());
    assert!(DecimalPolicy::new(2, 8, Included(upper.clone()), Included(lower)).is_err());
    assert!(DecimalPolicy::new(2, 8, Excluded(upper.clone()), Included(upper)).is_err());
    let mut state = EditState::new(policy, None, DisplayLocale::English).unwrap();
    input(&mut state, "12345678");
    let before = state.draft().to_owned();
    assert_eq!(
        state.input("123456789", EditSelection::end("123456789")),
        Err(EditError::Limit)
    );
    assert_eq!(state.draft(), before);
    assert_eq!(
        state.paste(&"x".repeat(385), EditSelection::default()),
        Err(EditError::Limit)
    );
    assert_eq!(
        EditState::new(
            decimal(),
            Some(DecimalValue::parse("1.001").unwrap()),
            DisplayLocale::English
        )
        .err(),
        Some(EditError::Precision)
    );
}
/// Draft locale is captured; future presentation changes do not reinterpret decimal marks.
#[test]
fn dirty_locale_and_region_context() {
    let mut state = EditState::new(decimal(), None, DisplayLocale::English).unwrap();
    input(&mut state, "1.25");
    state.set_locale(DisplayLocale::French);
    assert_eq!(state.draft(), "1.25");
    state.commit(false).unwrap();
    assert_eq!(state.draft(), "1,25");
    state.set_locale(DisplayLocale::English);
    assert_eq!(state.draft(), "1.25");
    input(&mut state, "2.");
    assert_eq!(state.set_policy(decimal()), Err(EditError::DirtyContext));
    assert_eq!(state.draft(), "2.");
    state.cancel();
    state.set_policy(decimal()).unwrap();
    assert_eq!(state.error(), None);
}
/// UTF-16 offsets, backward ranges and grouping remapping retain semantic digit positions.
#[test]
fn selection_and_paste() {
    let split = EditSelection {
        start: 1,
        end: 2,
        direction: SelectionDirection::Backward,
    };
    assert_eq!(split.bytes("𝟘"), Err(EditError::Selection));
    assert_eq!(EditSelection::end("𝟘").bytes("𝟘").unwrap(), (4, 4));
    let mut state = EditState::new(decimal(), None, DisplayLocale::English).unwrap();
    input(&mut state, "1234.50");
    state
        .select(EditSelection {
            start: 1,
            end: 4,
            direction: SelectionDirection::Backward,
        })
        .unwrap();
    state.commit(false).unwrap();
    assert_eq!(
        state.selection(),
        EditSelection {
            start: 2,
            end: 5,
            direction: SelectionDirection::Backward
        }
    );
    state.set_locale(DisplayLocale::French);
    assert_eq!(
        state.selection(),
        EditSelection {
            start: 2,
            end: 5,
            direction: SelectionDirection::Backward
        }
    );
    assert_eq!(state.draft(), "1\u{202f}234,50");
    let selection = EditSelection {
        start: 0,
        end: 8,
        direction: SelectionDirection::Backward,
    };
    assert!(state.paste("-0,5", selection).is_ok());
    assert_eq!(state.draft(), "1\u{202f}234,50");
    assert_eq!(state.selection(), selection);
    assert_eq!(state.paste("1\n2", selection), Err(EditError::Grammar));
    assert_eq!(state.paste("1.2,3", selection), Err(EditError::Grammar));
    assert_eq!(state.selection(), selection);
    assert_eq!(
        state.select(EditSelection {
            start: 9,
            end: 1,
            direction: SelectionDirection::Forward
        }),
        Err(EditError::Selection)
    );
}
/// Composition text stays verbatim; blur defers and final-input orders emit at most once.
#[test]
fn composition_orders_cancel_and_duplicate() {
    for final_before_end in [false, true] {
        let mut state = EditState::new(
            decimal(),
            Some(DecimalValue::parse("1.00").unwrap()),
            DisplayLocale::English,
        )
        .unwrap();
        state.composition_start();
        input(&mut state, "一");
        assert_eq!(state.draft(), "一");
        assert_eq!(state.error(), None);
        assert!(!state.commit(false).unwrap());
        if final_before_end {
            input(&mut state, "2.25");
        }
        state.composition_end();
        state.composition_end();
        if !final_before_end {
            input(&mut state, "2.25");
        }
        assert!(state.settle_composition());
        assert!(state.commit(false).unwrap());
        input(&mut state, "2.25");
        assert!(!state.commit(false).unwrap());
        assert!(!state.settle_composition());
        state.composition_start();
        input(&mut state, "雪");
        state.composition_cancel();
        assert_eq!(state.draft(), "2.25");
        assert!(!state.dirty());
        assert!(!state.composing());
        state.composition_start();
        state.composition_end();
        state.composition_start();
        assert!(!state.settle_composition());
        assert!(state.composing());
        state.composition_cancel();
    }
}
/// Calendar/time precision, leap years, offsets and exact instant bound comparisons.
#[test]
fn temporal_values_and_bounds() {
    for (kind, precision, text, expected) in [
        (TemporalKind::Date, 0, "2000-02-29", "2000-02-29"),
        (TemporalKind::Time, 3, "12:30", "12:30:00.000"),
        (TemporalKind::Time, 3, "23:59:59.12", "23:59:59.120"),
        (
            TemporalKind::Instant,
            0,
            "2024-02-29T12:30Z",
            "2024-02-29T12:30:00+00:00",
        ),
        (
            TemporalKind::Instant,
            9,
            "2024-02-29T12:30:00.123456789+14:00",
            "2024-02-29T12:30:00.123456789+14:00",
        ),
    ] {
        let p = TemporalPolicy::new(kind, precision, Unbounded, Unbounded).unwrap();
        let v = p.parse(text, DisplayLocale::French).unwrap();
        assert_eq!(v.as_str(), expected);
    }
    for (kind, text) in [
        (TemporalKind::Date, "1900-02-29"),
        (TemporalKind::Date, "0000-01-01"),
        (TemporalKind::Time, "24:00"),
        (TemporalKind::Time, "12:30:60"),
        (TemporalKind::Instant, "2024-02-29T12:30"),
        (TemporalKind::Instant, "2024-02-29T12:30-00:00"),
        (TemporalKind::Instant, "2024-02-29T12:30+14:01"),
    ] {
        assert!(IsoValue::parse(kind, text).is_err(), "{text}");
    }
    let p = TemporalPolicy::new(TemporalKind::Time, 0, Unbounded, Unbounded).unwrap();
    assert_eq!(
        p.parse("12:30:00.1", DisplayLocale::English),
        Err(EditError::Precision)
    );
    let min = IsoValue::parse(TemporalKind::Instant, "2026-11-01T01:30-04:00").unwrap();
    let max = IsoValue::parse(TemporalKind::Instant, "2026-11-01T01:30-05:00").unwrap();
    let p = TemporalPolicy::new(TemporalKind::Instant, 0, Excluded(min), Included(max)).unwrap();
    assert!(
        p.parse("2026-11-01T05:30Z", DisplayLocale::English)
            .is_err()
    );
    assert!(p.parse("2026-11-01T06:30Z", DisplayLocale::English).is_ok());
    assert!(TemporalPolicy::new(TemporalKind::Date, 1, Unbounded, Unbounded).is_err());
    assert!(TemporalPolicy::new(TemporalKind::Time, 10, Unbounded, Unbounded).is_err());
    for partial in ["2", "2024-", "2024-02-"] {
        let p = TemporalPolicy::new(TemporalKind::Date, 0, Unbounded, Unbounded).unwrap();
        assert!(p.admit(partial, DisplayLocale::English).is_ok());
        let mut state = EditState::new(p, None, DisplayLocale::English).unwrap();
        input(&mut state, partial);
        assert!(state.commit(true).is_err());
        assert_eq!(state.draft(), partial);
    }
}
/// Explicit regions, international paste and extension exclusion preserve canonical E.164.
#[test]
fn phone_drafts_and_switching() {
    for (region, draft, wire) in [
        ("FR", "01 23 45 67 89", "+33123456789"),
        ("US", "(415) 555-2671", "+14155552671"),
        ("FR", "0044 20 7946 0018", "+442079460018"),
        ("US", "+33 1 23 45 67 89", "+33123456789"),
    ] {
        let p = PhonePolicy::new(PhoneRegion::parse(region).unwrap());
        let value = p.parse(draft, DisplayLocale::English).unwrap();
        assert_eq!(value.as_str(), wire);
    }
    let p = PhonePolicy::new(PhoneRegion::parse("FR").unwrap());
    for draft in ["+", "+33"] {
        let mut state = EditState::new(p.clone(), None, DisplayLocale::English).unwrap();
        input(&mut state, draft);
        assert!(state.commit(false).is_err());
        assert_eq!(state.draft(), draft);
    }
    for draft in [
        "+33123456789x2",
        "+33123456789;ext=2",
        "+33123456789 #2",
        "+３３１２３４５６７８９",
    ] {
        assert_eq!(
            p.admit(draft, DisplayLocale::French),
            Err(EditError::Grammar)
        );
    }
    for region in ["fr", "ZZ", "FRA", ""] {
        assert!(PhoneRegion::parse(region).is_err());
    }
    let mut state = EditState::new(p, None, DisplayLocale::English).unwrap();
    input(&mut state, "01");
    assert_eq!(
        state.set_policy(PhonePolicy::new(PhoneRegion::parse("US").unwrap())),
        Err(EditError::DirtyContext)
    );
    state.cancel();
    state
        .set_policy(PhonePolicy::new(PhoneRegion::parse("US").unwrap()))
        .unwrap();
    input(&mut state, "4155552671");
    state.commit(false).unwrap();
    assert_eq!(state.canonical().as_deref(), Some("+14155552671"));
    assert_eq!(
        state.value(),
        Some(&E164Phone::parse("+14155552671").unwrap())
    );
}

/// Native middle edits move old group boundaries; strict clipboard groups remain unambiguous.
#[test]
fn middle_edit_grouping_and_locale_during_composition() {
    let mut state = EditState::new(decimal(), None, DisplayLocale::English).unwrap();
    input(&mut state, "1,2934.12");
    assert_eq!(state.error(), None);
    state.commit(false).unwrap();
    assert_eq!(state.draft(), "12,934.12");
    input(&mut state, "12,34.12");
    state.commit(false).unwrap();
    assert_eq!(state.draft(), "1,234.12");
    assert_eq!(
        state.paste(
            "1,23.45",
            EditSelection {
                start: 0,
                end: 8,
                direction: SelectionDirection::Forward
            }
        ),
        Err(EditError::Grammar)
    );
    state.cancel();
    state.composition_start();
    state.set_locale(DisplayLocale::French);
    input(&mut state, "2.25");
    state.composition_end();
    state.settle_composition();
    state.commit(false).unwrap();
    assert_eq!(state.draft(), "2,25");
    let initial = state.initial().to_owned();
    state.cancel();
    state.adopt("2,25", EditSelection::end("2,25")).unwrap();
    assert!(!state.dirty());
    assert_eq!(state.initial(), initial);
}

/// All three families render native labels/errors and a separate canonical successful field.
#[cfg(feature = "ssr")]
#[test]
fn escaped_ssr_editor_contract() {
    use leptos::prelude::*;
    let owner = Owner::new();
    let html=owner.with(||{
        let state=RwSignal::new(EditState::new(decimal(),Some(DecimalValue::parse("0.00").unwrap()),DisplayLocale::French).unwrap());
        view!{<FormattedInput id="amount" name="amount-value" label="<script>bad</script>" state nullable=true help="Use the explicit locale"/>}.to_html()
    });
    owner.cleanup();
    assert!(html.contains("for=\"amount\""));
    assert!(html.contains("aria-describedby=\"amount-help amount-error\""));
    assert!(html.contains("value=\"0,00\""));
    assert!(html.contains("name=\"amount-value\""));
    assert!(html.contains("value=\"0.00\""));
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>"));
}

/// Blur preserves native text while committing exact rounded value; a new IME owns its blur.
#[test]
fn preserved_draft_and_fresh_composition() {
    let mut state = EditState::new(decimal(), None, DisplayLocale::English).unwrap();
    input(&mut state, "1234.125");
    let selection = state.selection();
    assert!(state.commit_preserving_draft(false).unwrap());
    assert_eq!(state.canonical().as_deref(), Some("1234.12"));
    assert_eq!(state.draft(), "1234.125");
    assert_eq!(state.selection(), selection);
    assert!(state.dirty());
    assert!(!state.commit(false).unwrap());
    assert_eq!(state.draft(), "1,234.12");
    state.composition_start();
    state.commit(false).unwrap();
    state.composition_end();
    state.composition_start();
    state.composition_end();
    assert!(
        !state.settle_composition(),
        "new IME must not inherit the old deferred blur"
    );
    let locale = state.draft_locale();
    state.retain_native("1234.125", EditSelection::end("1234.125"), locale);
    assert_eq!(state.draft(), "1234.125");
    assert_eq!(state.canonical().as_deref(), Some("1234.12"));
}

/// Native snapshot reads and duplicate final events cannot fabricate a dirty draft.
#[test]
fn identical_native_read_keeps_clean_cancellation_base() {
    let p = PhonePolicy::new(PhoneRegion::parse("FR").unwrap());
    let mut state = EditState::new(
        p,
        Some(E164Phone::parse("+33123456789").unwrap()),
        DisplayLocale::English,
    )
    .unwrap();
    let initial = state.draft().to_owned();
    input(&mut state, &initial);
    assert!(!state.dirty());
    state.composition_start();
    input(&mut state, "雪");
    state.composition_cancel();
    assert!(!state.dirty());
    state
        .set_policy(PhonePolicy::new(PhoneRegion::parse("US").unwrap()))
        .unwrap();
    assert_eq!(state.draft(), initial);
}

/// Offset suffix tails must not pass clipboard grammar as if Z were a sign.
#[test]
fn explicit_offset_suffix_grammar() {
    let p = TemporalPolicy::new(TemporalKind::Instant, 0, Unbounded, Unbounded).unwrap();
    for input in [
        "2024-02-29T12:30Z1",
        "2024-02-29T12:30+01-00",
        "2024-02-29T12:30+0100",
    ] {
        assert_eq!(
            p.admit(input, DisplayLocale::English),
            Err(EditError::Grammar)
        );
    }
    assert!(
        p.admit("2024-02-29T12:30+01:", DisplayLocale::English)
            .is_ok()
    );
}

/// Browser transport admits bytes/digits before allocation, without UTF-16 coercion.
#[test]
fn utf16_transport_budgets() {
    use super::transport::admit_utf16;
    for (text, bytes) in [("A", 1), ("é", 2), ("\u{202f}", 3), ("𝟘", 4)] {
        assert!(admit_utf16(text.encode_utf16(), bytes, 1).is_ok());
        assert_eq!(
            admit_utf16(text.encode_utf16(), bytes - 1, 1),
            Err(EditError::Limit)
        );
    }
    assert!(admit_utf16("123".encode_utf16(), 3, 3).is_ok());
    assert_eq!(
        admit_utf16("1234".encode_utf16(), 4, 3),
        Err(EditError::Limit)
    );
    for units in [vec![0xd800], vec![0xdc00], vec![0xd800, u16::from(b'A')]] {
        assert_eq!(
            admit_utf16(units.into_iter(), 8, 8),
            Err(EditError::Grammar)
        );
    }
    let mut state = EditState::new(decimal(), None, DisplayLocale::French).unwrap();
    assert_eq!(state.transport_budget(), (384, 40));
    input(&mut state, "1,5");
    state.reject_native(EditError::Limit, EditSelection::end("1,5"));
    assert_eq!(state.draft(), "1,5");
    assert_eq!(state.error(), Some(EditError::Limit));
    assert_eq!(state.canonical(), None);
}
