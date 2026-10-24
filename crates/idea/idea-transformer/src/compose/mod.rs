//! Finite per-run import composition with immutable child snapshots and effective provenance.
//!
//! This layer returns non-final config and actual typed reference sites. Call the parser's
//! `finalize_with_references` on the completed result to resolve only those sites. Literal
//! strings such as `${Name}` remain ordinary data throughout import composition.

mod compile;
mod json;
mod merge;

use std::collections::{BTreeMap, HashMap};
use std::io::{self, Write};

use stackpress_idea_parser::{CompileInputs, Reference, SchemaConfig};

use compile::compile_source;

use crate::{LoadedSource, Result, SourceId, SourceLoader, TransformError};

/// Lowerable composition budgets; all counters reset for every [`Composer::compose`] call.
#[derive(Debug, Clone, Copy)]
pub struct CompositionLimits {
    /// Maximum bytes acquired from any single source.
    pub max_source_bytes: usize,
    /// Total acquired bytes, including repeated alias/diamond reads used to confirm identity.
    pub max_total_bytes: usize,
    /// Maximum distinct canonical inputs in the source map.
    pub max_files: usize,
    /// Maximum active import depth including the root; capped at [`Self::HARD_DEPTH`].
    pub max_depth: usize,
    /// Maximum import edges visited, including repeated edges in distinct parents.
    pub max_imports: usize,
    /// Cumulative config/reference bytes charged before publishing or copying snapshots.
    /// This bounds diamond expansion work as well as retained config; collisions still charge.
    pub max_expanded_bytes: usize,
    /// Maximum JSON value nodes per normalized input, counted before deserializing each node.
    pub max_json_nodes: usize,
}

impl CompositionLimits {
    /// Maximum supported native recursive composition depth, even with caller-raised limits.
    pub const HARD_DEPTH: usize = 64;
}

impl Default for CompositionLimits {
    /// Supply finite development defaults independent of process environment.
    fn default() -> Self {
        Self {
            max_source_bytes: 1024 * 1024,
            max_total_bytes: 16 * 1024 * 1024,
            max_files: 256,
            max_depth: 32,
            max_imports: 4096,
            max_expanded_bytes: 16 * 1024 * 1024,
            max_json_nodes: 100_000,
        }
    }
}

/// One resolved import in traversal order; a diamond has distinct incoming edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEdge {
    /// Canonical importing file.
    pub importer: SourceId,
    /// Literal specifier, retained without resolving against a later caller cwd.
    pub specifier: String,
    /// Canonical imported file.
    pub imported: SourceId,
}

/// Owned composition result, including source snapshots and the origin of every surviving part.
///
/// Config imports are consumed; props and unresolved references remain non-final. Child plugin
/// declarations are inert and never enter the root plugin selection. Cloning copies ordinary
/// data deeply. Source snapshots may contain secrets and are for deliberate developer access.
/// Finalization is explicit and uses the parser's checked reference-aware boundary; it does not
/// load files or reinterpret placeholder-looking literal strings.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::{CompileInputs, finalize_with_references};
/// use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};
/// let mut loader = MemoryLoader::new("/project")?;
/// loader.insert("root.idea", "use \"child\"");
/// loader.insert("child.idea", "prop Name { label \"resolved\" } \
///     model Read @value(Name \"${Name}\") { id String }");
/// let composed = Composer::new(loader, CompositionLimits::default()).compose("root")?;
/// let schema = finalize_with_references(&composed.config, &composed.references,
///     &CompileInputs::default())?;
/// let json = serde_json::to_value(schema)?;
/// assert_eq!(json["model"]["Read"]["attributes"]["value"],
///     serde_json::json!([{"label":"resolved"}, "${Name}"]));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
pub struct ComposedSchema {
    /// Effective ordered non-final configuration.
    pub config: SchemaConfig,
    /// Unique canonical inputs in first-acquisition DFS order, root first.
    pub sources: Vec<LoadedSource>,
    /// Import edges in DFS encounter order, including diamond edges.
    pub imports: Vec<ImportEdge>,
    /// Origins keyed by structural paths; declaration roots, attributes and whole columns
    /// receive separate entries so a mutable merge retains precise winning provenance.
    pub provenance: BTreeMap<Vec<String>, SourceId>,
    /// Actual parser references with their original source spans and source-local forward flags.
    /// A typed JSON envelope can retain an original Idea filename; `provenance` and `imports`
    /// separately identify the JSON artifact through which that declaration was acquired.
    pub references: Vec<Reference>,
}

