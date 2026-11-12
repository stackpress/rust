//! Public source table, transform selection and root-owned presentation contracts.
#![cfg(all(feature = "codegen", feature = "source-ssr"))]

use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_admin::{
    RoleContext, codegen,
    components::TextField,
    pages::{Action, AdminConfig, EditorAction, FormIssues, LayoutAdmin, PageSpec, PendingEdits},
    source::pages::table,
};
use stackpress_framework_schema::{
    FieldIssue, Input, Role, Schema, SchemaError, SchemaErrorKind, SchemaResult,
};
use stackpress_idea_transformer::{OutputPlan, Registry, TransformContext, TransformSpec};

/// Real source metadata includes a hidden-from-list field to check column selection.
fn spec() -> PageSpec {
    let parsed = stackpress_idea_parser::parse(
        "boundaries.idea",
        "model Proof { id String @id @list.string name String @field.input @list.string note String? }",
        &Default::default(),
    )
    .unwrap();
    let graph = Schema::normalize_source(&parsed, Default::default(), &[]).unwrap();
    let mut ui = graph
        .project_source_ui(&codegen::source_definitions().unwrap())
        .unwrap();
    PageSpec {
        module: "proof".into(),
        model: ui.models.remove(0),
        soft_remove: false,
    }
}

/// The callback receives the unwrapped record and a stable root-local row identity.
fn cells(row: &Value, identity: &str) -> SchemaResult<Vec<AnyView>> {
    let text = row
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| SchemaError::new(SchemaErrorKind::Invalid, "test row name missing"))?;
    Ok(vec![
        view! { <span data-instance=identity.to_owned()>{text.to_owned()}</span> }.into_any(),
    ])
}

/// Source tables accept their source envelopes and propagate callback failures unchanged.
#[test]
fn source_table_preserves_rows_columns_and_callback_rejection() {
    let spec = spec();
    for result in [
        json!([{"id":"a","name":"<Ada>"},{"id":"b","name":"Grace"}]),
        json!({"rows":[{"row":{"id":"a","name":"<Ada>"}},{"row":{"id":"b","name":"Grace"}}]}),
    ] {
        let owner = Owner::new();
        let html = owner.with(|| table(&spec, &result, "search", cells).unwrap().to_html());
        for text in ["search-0", "search-1", "&lt;Ada&gt;", "Grace", "name", "id"] {
            assert!(html.contains(text), "missing {text}: {html}");
        }
        assert!(
            !html.contains(">note<"),
            "non-list field became a column: {html}"
        );
        assert!(
            !html.contains("<Ada>"),
            "record text was not escaped: {html}"
        );
        owner.cleanup();
    }
    let owner = Owner::new();
    owner.with(|| {
        for result in [json!(null), json!({"rows":{}}), json!({"rows":null})] {
            let error = table(&spec, &result, "search", cells).err().unwrap();
            assert_eq!(error.to_string(), "source search rows missing");
        }
        let error = table(&spec, &json!([{"id":"a"}]), "search", cells)
            .err()
            .unwrap();
        assert_eq!(error.to_string(), "test row name missing");
        // This portable source adapter must not acquire the Typed adapter's 100-row cap.
        let rows = Value::Array((0..101).map(|n| json!({"id":n,"name":"row"})).collect());
        let html = table(&spec, &rows, "source", cells).unwrap().to_html();
        assert!(html.contains("source-100"));
        assert!(
            table(&spec, &json!([]), "empty", cells)
                .unwrap()
                .to_html()
                .contains("<table")
        );
    });
    owner.cleanup();
}

/// A dependency callback contributes no files, while the selected admin callback owns UI.
fn prerequisite(
    _: &TransformContext<'_>,
    _: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    Ok(())
}
fn emit(
    _: &TransformContext<'_>,
    plan: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    plan.write("ui/selected.rs", "pub const SELECTED: bool = true;\n")
}

