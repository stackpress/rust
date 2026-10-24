//! End-to-end convenience orchestration, including parse failure and no-write check.

use stackpress_idea::{generate, parser, transformer};
use transformer::{GeneratedFile, OutputWorkspace, Registry};

/// Render one declared prop to prove the wrapper forwards real parsed metadata.
fn emit(schema: &parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new(
        "data/value.txt",
        schema.prop["Input"]["value"].as_str().unwrap(),
    )])
}

/// Own a root index independently of the plugin's partition.
fn aggregate(_: &parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new("index.txt", "generated")])
}

/// Success, check drift and syntax failure share one path without implicit repair.
#[test]
fn parse_plan_install_and_check() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let mut registry = Registry::new();
    registry.register("text", &["data"], emit).unwrap();
    let inputs = parser::CompileInputs::default();
    let source = "prop Input { value \"original\" }";
    assert!(
        !generate("s", source, &inputs, &registry, aggregate, &workspace, true)
            .unwrap()
            .is_current()
    );
    assert!(!root.exists());
    generate(
        "s", source, &inputs, &registry, aggregate, &workspace, false,
    )
    .unwrap();
    assert!(
        generate("s", source, &inputs, &registry, aggregate, &workspace, true)
            .unwrap()
            .is_current()
    );
    let error = generate(
        "broken.idea",
        "model X {",
        &inputs,
        &registry,
        aggregate,
        &workspace,
        false,
    )
    .unwrap_err();
    assert!(error.to_string().starts_with("broken.idea:1:"));
    assert_eq!(
        std::fs::read_to_string(root.join("data/value.txt")).unwrap(),
        "original"
    );
}
