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

/// A preflight regression: invoking this callback is itself a test failure.
fn never(_: &TransformContext<'_>, _: &mut OutputPlan) -> Result<()> {
    panic!("callback ran before full preflight");
}

/// A dependency-only plugin has no required output partition.
fn noop(_: &TransformContext<'_>, _: &mut OutputPlan) -> Result<()> {
    Ok(())
}

/// Emit modules out of alphabetic order and read only declared inputs and selection metadata.
fn contextual(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    assert_eq!(context.id(), "context");
    assert!(context.schema().model.is_empty());
    assert!(context.config().is_none());
    assert_eq!(context.contract("runtime"), Some("1"));
    let body = context.input("templates/z.txt")?;
    assert_eq!(context.input("templates/z.txt")?, body);
    plan.write("contracts/z.rs", body)?;
    plan.contribute_module("contracts", "z")?;
    plan.write("contracts/a.rs", "pub struct A;\n")?;
    plan.contribute_module("contracts", "a")
}

/// Failed preflight validates every prerequisite before invoking even the first callback.
#[test]
fn complete_preflight_missing_disabled_self_cycle_order_and_contract() {
    for required in ["missing", "first", "later"] {
        let mut registry = Registry::new();
        registry
            .add(TransformSpec::new("first").requires(&[required]), never)
            .unwrap();
        registry.add(TransformSpec::new("later"), never).unwrap();
        assert!(registry.plan(&SchemaConfig::default(), vec![]).is_err());
    }
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("first").requires(&["later"]), never)
        .unwrap();
    registry
        .add(TransformSpec::new("later").requires(&["first"]), never)
        .unwrap();
    assert!(registry.plan(&SchemaConfig::default(), vec![]).is_err());

    // Optional dependencies are not silently moved earlier when present.
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("first").after_if_present(&["later"]),
            never,
        )
        .unwrap();
    registry.add(TransformSpec::new("later"), never).unwrap();
    assert!(registry.plan(&SchemaConfig::default(), vec![]).is_err());
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("first").after_if_present(&["absent"]),
            noop,
        )
        .unwrap();
    registry
        .add(TransformSpec::new("later").requires(&["first"]), noop)
        .unwrap();
    assert_eq!(
        registry
            .plan(&SchemaConfig::default(), vec![])
            .unwrap()
            .executed(),
        ["first", "later"]
    );

    // Runtime contracts fail before the first plugin can consume configuration.
    let mut registry = Registry::new();
    registry.add(TransformSpec::new("first"), never).unwrap();
    registry
        .add(
            TransformSpec::new("later").requires_contract("runtime", "1"),
            never,
        )
        .unwrap();
    let mut selection = TransformSelection::new();
    selection.contract("runtime", "2").unwrap();
    assert!(
        registry
            .plan_with(&SchemaConfig::default(), &selection, vec![])
            .is_err()
    );
}

/// No-block defaults, empty selections and exact configured selections follow the accepted table.
#[test]
fn selection_defaults_empty_and_exact_match() {
    let empty = SchemaConfig::default();
    let plan = Registry::new()
        .plan(&empty, vec![GeneratedFile::new("index.txt", "empty")])
        .unwrap();
    assert!(plan.executed().is_empty());
    assert_eq!(plan.files().count(), 1);
    let mut registry = Registry::new();
    registry.register("z", &["z"], z).unwrap();
    assert_eq!(registry.plan(&empty, vec![]).unwrap().executed(), ["z"]);
    let inputs = CompileInputs::default();
    for source in [
        "plugin \"unknown\" {}",
        "plugin \"z\" {} plugin \"unknown\" {}",
    ] {
        let schema = parse("s.idea", source, &inputs).unwrap();
        assert!(registry.plan(&schema, vec![]).is_err());
        assert!(Registry::new().plan(&schema, vec![]).is_err());
    }
    let schema = parse("s.idea", "plugin \"a\" {}", &inputs).unwrap();
    assert!(registry.plan(&schema, vec![]).is_err());
    registry.register("a", &["a"], a).unwrap();
    assert!(registry.plan(&schema, vec![]).is_err());
}

/// Context reads are immutable snapshots; reports disclose names and preserve module order.
#[test]
fn context_snapshots_and_aggregate_modules() {
    let mut inputs = TransformInputs::new();
    inputs.insert("templates/z.txt", "pub struct Z;\n").unwrap();
    inputs.insert("unused.txt", "secret-unused").unwrap();
    assert!(inputs.insert("unused.txt", "replacement").is_err());
    assert!(inputs.insert("../escape", "x").is_err());
    let mut selection = TransformSelection::new().with_inputs(inputs);
    selection.contract("runtime", "1").unwrap();
    assert!(selection.contract("runtime", "1").is_err());
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("context")
                .owns(&["contracts"])
                .requires_contract("runtime", "1"),
            contextual,
        )
        .unwrap();
    let plan = registry
        .plan_with(&SchemaConfig::default(), &selection, vec![])
        .unwrap();
    assert_eq!(plan.inputs(), ["templates/z.txt"]);
    assert_eq!(plan.files["contracts/mod.rs"].1, "pub mod z;\npub mod a;\n");
    assert_eq!(plan.files["mod.rs"].1, "pub mod contracts;\n");
    assert_eq!(plan.files["contracts/mod.rs"].0, "aggregate");
    assert_eq!(plan.contributions()[0].module, "z");
    assert!(!format!("{plan:?}").contains("secret-unused"));
}

