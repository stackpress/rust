//! Public descriptor wording, readable links and context-sensitive presence controls.
#![cfg(all(feature = "codegen", feature = "ssr"))]
use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::{
    Draft, RoleContext,
    components::TextField,
    pages::{Action, EditorAction},
    presentation::{issue_message, record_label, submitted_issues},
};
use stackpress_framework_schema::{
    FieldAccess, FieldIssue, Role, Schema, UiModel,
    codegen::{DefinitionRegistry, NormalizeOptions},
};

fn model() -> UiModel {
    let source = "model Example { id String @id @field.input name String @label(\"Name\") @field.input @filter.input @is.ne note String? @field.input @is.cle(10) custom String @is.ne(\"Please provide a custom value\") @field.input }";
    let parsed =
        stackpress_idea_parser::parse("fixture.idea", source, &Default::default()).unwrap();
    let mut registry = DefinitionRegistry::standard();
    for name in ["id", "name", "note", "custom"] {
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
    let registry =
        stackpress_framework_admin::codegen::register_components(registry, vec![]).unwrap();
    Schema::normalize(&parsed, &registry, NormalizeOptions::default())
        .unwrap()
        .project_ui()
        .unwrap()
        .models
        .remove(0)
}
#[test]
fn public_wording_and_labels() {
    let model = model();
    let issue = |code: &str| FieldIssue {
        path: vec!["name".into()],
        code: code.into(),
        message: "field validation failed".into(),
    };
    assert_eq!(
        issue_message(&model.fields[1], &issue("required")),
        "Name is required"
    );
    assert_eq!(
        issue_message(&model.fields[2], &issue("cle")),
        "note must contain at most 10 characters"
    );
    assert_eq!(
        issue_message(&model.fields[3], &issue("ne")),
        "Please provide a custom value"
    );
    assert_eq!(record_label(&model, &json!({"id":"x","name":"Ada"})), "Ada");
    assert_eq!(record_label(&model, &json!({"id":"x"})), "View x");
    assert_eq!(record_label(&model, &json!({})), "View record");
    let mut invalid = issue("invalid");
    let specific = submitted_issues(&model, vec![invalid.clone()], &json!({"name":""}));
    assert_eq!(specific[0].message, "Name is required");
    invalid.path = vec!["note".into()];
    let specific = submitted_issues(
        &model,
        vec![invalid.clone()],
        &json!({"note":"too many characters"}),
    );
    assert_eq!(
        specific[0].message,
        "note must contain at most 10 characters"
    );
    invalid.path = vec!["custom".into()];
    let specific = submitted_issues(&model, vec![invalid.clone()], &json!({"custom":""}));
    assert_eq!(specific[0].message, "Please provide a custom value");
    invalid.path = vec!["name".into(), "unknown".into()];
    assert_eq!(
        submitted_issues(&model, vec![invalid], &json!({"name":""}))[0].code,
        "invalid"
    );
}
#[test]
fn presence_is_only_offered_when_meaningful() {
    for (action, role, nullable, shown, label) in [
        (Action::Create, Role::Field, false, false, ""),
        (Action::Copy, Role::Field, false, false, ""),
        (Action::Update, Role::Field, false, true, "Leave unchanged"),
        (Action::Create, Role::Filter, false, true, "Any value"),
        (Action::Create, Role::Field, true, true, "Clear (empty)"),
    ] {
        let owner = Owner::new();
        owner.with(|| {
            provide_context(EditorAction(action));
            let model = model();
            let field = model.fields[if nullable { 2 } else { 1 }].clone();
            let draft = Draft::new(&model.fields, &json!({}), role).unwrap();
            let context = RoleContext {
                id: "presence-proof".into(),
                value: draft.get(&field.wire_name).unwrap(),
                field,
                role,
            };
            let html = view! {<TextField context/>}.to_html();
            assert_eq!(html.contains("presence-proof-presence"), shown);
            assert!(!html.contains("value state"));
            assert!(!html.contains(">Omitted<"));
            if shown {
                assert!(html.contains(label), "{html}");
            }
        });
        owner.cleanup();
    }
}
