//! Compile the aggregate source-client output from actual Idea input, with freshness proof.
use stackpress_framework::commands::schema::rust::transformer::*;
use stackpress_framework_schema::{Schema, codegen::transform_error};

/// The aggregate emitter is the same boundary that previously refused the blog editor.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let graph = Schema::normalize_source(context.schema(), Default::default(), &[])
        .map_err(transform_error)?;
    let definitions =
        stackpress_framework_admin::codegen::source_definitions().map_err(transform_error)?;
    let options = stackpress_framework::commands::source_client::Options::new(
        "editor-proof",
        "stackpress_framework",
    );
    stackpress_framework::commands::source_client::emit(&graph, &definitions, &options, output)
}

/// The proof owns only its disposable consumer's generated directory.
fn main() {
    let source = std::fs::read_to_string("schema.idea").unwrap();
    let config =
        stackpress_idea_parser::parse("schema.idea", &source, &Default::default()).unwrap();
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("editor").owns(&["ui", "source", "stores", "client"]),
            emit,
        )
        .unwrap();
    let workspace = OutputWorkspace::new("src/generated").unwrap();
    workspace
        .install(registry.plan(&config, Vec::new()).unwrap())
        .unwrap();
    assert!(
        workspace
            .check(&registry.plan(&config, Vec::new()).unwrap())
            .unwrap()
            .is_current()
    );
    println!("cargo:rerun-if-changed=schema.idea");
}
