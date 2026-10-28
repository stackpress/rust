//! Explicit compiled plugin selection and deterministic dependency preflight.

use crate::builder::{BuildError, BuildResult, ServerBuilder, StartupStage};
use stackpress_lib::{Error, Result, SourceLocation};
use std::collections::{HashMap, HashSet};

/// Selected presence/order requirements; this never loads code dynamically.
///
/// Required and present optional prerequisites must occur earlier in the application's list.
/// Build rejects missing requirements, cycles and wrong order before any callback runs.
/// It preserves supplied order instead of sorting priority-sensitive plugin composition.
///
/// # Examples
///
/// ```rust
/// use stackpress_server::PluginSpec;
/// let spec = PluginSpec::new("profiles")
///     .requires(&["storage"])
///     .after_if_present(&["metrics"]);
/// # let _ = spec;
/// ```
/// Discarded specification chains are diagnosed; use an explicit binding for intentional discard.
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use stackpress_server::PluginSpec;
/// PluginSpec::new("profiles").requires(&["storage"]);
/// ```
#[derive(Clone, Debug)]
#[must_use = "the configured value must be used"]
pub struct PluginSpec {
    /// Stable bounded application-selected identity.
    pub(crate) id: String,
    /// These plugins must be selected and precede this plugin.
    required: Vec<String>,
    /// Selected optional prerequisites must precede this plugin.
    optional: Vec<String>,
}

impl PluginSpec {
    /// Describe one plugin; validation occurs when it enters a builder.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_server::PluginSpec;
    /// let spec = PluginSpec::new("profiles")
    ///     .requires(&["storage"])
    ///     .after_if_present(&["metrics"]);
    /// # let _ = spec;
    /// ```
    pub fn new(id: &str) -> Self {
        // Specification construction is data-only; invalid data cannot execute callbacks.
        Self {
            id: id.into(),
            required: Vec::new(),
            optional: Vec::new(),
        }
    }
    /// Require selected plugins in the listed application order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_server::PluginSpec;
    /// let spec = PluginSpec::new("profiles")
    ///     .requires(&["storage"])
    ///     .after_if_present(&["metrics"]);
    /// # let _ = spec;
    /// ```
    pub fn requires(mut self, ids: &[&str]) -> Self {
        // Keep the explicit requirements; preflight validates the complete graph.
        self.required.extend(ids.iter().map(|id| (*id).into()));
        self
    }
    /// Order after these plugins only when the application selected them.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_server::PluginSpec;
    /// let spec = PluginSpec::new("profiles")
    ///     .requires(&["storage"])
    ///     .after_if_present(&["metrics"]);
    /// # let _ = spec;
    /// ```
    pub fn after_if_present(mut self, ids: &[&str]) -> Self {
        // Absent optional plugins are never loaded or inferred from configuration.
        self.optional.extend(ids.iter().map(|id| (*id).into()));
        self
    }
    /// Validate metadata and bound graph work before selection publication.
    pub(crate) fn validate(&self) -> Result<()> {
        // Validate diagnostic identity before inspecting dependency declarations.
        identity(&self.id)?;
        // Bound adjacency work independently of how many plugins are ultimately present.
        if self.required.len() + self.optional.len() > 1024 {
            return Err(Error::limit("plugin dependency budget exceeded"));
        }
        for id in self.required.iter().chain(&self.optional) {
            // Every prerequisite name is bounded even when that optional plugin is absent.
            identity(id)?;
            // Self-dependency cannot be repaired by ordering and must fail before callback entry.
            if id == &self.id {
                return Err(Error::invalid(format!(
                    "plugin {} depends on itself",
                    self.id
                )));
            }
        }
        Ok(())
    }
}

/// Read-only selected plugin provenance in the original application order.
#[derive(Clone, Debug)]
pub struct PluginDescriptor {
    /// Explicit identity; not a package path or a semver constraint.
    pub id: String,
    /// Required presence/order declarations.
    pub required: Vec<String>,
    /// Optional-present order declarations.
    pub after_if_present: Vec<String>,
    /// Application selection site.
    pub registered: SourceLocation,
}

/// A callback is owned by construction only and never enters HostContext.
pub(crate) struct Plugin {
    /// Frozen public identity and declarations.
    pub(crate) descriptor: PluginDescriptor,
    /// Normal compiled registration function.
    pub(crate) register: fn(&mut ServerBuilder) -> Result<()>,
}

