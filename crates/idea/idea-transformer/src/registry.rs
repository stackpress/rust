//! Sequential function transforms with explicit preflight and exclusive output partitions.

use std::cell::RefCell;
use std::collections::BTreeSet;

use stackpress_idea_parser::SchemaConfig;

pub(crate) use crate::plan::valid_path;
use crate::{Result, TransformError};

mod context;
pub use crate::plan::OutputPlan;
pub use context::{TransformContext, TransformInputs, TransformSelection};

/// One inspectable UTF-8 source file claimed by a transform or aggregate owner.
#[derive(Debug)]
pub struct GeneratedFile {
    /// Validated relative slash-separated path beneath the generated root.
    pub path: String,
    /// Complete new file contents; transforms never patch prior files.
    pub contents: String,
}

impl GeneratedFile {
    /// Construct a proposed output; planning validates ownership, path and byte limits.
    pub fn new(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
        }
    }
}

/// M0 synchronous callback, retained for independently compiled consumers.
pub type Transform = fn(&SchemaConfig) -> Result<Vec<GeneratedFile>>;

/// Synchronous named callback with checked ownership and explicit immutable inputs.
pub type ContextTransform = fn(&TransformContext<'_>, &mut OutputPlan) -> Result<()>;

/// Compiled plugin metadata checked as a complete selection before callbacks execute.
#[derive(Debug)]
pub struct TransformSpec {
    /// Unique ID, independent of the function's Rust symbol.
    id: String,
    /// Exclusive first-component output partitions.
    partitions: Vec<String>,
    /// Required IDs must exist and precede this registration.
    requires: Vec<String>,
    /// Optional IDs need precede this registration only when selected.
    optional: Vec<String>,
    /// Exact app-owned runtime contract revisions required by this generator.
    contracts: Vec<(String, String)>,
}

impl TransformSpec {
    /// Describe one compiled plugin; validation occurs atomically in [`Registry::add`].
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            partitions: Vec::new(),
            requires: Vec::new(),
            optional: Vec::new(),
            contracts: Vec::new(),
        }
    }

    /// Declare exclusive single-component partitions; an empty list permits contribution-only work.
    pub fn owns(mut self, partitions: &[&str]) -> Self {
        self.partitions = partitions.iter().map(|value| (*value).into()).collect();
        self
    }

    /// Require these generators earlier in the listed selection, without auto-reordering.
    pub fn requires(mut self, ids: &[&str]) -> Self {
        self.requires = ids.iter().map(|value| (*value).into()).collect();
        self
    }

    /// Require earlier ordering only for these optional generators that are actually selected.
    pub fn after_if_present(mut self, ids: &[&str]) -> Self {
        self.optional = ids.iter().map(|value| (*value).into()).collect();
        self
    }

    /// Require an exact app-owned runtime contract revision supplied to [`Registry::plan_with`].
    pub fn requires_contract(mut self, id: &str, revision: &str) -> Self {
        self.contracts.push((id.into(), revision.into()));
        self
    }

    /// Bound prerequisite/contract metadata before the registry records any registration.
    fn validate_requirements(&self) -> Result<()> {
        // Required and optional lists share syntax/duplicate checks, but not presence policy.
        for requirements in [&self.requires, &self.optional] {
            if requirements.len() > 1024 {
                return Err(TransformError::new("prerequisite budget"));
            }
            for (index, id) in requirements.iter().enumerate() {
                if id.is_empty() || id.len() > 128 || requirements[..index].contains(id) {
                    return Err(TransformError::new("invalid or duplicate prerequisite"));
                }
            }
        }
        if self.contracts.len() > 1024 || self.partitions.len() > 1024 {
            return Err(TransformError::new("transform metadata budget"));
        }
        for (index, (id, revision)) in self.contracts.iter().enumerate() {
            context::valid_contract(id, revision)?;
            if self.contracts[..index].iter().any(|(prior, _)| prior == id) {
                return Err(TransformError::new("duplicate required runtime contract"));
            }
        }
        Ok(())
    }
}

/// Either frozen M0 callbacks or the additive contextual function API.
enum Callback {
    /// Original schema-to-files callback.
    Legacy(Transform),
    /// Contextual callback writing fresh claims into the bounded plan.
    Context(ContextTransform),
}

/// One selected compiled function and its explicit requirements.
struct Registration {
    /// Metadata belongs to registration, never to arbitrary strings in imported sources.
    spec: TransformSpec,
    /// No trait implementation, dynamic loader or asynchronous runtime is required.
    callback: Callback,
}