/// Duplicate registration attempts fail without reserving a partition or poisoning later adds.
#[test]
fn metadata_conflicts_are_atomic_and_case_portable() {
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("one").owns(&["Contracts"]), noop)
        .unwrap();
    for spec in [
        TransformSpec::new("one").owns(&["free"]),
        TransformSpec::new("two").owns(&["contracts"]),
        TransformSpec::new("two").owns(&["free", "FREE"]),
        TransformSpec::new("two").owns(&["x/y"]),
        TransformSpec::new("aggregate"),
        TransformSpec::new("two").requires(&["one", "one"]),
    ] {
        assert!(registry.add(spec, noop).is_err());
    }
    registry
        .add(TransformSpec::new("two").owns(&["free"]), noop)
        .unwrap();
    assert_eq!(
        registry
            .plan(&SchemaConfig::default(), vec![])
            .unwrap()
            .executed(),
        ["one", "two"]
    );
}

/// A new-style plugin failure retains its cause but safe display omits callback input values.
#[test]
fn contextual_errors_preserve_cause_without_disclosing_values() {
    /// Echoing a private value into an error must not leak through the safe outer error.
    fn fails(context: &TransformContext<'_>, _: &mut OutputPlan) -> Result<()> {
        Err(TransformError::new(context.input("private.txt")?))
    }
    /// Access to undeclared input is rejected without opening any ambient path.
    fn undeclared(context: &TransformContext<'_>, _: &mut OutputPlan) -> Result<()> {
        context.input("not-declared.txt").map(|_| ())
    }
    let mut inputs = TransformInputs::new();
    inputs.insert("private.txt", "secret-sentinel").unwrap();
    let selection = TransformSelection::new().with_inputs(inputs);
    let mut registry = Registry::new();
    registry.add(TransformSpec::new("failure"), fails).unwrap();
    let error = registry
        .plan_with(&SchemaConfig::default(), &selection, vec![])
        .unwrap_err();
    assert!(!error.to_string().contains("secret-sentinel"));
    assert!(std::error::Error::source(&error).is_some());
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("failure"), undeclared)
        .unwrap();
    assert!(registry.plan(&SchemaConfig::default(), vec![]).is_err());
}

/// Later functions inspect earlier declarations without patching earlier ownership claims.
#[test]
fn later_transform_observes_prior_contributions_and_typed_config() {
    /// First selected plugin emits an owned module.
    fn first(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
        plan.write("contracts/z.rs", "pub struct Z;\n")?;
        plan.contribute_module("contracts", "z")
    }
    /// The next plugin borrows both typed config and an immutable prior-contribution view.
    fn second(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
        assert_eq!(context.config().unwrap()["enabled"], true);
        assert_eq!(plan.contributions()[0].module, "z");
        assert!(plan.write("contracts/z.rs", "replacement").is_err());
        Ok(())
    }
    let schema = parse(
        "s.idea",
        "plugin \"first\" {} plugin \"second\" { enabled true }",
        &CompileInputs::default(),
    )
    .unwrap();
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("first").owns(&["contracts"]), first)
        .unwrap();
    registry
        .add(TransformSpec::new("second").requires(&["first"]), second)
        .unwrap();
    assert_eq!(
        registry.plan(&schema, vec![]).unwrap().executed(),
        ["first", "second"]
    );
}

/// Required and selected contracts share bounded revision rules; duplicates never overwrite.
#[test]
fn contract_metadata_and_auxiliary_budget_boundaries() {
    let mut registry = Registry::new();
    for spec in [
        TransformSpec::new("bad").requires_contract("runtime", ""),
        TransformSpec::new("bad").requires_contract("../runtime", "1"),
        TransformSpec::new("bad")
            .requires_contract("runtime", "1")
            .requires_contract("runtime", "2"),
    ] {
        assert!(registry.add(spec, never).is_err());
    }
    let mut selection = TransformSelection::new();
    assert!(selection.contract("runtime", "\nsecret").is_err());
    selection.contract("runtime", "1").unwrap();
    let mut inputs = TransformInputs::new();
    inputs.insert("max", "x".repeat(1024 * 1024)).unwrap();
    assert!(inputs.insert("over", "x").is_err());
    let mut inputs = TransformInputs::new();
    for index in 0..1024 {
        inputs.insert(&format!("input-{index}"), "").unwrap();
    }
    assert!(inputs.insert("over", "").is_err());
}

/// M0 generators may still own partition module files when no aggregate claims those paths.
#[test]
fn legacy_partition_module_files_remain_supported() {
    /// A plugin can construct its own nested module tree without the contribution API.
    fn modules(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
        Ok(vec![
            GeneratedFile::new("contracts/mod.rs", "pub mod profile;\n"),
            GeneratedFile::new("contracts/profile.rs", "pub struct Profile;\n"),
        ])
    }
    let mut registry = Registry::new();
    registry
        .register("legacy", &["contracts"], modules)
        .unwrap();
    let plan = registry
        .plan(
            &SchemaConfig::default(),
            vec![GeneratedFile::new("mod.rs", "pub mod contracts;\n")],
        )
        .unwrap();
    assert_eq!(plan.files["contracts/mod.rs"].0, "legacy");
    assert_eq!(plan.files["mod.rs"].0, "aggregate");
}
