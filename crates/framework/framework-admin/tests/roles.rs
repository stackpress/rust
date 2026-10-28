//! Frozen schema normalization, source diagnostics and native public-state regression matrix.
#![cfg(all(feature = "codegen", feature = "ssr"))]

use leptos::prelude::*;
use serde_json::json;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{
    FieldAccess, Input, Role, Schema, SchemaError, UiSchema,
    codegen::{DefinitionRegistry, DiagnosticMap, NormalizeOptions},
};

#[cfg(feature = "codegen")]
mod generation {
    use super::*;
    use stackpress_framework_admin::codegen::{components, emit_roles, register_components};
    use stackpress_idea_transformer::{
        Composer, MemoryLoader, OutputPlan, Registry, TransformContext, TransformSpec,
    };

    /// Every fixture uses the real parser/composer and a deny-by-default app policy.
    fn fixture(
        source: &str,
    ) -> (
        stackpress_idea_transformer::ComposedSchema,
        DiagnosticMap,
        DefinitionRegistry,
    ) {
        let mut loader = MemoryLoader::new("/admin").unwrap();
        loader.insert("fixture.idea", source);
        let composed = Composer::new(loader, Default::default())
            .compose("fixture.idea")
            .unwrap();
        let diagnostics = DiagnosticMap::from_composed(&composed).unwrap();
        let mut definitions = DefinitionRegistry::standard();
        for name in ["id", "name", "flag", "count", "choice", "notes"] {
            definitions = definitions
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
        (
            composed,
            diagnostics,
            register_components(definitions, Vec::new()).unwrap(),
        )
    }

    /// Normalize and project actual revision-3 metadata with original source coordinates.
    pub(super) fn normalize(source: &str) -> Result<UiSchema, SchemaError> {
        let (composed, diagnostics, definitions) = fixture(source);
        Schema::normalize_with_diagnostics(
            &composed.config,
            &definitions,
            NormalizeOptions::default(),
            &diagnostics,
        )?
        .project_ui()
    }

    /// Errors must resolve to the original attribute rather than a guessed offset.
    #[test]
    fn unknown_props_components_alias_collision_and_coordinates() {
        for attribute in [
            "@field.unknown",
            "@field.input({ unknown true })",
            "@field.input({ disabled 0 })",
            "@field.string @field.input",
        ] {
            let source =
                format!("// UTF-8: 🦀\nmodel Example {{ id String @id name String {attribute} }}");
            let error = normalize(&source).unwrap_err();
            let location = error.location().unwrap();
            let (start, end) = location.bytes.unwrap();
            assert!(
                source[start..end].contains("field."),
                "{source:?} {location:?}"
            );
            assert_eq!(location.path[1], "Example");
            assert_eq!(error.path()[3], "name");
        }
        let ui = normalize("model Example { id String @id name String @field.string @filter.string @span.string @list.string @view.string }").unwrap();
        let roles = &ui.models[0]
            .fields
            .iter()
            .find(|field| field.wire_name == "name")
            .unwrap()
            .roles;
        assert_eq!(
            roles.iter().map(|binding| binding.role).collect::<Vec<_>>(),
            [
                Role::Field,
                Role::Filter,
                Role::List,
                Role::Span,
                Role::View
            ]
        );
        assert!(roles.iter().all(|binding| binding.available));
    }

    /// A real compiled replacement's fixed values override supplied props, including false.
    #[test]
    fn fixed_precedence_and_registry_rejections() {
        let mut component = components().remove(0);
        component
            .definition
            .fixed
            .insert("placeholder".into(), "fixed \"🦀\"".into());
        component
            .definition
            .fixed
            .insert("disabled".into(), false.into());
        let source = "model Example { id String @id name String @field.string({ placeholder \"idea\" disabled true }) }";
        let (composed, _, _) = fixture(source);
        let registry = register_components(
            DefinitionRegistry::standard()
                .with_field_policy(
                    "Example",
                    "name",
                    FieldAccess {
                        readable: true,
                        writable: true,
                        ..Default::default()
                    },
                )
                .unwrap(),
            vec![component.clone()],
        )
        .unwrap();
        let ui = Schema::normalize(&composed.config, &registry, Default::default())
            .unwrap()
            .project_ui()
            .unwrap();
        let props = &ui.models[0].fields[0].roles[0].props;
        assert_eq!(props["placeholder"], "fixed \"🦀\"");
        assert_eq!(props["disabled"], false);
        assert!(
            register_components(
                DefinitionRegistry::standard(),
                vec![component.clone(), component.clone()]
            )
            .is_err()
        );
        for module in ["crate; panic!()", "x::\"bad", "../escape"] {
            component.import.module = module.into();
            assert!(
                register_components(DefinitionRegistry::standard(), vec![component.clone()])
                    .is_err()
            );
        }
    }

    /// Run the real function transform with a declared catalog and ownership grant.
    fn plan(
        context: &TransformContext<'_>,
        output: &mut OutputPlan,
    ) -> stackpress_idea_transformer::Result<()> {
        let ui: UiSchema = serde_json::from_str(context.input("ui.json")?).unwrap();
        let symbol = context.input("symbol.txt")?;
        let module = context.input("module.txt")?;
        let mut input = components().remove(0);
        input.import.symbol = symbol.into();
        input.import.module = module.into();
        let definitions = register_components(DefinitionRegistry::standard(), vec![input]).unwrap();
        let diagnostics: DiagnosticMap =
            serde_json::from_str(context.input("diagnostics.json")?).unwrap();
        emit_roles(&ui, &definitions, &diagnostics, output)
    }

    /// Shared writer output is deterministic, escaped, independent per-role and public only.
    #[test]
    fn deterministic_escaped_public_wrappers_and_import_diagnostics() {
        let original = "model Example { id String @id name String @label(\"</script> 🦀\") @field.string @list.text secret String @field.input }";
        let (_, diagnostics, _) = fixture(original);
        let mut ui = normalize(original).unwrap();
        ui.models[0].fields[1]
            .label
            .push_str("\"; panic!(\"injection\"); //\n");
        assert!(
            !ui.models[0]
                .fields
                .iter()
                .any(|field| field.wire_name == "secret")
        );
        let mut sensitive = ui.models[0].fields[1].clone();
        sensitive.wire_name = "sensitive_sentinel".into();
        sensitive.sensitive = true;
        ui.models[0].fields.push(sensitive);
        let generate = |ui: &UiSchema, module: &str, symbol: &str| {
            let mut inputs = stackpress_idea_transformer::TransformInputs::new();
            inputs
                .insert("ui.json", serde_json::to_string(ui).unwrap())
                .unwrap();
            inputs
                .insert(
                    "diagnostics.json",
                    serde_json::to_string(&diagnostics).unwrap(),
                )
                .unwrap();
            inputs.insert("symbol.txt", symbol).unwrap();
            inputs.insert("module.txt", module).unwrap();
            let mut registry = Registry::new();
            registry
                .add(TransformSpec::new("ui").owns(&["ui"]), plan)
                .unwrap();
            registry.plan_with(
                &Default::default(),
                &stackpress_idea_transformer::TransformSelection::new().with_inputs(inputs),
                vec![],
            )
        };
        let first = generate(&ui, "stackpress_framework_admin::components", "TextField").unwrap();
        let second = generate(&ui, "stackpress_framework_admin::components", "TextField").unwrap();
        assert_eq!(
            first.files().collect::<Vec<_>>(),
            second.files().collect::<Vec<_>>()
        );
        let text = first
            .files()
            .find(|(path, _, _)| *path == "ui/example.rs")
            .unwrap()
            .2;
        assert!(text.contains("pub fn field("));
        assert!(text.contains("pub fn list("));
        assert!(!text.contains("pub fn span("));
        assert!(!text.contains("pub fn filter("));
        assert!(!text.contains("sensitive_sentinel"));
        assert!(!text.contains("secret"));
        assert!(!text.contains("fixture.idea"));
        assert!(text.contains("🦀"));
        for (module, symbol) in [
            ("fn", "TextField"),
            ("safe", "fn"),
            ("safe", "UnknownAdapter"),
        ] {
            let result = generate(&ui, module, symbol);
            if symbol == "UnknownAdapter" {
                assert!(result.is_ok());
                continue;
            }
            let error = result.unwrap_err();
            let mut source: &dyn std::error::Error = &error;
            let cause = loop {
                if let Some(cause) = source.downcast_ref::<SchemaError>() {
                    break cause;
                }
                source = source
                    .source()
                    .expect("registry must preserve the concrete schema cause");
            };
            let (start, end) = cause.location().unwrap().bytes.unwrap();
            assert!(original[start..end].contains("field.string"));
            assert_eq!(cause.path().last().unwrap(), "field.string");
        }
        // An old projected descriptor cannot mix with the selected revision-3 library.
        assert_eq!(ui.contract_revision, 3);
        for revision in [2, 4] {
            ui.contract_revision = revision;
            assert!(generate(&ui, "safe", "Adapter").is_err());
        }
        ui.contract_revision = 3;
        assert!(generate(&ui, "safe", "Adapter").is_ok());
        let mut relation = ui.clone();
        relation.models[0].fields[1].value_type.value =
            stackpress_framework_schema::ValueType::Model("Other".into());
        relation.models[0].fields[1]
            .roles
            .retain(|binding| binding.role == Role::List);
        assert!(
            generate(
                &relation,
                "stackpress_framework_admin::components",
                "TextField"
            )
            .is_err(),
            "virtual relations have no Public DTO values and must not render blank"
        );
        let duplicate = ui.models[0].fields[1].roles[0].clone();
        ui.models[0].fields[1].roles.push(duplicate);
        assert!(generate(&ui, "safe", "Adapter").is_err());
    }

    /// Unavailable rich editors and incompatible native props refuse with field source positions.
    #[test]
    fn incompatible_control_and_json_path_diagnostics() {
        let source = "model Example { id String @id name String @field.code }";
        let (mut composed, map, definitions) = fixture(source);
        let schema = Schema::normalize(&composed.config, &definitions, Default::default()).unwrap();
        assert!(schema.require_available_roles().is_err());
        assert!(
            map.entries
                .iter()
                .any(|entry| entry.path.last().is_some_and(|part| part == "field.code"))
        );
        composed.config.model["Example"].columns[1]
            .attributes
            .insert(
                "field.input".into(),
                stackpress_idea_parser::AttributeValue::Array(vec![json!({"unknown":true})]),
            );
        let mut loader = MemoryLoader::new("/admin").unwrap();
        loader.insert(
            "source.json",
            serde_json::to_string(&composed.config).unwrap(),
        );
        let json = Composer::new(loader, Default::default())
            .compose("source.json")
            .unwrap();
        let diagnostics = DiagnosticMap::from_composed(&json).unwrap();
        let path = vec![
            "model".into(),
            "Example".into(),
            "columns".into(),
            "name".into(),
        ];
        let location = diagnostics.resolve(&path).unwrap();
        assert!(location.bytes.is_none());
        assert!(location.source.ends_with("source.json"));
        let error = Schema::normalize_with_diagnostics(
            &json.config,
            &definitions,
            Default::default(),
            &diagnostics,
        )
        .unwrap_err();
        assert!(error.location().unwrap().bytes.is_none());
        assert!(error.path().starts_with(&path));
    }
}

/// Exercise actual tri-state drafts, public defaults and presentation without coercion.
#[test]
fn drafts_preserve_missing_null_false_zero_defaults_and_projection() {
    let ui = generation::normalize("model Example { id String @id name String? @default(\"seed\") @field.input flag Boolean @default(false) @field.checkbox count Integer @default(0) @list.text secret String }").unwrap();
    let mut fields = ui.models[0].fields.clone();
    let mut secret = fields[1].clone();
    secret.wire_name = "secret".into();
    secret.sensitive = true;
    fields.push(secret);
    let owner = Owner::new();
    owner.with(|| {
        let draft = Draft::new(
            &fields,
            &json!({"name":null,"flag":false,"count":0,"secret":"sentinel"}),
            Role::Field,
        )
        .unwrap();
        assert_eq!(draft.get("name").unwrap().get_untracked(), Input::Null);
        assert_eq!(
            draft.get("flag").unwrap().get_untracked(),
            Input::Value(json!(false))
        );
        assert!(draft.get("secret").is_none());
        let defaults = Draft::new(&fields, &json!({}), Role::Field).unwrap();
        assert_eq!(
            defaults.snapshot().unwrap(),
            json!({"name":"seed","flag":false,"count":0})
        );
        let filter = Draft::new(&fields, &json!({}), Role::Filter).unwrap();
        assert_eq!(filter.snapshot().unwrap(), json!({}));
        let list = Draft::new(&fields, &json!({"count":0}), Role::List).unwrap();
        let context = RoleContext {
            id: "count".into(),
            field: fields
                .iter()
                .find(|field| field.wire_name == "count")
                .unwrap()
                .clone(),
            role: Role::List,
            value: list.get("count").unwrap(),
        };
        assert_eq!(context.text(), "0");
        context.value.set(Input::Missing);
        assert_eq!(context.text(), "");
        context.value.set(Input::Null);
        assert_eq!(context.text(), "—");
        context.value.set(Input::Value(json!(false)));
        assert_eq!(context.text(), "false");
        context.value.set(Input::Value(json!("")));
        assert_eq!(context.text(), "");
        assert!(!context.bool_prop("disabled", false));
        context
            .value
            .set(Input::Value(json!("x".repeat(1_048_577))));
        assert!(list.snapshot().is_err());
        assert!(Draft::new(&fields, &json!("x".repeat(1_048_577)), Role::Field).is_err());
        assert!(Draft::new(&fields, &json!([]), Role::Field).is_err());
        let mut duplicate = fields.clone();
        duplicate.push(fields[1].clone());
        assert!(Draft::new(&duplicate, &json!({}), Role::Field).is_err());
    });
    owner.cleanup();
}

/// SSR all registered adapters over exact public scalar types and finite choices.
#[test]
fn concrete_adapters_render_escaped_semantic_controls() {
    use stackpress_framework_admin::components::*;
    let ui = generation::normalize("enum Choice { A \"a\" B \"b\" } model Example { id String @id name String? @field.input({ placeholder \"<script>\" }) @view.uppercase notes String @field.textarea flag Boolean @field.checkbox @view.yesno choice Choice @field.select }").unwrap();
    let owner = Owner::new();
    owner.with(|| {
        let fields = &ui.models[0].fields;
        let draft = Draft::new(fields, &json!({"name":"<script>bad()</script>","notes":"</textarea><script>bad()</script>","flag":false,"choice":"a"}), Role::Field).unwrap();
        let context = |name: &str, role| RoleContext { id: format!("fixture-{name}"), field: fields.iter().find(|field| field.wire_name == name).unwrap().clone(), role, value: draft.get(name).unwrap() };
        let text = context("name", Role::Field);
        let area = context("notes", Role::Field);
        let check = context("flag", Role::Field);
        let switch = check.clone();
        let select = context("choice", Role::Field);
        let display = context("name", Role::View);
        let yesno = context("flag", Role::View);
        let plain = display.clone();
        let html = view! { <TextField context=text/><TextareaField context=area/><CheckboxField context=check/><SwitchField context=switch/><SelectField context=select/><CaseView context=display/><BooleanView context=yesno/><TextView context=plain/> }.to_html();
        assert!(html.contains("type=\"checkbox\"")); assert!(html.contains("role=\"switch\""));
        assert!(html.contains("text-transform:uppercase")); assert!(html.contains("No"));
        assert!(!html.contains("<script>bad()")); assert!(!html.contains("</textarea><script>"));
        assert!(html.contains("fixture-name-help fixture-name-error"));
    });
    owner.cleanup();
}

/// List presentation is readable and escaped while exact arrays remain in the draft.
#[test]
fn text_view_formats_lists_without_mutating_values() {
    use stackpress_framework_admin::components::TextView;
    let ui =
        generation::normalize("model Example { id String @id notes String[] @list.text }").unwrap();
    let owner = Owner::new();
    owner.with(|| {
        for (value, expected) in [
            (json!([]), "None"),
            (
                json!(["rust", "0", "雪", "<script>"]),
                "rust, 0, 雪, &lt;script&gt;",
            ),
        ] {
            let draft =
                Draft::new(&ui.models[0].fields, &json!({"notes":value}), Role::List).unwrap();
            let context = RoleContext {
                id: "notes".into(),
                field: ui.models[0]
                    .fields
                    .iter()
                    .find(|field| field.wire_name == "notes")
                    .unwrap()
                    .clone(),
                role: Role::List,
                value: draft.get("notes").unwrap(),
            };
            let html = view! {<TextView context/>}.to_html();
            assert!(html.contains(expected), "{html}");
            assert_eq!(draft.snapshot().unwrap()["notes"], value);
        }
    });
    owner.cleanup();
}
