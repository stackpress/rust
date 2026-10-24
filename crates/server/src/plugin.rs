//! Explicit compiled plugin selection and deterministic dependency preflight.

use crate::builder::{BuildError, BuildResult, ServerBuilder, StartupStage};
use stackpress_lib::{Error, Result, SourceLocation};
use std::collections::{HashMap, HashSet};

/// Selected presence/order requirements; this never loads code dynamically.
#[derive(Clone, Debug)]
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
    pub fn new(id: &str) -> Self {
        // Specification construction is data-only; invalid data cannot execute callbacks.
        Self {
            id: id.into(),
            required: Vec::new(),
            optional: Vec::new(),
        }
    }
    /// Require selected plugins in the listed application order.
    pub fn requires(mut self, ids: &[&str]) -> Self {
        // Keep the explicit requirements; preflight validates the complete graph.
        self.required.extend(ids.iter().map(|id| (*id).into()));
        self
    }
    /// Order after these plugins only when the application selected them.
    pub fn after_if_present(mut self, ids: &[&str]) -> Self {
        // Absent optional plugins are never loaded or inferred from configuration.
        self.optional.extend(ids.iter().map(|id| (*id).into()));
        self
    }
    /// Validate metadata and bound graph work before selection publication.
    pub(crate) fn validate(&self) -> Result<()> {
        identity(&self.id)?;
        // Bound adjacency work independently of how many plugins are ultimately present.
        if self.required.len() + self.optional.len() > 1024 {
            return Err(Error::limit("plugin dependency budget exceeded"));
        }
        for id in self.required.iter().chain(&self.optional) {
            identity(id)?;
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
    let positions: HashMap<_, _> = plugins
        .iter()
        .enumerate()
        .map(|(i, p)| (p.descriptor.id.as_str(), i))
        .collect();
    let mut edges = vec![HashSet::new(); plugins.len()];
    for (index, plugin) in plugins.iter().enumerate() {
        // Missing required presence fails before registration can produce side effects.
        for required in &plugin.descriptor.required {
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
                edges[index].insert(dependency);
            }
        }
    }
    // Iterative elimination detects cycles without a recursive stack proportional to input.
    let mut visited = vec![false; plugins.len()];
    loop {
        let next =
            (0..plugins.len()).find(|&i| !visited[i] && edges[i].iter().all(|&d| visited[d]));
        match next {
            Some(i) => visited[i] = true,
            None => break,
        }
    }
    if let Some(i) = visited.iter().position(|done| !done) {
        return Err(plugins[i].failure(Error::invalid("plugin dependency cycle")));
    }
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
