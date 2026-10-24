//! Adapt checked Idea/JSON compilation into one snapshot before import precedence is applied.
//!
//! Parser references are retained as their real owned type. Source spans stay with their sites;
//! the composer owns acquisition/import graphs while the merge module owns winner selection.

use stackpress_idea_parser::{
    CompileInputs, CompiledSchema, SourceFile, compile_ast_with_references, parse_ast,
};

use super::{CompositionLimits, Snapshot, context, json, merge};
use crate::{LoadedSource, Result, SourceKind};

/// Compile one source and preserve actual reference syntax before any import precedence runs.
pub(super) fn compile_source(
    source: &LoadedSource,
    inputs: &CompileInputs,
    limits: CompositionLimits,
) -> Result<Snapshot> {
    // Both grammars share caller-lowered byte/depth policy; JSON also has a node counter.
    let compiled = match source.kind {
        SourceKind::Idea => {
            // Acquisition already bounded this text; syntax and compilation use explicit inputs.
            let file = SourceFile::new(&source.name, source.text.as_str());
            parse_ast(&file, inputs.limits)
                .and_then(|ast| compile_ast_with_references(&ast, inputs))
                .map_err(|cause| context(format!("Idea source {} failed", source.name), cause))?
        }
        SourceKind::Json => json::parse(source, inputs.limits, limits.max_json_nodes)?,
    };
    // Move the compiler's real sites with their config instead of deriving sites from strings.
    let CompiledSchema { config, references } = compiled;
    let mut snapshot = Snapshot {
        config,
        references,
        ..Snapshot::default()
    };
    merge::record_origins(&mut snapshot, &source.id);
    Ok(snapshot)
}
