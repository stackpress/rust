//! Small generation orchestration over independently usable parser and transformer crates.
//!
//! This synchronous build-host API starts no executor and accepts explicit source/environment.
//! The caller owns source acquisition, selected function registrations and the generated root.

pub use stackpress_idea_parser as parser;
pub use stackpress_idea_transformer as transformer;

use parser::CompileInputs;
use transformer::{CheckReport, GeneratedFile, OutputWorkspace, Registry, Result};

/// Parse one named source, execute selected transforms and compare or install the owned plan.
///
/// # Errors
/// Preserves source diagnostics and returns callback/ownership/IO/recovery errors. Planning
/// failures never change output. Check mode never changes generated files or their manifest.
pub fn generate(
    file: &str,
    source: &str,
    inputs: &CompileInputs,
    registry: &Registry,
    aggregates: fn(&parser::SchemaConfig) -> Result<Vec<GeneratedFile>>,
    workspace: &OutputWorkspace,
    check: bool,
) -> Result<CheckReport> {
    let schema = parser::parse(file, source, inputs)?;
    let plan = registry.plan(&schema, aggregates(&schema)?)?;
    // Both paths use the same planner and formatter; check cannot silently repair drift.
    if check {
        workspace.check(&plan)
    } else {
        workspace.install(plan)
    }
}
