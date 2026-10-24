//! Sequential function transforms with explicit partitions and one aggregate owner.

use std::collections::BTreeMap;

use stackpress_idea_parser::SchemaConfig;

use crate::{Result, TransformError};

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

/// Named synchronous generator callback; it has no managed filesystem handle.
pub type Transform = fn(&SchemaConfig) -> Result<Vec<GeneratedFile>>;

/// One selected function and its exclusive first-component output partitions.
struct Registration {
    /// Unique compiled plugin identifier.
    id: String,
    /// Exclusive source partitions such as contracts or server.
    partitions: Vec<String>,
    /// Selected compiled function, invoked once in registration order.
    transform: Transform,
}

/// Builder-time selected transforms; arbitrary schema strings never load executable code.
#[derive(Default)]
pub struct Registry {
    /// Insertion order is the execution order; no sorting/reordering is performed.
    registrations: Vec<Registration>,
}

/// Fully checked fresh output, independent of prior files on disk.
#[derive(Debug)]
pub struct OutputPlan {
    /// Stable path order is for writing only; callbacks already ran in registry order.
    pub(crate) files: BTreeMap<String, (String, String)>,
    /// Executed IDs make the actual selected order inspectable.
    executed: Vec<String>,
}

#[cfg(test)]
mod tests;

impl OutputPlan {
    /// Return selected IDs in their actual execution order.
    pub fn executed(&self) -> &[String] {
        &self.executed
    }
}

impl Registry {
    /// Start an empty compiled selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one function with disjoint, nonempty, single-component partitions.
    ///
    /// # Errors
    /// Rejects duplicate IDs, overlapping/invalid partitions and an empty ID.
    pub fn register(&mut self, id: &str, partitions: &[&str], transform: Transform) -> Result<()> {
        if id.is_empty() || self.registrations.iter().any(|item| item.id == id) {
            return Err(TransformError::new("empty or duplicate generator ID"));
        }
        // Preflight the complete contribution before mutating the registry.
        for (index, part) in partitions.iter().enumerate() {
            valid_path(part)?;
            if part.contains('/')
                || partitions[..index].contains(part)
                || self
                    .registrations
                    .iter()
                    .any(|item| item.partitions.iter().any(|p| p == part))
            {
                return Err(TransformError::new("generator partition conflict"));
            }
        }
        self.registrations.push(Registration {
            id: id.into(),
            partitions: partitions.iter().map(|part| (*part).into()).collect(),
            transform,
        });
        Ok(())
    }

    /// Execute selected functions after validating root plugin order, then own aggregate files.
    ///
    /// # Errors
    /// Any callback, selection, collision, path or output-budget failure produces no disk writes.
    pub fn plan(
        &self,
        schema: &SchemaConfig,
        aggregates: Vec<GeneratedFile>,
    ) -> Result<OutputPlan> {
        let selected: Vec<_> = self
            .registrations
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        let declared: Vec<_> = schema.plugin.keys().map(String::as_str).collect();
        if selected.is_empty() || (!declared.is_empty() && declared != selected) {
            return Err(TransformError::new(
                "Idea plugins must match selected generator order",
            ));
        }
        let mut plan = OutputPlan {
            files: BTreeMap::new(),
            executed: Vec::new(),
        };
        // Aggregate callers own only root files; plugins own partition contents exclusively.
        for file in aggregates {
            if file.path.contains('/') {
                return Err(TransformError::new("invalid aggregate path"));
            }
            insert(&mut plan, "aggregate", file)?;
        }
        for item in &self.registrations {
            for file in (item.transform)(schema)? {
                let prefix = file.path.split('/').next().unwrap_or("");
                if !item.partitions.iter().any(|part| part == prefix) || !file.path.contains('/') {
                    return Err(TransformError::new("transform wrote outside its partition"));
                }
                insert(&mut plan, &item.id, file)?;
            }
            plan.executed.push(item.id.clone());
        }
        Ok(plan)
    }
}

/// Validate a portable output path before any filesystem operation.
pub(crate) fn valid_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 240
        || !path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-./".contains(&byte))
        || path.chars().any(char::is_control)
        || path.starts_with('.')
        || path.split('/').count() > 16
        || path.split('/').any(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with('.')
                || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.as_bytes()[3].is_ascii_digit())
        })
    {
        return Err(TransformError::new("invalid relative generated path"));
    }
    Ok(())
}

/// Bound aggregate output and reject case-insensitive collisions before publishing a plan.
fn insert(plan: &mut OutputPlan, owner: &str, file: GeneratedFile) -> Result<()> {
    valid_path(&file.path)?;
    if plan.files.len() >= 1024
        || plan
            .files
            .values()
            .map(|(_, text)| text.len())
            .sum::<usize>()
            .checked_add(file.contents.len())
            .is_none_or(|bytes| bytes > 1024 * 1024)
        || plan.files.keys().any(|path| {
            let prior = path.to_ascii_lowercase();
            let next = file.path.to_ascii_lowercase();
            prior == next
                || prior.starts_with(&format!("{next}/"))
                || next.starts_with(&format!("{prior}/"))
        })
    {
        return Err(TransformError::new(
            "generated output budget or path collision",
        ));
    }
    plan.files.insert(file.path, (owner.into(), file.contents));
    Ok(())
}