/// Role-only and native-page registrations keep distinct, explicit prerequisites.
#[test]
fn registration_requires_selected_dependencies_before_admin() {
    let config = stackpress_idea_parser::parse("empty.idea", "", &Default::default()).unwrap();
    for pages in [false, true] {
        let mut missing = Registry::new();
        if pages {
            codegen::register_pages(&mut missing, emit).unwrap();
        } else {
            codegen::register(&mut missing, emit).unwrap();
        }
        assert!(missing.plan(&config, Vec::new()).is_err());
        let mut selected = Registry::new();
        selected
            .add(TransformSpec::new("framework-schema"), prerequisite)
            .unwrap();
        if pages {
            codegen::register_pages(&mut selected, emit).unwrap();
            assert!(selected.plan(&config, Vec::new()).is_err());
            // Late SQL selection must not silently reorder or satisfy the page dependency.
            selected
                .add(TransformSpec::new("framework-sql"), prerequisite)
                .unwrap();
            assert!(selected.plan(&config, Vec::new()).is_err());
            selected = Registry::new();
            selected
                .add(TransformSpec::new("framework-schema"), prerequisite)
                .unwrap();
            selected
                .add(TransformSpec::new("framework-sql"), prerequisite)
                .unwrap();
            codegen::register_pages(&mut selected, emit).unwrap();
        } else {
            codegen::register(&mut selected, emit).unwrap();
        }
        let plan = selected.plan(&config, Vec::new()).unwrap();
        let expected = if pages {
            vec!["framework-schema", "framework-sql", "framework-admin"]
        } else {
            vec!["framework-schema", "framework-admin"]
        };
        assert_eq!(plan.executed(), expected);
        assert!(
            plan.files()
                .any(|(path, owner, text)| path == "ui/selected.rs"
                    && owner == "framework-admin"
                    && text.contains("SELECTED"))
        );
        assert!(codegen::register(&mut selected, emit).is_err());
        assert!(codegen::register_pages(&mut selected, emit).is_err());
    }
}

/// Checked navigation renders the supplied root's links/content as escaped data.
#[test]
fn layout_and_pending_edits_stay_owned_by_the_current_root() {
    let owner = Owner::new();
    owner.with(|| {
        let pending = PendingEdits::default();
        assert!(!pending.blocked());
        pending.0.update(|fields| { fields.insert("partial".into(), true); fields.insert("valid".into(), false); });
        assert!(pending.blocked());
        pending.0.update(|fields| { fields.remove("partial"); });
        assert!(!pending.blocked());
        let config = AdminConfig::new("/console/", vec!["proof".into(), "other".into()]).unwrap();
        let actions = vec![(Action::Create, "/console/proof/create?next=1&x=2".into())];
        let html = view! { <LayoutAdmin title="<Proof>".to_owned() config actions><p>"root content"</p></LayoutAdmin> }.to_html();
        for text in ["/console/proof/search", "/console/other/search", "data-admin-action=\"create\"", "&lt;Proof&gt;", "root content"] {
            assert!(html.contains(text), "missing {text}: {html}");
        }
        assert!(!html.contains("<Proof>"));
    });
    owner.cleanup();
    for models in [
        vec!["a".repeat(129)],
        vec![String::new()],
        (0..129).map(|n| format!("model{n}")).collect(),
    ] {
        assert!(AdminConfig::new("/", models).is_err());
    }
}

/// Matching public issues stay tied to the submitted version and never leak other fields.
#[test]
fn field_errors_render_matching_paths_and_escape_host_messages() {
    let field = spec().model.fields.remove(1);
    for action in [Action::Search, Action::Update] {
        let owner = Owner::new();
        let html = owner.with(|| {
            provide_context(EditorAction(action));
            provide_context(FormIssues(RwSignal::new(vec![
                FieldIssue {
                    path: vec!["name".into()],
                    code: "required".into(),
                    message: String::new(),
                },
                FieldIssue {
                    path: vec!["other".into()],
                    code: "invalid".into(),
                    message: "OTHER-FIELD".into(),
                },
                FieldIssue {
                    path: vec!["name".into(), "child".into()],
                    code: "invalid".into(),
                    message: "<host message>".into(),
                },
            ])));
            let context = RoleContext {
                id: "name".into(),
                field: field.clone(),
                role: Role::Field,
                value: RwSignal::new(Input::Value(json!("edited after submit"))),
            };
            view! { <TextField context/> }.to_html()
        });
        assert!(html.contains("name is required"), "{html}");
        assert!(html.contains("&lt;host message&gt;"), "{html}");
        assert!(!html.contains("OTHER-FIELD"), "{html}");
        assert!(!html.contains("<host message>"), "{html}");
        assert_eq!(
            html.contains("Submitted version:"),
            action == Action::Update
        );
        owner.cleanup();
    }
}
