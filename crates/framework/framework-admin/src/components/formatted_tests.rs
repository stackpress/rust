//! Native controller/bridge assertions use real normalized public metadata.
use super::*;
use serde_json::json;
use stackpress_framework_schema::{FieldAccess, Role, Schema, codegen::DefinitionRegistry};
use stackpress_framework_ui::format::edit::EditSelection;

fn field(kind: &str, selection: &str) -> UiField {
    let source = format!("model Example {{ id String @id value {kind} {selection} }}");
    let parsed =
        stackpress_idea_parser::parse("format.idea", &source, &Default::default()).unwrap();
    let mut registry = DefinitionRegistry::standard();
    for name in ["id", "value"] {
        registry = registry
            .with_field_policy(
                "Example",
                name,
                FieldAccess {
                    readable: true,
                    writable: true,
                    sensitive: false,
                    publish_default: true,
                },
            )
            .unwrap();
    }
    let registry = crate::codegen::register_components(registry, vec![]).unwrap();
    Schema::normalize(&parsed, &registry, Default::default())
        .unwrap()
        .project_ui()
        .unwrap()
        .models
        .remove(0)
        .fields
        .remove(1)
}
#[test]
fn typed_policies_and_wire_boundaries() {
    for (kind, selection, value) in [
        ("String?", "@field.number", json!("-0.00")),
        ("Integer?", "@field.integer", json!(0)),
        ("Number?", "@field.number", json!(12.5)),
    ] {
        let field = field(kind, selection);
        let binding = &field.roles[0];
        configuration(&field, binding).unwrap();
        let canonical = if let Some(value) = value.as_str() {
            value.into()
        } else {
            value.to_string()
        };
        assert_eq!(
            decimal_wire(
                &field.value_type.value,
                DecimalValue::parse(&canonical).unwrap()
            )
            .unwrap(),
            value
        );
    }
    assert!(decimal_wire(&ValueType::Integer, DecimalValue::parse("1.25").unwrap()).is_err());
    assert!(decimal_wire(&ValueType::Boolean, DecimalValue::parse("1").unwrap()).is_err());
    for (kind, text, expected) in [
        (TemporalKind::Date, "2024-02-29", "2024-02-29"),
        (TemporalKind::Time, "12:30:00.000", "12:30:00"),
        (
            TemporalKind::Instant,
            "2024-02-29T12:30:00+02:00",
            "2024-02-29T10:30:00Z",
        ),
    ] {
        assert_eq!(
            temporal_wire(kind, IsoValue::parse(kind, text).unwrap()).unwrap(),
            json!(expected)
        );
    }
    let field = field("String?", "@field.phone({ region \"US\" })");
    let binding = &field.roles[0];
    let policy = phone(&field, binding).unwrap();
    let mut state = EditState::new(policy, None, DisplayLocale::English).unwrap();
    state
        .input("2025550123", EditSelection::end("2025550123"))
        .unwrap();
    assert!(pending_edit(&state, true));
    state.commit(true).unwrap();
    assert_eq!(state.canonical().as_deref(), Some("+12025550123"));
    assert!(!pending_edit(&state, true));
    for partial in ["+33", "garbage"] {
        state.input(partial, EditSelection::end(partial)).unwrap();
        assert!(state.commit(true).is_err());
        assert_eq!(state.canonical().as_deref(), Some("+12025550123"));
        assert!(pending_edit(&state, true));
    }
}
#[test]
fn partials_and_valid_blur_keep_distinct_state() {
    let field = field("Number?", "@field.number");
    let policy = decimal(&field, &field.roles[0]).unwrap();
    let mut state = EditState::new(
        policy,
        Some(DecimalValue::parse("1234.50").unwrap()),
        DisplayLocale::English,
    )
    .unwrap();
    for partial in ["-", "1."] {
        state.input(partial, EditSelection::end(partial)).unwrap();
        assert!(pending_edit(&state, true));
        assert!(state.commit(true).is_err());
        assert_eq!(state.canonical().as_deref(), Some("1234.50"));
        state.cancel();
    }
    state
        .input("1234.50", EditSelection::end("1234.50"))
        .unwrap();
    state.commit_preserving_draft(true).unwrap();
    assert!(state.dirty());
    assert!(!pending_edit(&state, true));
    state.input("9.25", EditSelection::end("9.25")).unwrap();
    assert!(pending_edit(&state, true));
    state.commit(true).unwrap();
    assert!(!pending_edit(&state, true));
}
#[test]
fn temporal_presentation_keeps_exact_commits_and_declared_precision() {
    for (kind, selection, raw, shown) in [
        ("Time?", "@field.time", "12:30:00", "12:30:00"),
        ("Time?", "@field.time", "12:30:00.500000000", "12:30:00.5"),
        (
            "Time?",
            "@field.time({ precision 3 })",
            "12:30:00.5",
            "12:30:00.500",
        ),
        (
            "Datetime?",
            "@field.datetime",
            "2024-02-29T12:30:00.000000000-02:00",
            "2024-02-29T12:30:00-02:00",
        ),
        (
            "Datetime?",
            "@field.datetime({ precision 3 })",
            "2024-02-29T12:30:00-02:00",
            "2024-02-29T12:30:00.000-02:00",
        ),
    ] {
        let field = field(kind, selection);
        let (kind, policy) = temporal(&field, &field.roles[0]).unwrap();
        let value = IsoValue::parse(kind, raw).unwrap();
        assert_eq!(policy.display(&value, DisplayLocale::English), shown);
        assert_eq!(
            policy.canonical(&value),
            raw,
            "presentation must not rewrite storage"
        );
        let mut state = EditState::new(policy, Some(value), DisplayLocale::English).unwrap();
        state.commit(true).unwrap();
        assert_eq!(
            temporal_wire(kind, state.value().unwrap().clone()).unwrap(),
            temporal_wire(kind, IsoValue::parse(kind, raw).unwrap()).unwrap()
        );
    }
    assert_eq!(
        temporal_text("12:30:00.500000000 PM", None),
        "12:30:00.5 PM"
    );
    assert_eq!(
        temporal_text("Feb 29, 2024 · 12:30:00.000000000 PM UTC+02:00", None),
        "Feb 29, 2024 · 12:30:00 PM UTC+02:00"
    );
}
#[test]
fn props_types_and_all_rendered_families() {
    let owner = Owner::new();
    owner.with(|| {
        for (kind, selection, value) in [
            (
                "Number?",
                "@label(\"</script> 🦀\") @field.number @view.number",
                json!(1234.5),
            ),
            (
                "String?",
                "@field.price({ locale \"fr\" }) @view.price({ locale \"fr\" })",
                json!("1234.50"),
            ),
            ("Date?", "@field.date @view.date", json!("2024-02-29")),
            ("Time?", "@field.time @view.date", json!("12:30:00")),
            (
                "Datetime?",
                "@field.datetime @view.date",
                json!("2024-02-29T12:30:00Z"),
            ),
            (
                "String?",
                "@field.phone({ region \"US\" }) @view.phone",
                json!("+12025550123"),
            ),
        ] {
            let field = field(kind, selection);
            let mut context = RoleContext {
                id: "formatted-proof".into(),
                field: field.clone(),
                role: Role::Field,
                value: RwSignal::new(Input::Value(value)),
            };
            let key = field
                .roles
                .iter()
                .find(|b| b.role == Role::Field)
                .unwrap()
                .component
                .clone();
            let html = if key.ends_with("phone") {
                view! {<PhoneField context=context.clone()/>}.to_html()
            } else if matches!(
                field.value_type.value,
                ValueType::Date | ValueType::Time | ValueType::Datetime
            ) {
                view! {<TemporalField context=context.clone()/>}.to_html()
            } else {
                view! {<DecimalField context=context.clone()/>}.to_html()
            };
            assert!(html.contains("formatted-proof-error"));
            assert!(!html.contains("incompatible"));
            assert!(!html.contains("</script>"));
            if field.value_type.value == ValueType::Number {
                assert!(html.contains("&lt;/script&gt;"));
            }
            context.role = Role::View;
            assert!(
                !view! {<FormattedView context=context.clone()/>}
                    .to_html()
                    .contains("incompatible")
            );
            context.value.set(Input::Null);
            assert!(view! {<FormattedView context/>}.to_html().contains('—'));
            let mut binding = field.roles[0].clone();
            binding.props.insert("locale".into(), json!("ambient"));
            assert!(configuration(&field, &binding).is_err());
        }
        let field = field("Integer?", "@field.integer");
        let mut binding = field.roles[0].clone();
        binding.props.insert("scale".into(), json!(1));
        assert!(configuration(&field, &binding).is_err());
        binding.props.insert("scale".into(), json!(-1));
        assert!(configuration(&field, &binding).is_err());
    });
    owner.cleanup();
}
