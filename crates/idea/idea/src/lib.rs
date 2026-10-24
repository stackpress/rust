//! Small generation orchestration over independently usable parser and transformer crates.
//!
//! This synchronous build-host API composes imports through a caller-selected source loader,
//! resolves typed references, and plans selected function transforms before touching output.
//! The caller owns environment snapshots, limits, registrations and the dedicated generated root.
//! No executor, listener, process-environment read or second plugin system is introduced.

pub use stackpress_idea_parser as parser;
pub use stackpress_idea_transformer as transformer;

use parser::CompileInputs;
use transformer::{
    CheckReport, Composer, CompositionLimits, GeneratedFile, OutputWorkspace, Registry, Result,
    SourceLoader, TransformSelection,
};

/// Choose whether generation only compares output or installs the completed plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationMode {
    /// Compare formatted output without changing generated files or their ownership manifest.
    Check,
    /// Install formatted output using the workspace's cooperative ownership/recovery contract.
    Install,
}

/// Explicit inputs and app-owned aggregation for one finite generation run.
///
/// The selected registry and output workspace remain independently owned arguments to
/// [`generate`]. Transform callbacks and this aggregate function receive the same finalized
/// config: imports/props are consumed, actual references are resolved and literal strings survive.
pub struct GenerateRequest<'a> {
    /// Entry specifier resolved by the supplied loader, independent of the output root.
    pub entry: &'a str,
    /// Explicit environment snapshot and parser/finalization budgets.
    pub inputs: CompileInputs,
    /// Acquisition, import traversal and cumulative composition budgets.
    pub limits: CompositionLimits,
    /// Runtime contract metadata and declared auxiliary snapshots for production planning.
    pub selection: TransformSelection,
    /// App-owned root-file renderer, run after finalization and before registry planning.
    pub aggregates: fn(&parser::SchemaConfig) -> Result<Vec<GeneratedFile>>,
    /// The single check-versus-install decision for this run.
    pub mode: GenerationMode,
}

impl<'a> GenerateRequest<'a> {
    /// Use finite default budgets, no environment/auxiliary inputs, and explicit output mode.
    pub fn new(
        entry: &'a str,
        aggregates: fn(&parser::SchemaConfig) -> Result<Vec<GeneratedFile>>,
        mode: GenerationMode,
    ) -> Self {
        // Ordinary owned options are per-run policy, never a shared registry or mutable cache.
        Self {
            entry,
            inputs: CompileInputs::default(),
            limits: CompositionLimits::default(),
            selection: TransformSelection::new(),
            aggregates,
            mode,
        }
    }
}

/// Compose, finalize, plan and then compare or install one generated application.
///
/// Loading, reference resolution, aggregation and transform planning complete before workspace
/// IO begins. Source diagnostics retain typed causes and original imported-file coordinates.
/// The source loader's root is a resolution policy, not a sandbox. Callbacks are trusted Rust;
/// their own direct IO is outside the output workspace's managed-write guarantees.
///
/// # Examples
/// ```no_run
/// use stackpress_idea::{generate, GenerateRequest, GenerationMode, parser, transformer};
/// use transformer::{GeneratedFile, OutputWorkspace, ProjectLoader, Registry};
/// fn aggregate(_: &parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>> {
///     // Empty selections can still produce an app-owned root module.
///     Ok(vec![GeneratedFile::new("mod.rs", "//! Generated registry.\n")])
/// }
/// let loader = ProjectLoader::new("/project")?;
/// let workspace = OutputWorkspace::new("/project/src/generated")?;
/// let request = GenerateRequest::new("schema.idea", aggregate, GenerationMode::Check);
/// let report = generate(request, loader, &Registry::new(), &workspace)?;
/// assert!(report.is_current());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// Returns source/composition/finalization, callback/preflight, ownership, formatting, IO or
/// recovery errors without erasing their cause chains. Pre-install failures leave output intact.
/// Check mode acquires a transient writer lock but changes no generated files or manifest.
/// Callback panics propagate; trusted loaders/renderers are responsible for their own side effects.
pub fn generate(
    request: GenerateRequest<'_>,
    loader: impl SourceLoader,
    registry: &Registry,
    workspace: &OutputWorkspace,
) -> Result<CheckReport> {
    // Environment expansion finishes during composition. Finalization needs only value limits;
    // move the snapshot into the composer rather than cloning it before parser budget checks.
    let finalization = CompileInputs {
        limits: request.inputs.limits,
        ..Default::default()
    };
    let composed = Composer::new(loader, request.limits)
        .with_inputs(request.inputs)
        .compose(request.entry)?;
    // Resolve actual reference sites after all import precedence is settled, preserving literals.
    let finalized =
        parser::finalize_with_references(&composed.config, &composed.references, &finalization)?;
    // Registry callbacks use SchemaConfig; move finalized sections without JSON roundtripping.
    let schema = parser::SchemaConfig {
        plugin: finalized.plugin,
        r#enum: finalized.r#enum,
        r#type: finalized.r#type,
        model: finalized.model,
        ..Default::default()
    };
    // App aggregation is pure rendering; the registry then preflights every selected transform.
    let plan = registry.plan_with(&schema, &request.selection, (request.aggregates)(&schema)?)?;
    // Both modes share composition/planning/formatting; checking cannot silently repair drift.
    match request.mode {
        GenerationMode::Check => workspace.check(&plan),
        GenerationMode::Install => workspace.install(plan),
    }
}
