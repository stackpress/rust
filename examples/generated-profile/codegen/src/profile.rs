//! App-owned Profile mapping, registration and deterministic generated source.
//!
//! This module owns the advertised demo shape and runtime template. The tool adapter composes
//! sources and installs output; the generated application never depends on this module.

use stackpress_idea::parser::SchemaConfig;
use stackpress_idea::transformer::{
    GeneratedFile, Registry, Result as GenerateResult, TransformInputs, TransformSelection,
};
#[cfg(feature = "profiles")]
use stackpress_idea::transformer::{OutputPlan, TransformContext, TransformError, TransformSpec};

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
    let mut contract = String::from("//! Public model fields generated from schema.idea.\n\n");
    contract.push_str("/// Public profile data; generated from the declared model.\n");
    contract
        .push_str("#[derive(Debug, serde::Deserialize, serde::Serialize)]\npub struct Profile {\n");
    for column in &model.columns {
        // Names are validated above, so no arbitrary identifier/source interpolation is possible.
        contract.push_str(&format!(
            "    /// Public {} field.\n    pub {}: String,\n",
            column.name, column.name
        ));
    }
    contract.push_str("}\n");
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
    let mut source = format!(
        "//! Generated module registry; regenerate instead of editing.\n\n\
         const _: () = assert!({}crate::selection::ENABLED);\n\
         const _: () = assert!(crate::selection::REVISION == {});\n\n",
        if selection::ENABLED { "" } else { "!" },
        selection::REVISION,
    );
    let present = selection::ENABLED && schema.model.contains_key("Profile");
    if present {
        source.push_str("/// Public data without host dependencies.\npub mod contracts {\n");
        source.push_str("    /// Model Profile.\n    pub mod profile;\n}\n");
        source.push_str("/// Native capability implementation.\npub mod server {\n");
        source.push_str("    /// In-memory profiles plugin.\n    pub mod profile;\n}\n\n");
    }
    source.push_str("/// Register the selected generated contributions.\n");
    source.push_str("///\n/// # Errors\n/// Returns plugin/host registration failures.\n");
    source.push_str("pub fn register(app: &mut stackpress_server::ServerBuilder) ");
    source.push_str("-> stackpress_lib::Result<()> {\n");
    // Removing the model removes both the module declarations and their registration references.
    if present {
        source.push_str("    app.plugin(\"profiles\", server::profile::register)?;\n");
    } else {
        source.push_str("    let _ = app;\n");
    }
    source.push_str("    Ok(())\n}\n");
    Ok(vec![GeneratedFile::new("mod.rs", source)])
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
