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

/// Actual catalog normalization carries URL type and both Boolean filter variants.
fn parity_model() -> UiModel {
    let parsed = stackpress_idea_parser::parse("parity.idea", "model Example { id String @id site String @field.url @filter.url active Boolean? @field.switch @filter.switch checked Boolean @filter.checkbox }", &Default::default()).unwrap();
    let mut registry = DefinitionRegistry::standard();
    for name in ["id", "site", "active", "checked"] {
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
    Schema::normalize(&parsed, &registry, Default::default())
        .unwrap()
        .project_ui()
        .unwrap()
        .models
        .remove(0)
}

/// A Boolean filter's hidden zero is reflected in both native markup and its typed draft.
#[test]
fn boolean_filters_have_only_source_controls_and_values() {
    use stackpress_framework_admin::components::{CheckboxField, SwitchField};
    use stackpress_framework_schema::Input;
    for (role, nullable, expected_presence) in [
        (Role::Filter, false, false),
        (Role::Filter, true, false),
        (Role::Field, true, true),
    ] {
        for initial in [
            Input::Missing,
            Input::Null,
            Input::Value(json!(false)),
            Input::Value(json!(true)),
        ] {
            let owner = Owner::new();
            owner.with(|| {
                let mut field = parity_model().fields.remove(2);
                field.nullable = nullable;
                let value = RwSignal::new(initial.clone());
                let context = RoleContext {
                    id: "active".into(),
                    field,
                    role,
                    value,
                };
                let html = view! {<SwitchField context/>}.to_html();
                assert_eq!(html.contains("active-presence"), expected_presence);
                if role == Role::Filter {
                    assert!(html.contains("type=\"hidden\" name=\"active\" value=\"0\""));
                    assert!(html.contains("name=\"active\" value=\"1\""));
                    assert_eq!(
                        value.get_untracked(),
                        Input::Value(json!(initial == Input::Value(json!(true))))
                    );
                } else {
                    assert!(!html.contains("type=\"hidden\""));
                    assert_eq!(value.get_untracked(), initial);
                }
            });
            owner.cleanup();
        }
    }
    let owner = Owner::new();
    owner.with(|| {
        let field = parity_model().fields.remove(3);
        let context = RoleContext {
            id: "checked".into(),
            field,
            role: Role::Filter,
            value: RwSignal::new(Input::Missing),
        };
        let html = view! {<CheckboxField context/>}.to_html();
        assert!(!html.contains("checked-presence"));
        assert!(html.contains("type=\"hidden\" name=\"checked\" value=\"0\""));
        assert!(html.contains("name=\"checked\" value=\"1\""));
    });
    owner.cleanup();
}

/// URL selection reaches the native input through the actual normalized fixed props.
#[test]
fn url_catalog_preserves_native_type_for_field_and_filter() {
    for role in [Role::Field, Role::Filter] {
        let owner = Owner::new();
        owner.with(|| {
            let field = parity_model().fields.remove(1);
            let value = RwSignal::new(stackpress_framework_schema::Input::Value(json!(
                "https://example.invalid/path?a=1&b=2"
            )));
            let context = RoleContext {
                id: "site".into(),
                field,
                role,
                value,
            };
            let html = view! {<TextField context/>}.to_html();
            assert!(html.contains("type=\"url\""));
            assert!(html.contains("https://example.invalid/path?a=1&amp;b=2"));
        });
        owner.cleanup();
    }
}

/// Source checks GET before href selection; public schema keys determine composite paths.
#[test]
fn row_actions_preserve_permission_order_paths_and_optional_href() {
    use stackpress_framework_admin::row_actions::{RowActions, RowDetail, detail_action};
    use std::sync::{Arc, Mutex};
    let row = json!({"tenant":"north", "id":7, "title":"Other"});
    let owner = Owner::new();
    owner.with(|| {
        assert!(
            !detail_action("example", &["id"], &row)
                .to_html()
                .contains("<a")
        )
    });
    owner.cleanup();
    for permitted in [false, true] {
        let owner = Owner::new();
        owner.with(|| {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let capture = calls.clone();
            let can = Callback::new(move |(method, route): (String, String)| {
                capture.lock().unwrap().push(format!("{method} {route}"));
                permitted
            });
            let capture = calls.clone();
            let href = Callback::new(move |row: RowDetail| {
                assert_eq!(row.model, "example");
                assert_eq!(row.key, json!({"id":7,"tenant":"north"}));
                assert_eq!(row.path, "north/7");
                capture.lock().unwrap().push("href".into());
                "detail?key=7&tenant=north".into()
            });
            provide_context(RowActions::new("/admin", can).with_href(href));
            assert!(!detail_action("example", &[], &row).to_html().contains("<a"));
            let html = detail_action("example", &["tenant", "id"], &row).to_html();
            let expected = if permitted {
                vec!["GET /admin/example/detail/north/7", "href"]
            } else {
                vec!["GET /admin/example/detail/north/7"]
            };
            assert_eq!(*calls.lock().unwrap(), expected);
            assert_eq!(html.contains("aria-label=\"View details\""), permitted);
            if permitted {
                assert!(html.contains("href=\"detail?key=7&amp;tenant=north\""));
                assert!(html.contains("aria-hidden=\"true\""));
                assert!(html.contains("m9 18 6-6-6-6"));
            } else {
                assert!(!html.contains("<a"));
            }
        });
        owner.cleanup();
    }
    let owner = Owner::new();
    owner.with(|| {
        provide_context(RowActions::new("", Callback::new(|_| true)));
        let html = detail_action("example", &["tenant", "id"], &row).to_html();
        assert!(html.contains("href=\"detail/north/7\""));
        // Attribute escaping, rather than a new URL admission policy, matches the source.
        let html = detail_action("example", &["id"], &json!({"id":"<\"&"})).to_html();
        assert!(html.contains("href=\"detail/&lt;&quot;&amp;\""));
    });
    owner.cleanup();
}

/// Source base interpolation is literal; joining does not normalize application URLs.
#[test]
fn row_permission_base_is_used_verbatim() {
    use stackpress_framework_admin::row_actions::{RowActions, detail_action};
    let owner = Owner::new();
    owner.with(|| {
        provide_context(RowActions::new(
            "/admin/",
            Callback::new(|(method, route): (String, String)| {
                assert_eq!(method, "GET");
                assert_eq!(route, "/admin//example/detail/x");
                true
            }),
        ));
        assert!(
            detail_action("example", &["id"], &json!({"id":"x"}))
                .to_html()
                .contains("href=\"detail/x\"")
        );
    });
    owner.cleanup();
}

/// Reference JS template interpolation preserves zero, integral reals and exponent boundaries.
#[test]
fn row_action_numbers_use_source_template_spelling() {
    use stackpress_framework_admin::row_actions::{RowActions, detail_action};
    for (value, expected) in [
        (1.0, "1"),
        (-0.0, "0"),
        (1e21, "1e+21"),
        (1e20, "100000000000000000000"),
        (1e-6, "0.000001"),
        (1e-7, "1e-7"),
        (-1e21, "-1e+21"),
        (0.30000000000000004, "0.30000000000000004"),
    ] {
        let owner = Owner::new();
        owner.with(|| {
            let can = Callback::new(move |(method, route): (String, String)| {
                assert_eq!(method, "GET");
                assert_eq!(route, format!("/admin/example/detail/{expected}"));
                true
            });
            provide_context(RowActions::new("/admin", can));
            let html = detail_action("example", &["id"], &json!({"id":value})).to_html();
            assert!(
                html.contains(&format!("href=\"detail/{expected}\"")),
                "{html}"
            );
        });
        owner.cleanup();
    }
}
