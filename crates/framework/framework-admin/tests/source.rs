//! Source-native UI uses the full composed graph, without Typed DTO or policy adapters.
#![cfg(all(feature = "codegen", feature = "source-ssr"))]
use stackpress_framework_admin::codegen::{emit_source_pages, source_definitions};
use stackpress_framework_schema::{
    Role, Schema,
    codegen::{NormalizeOptions, transform_error},
};
use stackpress_idea_transformer::{
    Composer, MemoryLoader, OutputPlan, PackageLoader, Registry, TransformContext, TransformSpec,
};

#[path = "source/editor.rs"]
mod editor;
#[path = "source/remaining.rs"]
mod remaining;

fn config() -> stackpress_idea_parser::SchemaConfig {
    let mut app = MemoryLoader::new("/source-admin").unwrap();
    app.insert("schema.idea","use \"stackpress-api/schema.idea\"\nmodel Profile { nickname String? @default(\"Ada\") @field.input @list.string @view.string @filter.input }");
    let loader = PackageLoader::new(app)
        .with_source(
            "stackpress-session/schema.idea",
            include_str!("../../framework-session/schema.idea"),
        )
        .unwrap()
        .with_source(
            "stackpress-api/schema.idea",
            include_str!("../../framework-api/schema.idea"),
        )
        .unwrap();
    Composer::new(loader, Default::default())
        .compose("schema.idea")
        .unwrap()
        .config
}
fn emit(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let graph = Schema::normalize_source(context.schema(), NormalizeOptions::default(), &[])
        .map_err(transform_error)?;
    let definitions = source_definitions().map_err(transform_error)?;
    let ui = graph
        .project_source_ui(&definitions)
        .map_err(transform_error)?;
    emit_source_pages(&graph, &ui, &definitions, output)
}
#[test]
#[cfg(feature = "format")]
fn complete_shipped_source_ui_and_extension_emit_without_typed_projection() {
    let config = config();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let definitions = source_definitions().unwrap();
    let ui = graph.project_source_ui(&definitions).unwrap();
    assert_eq!(
        ui.models.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(),
        ["Profile", "Application", "Session", "Auth"]
    );
    let profile = &ui.models[0];
    let extension = profile.fields.last().unwrap();
    assert_eq!(extension.wire_name, "nickname");
    assert!(extension.roles.iter().any(|role| role.role == Role::Field));
    assert!(
        ui.models
            .iter()
            .flat_map(|model| &model.fields)
            .flat_map(|field| &field.roles)
            .all(|role| role.available)
    );
    let json = serde_json::to_string(&ui).unwrap();
    for private in [
        "encrypted",
        "hashed",
        "cuid()",
        "database-seed",
        "is.required",
        "documentation",
        "examples",
    ] {
        assert!(!json.contains(private), "{private}");
    }
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("source-ui").owns(&["ui"]), emit)
        .unwrap();
    let plan = registry.plan(&config, Vec::new()).unwrap();
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = dir.path().join("generated");
    let workspace = stackpress_idea_transformer::OutputWorkspace::new(&root).unwrap();
    workspace.install(plan).unwrap();
    let profile = std::fs::read_to_string(root.join("ui/profile.rs")).unwrap();
    assert!(profile.contains("nickname"));
    assert!(!profile.contains("contracts::public"));
    assert!(profile.contains("stackpress_framework_schema::__private::serde_json"));
    let pages = std::fs::read_to_string(root.join("ui/pages.rs")).unwrap();
    assert!(pages.contains("\"profile\""));
    let host = std::fs::read_to_string(root.join("ui/host.rs")).unwrap();
    assert!(host.contains("\"plural\":\"Profiles\""));
    assert!(host.contains("\"icon\":\"user\""));
    assert!(!host.contains("@encrypted"));
    assert!(
        workspace
            .check(&registry.plan(&config, Vec::new()).unwrap())
            .unwrap()
            .is_current()
    );
}

#[test]
#[cfg(not(feature = "format"))]
fn source_unselected_format_roles_remain_metadata_only() {
    let config = config();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    assert!(
        ui.models
            .iter()
            .flat_map(|model| &model.fields)
            .flat_map(|field| &field.roles)
            .any(|role| !role.available)
    );
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("source-ui").owns(&["ui"]), emit)
        .unwrap();
    assert!(registry.plan(&config, Vec::new()).is_err());
}

