//! Real descriptor generation retains encoded-data headroom and managed writer budgets.
#![cfg(feature = "codegen")]

use stackpress_framework_admin::codegen::{emit_roles, register_components};
use stackpress_framework_schema::{
    FieldAccess, Schema, UiSchema,
    codegen::{DefinitionRegistry, DiagnosticMap},
};
use stackpress_idea_transformer::{
    Composer, MemoryLoader, OutputPlan, OutputWorkspace, Registry, TransformContext,
    TransformInputs, TransformSelection, TransformSpec,
};
use std::error::Error;

/// Use the actual public component registry and existing field access rules.
fn definitions() -> DefinitionRegistry {
    let mut definitions = DefinitionRegistry::standard();
    for field in ["id", "name"] {
        definitions = definitions
            .with_field_policy(
                "Example",
                field,
                FieldAccess {
                    readable: true,
                    writable: true,
                    sensitive: false,
                    publish_default: true,
                },
            )
            .unwrap();
    }
    register_components(definitions, Vec::new()).unwrap()
}

/// Contribute through the real ownership-granted transform, without touching generated examples.
fn transform(
    context: &TransformContext<'_>,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let ui: UiSchema = serde_json::from_str(context.input("ui")?).unwrap();
    emit_roles(
        &ui,
        &definitions(),
        &DiagnosticMap { entries: vec![] },
        output,
    )
}

/// Parse and normalize a real source label, then plan its public UI through the existing writer.
fn label_plan(repeats: usize) -> stackpress_idea_transformer::Result<OutputPlan> {
    let hostile = "\"\\\n".repeat(repeats) + "[%missing%] </script> 🦀";
    let mut loader = MemoryLoader::new("/admin-templates").unwrap();
    loader.insert(
        "schema.idea",
        "model Example { id String @id name String @field.input }",
    );
    let mut config = Composer::new(loader, Default::default())
        .compose("schema.idea")
        .unwrap()
        .config;
    // The normalized JSON source supports the full hostile literal corpus directly.
    config
        .model
        .get_mut("Example")
        .unwrap()
        .columns
        .iter_mut()
        .find(|field| field.name == "name")
        .unwrap()
        .attributes
        .insert(
            "label".into(),
            stackpress_idea_parser::AttributeValue::Array(vec![serde_json::json!(hostile)]),
        );
    let mut loader = MemoryLoader::new("/admin-templates").unwrap();
    loader.insert("schema.json", serde_json::to_string(&config).unwrap());
    let composed = Composer::new(loader, Default::default())
        .compose("schema.json")
        .unwrap();
    let ui = Schema::normalize(&composed.config, &definitions(), Default::default())
        .unwrap()
        .project_ui()
        .unwrap();
    assert_eq!(
        ui.models[0]
            .fields
            .iter()
            .find(|field| field.wire_name == "name")
            .unwrap()
            .label,
        hostile
    );
    let mut inputs = TransformInputs::new();
    inputs
        .insert("ui", serde_json::to_string(&ui).unwrap())
        .unwrap();
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("admin").owns(&["ui"]), transform)
        .unwrap();
    registry.plan_with(
        &Default::default(),
        &TransformSelection::new().with_inputs(inputs),
        vec![],
    )
}

/// Heavy real-source literal escaping remains accepted near the unchanged one-MiB writer ceiling.
#[test]
fn escaped_descriptors_keep_existing_writer_budget_and_literal_contents() {
    let plan = label_plan(94_000).unwrap();
    let source = plan
        .files()
        .find(|(path, _, _)| *path == "ui/example.rs")
        .unwrap()
        .2;
    assert!(source.len() > 1_000_000);
    assert!(source.len() < 1024 * 1024);
    assert!(source.contains("[%missing%] </script> 🦀"));
    // The ordinary writer performs actual rustfmt and unchanged aggregate artifact accounting.
    let temporary = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = temporary.path().join("generated");
    OutputWorkspace::new(&root).unwrap().install(plan).unwrap();
    let generated = std::fs::read_to_string(root.join("ui/example.rs")).unwrap();
    assert!(generated.contains("[%missing%] </script> 🦀"));
    assert!(generated.len() < 1024 * 1024);
}

/// Oversize real generated source retains the public writer Limit category after migration.
#[test]
fn oversized_real_source_retains_transform_limit_classification() {
    let error = label_plan(100_000).unwrap_err();
    // Registry has always wrapped callbacks; preserve the concrete emitter category beneath it.
    assert_eq!(
        error.kind(),
        stackpress_idea_transformer::TransformErrorKind::Callback
    );
    let emitted = error
        .source()
        .unwrap()
        .downcast_ref::<stackpress_idea_transformer::TransformError>()
        .unwrap();
    assert_eq!(
        emitted.kind(),
        stackpress_idea_transformer::TransformErrorKind::Limit
    );
}