/// Builder-time selected transforms; arbitrary schema strings never load executable code.
#[derive(Default)]
pub struct Registry {
    /// Insertion order is execution order; no sorting/reordering is performed.
    registrations: Vec<Registration>,
}

/// Explicit name for the registry, retaining the frozen [`Registry`] entry point.
pub type TransformRegistry = Registry;

#[cfg(test)]
mod tests;

impl Registry {
    /// Start an empty compiled selection; empty schemas permit aggregate-only plans.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an M0 function with exclusive, single-component partitions.
    ///
    /// Each declared partition must be nonempty. An empty list is valid for functions that
    /// emit no files; legacy functions still run in the same listed order as contextual ones.
    ///
    /// # Errors
    /// Rejects duplicate IDs, overlapping/invalid partitions and an empty ID without mutation.
    pub fn register(&mut self, id: &str, partitions: &[&str], transform: Transform) -> Result<()> {
        self.insert(
            TransformSpec::new(id).owns(partitions),
            Callback::Legacy(transform),
        )
    }

    /// Add an ordinary named contextual function, preserving the supplied registration order.
    ///
    /// # Errors
    /// Rejects invalid metadata and duplicate ID/partition ownership atomically. Prerequisite
    /// presence/order and runtime-contract matching are checked during planning, before effects.
    ///
    /// ```
    /// use stackpress_idea_transformer::{OutputPlan, Result, TransformContext};
    /// use stackpress_idea_transformer::{TransformRegistry, TransformSpec};
    /// fn emit(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    ///     plan.write("contracts/profile.rs", "pub struct Profile;\n")?;
    ///     plan.contribute_module("contracts", "profile")
    /// }
    /// let mut registry = TransformRegistry::new();
    /// registry.add(TransformSpec::new("profile").owns(&["contracts"]), emit)?;
    /// let plan = registry.plan(&Default::default(), vec![])?;
    /// assert_eq!(plan.executed(), &["profile"]);
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn add(&mut self, spec: TransformSpec, transform: ContextTransform) -> Result<()> {
        self.insert(spec, Callback::Context(transform))
    }

    /// Validate a whole registration before granting it output authority.
    fn insert(&mut self, spec: TransformSpec, callback: Callback) -> Result<()> {
        valid_path(&spec.id)?;
        if spec.id.is_empty()
            || spec.id.len() > 128
            || spec.id == "aggregate"
            || self.registrations.len() >= 1024
            || self
                .registrations
                .iter()
                .any(|item| item.spec.id == spec.id)
        {
            return Err(TransformError::new(
                "empty, reserved or duplicate generator ID or budget",
            ));
        }
        // Preserve validation precedence while keeping ownership and metadata checks separate.
        self.validate_partitions(&spec)?;
        spec.validate_requirements()?;
        self.registrations.push(Registration { spec, callback });
        Ok(())
    }

    /// Reject partition aliases before a failed registration can reserve any ownership.
    fn validate_partitions(&self, spec: &TransformSpec) -> Result<()> {
        // Partition case aliases are conflicts even when this machine is case-sensitive.
        for (index, part) in spec.partitions.iter().enumerate() {
            valid_path(part)?;
            if part.contains('/')
                || spec.partitions[..index]
                    .iter()
                    .any(|prior| prior.eq_ignore_ascii_case(part))
                || self.registrations.iter().any(|item| {
                    item.spec
                        .partitions
                        .iter()
                        .any(|prior| prior.eq_ignore_ascii_case(part))
                })
            {
                return Err(TransformError::new("generator partition conflict"));
            }
        }
        Ok(())
    }

    /// Execute after full preflight using no auxiliary inputs or runtime contract declarations.
    ///
    /// With no schema plugin blocks the compiled registry supplies defaults, including an
    /// empty selection producing aggregate-only output. Otherwise blocks must match the exact
    /// selection and order; a declared plugin cannot be silently ignored.
    ///
    /// | Root plugin blocks | Compiled registry | Behavior |
    /// | --- | --- | --- |
    /// | None | Nonempty | Run registered transforms using defaults. |
    /// | None | Empty | Produce aggregate-only output. |
    /// | Present | Any | Require exact IDs and listed order before callbacks. |
    ///
    /// # Errors
    /// Callback, selection, collision, path or budget failures produce no disk writes.
    pub fn plan(
        &self,
        schema: &SchemaConfig,
        aggregates: Vec<GeneratedFile>,
    ) -> Result<OutputPlan> {
        self.plan_with(schema, &TransformSelection::default(), aggregates)
    }