#[test]
fn source_native_date_keeps_local_time_in_fresh_processes() {
    // Local-time selection is process-owned. Do not mutate TZ while other tests run.
    for (timezone, expected) in [
        ("UTC", "0 4, 1970 12:iAM"),
        ("Pacific/Honolulu", "0 3, 1969 2:iPM"),
    ] {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "source_native_date_child",
                "--ignored",
                "--nocapture",
            ])
            .env("TZ", timezone)
            .env("SOURCE_EXPECTED_DATE", expected)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stdout)
        );
    }
}

#[test]
#[ignore = "fresh process with explicit local-time environment"]
fn source_native_date_child() {
    use leptos::prelude::*;
    use stackpress_framework_schema::Input;
    let graph = Schema::normalize_source(&config(), Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let field = ui.models[0]
        .fields
        .iter()
        .find(|field| field.wire_name == "created")
        .unwrap()
        .clone();
    let owner = Owner::new();
    let html = owner.with(|| {
        let context = stackpress_framework_admin::RoleContext {
            id: "date".into(),
            field,
            role: Role::View,
            value: RwSignal::new(Input::Value(serde_json::json!("1970-01-01T00:00:00Z"))),
        };
        view! {<stackpress_framework_admin::source::roles::DateView context/>}.to_html()
    });
    assert!(
        html.contains(&std::env::var("SOURCE_EXPECTED_DATE").unwrap()),
        "{html}"
    );
    owner.cleanup();
}

#[test]
fn source_projection_requires_explicit_compiled_role_selection() {
    let graph = Schema::normalize_source(&config(), Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&stackpress_framework_schema::codegen::source_definitions().unwrap())
        .unwrap();
    assert!(
        ui.models[0]
            .fields
            .iter()
            .flat_map(|field| &field.roles)
            .any(|role| !role.available)
    );
    // Native source @secret supplies a password editor, not a read/list role or provider.
    let secret = ui
        .models
        .iter()
        .find(|model| model.id == "Auth")
        .unwrap()
        .fields
        .iter()
        .find(|field| field.wire_name == "secret")
        .unwrap();
    assert!(secret.writable);
    assert!(!secret.readable);
}

#[test]
fn source_select_uses_authored_option_order_with_the_shared_control() {
    use leptos::prelude::*;
    use stackpress_framework_schema::Input;
    let config=stackpress_idea_parser::parse("select.idea",r#"enum State { A "A" B "B" C "C" } model Article { status State @field.select({ options ["B" "A"] }) }"#,&Default::default()).unwrap();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let field = ui.models[0].fields[0].clone();
    let owner = Owner::new();
    let html = owner.with(|| {
        let context = stackpress_framework_admin::RoleContext {
            id: "status".into(),
            field,
            role: Role::Field,
            value: RwSignal::new(Input::Value(serde_json::json!("A"))),
        };
        view! {<stackpress_framework_admin::source::roles::SelectField context/>}.to_html()
    });
    assert!(html.find("value=\"B\"").unwrap() < html.find("value=\"A\"").unwrap());
    assert!(!html.contains("value=\"C\""));
    owner.cleanup();
}

#[test]
fn source_declared_but_unimplemented_adapter_props_are_not_silently_ignored() {
    for source in [
        "model Article { status String @field.select({ options [\"A\"] className \"source-style\" }) }",
        "model Article { created Datetime @view.date(\"YYYY\") }",
        "model Article { metadata Hash @view.metadata({ style { color \"red\" } }) }",
    ] {
        let config =
            stackpress_idea_parser::parse("unavailable.idea", source, &Default::default()).unwrap();
        // Metadata acceptance does not promise an application styling adapter.
        let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
        assert!(
            graph
                .project_source_ui(&source_definitions().unwrap())
                .is_ok()
        );
        let mut registry = Registry::new();
        registry
            .add(TransformSpec::new("source-ui").owns(&["ui"]), emit)
            .unwrap();
        assert!(registry.plan(&config, Vec::new()).is_err());
    }
}

/// Build the actual source role context; test data never changes generated expectations.
fn display(source: &str, value: serde_json::Value, metadata: bool) -> String {
    use leptos::prelude::*;
    use stackpress_framework_schema::Input;
    let config =
        stackpress_idea_parser::parse("display.idea", source, &Default::default()).unwrap();
    let graph = Schema::normalize_source(&config, Default::default(), &[]).unwrap();
    let ui = graph
        .project_source_ui(&source_definitions().unwrap())
        .unwrap();
    let owner = Owner::new();
    let html = owner.with(|| {
        let context = stackpress_framework_admin::RoleContext {
            id: "display".into(),
            field: ui.models[0].fields[0].clone(),
            role: Role::View,
            value: RwSignal::new(Input::Value(value)),
        };
        if metadata {
            view! {<stackpress_framework_admin::source::roles::MetadataView context/>}.to_html()
        } else {
            view! {<stackpress_framework_admin::source::roles::OverflowView context/>}.to_html()
        }
    });
    owner.cleanup();
    html
}

#[test]
fn source_overflow_matches_frui_count_slice_words_and_ellipsis_boundaries() {
    // Frui src/view/TextOverflow.tsx:21-49: Number(length), truthiness and JS slice.
    // In particular, words splits on a literal space and length counts UTF-16 units.
    for (props, text, expected) in [
        ("", "abcdef", "abcdef"),
        ("(0)", "abcdef", "abcdef"),
        ("(\"invalid\")", "abcdef", "abcdef"),
        ("(\"Infinity\")", "abcdef", "abcdef"),
        ("(\"0x2\" false true)", "abcdef", "ab…"),
        ("(-1.9 false true)", "abcdef", "abcde…"),
        ("(3 false true)", "A🦀BC", "A🦀…"),
        ("(2 false true)", "A🦀BC", "A�…"),
        ("(3 true true)", "one  two three", "one  two…"),
        ("(2 true false)", "one two", "one two"),
        ("(-2 true true)", "one two three", "one…"),
    ] {
        let source = format!("model Article {{ text String @view.overflow{props} }}");
        assert_eq!(
            display(&source, serde_json::json!(text), false),
            expected,
            "{props}"
        );
    }
    assert_eq!(
        display(
            "model Article { text String @view.clip(3) }",
            serde_json::json!("<x>"),
            false
        ),
        "&lt;x&gt;"
    );
}

#[test]
fn source_metadata_keeps_two_column_order_escaping_and_primitive_child_semantics() {
    // Frui src/view/Metadata.tsx:23-33: Object.entries with a class on both cells.
    let source = "model Article { metadata Hash @view.metadata({ className \"source-cell\" }) }";
    let html = display(
        source,
        serde_json::json!({"z":"<script>","a":2,"empty":false,"items":["x",0,null,true]}),
        true,
    );
    assert!(html.starts_with("<table><tbody>"), "{html}");
    assert!(
        html.find(">z<").unwrap() < html.find(">a<").unwrap(),
        "{html}"
    );
    assert_eq!(html.matches("class=\"source-cell\"").count(), 8);
    assert!(html.contains("&lt;script&gt;"), "{html}");
    assert!(!html.contains("<script>"));
    assert!(html.contains(">2</td>"));
    assert!(html.contains(">x0</td>"));
    assert!(!html.contains("false") && !html.contains("null") && !html.contains("true"));
    let array = display(source, serde_json::json!(["first", "second"]), true);
    assert!(array.find(">0<").unwrap() < array.find(">1<").unwrap());
    let text = display(source, serde_json::json!("ab"), true);
    assert!(text.contains(">a</td>") && text.contains(">b</td>"));
    // Leptos may retain an empty hydration marker; no rendered table or data remains.
    assert!(!display(source, serde_json::Value::Null, true).contains("<table"));
    let empty = display(source, serde_json::json!(4), true);
    assert!(empty.starts_with("<table><tbody>") && empty.ends_with("</tbody></table>"));
    assert!(!empty.contains("<tr>"));
}

#[test]
#[should_panic(expected = "source metadata object is not a renderable child")]
fn source_metadata_does_not_invent_a_display_for_non_renderable_object_children() {
    let _ = display(
        "model Article { metadata Hash @view.metadata }",
        serde_json::json!({"object":{"x":1}}),
        true,
    );
}

#[test]
fn source_collection_labels_forward_authored_props_and_preserve_default_markup() {
    use leptos::prelude::*;
    use stackpress_framework_schema::Input;
    use stackpress_leptos::components::{BoundedList, MetadataInput, TextList};
    // Same IDs, draft, callbacks and ARIA paths: an explicit old label must render
    // byte-identically to an omitted prop, and custom labels change only that text.
    let owner = Owner::new();
    owner.with(|| {
        let text = RwSignal::new(BoundedList::<String>::new(Vec::new(), 100).unwrap());
        let metadata = RwSignal::new(BoundedList::new(Vec::new(), 100).unwrap());
        let old = view! {<TextList id="roles" label="Roles" list=text/>}.to_html();
        let explicit = view! {<TextList id="roles" label="Roles" list=text add_label="Add row"/>}.to_html();
        let custom = view! {<TextList id="roles" label="Roles" list=text add_label="Add Role"/>}.to_html();
        assert_eq!(old, explicit);
        assert_eq!(old, custom.replace("Add Role", "Add row"));
        let old = view! {<MetadataInput id="references" label="References" list=metadata/>}.to_html();
        let explicit = view! {<MetadataInput id="references" label="References" list=metadata add_label="Add row"/>}.to_html();
        let custom = view! {<MetadataInput id="references" label="References" list=metadata add_label="Add Reference"/>}.to_html();
        assert_eq!(old, explicit);
        assert_eq!(old, custom.replace("Add Reference", "Add row"));
        // Real shipped role bindings, rather than a fabricated catalog, reach both adapters.
        let graph = Schema::normalize_source(&config(), Default::default(), &[]).unwrap();
        let ui = graph.project_source_ui(&source_definitions().unwrap()).unwrap();
        for (name, label, value) in [("roles", "Add Role", serde_json::json!([])), ("references", "Add Reference", serde_json::json!({}))] {
            let context = stackpress_framework_admin::RoleContext {
                id: name.into(),
                field: ui.models[0].fields.iter().find(|field| field.wire_name == name).unwrap().clone(),
                role: Role::Field,
                value: RwSignal::new(Input::Value(value)),
            };
            let html = if name == "roles" {
                view! {<stackpress_framework_admin::source::roles::TextListField context/>}.to_html()
            } else {
                view! {<stackpress_framework_admin::source::roles::MetadataField context/>}.to_html()
            };
            assert!(html.contains(&format!(">{label}</button>")), "{html}");
        }
    });
    owner.cleanup();
}

#[test]
fn source_metadata_field_projects_original_string_entries_without_mutating_draft() {
    use leptos::prelude::*;
    use stackpress_framework_schema::Input;
    // stackpress-view/src/transform/form/field.ts:139–141 uses String(val),
    // independently of the detail view's refusal to render a nested object child.
    let owner = Owner::new();
    owner.with(|| {
        let graph = Schema::normalize_source(&config(), Default::default(), &[]).unwrap();
        let ui = graph
            .project_source_ui(&source_definitions().unwrap())
            .unwrap();
        let original = Input::Value(serde_json::json!({
            "number":1.25,"flag":true,"nothing":null,"list":["a",null,2],"object":{"x":1}
        }));
        let draft = RwSignal::new(original.clone());
        let context = stackpress_framework_admin::RoleContext {
            id: "source-metadata".into(),
            field: ui.models[0]
                .fields
                .iter()
                .find(|field| field.wire_name == "references")
                .unwrap()
                .clone(),
            role: Role::Field,
            value: draft,
        };
        let html =
            view! {<stackpress_framework_admin::source::roles::MetadataField context/>}.to_html();
        for text in ["1.25", "true", "null", "a,,2", "[object Object]"] {
            assert!(
                html.contains(&format!("value=\"{text}\"")),
                "{text}: {html}"
            );
        }
        assert!(html.contains(">Add Reference</button>"));
        assert_eq!(
            draft.get_untracked(),
            original,
            "presentation is not a data mutation"
        );
    });
    owner.cleanup();
}