impl Plugin {
    /// Retain the actual application caller while consuming validated spec data.
    #[track_caller]
    pub(crate) fn new(spec: PluginSpec, register: fn(&mut ServerBuilder) -> Result<()>) -> Self {
        // Function identity does not substitute for the application's selected ID.
        Self {
            descriptor: PluginDescriptor {
                id: spec.id,
                required: spec.required,
                after_if_present: spec.optional,
                registered: SourceLocation::caller(),
            },
            register,
        }
    }
    /// Attribute preflight failures before any plugin callback is entered.
    pub(crate) fn failure(&self, error: Error) -> BuildError {
        // Honest registration metadata exists even when no listener or startup task exists yet.
        BuildError {
            stage: StartupStage::Registration,
            plugin: Some(self.descriptor.id.clone()),
            label: "preflight".into(),
            registered: self.descriptor.registered.clone(),
            error,
        }
    }
}

/// Check the whole graph before callbacks, preserving supplied order rather than sorting it.
pub(crate) fn preflight(plugins: &[Plugin]) -> BuildResult<()> {
    // Resolve presence first; no selected function may run against an incomplete graph.
    let edges = dependency_edges(plugins)?;
    // Diagnose cycles before wrong-order errors so a cyclic graph has one stable failure class.
    reject_cycles(plugins, &edges)?;
    // Cycles are reported distinctly; acyclic but wrongly listed dependencies name both IDs.
    for (i, plugin) in plugins.iter().enumerate() {
        if let Some(&dependency) = edges[i].iter().filter(|&&d| d >= i).min() {
            return Err(plugin.failure(Error::invalid(format!(
                "plugin {} must follow {}",
                plugin.descriptor.id, plugins[dependency].descriptor.id
            ))));
        }
    }
    Ok(())
}

/// Resolve required/present optional prerequisites without executing registration callbacks.
fn dependency_edges(plugins: &[Plugin]) -> BuildResult<Vec<HashSet<usize>>> {
    // Map explicit selection positions; these positions remain the execution order.
    let positions: HashMap<_, _> = plugins
        .iter()
        .enumerate()
        .map(|(i, p)| (p.descriptor.id.as_str(), i))
        .collect();
    // Deduplicated adjacency affects validation only, never registration priority or ties.
    let mut edges = vec![HashSet::new(); plugins.len()];
    for (index, plugin) in plugins.iter().enumerate() {
        // Missing required presence fails before registration can produce side effects.
        for required in &plugin.descriptor.required {
            // An absent required plugin is an error; the host never loads code to satisfy it.
            if !positions.contains_key(required.as_str()) {
                return Err(plugin.failure(Error::invalid(format!(
                    "plugin {} requires missing {required}",
                    plugin.descriptor.id
                ))));
            }
        }
        // Deduplicate graph edges without altering callback or declaration order.
        for id in plugin
            .descriptor
            .required
            .iter()
            .chain(&plugin.descriptor.after_if_present)
        {
            if let Some(&dependency) = positions.get(id.as_str()) {
                // Only present optional prerequisites create order/cycle edges.
                edges[index].insert(dependency);
            }
        }
    }
    Ok(edges)
}

/// Reject cycles iteratively before diagnosing any acyclic supplied-order violation.
fn reject_cycles(plugins: &[Plugin], edges: &[HashSet<usize>]) -> BuildResult<()> {
    // Iterative elimination detects cycles without a recursive stack proportional to input.
    let mut visited = vec![false; plugins.len()];
    loop {
        let next =
            (0..plugins.len()).find(|&i| !visited[i] && edges[i].iter().all(|&d| visited[d]));
        match next {
            // Mark a node only after all its dependencies have been removed from consideration.
            Some(i) => visited[i] = true,
            // No progress means either completion or a remaining cycle; inspect visited next.
            None => break,
        }
    }
    // A remaining node proves a cycle; use the first unresolved selection for stable diagnostics.
    if let Some(i) = visited.iter().position(|done| !done) {
        return Err(plugins[i].failure(Error::invalid("plugin dependency cycle")));
    }
    Ok(())
}

/// Validate bounded printable identities used by config, plugins and service names.
pub(crate) fn identity(value: &str) -> Result<()> {
    // Reject terminal injection before the value can enter a report or source label.
    if value.is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
        return Err(Error::invalid(
            "identity must contain 1..=256 bytes without controls",
        ));
    }
    Ok(())
}