    /// Plan with explicit immutable auxiliary snapshots and runtime contract metadata.
    ///
    /// All selection/dependency/contract checks precede every callback. Callbacks run once in
    /// listed order. A callback panic propagates; there is no promise to recover arbitrary
    /// plugin panics. No output workspace is touched until a completed plan is installed.
    ///
    /// # Errors
    /// Returns preflight or callback failures, preserving contextual callback errors as causes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_idea_transformer::{OutputPlan, Registry, Result, TransformContext};
    /// use stackpress_idea_transformer::{TransformInputs, TransformSelection, TransformSpec};
    /// fn emit(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    ///     plan.write("data/model.txt", context.input("templates/model.txt")?)
    /// }
    /// let mut inputs = TransformInputs::new();
    /// inputs.insert("templates/model.txt", "model body")?;
    /// let selection = TransformSelection::new().with_inputs(inputs);
    /// let mut registry = Registry::new();
    /// registry.add(TransformSpec::new("model").owns(&["data"]), emit)?;
    /// let plan = registry.plan_with(&Default::default(), &selection, vec![])?;
    /// assert_eq!(plan.inputs(), &["templates/model.txt"]);
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn plan_with(
        &self,
        schema: &SchemaConfig,
        selection: &TransformSelection,
        aggregates: Vec<GeneratedFile>,
    ) -> Result<OutputPlan> {
        self.preflight(schema, selection)?;
        let mut plan = OutputPlan::default();
        // Only the app's aggregate owner can claim root-level files.
        for file in aggregates {
            if file.path.contains('/')
                || self.registrations.iter().any(|item| {
                    item.spec
                        .partitions
                        .iter()
                        .any(|part| part.eq_ignore_ascii_case(&file.path))
                })
            {
                return Err(TransformError::new(
                    "invalid aggregate path or partition collision",
                ));
            }
            plan.insert("aggregate", file)?;
        }
        // The access log is run-local; context accessors release every borrow before returning.
        let used = RefCell::new(BTreeSet::new());
        for item in &self.registrations {
            let spec = &item.spec;
            plan.begin(&spec.id, &spec.partitions);
            match item.callback {
                Callback::Legacy(transform) => {
                    // Legacy callbacks still use the same exact partition and collision checks.
                    for file in transform(schema)? {
                        plan.write(file.path, file.contents)?;
                    }
                }
                Callback::Context(transform) => {
                    let context = TransformContext {
                        schema,
                        id: &spec.id,
                        selection,
                        used: &used,
                    };
                    transform(&context, &mut plan).map_err(|error| TransformError {
                        message: format!("transform callback failed for {}", spec.id),
                        cause: Some(Box::new(error)),
                    })?;
                }
            }
            // Close authority before recording completion or invoking the next plugin.
            plan.end();
            plan.executed.push(spec.id.clone());
        }
        // BTreeSet order exposes consumed names deterministically without retaining their bytes.
        plan.inputs = used.into_inner().into_iter().collect();
        plan.finish_modules()?;
        Ok(plan)
    }

    /// Validate exact root selection and every required/optional dependency before callbacks.
    fn preflight(&self, schema: &SchemaConfig, selection: &TransformSelection) -> Result<()> {
        let selected: Vec<_> = self
            .registrations
            .iter()
            .map(|item| item.spec.id.as_str())
            .collect();
        let declared: Vec<_> = schema.plugin.keys().map(String::as_str).collect();
        if !declared.is_empty() && declared != selected {
            return Err(TransformError::new(
                "Idea plugins must match selected generator order",
            ));
        }
        for (index, item) in self.registrations.iter().enumerate() {
            for (required, ids) in [(true, &item.spec.requires), (false, &item.spec.optional)] {
                for id in ids {
                    // A later/self prerequisite fails, including every cycle; no sorting occurs.
                    match selected.iter().position(|selected| *selected == id) {
                        Some(position) if position < index => {}
                        None if !required => {}
                        _ => {
                            return Err(TransformError::new(
                                "missing or out-of-order prerequisite",
                            ));
                        }
                    }
                }
            }
            for (id, revision) in &item.spec.contracts {
                if selection.contracts.get(id) != Some(revision) {
                    return Err(TransformError::new(
                        "selected runtime contract revision mismatch",
                    ));
                }
            }
        }
        Ok(())
    }
}
