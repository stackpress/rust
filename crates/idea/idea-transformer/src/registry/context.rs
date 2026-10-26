//! Explicit immutable auxiliary snapshots and selected runtime contract metadata.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use stackpress_idea_parser::SchemaConfig;

use super::valid_path;
use crate::{Result, TransformError};

/// Bounded, owned auxiliary input snapshots supplied by the build tool before planning.
///
/// Names are portable logical paths, never absolute host paths. Values stay out of reports.
#[derive(Debug, Default)]
pub struct TransformInputs {
    /// A run owns its bytes, without aliases to mutable loader caches.
    values: BTreeMap<String, String>,
    /// Aggregate accounting bounds many individually small input snapshots.
    bytes: usize,
}

impl TransformInputs {
    /// Start an empty input snapshot; no implicit filesystem or environment reads occur.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a unique logical input before callbacks execute.
    ///
    /// # Errors
    /// Rejects invalid/duplicate names, more than 1024 inputs, or over 1 MiB of input bytes.
    ///
    /// ```
    /// use stackpress_idea_transformer::TransformInputs;
    /// let mut inputs = TransformInputs::new();
    /// inputs.insert("templates/model.txt", "model body")?;
    /// # Ok::<(), stackpress_idea_transformer::TransformError>(())
    /// ```
    pub fn insert(&mut self, name: &str, contents: impl Into<String>) -> Result<()> {
        valid_path(name)?;
        let contents = contents.into();
        let bytes = self.bytes.checked_add(contents.len());
        if self.values.contains_key(name)
            || self.values.len() >= 1024
            || bytes.is_none_or(|total| total > 1024 * 1024)
        {
            return Err(TransformError::classified(
                if self.values.contains_key(name) {
                    crate::TransformErrorKind::Selection
                } else {
                    crate::TransformErrorKind::Limit
                },
                "auxiliary input duplicate or budget exceeded",
            ));
        }
        self.bytes = bytes.unwrap_or_default();
        self.values.insert(name.into(), contents);
        Ok(())
    }
}

/// App-supplied selection contracts and immutable auxiliary input snapshots for one run.
#[derive(Debug, Default)]
#[must_use = "the configured value must be used"]
pub struct TransformSelection {
    /// Required contract IDs/revisions are checked before any callback.
    pub(crate) contracts: BTreeMap<String, String>,
    /// Inputs can only change before the selection is borrowed for planning.
    pub(crate) inputs: TransformInputs,
}

impl TransformSelection {
    /// Select no runtime contracts and no auxiliary inputs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a unique selected runtime contract revision.
    ///
    /// # Errors
    /// Rejects invalid IDs, duplicates, empty/oversized revisions and excessive contracts.
    pub fn contract(&mut self, id: &str, revision: &str) -> Result<()> {
        valid_contract(id, revision)?;
        if self.contracts.len() >= 1024 || self.contracts.contains_key(id) {
            return Err(TransformError::classified(
                if self.contracts.len() >= 1024 {
                    crate::TransformErrorKind::Limit
                } else {
                    crate::TransformErrorKind::Selection
                },
                "invalid or duplicate runtime contract",
            ));
        }
        self.contracts.insert(id.into(), revision.into());
        Ok(())
    }

    /// Set the complete auxiliary snapshot, consumed through [`TransformContext::input`].
    pub fn with_inputs(mut self, inputs: TransformInputs) -> Self {
        self.inputs = inputs;
        self
    }
}

/// Use the same bounded contract vocabulary for required and supplied revisions.
pub(super) fn valid_contract(id: &str, revision: &str) -> Result<()> {
    valid_path(id).map_err(|mut error| {
        // Contract IDs use portable path syntax but fail at the selection boundary.
        error.kind = crate::TransformErrorKind::Selection;
        error
    })?;
    if revision.is_empty()
        || revision.len() > 128
        || !revision
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Selection,
            "invalid runtime contract revision",
        ));
    }
    Ok(())
}

/// Immutable schema/config/selection view for one synchronous trusted function plugin.
///
/// Auxiliary input access records logical names only. This does not sandbox trusted Rust
/// code that deliberately performs direct filesystem or process-environment access.
pub struct TransformContext<'a> {
    /// Immutable schema shared by sequential callbacks.
    pub(crate) schema: &'a SchemaConfig,
    /// Compiled plugin currently executing.
    pub(crate) id: &'a str,
    /// Explicit contract requirements and bounded input snapshots.
    pub(crate) selection: &'a TransformSelection,
    /// Per-run access log, borrowed only during individual accessor calls.
    pub(crate) used: &'a RefCell<BTreeSet<String>>,
}

impl<'a> TransformContext<'a> {
    /// Borrow the schema without granting mutation of another transform's input.
    pub fn schema(&self) -> &'a SchemaConfig {
        self.schema
    }

    /// Return this callback's compiled registration ID.
    pub fn id(&self) -> &'a str {
        self.id
    }

    /// Borrow the current plugin's typed JSON configuration, absent for registry defaults.
    pub fn config(&self) -> Option<&'a stackpress_idea_parser::PluginConfig> {
        self.schema.plugin.get(self.id)
    }

    /// Inspect an explicitly selected contract revision without consulting process state.
    pub fn contract(&self, id: &str) -> Option<&'a str> {
        self.selection.contracts.get(id).map(String::as_str)
    }

    /// Read a declared snapshot and record its logical name for deterministic reporting.
    ///
    /// # Errors
    /// Rejects undeclared inputs without reading the filesystem or exposing input values.
    pub fn input(&self, name: &str) -> Result<&'a str> {
        let value = self.selection.inputs.values.get(name).ok_or_else(|| {
            TransformError::classified(
                crate::TransformErrorKind::Selection,
                "transform requested undeclared auxiliary input",
            )
        })?;
        self.used.borrow_mut().insert(name.into());
        Ok(value)
    }
}
