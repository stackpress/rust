//! Real build plans distinguish role-only UI, native pages and operation output.
use super::*;
use stackpress_idea::transformer::{ContextTransform, MemoryLoader};

/// A preflight failure must occur before even the first callback can run.
fn unexpected(_: &TransformContext<'_>, _: &mut OutputPlan) -> Result<()> {
    panic!("invalid transform selection executed a callback")
}
/// Missing and later dependencies fail before any output or app callback effects.
#[test]
fn explicit_transform_dependencies_preflight_every_callback() {
    for order in [
        vec!["sql"],
        vec!["admin"],
        vec!["schema", "admin"],
        vec!["sql", "schema"],
        vec!["schema", "admin", "sql"],
    ] {
        let mut registry = Registry::new();
        for id in order {
            match id {
                "schema" => {
                    stackpress_framework_schema::codegen::register(&mut registry, unexpected)
                        .unwrap()
                }
                "sql" => {
                    stackpress_framework_sql::codegen::register(&mut registry, unexpected).unwrap()
                }
                "admin" => {
                    stackpress_framework_admin::codegen::register_pages(&mut registry, unexpected)
                        .unwrap()
                }
                _ => unreachable!(),
            }
        }
        assert!(
            registry
                .plan_with(&Default::default(), &Default::default(), vec![])
                .is_err()
        );
    }
}
/// Compose real checked input and immutable diagnostics exactly as the application tool does.
fn fixture() -> (SchemaConfig, TransformSelection) {
    let mut loader = MemoryLoader::new("/registration").unwrap();
    loader.insert(
        "schema.idea",
        "model Profile { id String @id name String @field.input }",
    );
    let composed = Composer::new(loader, Default::default())
        .compose("schema.idea")
        .unwrap();
    let mut inputs = TransformInputs::new();
    inputs
        .insert(
            "diagnostics.json",
            serde_json::to_string(&DiagnosticMap::from_composed(&composed).unwrap()).unwrap(),
        )
        .unwrap();
    inputs.insert("admin-plugin.json", "false").unwrap();
    (
        composed.config,
        TransformSelection::new().with_inputs(inputs),
    )
}
/// Select real SQL stores without operation emission to test the stronger output dependency.
fn stores_only(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    let (schema, _, _) = normalized(context)?;
    stackpress_framework_sql::codegen::emit_stores(&schema, plan)
}
/// Merely selecting a SQL transform does not satisfy the generated page host contract.
#[test]
fn required_operation_output_is_checked_before_publication() {
    let (config, selection) = fixture();
    for (callback, valid) in [
        (stores_only as ContextTransform, false),
        (sql_transform as ContextTransform, true),
    ] {
        let mut registry = Registry::new();
        stackpress_framework_schema::codegen::register(&mut registry, schema_transform).unwrap();
        stackpress_framework_sql::codegen::register(&mut registry, callback).unwrap();
        stackpress_framework_admin::codegen::register_pages(&mut registry, ui_transform).unwrap();
        let plan = registry.plan_with(&config, &selection, vec![]).unwrap();
        assert_eq!(require_page_operations(&plan, true).is_ok(), valid);
        assert!(require_page_operations(&plan, false).is_ok());
    }
}
/// Role-only generation remains SQL-free; selecting both alternatives fails singleton ownership.
#[test]
fn role_only_registration_and_duplicate_claims_use_existing_registry_contracts() {
    /// This role-only callback never requests the native page emitter.
    fn roles(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
        let (schema, definitions, diagnostics) = normalized(context)?;
        emit_roles(
            &schema.project_ui().map_err(transform_error)?,
            &definitions,
            &diagnostics,
            plan,
        )
    }
    let (config, selection) = fixture();
    let mut registry = Registry::new();
    stackpress_framework_schema::codegen::register(&mut registry, schema_transform).unwrap();
    assert!(
        stackpress_framework_schema::codegen::register(&mut registry, schema_transform).is_err()
    );
    stackpress_framework_admin::codegen::register(&mut registry, roles).unwrap();
    assert!(
        stackpress_framework_admin::codegen::register_pages(&mut registry, ui_transform).is_err()
    );
    let plan = registry.plan_with(&config, &selection, vec![]).unwrap();
    assert!(plan.files().any(|(path, _, _)| path == "ui/profile.rs"));
    assert!(
        !plan
            .files()
            .any(|(path, _, _)| path.starts_with("stores/") || path == "ui/pages_host.rs")
    );
    let mut registry = Registry::new();
    stackpress_framework_sql::codegen::register(&mut registry, unexpected).unwrap();
    assert!(stackpress_framework_sql::codegen::register(&mut registry, unexpected).is_err());
}
