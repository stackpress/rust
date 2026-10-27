//! App-owned schema/SQL/UI generator registry; no generated consumer is a dependency.

use std::path::Path;

use stackpress_framework_admin::codegen::{Component, components, emit_roles, register_components};
use stackpress_framework_schema::{
    Schema,
    codegen::{
        DefinitionRegistry, DiagnosticMap, NormalizeOptions, emit_contracts, transform_error,
    },
};
use stackpress_idea::{
    parser::SchemaConfig,
    transformer::{
        CheckReport, Composer, GeneratedFile, OutputPlan, OutputWorkspace, ProjectLoader, Registry,
        Result, TransformContext, TransformInputs, TransformSelection, TransformSpec,
    },
};

/// Build the application's single immutable catalog, optionally including its compiled plugin.
/// # Errors
/// Propagates checked definition/policy failures without substituting another registry.
pub fn definitions(plugin: bool) -> stackpress_framework_schema::SchemaResult<DefinitionRegistry> {
    let replacements = if plugin {
        vec![accent_plugin()]
    } else {
        Vec::new()
    };
    let mut definitions = generated_workspace_tool::definitions()?;
    for field in ["id", "text", "flag", "zero"] {
        definitions = definitions.with_field_policy(
            "RoleProof",
            field,
            stackpress_framework_schema::FieldAccess {
                readable: true,
                writable: true,
                sensitive: false,
                publish_default: true,
            },
        )?;
    }
    definitions = definitions.with_field_policy(
        "RoleProof",
        "sensitive",
        stackpress_framework_schema::FieldAccess {
            writable: true,
            sensitive: true,
            ..Default::default()
        },
    )?;
    register_components(definitions, replacements)
}

/// The app plugin replaces one component and fixes its label/placeholder ahead of schema props.
pub fn accent_plugin() -> Component {
    let mut component = components()
        .into_iter()
        .find(|component| component.definition.name == "field.input")
        .expect("the framework catalog includes its input component");
    component.definition.key = "app.accent.input".into();
    component.definition.revision = "app-accent-1".into();
    component
        .definition
        .fixed
        .insert("label".into(), "Plugin field label".into());
    component
        .definition
        .fixed
        .insert("placeholder".into(), "Plugin fixed hint".into());
    component.import.component = component.definition.key.clone();
    component.import.module = "crate::plugin".into();
    component.import.symbol = "AccentField".into();
    component
}

/// Normalize identically in every registered transform, using only declared snapshots.
fn normalized(
    context: &TransformContext<'_>,
) -> Result<(Schema, DefinitionRegistry, DiagnosticMap)> {
    let plugin: bool = serde_json::from_str(context.input("admin-plugin.json")?).map_err(codec)?;
    let diagnostics: DiagnosticMap =
        serde_json::from_str(context.input("diagnostics.json")?).map_err(codec)?;
    let definitions = definitions(plugin).map_err(transform_error)?;
    let schema = Schema::normalize_with_diagnostics(
        context.schema(),
        &definitions,
        NormalizeOptions::default(),
        &diagnostics,
    )
    .map_err(transform_error)?;
    Ok((schema, definitions, diagnostics))
}

/// Schema owns contracts and public metadata.
fn schema_transform(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let (schema, _, _) = normalized(context)?;
    emit_contracts(&schema, output)
}

/// SQL retains its own stores partition; the frontend never includes those modules.
fn sql_transform(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let (schema, _, _) = normalized(context)?;
    stackpress_framework_sql::codegen::emit_stores(&schema, output)
}

/// UI consumes only the approved revision-2 projection of the same normalized schema.
fn ui_transform(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let (schema, definitions, diagnostics) = normalized(context)?;
    emit_roles(
        &schema.project_ui().map_err(transform_error)?,
        &definitions,
        &diagnostics,
        output,
    )
}

/// One registry writer supplies aggregate module declarations for all selected partitions.
fn aggregate(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(Vec::new())
}

/// Generate/check the same marked app-owned root through schema, SQL and UI function plugins.
///
/// The loader is rooted at the repository so this example can import the original workspace
/// fixture without copying or editing it. Callers choose the output explicitly for disposable
/// model removal/rename tests. There is no listener, database open, second writer, or install.
/// # Errors
/// Preserves parser/composer/normalizer/output errors before publication.
pub fn generate(input: &Path, output: &Path, check: bool, plugin: bool) -> Result<CheckReport> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let input = std::path::absolute(input)?;
    let loader = ProjectLoader::new(root).map_err(|cause| {
        stackpress_idea::transformer::TransformError::from(std::io::Error::other(cause))
    })?;
    let entry = input.to_str().ok_or_else(|| {
        stackpress_idea::transformer::TransformError::new("input path must be UTF-8")
    })?;
    let composed = Composer::new(loader, Default::default()).compose(entry)?;
    let diagnostics = DiagnosticMap::from_composed(&composed)?;
    let finalized = stackpress_idea::parser::finalize_with_references(
        &composed.config,
        &composed.references,
        &Default::default(),
    )?;
    let config = SchemaConfig {
        plugin: finalized.plugin,
        r#enum: finalized.r#enum,
        r#type: finalized.r#type,
        model: finalized.model,
        ..Default::default()
    };
    let mut inputs = TransformInputs::new();
    inputs.insert(
        "diagnostics.json",
        serde_json::to_string(&diagnostics).map_err(codec)?,
    )?;
    inputs.insert("admin-plugin.json", plugin.to_string())?;
    let selection = TransformSelection::new().with_inputs(inputs);
    let mut registry = Registry::new();
    // Imported plugin blocks do not select generators. Root blocks must explicitly list the
    // compiled functions in dependency order; the registry diagnoses absent/out-of-order ones.
    for plugin in config.plugin.keys() {
        match plugin.as_str() {
            "framework-schema" => registry.add(
                TransformSpec::new(plugin).owns(&["contracts", "metadata"]),
                schema_transform,
            )?,
            "framework-sql" => registry.add(
                TransformSpec::new(plugin)
                    .owns(&["stores"])
                    .requires(&["framework-schema"]),
                sql_transform,
            )?,
            "framework-admin" => registry.add(
                TransformSpec::new(plugin)
                    .owns(&["ui"])
                    .requires(&["framework-schema"]),
                ui_transform,
            )?,
            _ => {
                return Err(stackpress_idea::transformer::TransformError::new(
                    "generator plugin is not compiled into this tool",
                ));
            }
        }
    }
    let plan = registry.plan_with(&config, &selection, aggregate(&config)?)?;
    let workspace = OutputWorkspace::new(output)?;
    if check {
        workspace.check(&plan)
    } else {
        workspace.install(plan)
    }
}

/// Keep the concrete JSON codec cause under the transform error policy.
fn codec(cause: serde_json::Error) -> stackpress_idea::transformer::TransformError {
    stackpress_idea::transformer::TransformError::from(std::io::Error::other(cause))
}
