//! Actual SQL source generation, managed formatting and resource-boundary regressions.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

use stackpress_framework_schema::codegen::{
    DefinitionRegistry, NormalizeOptions, ProviderDefinition, ValidatorDefinition, transform_error,
};
use stackpress_framework_schema::{FieldAccess, Schema, ValueType};
use stackpress_framework_sql::codegen::{emit_stores, emit_stores_with_operations};
use stackpress_idea_transformer::{
    Composer, CompositionLimits, MemoryLoader, OutputPlan, OutputWorkspace, Registry, Result,
    TransformContext, TransformError, TransformErrorKind, TransformSpec,
};

/// Compose real Idea without coupling this fixture to an application's plugin selections.
pub(super) fn config(source: &str) -> stackpress_idea_transformer::ComposedSchema {
    let source = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("plugin "))
        .collect::<Vec<_>>()
        .join("\n");
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("schema.idea", source);
    Composer::new(loader, CompositionLimits::default())
        .compose("schema.idea")
        .unwrap()
}

/// Approve fixture fields explicitly while keeping secret defaults host-owned.
pub(super) fn normalized(context: &TransformContext<'_>) -> Result<Schema> {
    let config = context.schema();
    let mut definitions = DefinitionRegistry::standard()
        .with_provider(ProviderDefinition {
            name: "nanoid".into(),
            revision: "fixture-1".into(),
            output: ValueType::String,
        })
        .map_err(transform_error)?
        .with_validator(ValidatorDefinition {
            name: "app.approved".into(),
            revision: "fixture-1".into(),
            types: vec![ValueType::String],
        })
        .map_err(transform_error)?;
    for (name, record) in config.model.iter().chain(&config.r#type) {
        for field in &record.columns {
            definitions = definitions
                .with_field_policy(
                    name,
                    &field.name,
                    FieldAccess {
                        readable: true,
                        writable: true,
                        sensitive: false,
                        publish_default: field.name != "secret",
                    },
                )
                .map_err(transform_error)?;
        }
    }
    Schema::normalize(config, &definitions, NormalizeOptions::default()).map_err(transform_error)
}

/// Native-only fixture callback exercises the public emitter under actual write authority.
fn native(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    emit_stores(&normalized(context)?, plan)
}

/// Host-operation fixture callback selects handlers without enabling a runtime server.
fn operations(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    emit_stores_with_operations(&normalized(context)?, plan)
}

/// Use one SQL partition so tests also cover module ownership and aggregation.
pub(super) fn plan(source: &str, with_operations: bool) -> Result<OutputPlan> {
    plan_config(&config(source), with_operations)
}

/// Retain direct normalized-JSON/typed-config literals the raw Idea quote grammar cannot spell.
fn plan_config(
    config: &stackpress_idea_transformer::ComposedSchema,
    with_operations: bool,
) -> Result<OutputPlan> {
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("sql").owns(&["stores"]),
        if with_operations { operations } else { native },
    )?;
    registry.plan(&config.config, vec![])
}

/// Snapshot sorted installed source, independent of temporary root names.
pub(super) fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, output: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, output);
            } else {
                output.insert(
                    path.strip_prefix(root).unwrap().to_str().unwrap().into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut output = BTreeMap::new();
    visit(root, root, &mut output);
    output
}

/// The full type and relation corpus formats, installs and repeats byte-for-byte.
#[test]
fn real_type_corpus_and_operation_shapes_repeat() {
    for source in [
        include_str!("../../../../../examples/generated-workspace/contracts/matrix.idea"),
        include_str!("../../../../../examples/generated-workspace/app/schema.idea"),
        include_str!("../../../../../examples/generated-workspace/app/native.idea"),
        include_str!("../../../../../examples/generated-workspace/app/names.idea"),
        "enum Code { A 1 B 2 } model Compound { id Boolean @id code Code @id value String? }
         model Schema { id Integer @id value String @unique active Boolean @active updated Datetime @timestamp }
         model Operations { id String @id value String? }",
        "model Bounds { id String @id @store.length(191) required String @store.length(12) optional String? @store.length(9) }
         model KeyOnly { id String @id @store.length(191) }",
    ] {
        for with_operations in [false, true] {
            let initial = plan(source, with_operations).unwrap();
            let raw: BTreeMap<_, _> = initial.files().map(|(path, _, text)| (path.to_owned(), text.to_owned())).collect();
            assert_eq!(raw.contains_key("stores/operations.rs"), with_operations);
            let root = tempfile::tempdir_in("/private/tmp").unwrap();
            let output = root.path().join("generated");
            let workspace = OutputWorkspace::new(&output).unwrap();
            workspace.install(initial).unwrap();
            let before = snapshot(&output);
            assert!(workspace.check(&plan(source, with_operations).unwrap()).unwrap().is_current());
            workspace.install(plan(source, with_operations).unwrap()).unwrap();
            assert_eq!(before, snapshot(&output));
        }
    }
}

/// Arbitrary literal delimiters remain inert, and private defaults remain named providers.
#[test]
fn hostile_literals_and_private_policy_keep_their_boundaries() {
    let source = r#"enum Choice { VALUE "safe" } model Literal {
        id String @id
        value String @default("[%missing%] {{also_missing}} Café \\ backslash")
        secret String @default("PRIVATE_DEFAULT_SENTINEL")
        choice Choice
    }"#;
    let mut composed = config(source);
    // Typed config can carry quotes/control characters without inventing Idea escape syntax.
    composed.config.r#enum["Choice"]["VALUE"] =
        serde_json::json!("quote \" newline\n tab\t null\0 [%unknown%] {{inert}} café");
    let plan = plan_config(&composed, true).unwrap();
    let (_, _, operations) = plan
        .files()
        .find(|(path, _, _)| *path == "stores/operations.rs")
        .unwrap();
    assert!(operations.contains("[%missing%] {{also_missing}}"));
    assert!(operations.contains("default:Literal.secret"));
    assert!(!operations.contains("PRIVATE_DEFAULT_SENTINEL"));
    assert!(!operations.contains("pub async fn remove("));
    assert!(!operations.contains("pub async fn restore("));
    let positions: Vec<_> = [
        "get", "detail", "search", "create", "update", "upsert", "batch", "delete",
    ]
    .into_iter()
    .map(|name| operations.find(&format!("pub async fn {name}(")).unwrap())
    .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    OutputWorkspace::new(root.path().join("generated"))
        .unwrap()
        .install(plan)
        .unwrap();
}

