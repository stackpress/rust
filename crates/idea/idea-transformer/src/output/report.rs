//! Deterministic path/ownership comparisons and public output receipts.

use std::collections::BTreeMap;

use serde::Serialize;

/// Result of a no-write output comparison.
#[derive(Debug, Serialize)]
pub struct CheckReport {
    /// Paths added, changed or removed, in deterministic order.
    pub differences: Vec<String>,
}

/// Deterministic install receipt containing logical names, relative paths and selected IDs.
///
/// No timestamp, native root, environment value or source contents enter this report.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct GenerationReport {
    /// Consumed auxiliary input names, sorted and deduplicated.
    pub inputs: Vec<String>,
    /// Actual transform execution order.
    pub selected: Vec<String>,
    /// Added or changed planned paths, sorted lexically.
    pub generated: Vec<String>,
    /// Byte- and owner-identical planned paths, sorted lexically.
    pub unchanged: Vec<String>,
    /// Previously owned paths omitted from the new plan, sorted lexically.
    pub removed: Vec<String>,
    /// Writer receipt/ownership contract revision, independent of app runtime contracts.
    pub contract_version: u32,
}

impl CheckReport {
    /// Whether every planned file and the exact ownership set match disk.
    pub fn is_current(&self) -> bool {
        self.differences.is_empty()
    }
}

/// Compare exact bytes and ownership, retaining removals in the deterministic path set.
pub(super) fn compare(
    previous: &BTreeMap<String, (String, String)>,
    files: &BTreeMap<String, (String, String)>,
) -> CheckReport {
    let mut differences: Vec<_> = previous
        .keys()
        .chain(files.keys())
        .filter(|path| previous.get(*path) != files.get(*path))
        .cloned()
        .collect();
    differences.sort();
    differences.dedup();
    CheckReport { differences }
}
