//! App-owned Profile mapping, registration and deterministic generated source.
//!
//! This module owns the advertised demo shape and runtime template. The tool adapter composes
//! sources and installs output; the generated application never depends on this module.

use stackpress_idea::parser::SchemaConfig;
use stackpress_idea::transformer::{
    GeneratedFile, Registry, Result as GenerateResult, TransformError, TransformInputs,
    TransformSelection,
};
#[cfg(feature = "profiles")]
use stackpress_idea::transformer::{OutputPlan, TransformContext, TransformSpec};

use crate::selection;

/// Register the example transform through an ordinary selected function.
///
/// The callback consumes the declared `templates/profile.rs` auxiliary snapshot. Use
/// [`crate::generate`] for the complete run, including that compiled asset snapshot and imports.
/// Registration itself performs no IO and cannot activate a feature excluded at compilation.
///
/// # Errors
/// Returns registry ID/partition conflicts before any callbacks execute.
pub fn register(registry: &mut Registry) -> GenerateResult<()> {
    // Disabled builds contain no generator callback; schema blocks cannot activate it.
    #[cfg(feature = "profiles")]
    registry.add(
        TransformSpec::new("profile-demo").owns(&["contracts", "server"]),
        emit_profile,
    )?;
    let _ = registry;
    Ok(())
}

/// Emit real typed public fields and the existing proved search implementation.
#[cfg(feature = "profiles")]
fn emit_profile(context: &TransformContext<'_>, plan: &mut OutputPlan) -> GenerateResult<()> {
    let schema = context.schema();
    // This generator has no configuration knobs; accepting them would imply false semantics.
    if schema.plugin.values().any(|config| !config.is_empty()) {
        return Err(TransformError::new(
            "profile-demo accepts no plugin configuration",
        ));
    }
    // Removing the model deliberately contributes no contracts or runtime implementation.
    if schema.model.is_empty() {
        return Ok(());
    }
    let model = schema
        .model
        .get("Profile")
        .ok_or_else(|| TransformError::new("profile-demo supports only the Profile model"))?;
    // This explicit demo mapping cannot silently turn unsupported metadata into false promises.
    if schema.model.len() != 1 || !supported_profile(model) {
        return Err(TransformError::new(
            "profile-demo requires exactly Profile { id String name String }",
        ));
    }
    // Only the exact checked demo names reach the structural field loop.
    let fields: Vec<_> = model
        .columns
        .iter()
        .map(|column| column.name.as_str())
        .collect();
    let contract = source(
        "templates/contract.rs.tpl",
        include_str!("templates/contract.rs.tpl"),
        &serde_json::json!({"fields":fields}),
    )?;
    // The runtime source is an explicitly declared immutable template snapshot for this plan.
    plan.write("contracts/profile.rs", contract)?;
    plan.write("server/profile.rs", context.input("templates/profile.rs")?)?;
    Ok(())
}

/// Accept exactly the demo's two ordered, required scalar String fields without metadata.
#[cfg(feature = "profiles")]
fn supported_profile(model: &stackpress_idea::parser::ModelConfig) -> bool {
    // Length is checked before zip, so an omitted or extra column cannot be silently ignored.
    model.columns.len() == 2
        && model.attributes.is_empty()
        && model
            .columns
            .iter()
            .zip(["id", "name"])
            .all(|(column, name)| {
                // These conditions define the demo mapping; Idea metadata stays parser-owned.
                column.name == name
                    && column.column_type == "String"
                    && column.required
                    && !column.multiple
                    && column.attributes.is_empty()
            })
}

/// Own the aggregate registry and compile-time selection/revision checks.
pub(crate) fn aggregate(schema: &SchemaConfig) -> GenerateResult<Vec<GeneratedFile>> {
    let present = selection::ENABLED && schema.model.contains_key("Profile");
    let source = source(
        "templates/aggregate.rs.tpl",
        include_str!("templates/aggregate.rs.tpl"),
        &serde_json::json!({
            "enabled":selection::ENABLED, "revision":selection::REVISION, "present":present,
        }),
    )?;
    Ok(vec![GeneratedFile::new("mod.rs", source)])
}

/// Render immutable build assets without interpreting inserted source or changing writer limits.
fn source(asset: &'static str, template: &str, data: &serde_json::Value) -> GenerateResult<String> {
    let render = || {
        let engine = stackpress_lib::TemplateEngine::new(stackpress_lib::TemplateOptions {
            delimiters: ["[%".into(), "%]".into()],
            ..Default::default()
        })?;
        engine.render(template, data)
    };
    render().map_err(|cause: stackpress_lib::TemplateError| {
        let kind = if cause.kind() == stackpress_lib::TemplateErrorKind::Limit {
            stackpress_idea::transformer::TransformErrorKind::Limit
        } else {
            stackpress_idea::transformer::TransformErrorKind::Callback
        };
        TransformError::with_source(
            kind,
            format!("profile source template failed: {asset}"),
            cause,
        )
        .unwrap_or_else(|error| error)
    })
}

/// Snapshot the compiled generator's assets before any transform callback runs.
pub(crate) fn transform_selection() -> GenerateResult<TransformSelection> {
    let inputs = TransformInputs::new();
    #[cfg(feature = "profiles")]
    let inputs = {
        // Disabled codegen excludes both the callback and its runtime-template bytes.
        let mut inputs = inputs;
        inputs.insert("templates/profile.rs", include_str!("profile.rs.txt"))?;
        inputs
    };
    Ok(TransformSelection::new().with_inputs(inputs))
}
