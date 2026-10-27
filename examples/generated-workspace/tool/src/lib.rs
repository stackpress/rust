//! Single app-owned generator registry, independent of every generated application module.

use std::path::Path;

pub mod maintenance;
mod operator_schema;

use stackpress_framework_schema::{
    Schema,
    codegen::{
        DefinitionRegistry, DiagnosticMap, NormalizeOptions, emit_contracts, transform_error,
    },
};
use stackpress_idea::{
    GenerateRequest, GenerationMode,
    parser::SchemaConfig,
    transformer::{
        CheckReport, GeneratedFile, OutputPlan, OutputWorkspace, ProjectLoader, Registry, Result,
        TransformContext, TransformSpec,
    },
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

/// Freeze the same compiled definitions for every selected transform.
pub fn definitions() -> stackpress_framework_schema::SchemaResult<DefinitionRegistry> {
    let mut registry = DefinitionRegistry::standard().with_provider(
        stackpress_framework_schema::codegen::ProviderDefinition {
            name: "nanoid".into(),
            revision: "app-fixture-1".into(),
            output: stackpress_framework_schema::ValueType::String,
        },
    )?;
    registry =
        registry.with_validator(stackpress_framework_schema::codegen::ValidatorDefinition {
            name: "app.approved".into(),
            revision: "app-rule-1".into(),
            types: vec![stackpress_framework_schema::ValueType::String],
        })?;
    // This application explicitly approves its fixtures' nonsecret fields and defaults,
    // including the quickstart's heading/summary and inverse links; no route is authorized here.
    for (model, fields) in [
        ("Profile", "id name active address projects links"),
        (
            "Project",
            "id ownerId title heading summary state tags tasks owner",
        ),
        ("Task", "id projectId title done project"),
        ("Address", "city country"),
        (
            "Matrix",
            "id text count real float flag day time stamp object choice truth tags numbers dates times instants objects address siblings favorite created token links",
        ),
        ("MatrixAddress", "city country"),
        ("Link", "id ownerId owner"),
        (
            "Compound",
            "region code enabled day clock instant ratio choice truth value note",
        ),
        ("Query", "id self payload"),
        ("Payload", "row state value"),
        ("Schema", "id value type active owner"),
        ("Operations", "id value schemaId row"),
    ] {
        for field in fields.split_whitespace() {
            registry = registry.with_field_policy(
                model,
                field,
                stackpress_framework_schema::FieldAccess {
                    readable: true,
                    writable: true,
                    sensitive: false,
                    publish_default: true,
                },
            )?;
        }
    }
    Ok(registry)
}

/// Interpret only finalized config using this app's frozen catalog.
fn normalize(config: &SchemaConfig, diagnostics: &DiagnosticMap) -> Result<Schema> {
    Schema::normalize_with_diagnostics(
        config,
        &definitions().map_err(transform_error)?,
        NormalizeOptions::default(),
        diagnostics,
    )
    .map_err(transform_error)
}

/// Schema owns contracts and metadata; no SQL files are patched here.
fn schema_transform(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let diagnostics: DiagnosticMap = serde_json::from_str(context.input("diagnostics.json")?)
        .map_err(|cause| {
            stackpress_idea::transformer::TransformError::from(std::io::Error::other(cause))
        })?;
    emit_contracts(&normalize(context.schema(), &diagnostics)?, output)
}

/// SQL uses the identical normalization function and owns stores exclusively.
fn sql_transform(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let diagnostics: DiagnosticMap = serde_json::from_str(context.input("diagnostics.json")?)
        .map_err(|cause| {
            stackpress_idea::transformer::TransformError::from(std::io::Error::other(cause))
        })?;
    stackpress_framework_sql::codegen::emit_stores_with_diagnostics(
        &normalize(context.schema(), &diagnostics)?,
        output,
        &diagnostics,
        true,
    )
}

/// The aggregate contributes only module declarations; callbacks own their source partitions.
fn aggregate(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(Vec::new())
}

/// Generate or check an explicitly selected Idea input and marked output root.
///
/// The tool never imports the app, so this works when its generated root is absent.
/// # Errors
/// Returns bounded source/semantic/ownership/formatting failures before publishing a result.
pub fn generate(input: &Path, output: &Path, check: bool) -> Result<CheckReport> {
    let input = std::path::absolute(input)?;
    let root = input
        .parent()
        .ok_or_else(|| stackpress_idea::transformer::TransformError::new("input has no parent"))?;
    let loader = ProjectLoader::new(root).map_err(|cause| {
        stackpress_idea::transformer::TransformError::from(std::io::Error::other(cause))
    })?;
    let entry = input.to_str().ok_or_else(|| {
        stackpress_idea::transformer::TransformError::new("input path must be UTF-8")
    })?;
    let mode = if check {
        GenerationMode::Check
    } else {
        GenerationMode::Install
    };
    let request = GenerateRequest::new(entry, aggregate, mode);
    let composed =
        stackpress_idea::transformer::Composer::new(loader, request.limits).compose(entry)?;
    let diagnostics = DiagnosticMap::from_composed(&composed)?;
    let mut inputs = stackpress_idea::transformer::TransformInputs::new();
    inputs.insert(
        "diagnostics.json",
        serde_json::to_string(&diagnostics).map_err(|cause| {
            stackpress_idea::transformer::TransformError::from(std::io::Error::other(cause))
        })?,
    )?;
    let selection = stackpress_idea::transformer::TransformSelection::new().with_inputs(inputs);
    let finalized = stackpress_idea::parser::finalize_with_references(
        &composed.config,
        &composed.references,
        &request.inputs,
    )?;
    let config = SchemaConfig {
        plugin: finalized.plugin,
        r#enum: finalized.r#enum,
        r#type: finalized.r#type,
        model: finalized.model,
        ..Default::default()
    };
    // Select only compiled functions explicitly listed by this finalized application input.
    // Unknown plugins and invalid dependency/order selections fail before output publication.
    let mut registry = Registry::new();
    for plugin in config.plugin.keys() {
        match plugin.as_str() {
            "framework-schema" => registry.add(
                TransformSpec::new("framework-schema").owns(&["contracts", "metadata"]),
                schema_transform,
            )?,
            "framework-sql" => registry.add(
                TransformSpec::new("framework-sql")
                    .owns(&["stores"])
                    .requires(&["framework-schema"]),
                sql_transform,
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
    match mode {
        GenerationMode::Check => workspace.check(&plan),
        GenerationMode::Install => workspace.install(plan),
    }
}

/// Await generation on an owned blocking worker, keeping CLI dispatch independent of the app.
async fn generate_event(
    req: &mut Request,
    res: &mut Response,
    _: Invocation<'_>,
) -> stackpress_lib::Result<Flow> {
    let app = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../app")
        .canonicalize()
        .map_err(|cause| {
            Error::new("application root resolution failed")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(cause)
        })?;
    let input = req
        .data
        .path_as::<String>("input")?
        .map(Into::into)
        .unwrap_or_else(|| app.join("schema.idea"));
    let output = req
        .data
        .path_as::<String>("output")?
        .map(Into::into)
        .unwrap_or_else(|| app.join("src/generated"));
    let check = match req.data.path_as::<bool>("check")? {
        Some(check) => check,
        None => req.data.path_as::<bool>("c")?.unwrap_or(false),
    };
    let report = tokio::task::spawn_blocking(move || generate(&input, &output, check))
        .await
        .map_err(|cause| {
            Error::new("generation worker failed")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(cause)
        })?
        .map_err(|cause| {
            Error::new("workspace generation failed")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(cause)
        })?;
    if check && !report.is_current() {
        return Err(Error::invalid("generated output is stale"));
    }
    res.results(&report)?;
    Ok(Flow::Continue)
}

/// Build an explicit app tool; only selected operator events open databases, never startup.
pub async fn build_tool(layers: ConfigLayers) -> BuildResult<Server> {
    let mut builder = ServerBuilder::new();
    builder.config_layers(layers)?;
    builder.on("generate", generate_event)?;
    for event in ["revision", "history-check", "migration-plan"] {
        builder.on(event, maintenance::event)?;
    }
    #[cfg(feature = "database-tools")]
    for event in [
        "db-status",
        "db-install",
        "db-upgrade",
        "db-downgrade",
        "db-purge",
        "db-uninstall",
        "db-populate",
    ] {
        builder.on(event, maintenance::event)?;
    }
    builder.build().await
}