/// Immutable configuration and source adapter for independent synchronous composition runs.
pub struct Composer<L> {
    /// Caller-owned adapter; no mutable cache survives a compose call.
    loader: L,
    /// Acquisition, traversal and expansion policy for each run.
    limits: CompositionLimits,
    /// Explicit environment and parser bounds; never read from process globals.
    inputs: CompileInputs,
}

impl<L: SourceLoader> Composer<L> {
    /// Construct a composer with finite import budgets and default pure parser inputs.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};
    /// let mut loader = MemoryLoader::new("/project")?;
    /// loader.insert("schema.idea", "use \"./base\" model User { name String }");
    /// loader.insert("base.idea", "model User { id String }");
    /// let result = Composer::new(loader, CompositionLimits::default()).compose("schema")?;
    /// assert_eq!(result.config.model["User"].columns[0].name, "id");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(loader: L, limits: CompositionLimits) -> Self {
        Self {
            loader,
            limits,
            inputs: CompileInputs::default(),
        }
    }

    /// Supply an explicit per-run environment snapshot and lowerable parser/data limits.
    pub fn with_inputs(mut self, inputs: CompileInputs) -> Self {
        self.inputs = inputs;
        self
    }

    /// Compose an entry and its importer-relative closure with no writes or cwd changes.
    ///
    /// Every call makes fresh owned snapshots. Resolution roots are not security boundaries;
    /// the loader decides which files may be imported. G19 deliberately replaces an immutable
    /// accumulated local with its imported child, even though reference 00022 says otherwise.
    ///
    /// # Errors
    /// Returns contextual parser/JSON/IO failures, active-stack cycles, changed canonical
    /// snapshots or exhausted budgets. `Error::source` retains the original typed cause.
    pub fn compose(&self, entry: &str) -> Result<ComposedSchema> {
        let mut run = Run::new(self);
        let root = run.visit(None, entry)?;
        // The root is complete; move it out rather than cloning an already charged snapshot.
        let snapshot = run
            .cache
            .remove(&root)
            .ok_or_else(|| TransformError::new("completed composition snapshot missing"))?;
        Ok(ComposedSchema {
            config: snapshot.config,
            sources: run.sources,
            imports: run.imports,
            provenance: snapshot.provenance,
            references: snapshot.references,
        })
    }
}

/// One completed file and its effective declaration metadata, never mutated as an import.
#[derive(Debug, Default)]
struct Snapshot {
    /// Ordered effective schema for this file's closure.
    config: SchemaConfig,
    /// Origins travel with declaration roots/attributes/whole columns.
    provenance: BTreeMap<Vec<String>, SourceId>,
    /// Typed-reference payload travels through exactly the same subtree operations.
    references: Vec<Reference>,
}

/// Mutable state belongs to one compose call; cached children are borrowed immutably.
struct Run<'a, L> {
    /// Loader and immutable policy shared only during this synchronous call.
    composer: &'a Composer<L>,
    /// Completed canonical source closures; active sources never appear here.
    cache: HashMap<SourceId, Snapshot>,
    /// Active identities with logical names for complete cycle diagnostics.
    active: Vec<(SourceId, String)>,
    /// First-acquisition snapshots for developer diagnostics and input receipts.
    sources: Vec<LoadedSource>,
    /// Identity lookup into the unique source snapshots.
    positions: HashMap<SourceId, usize>,
    /// Encounter-ordered graph including repeated incoming diamond edges.
    imports: Vec<ImportEdge>,
    /// Remaining acquisition budget, charged on every loader result.
    remaining_bytes: usize,
    /// Remaining config-copy work budget, charged before every merge or snapshot retention.
    remaining_expansion: usize,
}

impl<'a, L: SourceLoader> Run<'a, L> {
    /// Reset counters and state so failed/prior calls cannot poison future compositions.
    fn new(composer: &'a Composer<L>) -> Self {
        Self {
            composer,
            cache: HashMap::new(),
            active: Vec::new(),
            sources: Vec::new(),
            positions: HashMap::new(),
            imports: Vec::new(),
            remaining_bytes: composer.limits.max_total_bytes,
            remaining_expansion: composer.limits.max_expanded_bytes,
        }
    }

