//! Public planning preflight and output ownership contracts.

use super::*;
use stackpress_idea_parser::{CompileInputs, parse};

/// Emit an observable z-owned contribution.
fn z(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new("z/item.txt", "z")])
}

/// Emit a different owner contribution without mutable global recording.
fn a(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new("a/item.txt", "a")])
}

/// Misbehaving plugins cannot claim the aggregate owner's file.
fn outside(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new("mod.rs", "manual")])
}

/// Return a real transform failure before the output writer is involved.
fn failure(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Err(TransformError::new("test transform failed"))
}

/// Execution preserves listed order, and schema declarations must agree before callbacks run.
#[test]
fn execution_order_and_preflight_errors() {
    let inputs = CompileInputs::default();
    let schema = parse("s", "plugin \"z\" { } plugin \"a\" { }", &inputs).unwrap();
    let mut registry = Registry::new();
    registry.register("z", &["z"], z).unwrap();
    registry.register("a", &["a"], a).unwrap();
    assert_eq!(
        registry.plan(&schema, vec![]).unwrap().executed(),
        ["z", "a"]
    );
    let reversed = parse("s", "plugin \"a\" { } plugin \"z\" { }", &inputs).unwrap();
    assert!(
        registry
            .plan(&reversed, vec![])
            .unwrap_err()
            .to_string()
            .contains("order")
    );
    assert!(registry.register("z", &["third"], z).is_err());
    assert!(registry.register("third", &["z"], z).is_err());
    assert!(registry.register("third", &["x", "x"], z).is_err());
    assert!(registry.register("third", &["x/y"], z).is_err());
    assert!(Registry::new().plan(&schema, vec![]).is_err());
}

/// Paths must be portable and collision-free before staging creates any output.
#[test]
fn path_partition_and_budget_failures() {
    for path in [
        "", "../a", "a/../b", "/a", "a\\b", "a:b", "a/é", "CON", "a/x.", "a//b",
    ] {
        assert!(valid_path(path).is_err(), "accepted {path}");
    }
    let schema = SchemaConfig::default();
    let mut registry = Registry::new();
    registry.register("z", &["z"], z).unwrap();
    assert!(
        registry
            .plan(&schema, vec![GeneratedFile::new("z", "collision")])
            .is_err()
    );
    assert!(
        registry
            .plan(&schema, vec![GeneratedFile::new("x/y", "bad aggregate")])
            .is_err()
    );
    assert!(
        registry
            .plan(
                &schema,
                vec![GeneratedFile::new("X", "1"), GeneratedFile::new("x", "2")]
            )
            .is_err()
    );
    assert!(
        registry
            .plan(
                &schema,
                vec![GeneratedFile::new("large", "x".repeat(1024 * 1024))]
            )
            .is_err()
    );
    let mut registry = Registry::new();
    registry.register("outside", &["data"], outside).unwrap();
    assert!(registry.plan(&schema, vec![]).is_err());
    let mut registry = Registry::new();
    registry.register("failure", &[], failure).unwrap();
    assert_eq!(
        registry.plan(&schema, vec![]).unwrap_err().to_string(),
        "test transform failed"
    );
}
