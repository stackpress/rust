//! Real wrapper composition, reference resolution, production planning and owned output IO.

use std::error::Error as StdError;

use stackpress_idea::{GenerateRequest, GenerationMode, generate, parser, transformer};
use transformer::{
    GeneratedFile, MemoryLoader, OutputPlan, OutputWorkspace, Registry, TransformContext,
    TransformInputs, TransformSpec,
};

/// Render only finalized attribute values, keeping literal reference lookalikes distinct.
fn emit(context: &TransformContext<'_>, plan: &mut OutputPlan) -> transformer::Result<()> {
    let schema = context.schema();
    // The callback cannot accidentally use unresolved imports/props as runtime contracts.
    assert!(schema.r#use.is_empty());
    assert!(schema.prop.is_empty());
    let parser::AttributeValue::Array(values) = &schema.model["Entry"].attributes["value"] else {
        panic!("missing finalized values");
    };
    let value = values[0]["value"].as_str().unwrap();
    assert_eq!(values[1].as_str(), Some("${Input}"));
    let suffix = context.input("templates/suffix.txt")?;
    plan.write("data/value.txt", format!("{value}{suffix}"))
}

/// Own an app root index separately from plugin partitions.
fn aggregate(_: &parser::SchemaConfig) -> transformer::Result<Vec<GeneratedFile>> {
    // Aggregate callbacks share the same finalized view as the registered transform.
    Ok(vec![GeneratedFile::new("index.txt", "generated")])
}

/// Failed callbacks must prevent the wrapper from reaching output installation.
fn reject(_: &TransformContext<'_>, _: &mut OutputPlan) -> transformer::Result<()> {
    Err(transformer::TransformError::new(
        "deliberate transform failure",
    ))
}

/// Construct an imported prop/model and a per-run explicit environment reference.
fn sources() -> MemoryLoader {
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("schema.idea", "use \"./models\"");
    loader.insert(
        "models.idea",
        concat!(
            "prop Input { value env(\"VALUE\") } ",
            "model Entry @value(Input \"${Input}\") { id String }",
        ),
    );
    loader
}

/// Supply the actual planner's declared snapshot and required runtime contract.
fn request(mode: GenerationMode) -> GenerateRequest<'static> {
    let mut request = GenerateRequest::new("schema.idea", aggregate, mode);
    request
        .inputs
        .environment
        .insert("VALUE".into(), "original".into());
    let mut inputs = TransformInputs::new();
    inputs.insert("templates/suffix.txt", "!").unwrap();
    request.selection = request.selection.with_inputs(inputs);
    request.selection.contract("text-runtime", "1").unwrap();
    request
}

/// Register a context callback with a genuine runtime-contract preflight requirement.
fn registry() -> Registry {
    let mut registry = Registry::new();
    registry
        .add(
            TransformSpec::new("text")
                .owns(&["data"])
                .requires_contract("text-runtime", "1"),
            emit,
        )
        .unwrap();
    registry
}

/// Find retained typed diagnostics rather than depending on contextual Display wording.
fn diagnostic<'a>(error: &'a (dyn StdError + 'static)) -> &'a parser::IdeaError {
    let mut current = Some(error);
    for _ in 0..32 {
        let Some(error) = current else { break };
        if let Some(diagnostic) = error.downcast_ref::<parser::IdeaError>() {
            return diagnostic;
        }
        current = error.source();
    }
    panic!("missing parser cause");
}

/// Compose/import/finalize/plan/install/check use one shared orchestration path.
#[test]
fn imports_references_auxiliary_inputs_and_check_mode() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let mut loader = sources();
    let registry = registry();

    // A successful check plans and resolves without creating the generated root.
    assert!(
        !generate(
            request(GenerationMode::Check),
            &loader,
            &registry,
            &workspace
        )
        .unwrap()
        .is_current()
    );
    assert!(!root.exists());
    generate(
        request(GenerationMode::Install),
        &loader,
        &registry,
        &workspace,
    )
    .unwrap();
    let manifest = std::fs::read(root.join(".stackpress-generated.json")).unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("data/value.txt")).unwrap(),
        "original!"
    );
    assert!(
        generate(
            request(GenerationMode::Check),
            &loader,
            &registry,
            &workspace
        )
        .unwrap()
        .is_current()
    );

    // Changing an imported source is stale, but check preserves all installed bytes.
    loader.insert(
        "models.idea",
        concat!(
            "prop Input { value \"changed\" } ",
            "model Entry @value(Input \"${Input}\") { id String }"
        ),
    );
    assert!(
        !generate(
            request(GenerationMode::Check),
            &loader,
            &registry,
            &workspace
        )
        .unwrap()
        .is_current()
    );
    assert_eq!(
        manifest,
        std::fs::read(root.join(".stackpress-generated.json")).unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(root.join("data/value.txt")).unwrap(),
        "original!"
    );
}