    /// Acquire a bounded snapshot, detect active cycles and compose each canonical file once.
    fn visit(&mut self, importer: Option<&SourceId>, specifier: &str) -> Result<SourceId> {
        let loaded = self.acquire(importer, specifier)?;
        let id = loaded.id.clone();
        // Completed diamond targets share this call's immutable snapshot, never an active frame.
        if let Some(position) = self.positions.get(&id) {
            // Detect inconsistent custom/native snapshots rather than silently using stale data.
            if self.sources[*position] != loaded {
                return Err(TransformError::new(
                    "canonical source changed during composition",
                ));
            }
            return Ok(id);
        }
        if self.sources.len() >= self.composer.limits.max_files {
            return Err(TransformError::new("canonical input file budget exceeded"));
        }
        let mut snapshot = compile_source(&loaded, &self.composer.inputs, self.composer.limits)?;
        charge(
            &snapshot.config,
            &snapshot.references,
            &mut self.remaining_expansion,
        )?;
        let uses = std::mem::take(&mut snapshot.config.r#use);
        self.active.push((id.clone(), loaded.name.clone()));
        self.positions.insert(id.clone(), self.sources.len());
        self.sources.push(loaded);
        // Each direct import merges sequentially; child snapshots are never edited by a parent.
        // A failure discards this private Run, so its partially populated active stack cannot
        // poison a subsequent compose call. Only the successful path publishes its cache entry.
        for specifier in uses {
            let importer_name = self
                .active
                .last()
                .map(|(_, name)| name.clone())
                .unwrap_or_else(|| "entry".into());
            let child = self
                .visit(Some(&id), &specifier)
                .map_err(|cause| context(format!("import from {importer_name} failed"), cause))?;
            let child = self
                .cache
                .get(&child)
                .ok_or_else(|| TransformError::new("completed imported snapshot missing"))?;
            charge(
                &child.config,
                &child.references,
                &mut self.remaining_expansion,
            )?;
            merge::merge(&mut snapshot, child);
        }
        self.active.pop();
        self.cache.insert(id.clone(), snapshot);
        Ok(id)
    }

    /// Acquire and charge one edge before testing whether its canonical target was completed.
    fn acquire(&mut self, importer: Option<&SourceId>, specifier: &str) -> Result<LoadedSource> {
        let limits = self.composer.limits;
        // Check depth/edges before loading another potentially large source.
        if self.active.len() >= limits.max_depth.min(CompositionLimits::HARD_DEPTH) {
            return Err(TransformError::new("import depth budget exceeded"));
        }
        if importer.is_some() && self.imports.len() >= limits.max_imports {
            return Err(TransformError::new("import edge budget exceeded"));
        }
        let budget = limits.max_source_bytes.min(self.remaining_bytes);
        let loaded = self
            .composer
            .loader
            .load(importer, specifier, budget)
            .map_err(|cause| context("source acquisition failed", cause))?;
        // Defend against custom adapters returning more text than the negotiated budget.
        if loaded.text.len() > budget {
            return Err(TransformError::new("source adapter exceeded byte budget"));
        }
        self.remaining_bytes -= loaded.text.len();
        let id = loaded.id.clone();
        if let Some(importer) = importer {
            self.imports.push(ImportEdge {
                importer: importer.clone(),
                specifier: specifier.into(),
                imported: id.clone(),
            });
        }
        // A completed diamond node is reusable; only an identity on the active path cycles.
        if self.active.iter().any(|(active, _)| active == &id) {
            let chain = self
                .active
                .iter()
                .map(|(_, name)| name.as_str())
                .chain(std::iter::once(loaded.name.as_str()))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(TransformError::new(format!("import cycle: {chain}")));
        }
        Ok(loaded)
    }
}

/// Preserve typed causes while public Display supplies only the current safe phase context.
fn context(
    message: impl Into<String>,
    cause: impl std::error::Error + Send + Sync + 'static,
) -> TransformError {
    TransformError {
        message: message.into(),
        cause: Some(Box::new(cause)),
    }
}

/// Charge config and actual reference metadata before a snapshot is retained or copied.
fn charge(config: &SchemaConfig, references: &[Reference], remaining: &mut usize) -> Result<()> {
    // The writer refuses excess bytes as serialization proceeds, before cloning a child.
    let mut counter = Counter {
        remaining: *remaining,
    };
    serde_json::to_writer(&mut counter, config)
        .map_err(|cause| context("composed config expansion budget exceeded", cause))?;
    // Real reference payloads are copied with their owner; charge them before the merge too.
    if !references.is_empty() {
        serde_json::to_writer(&mut counter, references)
            .map_err(|cause| context("composed reference expansion budget exceeded", cause))?;
    }
    *remaining = counter.remaining;
    Ok(())
}

/// Nonallocating serialization sink implementing the cumulative expansion budget.
struct Counter {
    /// Remaining bytes, decremented only after a checked write succeeds.
    remaining: usize,
}

impl Write for Counter {
    /// Reject a write that would exhaust the declared bound without wrapping arithmetic.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining = self
            .remaining
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::other("config expansion budget exceeded"))?;
        Ok(bytes.len())
    }

    /// No buffered bytes or host IO exist to flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
