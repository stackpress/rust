//! Callers classify failures through a stable enum and separately inspect private sources.
use stackpress_idea_transformer::{
    Composer, CompositionLimits, GeneratedFile, MemoryLoader, OutputWorkspace, Registry,
    TransformError, TransformErrorKind as Kind,
};
use std::error::Error;

#[path = "support/watchdog.rs"]
mod watchdog;

/// Callers distinguish phases and original causes independently of diagnostic wording.
#[test]
fn categories_do_not_depend_on_display_text() {
    assert_eq!(TransformError::new("arbitrary wording").kind(), Kind::Other);
    let io = TransformError::from(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
    assert_eq!(io.kind(), Kind::Io);
    assert!(io.source().unwrap().is::<std::io::Error>());
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("root.idea", "model {");
    let parse = Composer::new(loader.clone(), CompositionLimits::default())
        .compose("root")
        .unwrap_err();
    assert_eq!(parse.kind(), Kind::Parse);
    assert!(parse.source().is_some());
    let limits = CompositionLimits {
        max_files: 0,
        ..CompositionLimits::default()
    };
    let limit = Composer::new(loader, limits).compose("root").unwrap_err();
    assert_eq!(limit.kind(), Kind::Limit);
    let mut registry = Registry::new();
    registry
        .register("x", &["data"], |_| {
            Err(std::io::Error::other("PRIVATE callback").into())
        })
        .unwrap();
    let callback = registry.plan(&Default::default(), vec![]).unwrap_err();
    assert_eq!(callback.kind(), Kind::Callback);
    assert!(callback.source().unwrap().to_string().contains("PRIVATE"));
    assert!(!callback.to_string().contains("PRIVATE"));
    assert_eq!(
        registry
            .register("x", &["other"], |_| Ok(vec![]))
            .unwrap_err()
            .kind(),
        Kind::Selection
    );
}

/// Real managed siblings exercise lock, recovery and unowned-tree rejection.
#[test]
#[ignore = "run by kind watchdog"]
fn output_lock_recovery_and_ownership_have_distinct_kinds() {
    let directory = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
    let workspace = OutputWorkspace::new(directory.path().join("generated")).unwrap();
    let plan = || {
        Registry::new()
            .plan(&Default::default(), vec![GeneratedFile::new("a.txt", "a")])
            .unwrap()
    };
    let lock = directory.path().join(".generated.stackpress-lock");
    // Use the documented fixed sibling spelling from OutputWorkspace construction.
    std::fs::write(&lock, "owner").unwrap();
    assert_eq!(
        workspace.check(&plan()).unwrap_err().kind(),
        Kind::LockExists
    );
    std::fs::remove_file(lock).unwrap();
    let journal = directory.path().join(".generated.stackpress-journal");
    std::fs::write(&journal, "intent").unwrap();
    assert_eq!(
        workspace.check(&plan()).unwrap_err().kind(),
        Kind::RecoveryRequired
    );
    std::fs::remove_file(journal).unwrap();
    std::fs::create_dir(directory.path().join("generated")).unwrap();
    std::fs::write(directory.path().join("generated/manual.txt"), "manual").unwrap();
    assert_eq!(
        workspace.check(&plan()).unwrap_err().kind(),
        Kind::Ownership
    );
}

/// Native ownership probes cannot hang the caller's test process.
#[test]
fn output_kind_contracts_are_watchdog_bounded() {
    let result = watchdog::run(
        std::process::Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "output_lock_recovery_and_ownership_have_distinct_kinds",
            "--nocapture",
        ]),
        "error-kinds",
        std::time::Duration::from_secs(2),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}

/// Parser and serde visitor budgets retain Limit even behind contextual source wrappers.
#[test]
fn parser_and_json_visitor_budgets_are_machine_readable() {
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("root.json", "{}");
    let limits = CompositionLimits {
        max_json_nodes: 0,
        ..CompositionLimits::default()
    };
    assert_eq!(
        Composer::new(loader.clone(), limits)
            .compose("root")
            .unwrap_err()
            .kind(),
        Kind::Limit
    );
    loader.insert("schema.idea", "model A { id String }");
    let inputs = stackpress_idea_parser::CompileInputs {
        limits: stackpress_idea_parser::ParseLimits {
            max_bytes: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        Composer::new(loader, CompositionLimits::default())
            .with_inputs(inputs)
            .compose("schema")
            .unwrap_err()
            .kind(),
        Kind::Limit
    );
}
