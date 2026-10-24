//! Independently compiled Profile codegen tool; never depends on the generated app.
//!
//! The deliberately small generator accepts only id/name String fields. It owns demo search
//! semantics rather than assigning framework meaning to arbitrary Idea metadata.

#[path = "../../selection.rs"]
pub mod selection;

mod profile;
pub use profile::register;

use std::error::Error as StdError;
use std::path::Path;

use stackpress_idea::transformer::{
    CheckReport, CompositionLimits, OutputWorkspace, ProjectLoader, Registry,
    Result as GenerateResult, TransformError,
};
use stackpress_idea::{GenerateRequest, GenerationMode};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

/// Generate/check one explicit schema and root, independent of the final application's build.
///
/// Relative input paths resolve from the caller's working directory. The ProjectLoader's root
/// stays anchored to the app for project aliases; ordinary imports resolve at their own files.
/// Explicit output paths retain the workspace's marked-root ownership requirements.
///
/// # Errors
/// Returns source/import/reference, unsupported-shape, formatting, drift, lock or recovery errors.
pub fn generate(input: &Path, output: &Path, check: bool) -> GenerateResult<CheckReport> {
    // Keep explicit caller paths relative to the caller's cwd; imports resolve at each source.
    let input = std::path::absolute(input)?;
    let entry = input
        .to_str()
        .ok_or_else(|| TransformError::new("schema path must be UTF-8"))?;
    let app = Path::new(env!("CARGO_MANIFEST_DIR")).join("../app");
    let loader = ProjectLoader::new(app).map_err(|cause| {
        // Preserve native root-resolution causes without disclosing arbitrary OS paths publicly.
        TransformError::from(std::io::Error::other(cause))
    })?;
    let mut registry = Registry::new();
    register(&mut registry)?;
    let workspace = OutputWorkspace::new(output)?;
    let mode = if check {
        GenerationMode::Check
    } else {
        GenerationMode::Install
    };
    let mut request = GenerateRequest::new(entry, profile::aggregate, mode);
    // The model module owns its declared assets; orchestration only forwards their snapshot.
    request.selection = profile::transform_selection()?;
    stackpress_idea::generate(request, loader, &registry, &workspace)
}

/// Disclose parser-owned safe coordinates while retaining internal/IO causes privately.
fn generation_error(error: TransformError) -> Error {
    // A typed cause check prevents an arbitrary plugin/OS message becoming public diagnostics.
    let mut cause: Option<&(dyn StdError + 'static)> = Some(&error);
    // One contextual cause per import must still leave room for the leaf parser diagnostic.
    for _ in 0..=CompositionLimits::HARD_DEPTH + 4 {
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
    // Host setup remains independent of the generated application and its selected registry.
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on("generate", generate_event)?;
    app.build().await
}
