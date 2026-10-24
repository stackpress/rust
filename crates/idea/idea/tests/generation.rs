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
    // Aggregation stays outside transform partitions, just as the app's module registry does.
    Ok(vec![GeneratedFile::new("index.txt", "generated")])
}

/// A failed transform must prevent the wrapper from reaching the writer.
fn reject(_: &parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>> {
    Err(transformer::TransformError::new(
        "deliberate transform failure",
    ))
}

/// Success, check drift and syntax failure share one path without implicit repair.
#[test]
fn parse_plan_install_and_check() {
    // Real owned files distinguish no-write check mode from install mode.
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

/// Callback/selection failures and owned-file conflicts preserve the previously installed tree.
#[test]
fn orchestration_preserves_output_on_plan_or_install_failure() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let inputs = parser::CompileInputs::default();
    let source = "prop Input { value \"original\" }";
    let mut registry = Registry::new();
    registry.register("text", &["data"], emit).unwrap();
    generate(
        "s", source, &inputs, &registry, aggregate, &workspace, false,
    )
    .unwrap();
    let original = std::fs::read(root.join(".stackpress-generated.json")).unwrap();

    // Failed callbacks retain their cause; schema selection is checked before output effects.
    let mut failing = Registry::new();
    failing.register("text", &["data"], reject).unwrap();
    let error = generate("s", source, &inputs, &failing, aggregate, &workspace, false).unwrap_err();
    assert!(error.to_string().contains("deliberate transform failure"));
    assert!(
        generate(
            "s",
            "plugin \"unknown\" { }",
            &inputs,
            &registry,
            aggregate,
            &workspace,
            false
        )
        .is_err()
    );
    assert_eq!(
        original,
        std::fs::read(root.join(".stackpress-generated.json")).unwrap()
    );

    // Unknown files cannot be erased by orchestration, including in freshness-only mode.
    let manual = root.join("manual.txt");
    std::fs::write(&manual, "keep me").unwrap();
    for check in [true, false] {
        assert!(
            generate(
                "s", source, &inputs, &registry, aggregate, &workspace, check
            )
            .is_err()
        );
        assert_eq!(std::fs::read_to_string(&manual).unwrap(), "keep me");
    }
}
