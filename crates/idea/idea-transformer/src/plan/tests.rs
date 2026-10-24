//! In-memory ownership, aggregation and budget boundary tests.

use super::*;

/// Test claim rules while invoking the same private scope transition as the registry.
#[test]
fn claims_respect_ownership_reserved_aggregates_and_duplicate_paths() {
    let mut plan = OutputPlan::default();
    assert!(plan.write("contracts/x.rs", "x").is_err());
    plan.begin("plugin", &["contracts".into()]);
    for path in [
        "mod.rs",
        "other/x.rs",
        "Contracts/x.rs",
        "contracts/../manual",
        "contracts//x",
    ] {
        assert!(plan.write(path, "x").is_err(), "accepted {path}");
    }
    plan.write("contracts/a.rs", "pub struct A;\n").unwrap();
    assert!(plan.write("contracts/A.rs", "pub struct A;\n").is_err());
    assert!(plan.write("contracts/a.rs/child", "x").is_err());
    plan.contribute_module("contracts", "a").unwrap();
    assert!(plan.contribute_module("contracts", "a").is_err());
    for module in ["mod", "self", "gen", "a-b", "a;panic!()", "_", "é"] {
        assert!(plan.contribute_module("contracts", module).is_err());
    }
    plan.end();
    plan.finish_modules().unwrap();
    assert_eq!(plan.files["contracts/mod.rs"].0, "aggregate");
}

/// Missing files and user-supplied aggregate collisions fail before any workspace writes.
#[test]
fn contribution_targets_and_aggregate_conflicts() {
    let mut plan = OutputPlan::default();
    plan.begin("plugin", &["contracts".into()]);
    plan.contribute_module("contracts", "missing").unwrap();
    plan.end();
    assert!(plan.finish_modules().is_err());
    let mut plan = OutputPlan::default();
    plan.insert(
        "aggregate",
        GeneratedFile::new("mod.rs", "manual aggregate"),
    )
    .unwrap();
    plan.begin("plugin", &["contracts".into()]);
    plan.write("contracts/a.rs", "pub struct A;\n").unwrap();
    plan.contribute_module("contracts", "a").unwrap();
    plan.end();
    assert!(plan.finish_modules().is_err());
    let mut plan = OutputPlan::default();
    plan.begin("plugin", &["contracts".into()]);
    plan.write("contracts/mod.rs", "pub mod a;\n").unwrap();
    plan.write("contracts/a.rs", "pub struct A;\n").unwrap();
    plan.contribute_module("contracts", "a").unwrap();
    plan.end();
    assert!(plan.finish_modules().is_err());
}

/// Output limits protect aggregate bytes and the exact maximum file count.
#[test]
fn output_budget_boundaries() {
    let mut plan = OutputPlan::default();
    plan.begin("plugin", &["data".into()]);
    plan.write("data/max", "x".repeat(1024 * 1024)).unwrap();
    assert!(plan.write("data/over", "x").is_err());
    let mut plan = OutputPlan::default();
    plan.begin("plugin", &["data".into()]);
    for index in 0..1024 {
        plan.write(format!("data/{index}"), "").unwrap();
    }
    assert!(plan.write("data/over", "").is_err());
}

/// Case aliases of a shared directory are conflicts even when their leaf names differ.
#[test]
fn ancestor_directory_case_aliases_are_collisions() {
    for (left, right) in [
        ("data/Models/a.rs", "data/models/b.rs"),
        ("Data/a.rs", "data/b.rs"),
        ("data/models/A.rs", "data/models/a.rs"),
        ("data/Models", "data/models/b.rs"),
    ] {
        assert!(
            paths_collide(left, right),
            "accepted alias {left} / {right}"
        );
        assert!(paths_collide(right, left));
    }
    assert!(!paths_collide("data/Models/a.rs", "data/Models/b.rs"));
    assert!(!paths_collide("data/a.rs", "other/A.rs"));
    let mut plan = OutputPlan::default();
    plan.begin("plugin", &["data".into()]);
    plan.write("data/Models/a.rs", "pub struct A;\n").unwrap();
    assert!(plan.write("data/models/b.rs", "pub struct B;\n").is_err());
}
