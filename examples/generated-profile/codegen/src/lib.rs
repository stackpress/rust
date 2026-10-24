//! Independently compiled Profile codegen tool; never depends on the generated app.
//!
//! The deliberately small generator accepts only id/name String fields. It owns demo search
//! semantics rather than assigning framework meaning to arbitrary Idea metadata.

#[path = "../../selection.rs"]
pub mod selection;

use std::error::Error as StdError;
use std::path::Path;

use stackpress_idea::parser::{CompileInputs, SchemaConfig};
use stackpress_idea::transformer::{
    CheckReport, GeneratedFile, OutputWorkspace, Registry, Result as GenerateResult,
    TransformError, read_source,
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

/// Register the example transform through an ordinary selected function.
///
/// # Errors
/// Returns registry ID/partition conflicts before any callbacks execute.
pub fn register(registry: &mut Registry) -> GenerateResult<()> {
    // Disabled builds contain no generator callback; schema blocks cannot activate it.
    #[cfg(feature = "profiles")]
    registry.register("profile-demo", &["contracts", "server"], emit_profile)?;
    let _ = registry;
    Ok(())
}

/// Emit real typed public fields and the existing proved search implementation.
#[cfg(feature = "profiles")]
fn emit_profile(schema: &SchemaConfig) -> GenerateResult<Vec<GeneratedFile>> {
    // This generator has no configuration knobs; accepting them would imply false semantics.
    if schema.plugin.values().any(|config| !config.is_empty()) {
        return Err(TransformError::new(
            "profile-demo accepts no plugin configuration",
        ));
    }
    // Removing the model deliberately contributes no contracts or runtime implementation.
    if schema.model.is_empty() {
        return Ok(Vec::new());
    }
    let model = schema
        .model
        .get("Profile")
        .ok_or_else(|| TransformError::new("profile-demo supports only the Profile model"))?;
    // This explicit demo mapping cannot silently turn unsupported metadata into false promises.
    if model.columns.len() != 2
        || !model.attributes.is_empty()
        || model
            .columns
            .iter()
            .zip(["id", "name"])
            .any(|(column, name)| {
                column.name != name
                    || column.column_type != "String"
                    || !column.required
                    || column.multiple
                    || !column.attributes.is_empty()
            })
        || schema.model.len() != 1
    {
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
    Ok(vec![
        GeneratedFile::new("contracts/profile.rs", contract),
        GeneratedFile::new("server/profile.rs", include_str!("profile.rs.txt")),
    ])
}

/// Own the aggregate registry and compile-time selection/revision checks.
fn aggregate(schema: &SchemaConfig) -> GenerateResult<Vec<GeneratedFile>> {
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

/// Generate/check one explicit schema and root, independent of the final application's build.
///
/// # Errors
/// Returns parser, unsupported-shape, formatting, output drift, lock or recovery errors.
pub fn generate(input: &Path, output: &Path, check: bool) -> GenerateResult<CheckReport> {
    let inputs = CompileInputs::default();
    let source = read_source(input, inputs.limits.max_bytes)?;
    let mut registry = Registry::new();
    register(&mut registry)?;
    let workspace = OutputWorkspace::new(output)?;
    stackpress_idea::generate(
        input
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("schema.idea"),
        &source,
        &inputs,
        &registry,
        aggregate,
        &workspace,
        check,
    )
}

/// Disclose parser-owned safe coordinates while retaining internal/IO causes privately.
fn generation_error(error: TransformError) -> Error {
    // A typed cause check prevents an arbitrary plugin/OS message becoming public diagnostics.
    let mut cause: Option<&(dyn StdError + 'static)> = Some(&error);
    for _ in 0..32 {
        let Some(current) = cause else { break };
        if let Some(diagnostic) = current.downcast_ref::<stackpress_idea::parser::IdeaError>() {
            return Error::invalid(diagnostic.to_string()).with_cause(error);
        }
        // A finite walk also protects the adapter from a malformed cyclic plugin error chain.
        cause = current.source();
    }
    Error::from_source(error)
}

/// Read capability data once, then await the owned blocking worker on the tool runtime.
async fn generate_event(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../app")
        .canonicalize()
        .map_err(Error::from_source)?;
    let input = req
        .data
        .path_as::<String>("input")?
        .map(Into::into)
        .unwrap_or_else(|| root.join("schema.idea"));
    let output = req
        .data
        .path_as::<String>("output")?
        .map(Into::into)
        .unwrap_or_else(|| root.join("src/generated"));
    // Presence wins over aliases, including an explicit false in the long form.
    let check = match req.data.path_as::<bool>("check")? {
        Some(check) => check,
        None => req.data.path_as::<bool>("c")?.unwrap_or(false),
    };
    // The worker owns its paths and writer lock. Runtime shutdown waits for blocking work;
    // cancelling this future does not release a lock while the worker can still write.
    let report = tokio::task::spawn_blocking(move || generate(&input, &output, check))
        .await
        .map_err(Error::from_source)?
        .map_err(generation_error)?;
    if check && !report.is_current() {
        return Err(Error::invalid(
            "generated output is stale; run emit generate",
        ));
    }
    res.results(&report)?;
    Ok(Flow::Continue)
}

/// Build a generation-only host with no generated-app or HTTP adapter dependency.
///
/// # Errors
/// Returns configuration, event registration or startup failures.
pub async fn build_tool(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on("generate", generate_event)?;
    app.build().await
}