/// Acquisition, syntax and finalization failures preserve typed causes and prior output.
#[test]
fn source_and_finalization_failures_preserve_output() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let mut loader = sources();
    let registry = registry();
    generate(
        request(GenerationMode::Install),
        &loader,
        &registry,
        &workspace,
    )
    .unwrap();
    let manifest = std::fs::read(root.join(".stackpress-generated.json")).unwrap();

    // Parser and reference errors carry the imported source's original logical coordinates.
    for (source, code) in [
        ("model X {", parser::ErrorCode::UnexpectedEof),
        (
            "model Entry @value(Missing) { id String }",
            parser::ErrorCode::UnknownReference,
        ),
    ] {
        loader.insert("models.idea", source);
        let error = generate(
            request(GenerationMode::Install),
            &loader,
            &registry,
            &workspace,
        )
        .unwrap_err();
        let diagnostic = diagnostic(&error);
        assert_eq!(diagnostic.file, "models.idea");
        assert_eq!(diagnostic.code, code);
        assert_eq!(
            manifest,
            std::fs::read(root.join(".stackpress-generated.json")).unwrap()
        );
    }

    // Missing imports and caller-lowered composition limits fail before any output changes.
    loader.insert("schema.idea", "use \"missing\"");
    assert!(
        generate(
            request(GenerationMode::Install),
            &loader,
            &registry,
            &workspace
        )
        .is_err()
    );
    let mut bounded = request(GenerationMode::Install);
    bounded.limits.max_files = 1;
    assert!(generate(bounded, sources(), &registry, &workspace).is_err());
    assert_eq!(
        manifest,
        std::fs::read(root.join(".stackpress-generated.json")).unwrap()
    );
}

/// Contract/callback failures and manual files cannot be hidden by convenience orchestration.
#[test]
fn planning_and_install_failures_preserve_owned_output() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let loader = sources();
    let registry = registry();
    generate(
        request(GenerationMode::Install),
        &loader,
        &registry,
        &workspace,
    )
    .unwrap();
    let manifest = std::fs::read(root.join(".stackpress-generated.json")).unwrap();

    // Production plan_with sees selection contracts rather than silently dropping them.
    let mut invalid = request(GenerationMode::Install);
    invalid.selection = Default::default();
    assert!(generate(invalid, &loader, &registry, &workspace).is_err());
    let mut failing = Registry::new();
    failing
        .add(TransformSpec::new("text").owns(&["data"]), reject)
        .unwrap();
    let error = generate(
        request(GenerationMode::Install),
        &loader,
        &failing,
        &workspace,
    )
    .unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("deliberate transform failure")
    );
    assert_eq!(
        manifest,
        std::fs::read(root.join(".stackpress-generated.json")).unwrap()
    );

    // Both modes retain unknown files and return ownership conflicts through the same Result.
    let manual = root.join("manual.txt");
    std::fs::write(&manual, "keep me").unwrap();
    for mode in [GenerationMode::Check, GenerationMode::Install] {
        assert!(generate(request(mode), &loader, &registry, &workspace).is_err());
        assert_eq!(std::fs::read_to_string(&manual).unwrap(), "keep me");
    }
}

/// Independent requests cannot retain a prior environment or override caller-lowered bounds.
#[test]
fn per_run_environment_and_limits_remain_explicit() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let loader = sources();
    let registry = registry();
    generate(
        request(GenerationMode::Install),
        &loader,
        &registry,
        &workspace,
    )
    .unwrap();
    let original = std::fs::read(root.join(".stackpress-generated.json")).unwrap();

    // One changed snapshot makes output stale; a subsequent original snapshot is current again.
    let mut changed = request(GenerationMode::Check);
    changed
        .inputs
        .environment
        .insert("VALUE".into(), "different".into());
    assert!(
        !generate(changed, &loader, &registry, &workspace)
            .unwrap()
            .is_current()
    );
    assert!(
        generate(
            request(GenerationMode::Check),
            &loader,
            &registry,
            &workspace
        )
        .unwrap()
        .is_current()
    );

    // Missing environment is explicit policy, never inherited from a previous request/process.
    let mut absent = request(GenerationMode::Install);
    absent.inputs.environment.clear();
    let error = generate(absent, &loader, &registry, &workspace).unwrap_err();
    assert_eq!(
        diagnostic(&error).code,
        parser::ErrorCode::MissingEnvironment
    );
    let mut compatibility = request(GenerationMode::Check);
    compatibility.inputs.environment.clear();
    compatibility.inputs.missing_env_is_empty = true;
    assert!(
        !generate(compatibility, &loader, &registry, &workspace)
            .unwrap()
            .is_current()
    );

    // The parser checks the moved environment snapshot against the caller's smaller budget.
    let mut bounded = request(GenerationMode::Install);
    bounded.inputs.limits.max_bytes = 128;
    bounded
        .inputs
        .environment
        .insert("VALUE".into(), "x".repeat(129));
    assert_eq!(
        diagnostic(&generate(bounded, &loader, &registry, &workspace).unwrap_err()).code,
        parser::ErrorCode::Limit
    );
    assert_eq!(
        original,
        std::fs::read(root.join(".stackpress-generated.json")).unwrap()
    );
}