/// Build a real normalized model whose repeated enum choices can approach writer limits.
pub(super) fn wide_source(fields: usize, choice_bytes: usize) -> String {
    let choices = (0..32)
        .map(|index| {
            format!(
                "V{index} {:?}",
                "x".repeat(choice_bytes) + &index.to_string()
            )
        })
        .collect::<Vec<_>>()
        .join(" ");
    let fields = (0..fields)
        .map(|index| format!("field{index} Choice"))
        .collect::<Vec<_>>()
        .join(" ");
    format!("enum Choice {{ {choices} }} model Wide {{ id String @id {fields} }}")
}

/// Observe the emitter's own classification inside the registry's unchanged callback context.
pub(super) fn emitter_error(error: &TransformError) -> &TransformError {
    error
        .source()
        .and_then(|cause| cause.downcast_ref::<TransformError>())
        .unwrap()
}

/// Real source overflow stays Limit, and failed planning cannot replace an installed root.
#[test]
fn source_output_limit_preserves_kind_and_installed_bytes() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let output = root.path().join("generated");
    let workspace = OutputWorkspace::new(&output).unwrap();
    workspace
        .install(plan("model Small { id String @id }", false).unwrap())
        .unwrap();
    let before = snapshot(&output);
    let error = plan(&wide_source(127, 512), false).unwrap_err();
    assert_eq!(error.kind(), TransformErrorKind::Callback);
    assert_eq!(
        emitter_error(&error).kind(),
        TransformErrorKind::Limit,
        "{error:#?}"
    );
    // This actual schema exceeds one asset, before the managed writer's aggregate check.
    let cause = emitter_error(&error)
        .source()
        .unwrap()
        .downcast_ref::<stackpress_lib::TemplateError>()
        .unwrap();
    assert_eq!(cause.kind(), stackpress_lib::TemplateErrorKind::Limit);
    assert_eq!(before, snapshot(&output));
}

/// This real case was byte-compared against the pre-migration producer at its accepted boundary.
#[test]
fn previously_accepted_near_limit_schema_stays_accepted() {
    let source = wide_source(56, 512);
    let plan = plan(&source, false).unwrap();
    let raw: usize = plan.files().map(|(_, _, source)| source.len()).sum();
    assert!(raw > 1024 * 1024 * 98 / 100 && raw <= 1024 * 1024);
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let output = root.path().join("generated");
    let workspace = OutputWorkspace::new(&output).unwrap();
    workspace.install(plan).unwrap();
    let formatted: usize = snapshot(&output)
        .iter()
        .filter(|(path, _)| path.ends_with(".rs"))
        .map(|(_, bytes)| bytes.len())
        .sum();
    assert!(formatted > 1024 * 1024 * 98 / 100 && formatted <= 1024 * 1024);
    assert!(
        workspace
            .check(&self::plan(&source, false).unwrap())
            .unwrap()
            .is_current()
    );
}

/// Empty/disabled output and a key-only model keep finite selected behavior.
#[test]
fn empty_and_key_only_shapes_do_not_invent_capabilities() {
    let empty = plan("", false).unwrap();
    assert_eq!(
        empty.files().map(|(path, _, _)| path).collect::<Vec<_>>(),
        ["mod.rs", "stores/mod.rs", "stores/schema.rs"]
    );
    let key_only = plan("model KeyOnly { id Integer @id }", true).unwrap();
    let (_, _, source) = key_only
        .files()
        .find(|(path, _, _)| *path == "stores/operations.rs")
        .unwrap();
    assert!(source.contains("model declares no mutable patch fields"));
    assert!(!source.contains("let mut value:"));
}

/// Private relation flags must remove eager members from every public depth, retaining trusted ones.
#[test]
fn private_relations_do_not_widen_public_views() {
    /// The default registry grants no field exposure; SQL cannot infer one from a relation.
    fn private(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
        let schema = Schema::normalize(
            context.schema(),
            &DefinitionRegistry::standard(),
            NormalizeOptions::default(),
        )
        .map_err(transform_error)?;
        emit_stores(&schema, output)
    }
    let source = "model Parent { id String @id children Child[] }
        model Child { id String @id parentId String? parent Parent? @relation({local \"parentId\" foreign \"id\" name \"children\"}) }";
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("sql").owns(&["stores"]), private)
        .unwrap();
    let plan = registry.plan(&config(source).config, vec![]).unwrap();
    let (_, _, parent) = plan
        .files()
        .find(|(path, _, _)| *path == "stores/parent.rs")
        .unwrap();
    assert!(parent.contains("pub children: Loaded<"));
    for depth in 0..=3 {
        let declaration = parent
            .split(&format!("pub struct PublicView{depth} {{"))
            .nth(1)
            .unwrap()
            .split('}')
            .next()
            .unwrap();
        assert!(!declaration.contains("children"));
    }
    assert!(parent.contains("public: false"));
}
