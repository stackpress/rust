//! Real generation, bootstrap and repeatability proof against the accepted three-model schema.

/// Generate from absent output; repeat/check must preserve every byte.
#[test]
fn real_schema_generates_repeatably() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../app/schema.idea");
    let output = root.path().join("generated");
    let result = generated_workspace_tool::generate(&input, &output, false);
    assert!(result.is_ok(), "{result:#?}");
    assert!(
        generated_workspace_tool::generate(&input, &output, true)
            .unwrap()
            .is_current()
    );
    assert!(generated_workspace_tool::generate(&input, &output, false).is_ok());
}

/// Run the actual CLI grammar/host/worker path with bounded logs and kill/reap on every exit.
#[test]
fn cli_generation_and_stale_check() {
    use std::process::{Child, Command, Stdio};
    use std::time::{Duration, Instant};
    struct Guard(Child);
    impl Drop for Guard {
        /// Reap both successful children and watchdog failures.
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let input = root.path().join("schema.idea");
    let output = root.path().join("generated");
    std::fs::write(&input, include_str!("../../app/schema.idea")).unwrap();
    generated_workspace_tool::generate(&input, &output, false).unwrap();
    for stale in [false, true] {
        if stale {
            std::fs::write(&input, "plugin \"framework-schema\" {} plugin \"framework-sql\" {} model Profile { id String @id name String }").unwrap();
        }
        let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
        let file = std::fs::File::create(log.path()).unwrap();
        let mut child = Guard(
            Command::new(env!("CARGO_BIN_EXE_generated-workspace-tool"))
                .args(["emit", "generate", "--input"])
                .arg(&input)
                .arg("--output")
                .arg(&output)
                .args(["--check", "true"])
                .stdout(Stdio::from(file.try_clone().unwrap()))
                .stderr(Stdio::from(file))
                .spawn()
                .unwrap(),
        );
        let start = Instant::now();
        let status = loop {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "CLI generation timed out"
            );
            assert!(log.as_file().metadata().unwrap().len() < 128 * 1024);
            if let Some(status) = child.0.try_wait().unwrap() {
                break status;
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let result = std::fs::read_to_string(log.path()).unwrap();
        assert_eq!(status.success(), !stale, "{result}");
    }
}

#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Direct caller-owned config must obey one aggregate budget, not one budget per default.
#[test]
fn direct_config_budget_is_global() {
    use stackpress_framework_schema::{
        Schema, SchemaErrorKind,
        codegen::{DefinitionRegistry, NormalizeOptions},
    };
    use stackpress_idea::parser::{AttributeValue, CompileInputs, parse};
    if std::env::var("STACKPRESS_S6_SCENARIO").as_deref() != Ok("schema-budget") {
        let (ok, expired, output) = watchdog::run(
            "direct_config_budget_is_global",
            "schema-budget",
            std::time::Duration::from_secs(10),
        );
        assert!(ok && !expired, "config timeout={expired}\n{output}");
        return;
    }
    let mut config = parse(
        "direct.idea",
        "model Example { id String @id a String b String }",
        &CompileInputs::default(),
    )
    .unwrap();
    for column in &mut config.model["Example"].columns[1..] {
        column.attributes.insert(
            "default".into(),
            AttributeValue::Array(vec![serde_json::Value::String("x".repeat(600_000))]),
        );
    }
    let result = Schema::normalize(
        &config,
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    );
    assert!(
        result.is_err(),
        "aggregate default bytes exceeded the schema budget without refusal"
    );
    assert_eq!(result.err().unwrap().kind(), SchemaErrorKind::Limit);
    let mut deep = serde_json::Value::Null;
    for _ in 0..34 {
        deep = serde_json::Value::Array(vec![deep]);
    }
    config.model["Example"].columns[1]
        .attributes
        .insert("default".into(), AttributeValue::Array(vec![deep]));
    assert_eq!(
        Schema::normalize(
            &config,
            &DefinitionRegistry::standard(),
            NormalizeOptions::default()
        )
        .err()
        .unwrap()
        .kind(),
        SchemaErrorKind::Limit
    );
}
