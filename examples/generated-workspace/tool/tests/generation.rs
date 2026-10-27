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
    // A fixed public prop reused across fields must consume one aggregate resolved-role budget.
    let component = stackpress_framework_schema::codegen::ComponentDefinition {
        name: "field.input".into(),
        key: "app.input".into(),
        revision: "1".into(),
        props: vec![stackpress_framework_schema::codegen::PropDefinition {
            name: "label".into(),
            value_type: stackpress_framework_schema::codegen::PropType::String,
            required: false,
            public: true,
        }],
        fixed: [("label".into(), serde_json::json!("x".repeat(600_000)))].into(),
        available: false,
    };
    let registry = DefinitionRegistry::standard()
        .with_component(component, None)
        .unwrap();
    let roles = parse(
        "roles.idea",
        "model X { id String @id a String @field.input b String @field.input }",
        &CompileInputs::default(),
    )
    .unwrap();
    assert_eq!(
        Schema::normalize(&roles, &registry, NormalizeOptions::default())
            .unwrap_err()
            .kind(),
        SchemaErrorKind::Limit
    );
    // Reusing one declared fieldset does not reset preparation's aggregate default budget.
    let mut config=parse("defaults.idea","type Payload { text String @default(\"x\") } model X { id String @id a Payload b Payload }",&CompileInputs::default()).unwrap();
    config.r#type["Payload"].columns[0].attributes.insert(
        "default".into(),
        AttributeValue::Array(vec![serde_json::Value::String("x".repeat(600_000))]),
    );
    let schema = Schema::normalize(
        &config,
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    )
    .unwrap();
    let input = stackpress_framework_schema::InputSchema {
        record: stackpress_framework_schema::RecordSchema {
            model: schema.models()[0].clone(),
            fieldsets: schema.fieldsets().to_vec(),
            enums: Vec::new(),
        },
        patch: false,
    };
    assert_eq!(
        input
            .prepare(
                serde_json::json!({"id":"x","a":{},"b":{}}),
                &stackpress_framework_schema::ValueContext::default()
            )
            .unwrap_err()
            .kind(),
        SchemaErrorKind::Limit
    );
}

/// Generate the entire accepted M1 type corpus independently of native application compilation.
#[test]
fn full_type_corpus_generates() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../contracts/matrix.idea");
    let result = generated_workspace_tool::generate(&input, &root.path().join("output"), false);
    assert!(result.is_ok(), "{result:#?}");
}

/// Owned rename/removal prunes stale fields/modules, preserves sentinels, and refuses edited output.
#[test]
fn remove_rename_and_failure_preserve_output() {
    use std::collections::BTreeMap;
    fn snapshot(path: &std::path::Path) -> BTreeMap<String, Vec<u8>> {
        fn walk(
            root: &std::path::Path,
            path: &std::path::Path,
            out: &mut BTreeMap<String, Vec<u8>>,
        ) {
            for entry in std::fs::read_dir(path).unwrap() {
                let entry = entry.unwrap();
                let p = entry.path();
                if p.is_dir() {
                    walk(root, &p, out);
                } else {
                    out.insert(
                        p.strip_prefix(root).unwrap().to_string_lossy().into(),
                        std::fs::read(p).unwrap(),
                    );
                }
            }
        }
        let mut out = BTreeMap::new();
        walk(path, path, &mut out);
        out
    }
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let input = root.path().join("schema.idea");
    let output = root.path().join("generated");
    let first = "plugin \"framework-schema\" {} plugin \"framework-sql\" {} model Old { id String @id title String } model Keep { id String @id old String }";
    std::fs::write(&input, first).unwrap();
    generated_workspace_tool::generate(&input, &output, false).unwrap();
    std::fs::write(root.path().join("sentinel.txt"), "owned by app").unwrap();
    let second = "plugin \"framework-schema\" {} plugin \"framework-sql\" {} model Keep { id String @id renamed String }";
    std::fs::write(&input, second).unwrap();
    assert!(
        !generated_workspace_tool::generate(&input, &output, true)
            .unwrap()
            .is_current()
    );
    generated_workspace_tool::generate(&input, &output, false).unwrap();
    assert!(!output.join("contracts/old.rs").exists());
    assert!(!output.join("stores/old.rs").exists());
    assert!(
        !std::fs::read_to_string(output.join("stores/operations.rs"))
            .unwrap()
            .contains("pub mod old")
    );
    assert!(
        std::fs::read_to_string(output.join("stores/keep.rs"))
            .unwrap()
            .contains("wire: \"renamed\"")
    );
    assert!(
        std::fs::read_to_string(output.join("contracts/keep.rs"))
            .unwrap()
            .contains("pub renamed:")
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("sentinel.txt")).unwrap(),
        "owned by app"
    );
    let expected = snapshot(&output);
    generated_workspace_tool::generate(&input, &output, false).unwrap();
    assert_eq!(snapshot(&output), expected);
    std::fs::write(
        &input,
        second.replace("renamed String", "renamed String @is.typo"),
    )
    .unwrap();
    assert!(generated_workspace_tool::generate(&input, &output, false).is_err());
    assert_eq!(snapshot(&output), expected);
    std::fs::write(&input, second).unwrap();
    std::fs::write(output.join("contracts/keep.rs"), "// manual edit\n").unwrap();
    let edited = snapshot(&output);
    assert!(generated_workspace_tool::generate(&input, &output, false).is_err());
    assert_eq!(snapshot(&output), edited);
}

/// Native generation failures retain winning Idea spans and structural-only normalized JSON paths.
#[test]
fn store_diagnostics_preserve_source() {
    use stackpress_framework_schema::{SchemaError, SchemaErrorKind};
    use stackpress_idea::parser::{CompileInputs, parse};
    use std::error::Error;
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let output = root.path().join("generated");
    let input = root.path().join("schema.idea");
    let selection = "@query(\"owner.projects.owner.projects.title\")";
    let text = format!(
        "plugin \"framework-schema\" {{}} plugin \"framework-sql\" {{}}\nmodel Profile {{ id String @id name String projects Project[] }}\nmodel Project {selection} {{ id String @id ownerId String title String owner Profile @relation({{ local \"ownerId\" foreign \"id\" name \"projects\" }}) }}"
    );
    // A valid prior install must survive either failed native plan unchanged.
    std::fs::write(&input, text.replace(selection, "")).unwrap();
    generated_workspace_tool::generate(&input, &output, false).unwrap();
    let before = std::fs::read(output.join("stores/project.rs")).unwrap();
    for normalized in [false, true] {
        let source = if normalized {
            let config = parse("source.idea", &text, &CompileInputs::default()).unwrap();
            let path = root.path().join("schema.json");
            std::fs::write(&path, serde_json::to_vec(&config).unwrap()).unwrap();
            path
        } else {
            std::fs::write(&input, &text).unwrap();
            input.clone()
        };
        let error = generated_workspace_tool::generate(&source, &output, false).unwrap_err();
        let mut current: &(dyn Error + 'static) = &error;
        let diagnostic = loop {
            if let Some(diagnostic) = current.downcast_ref::<SchemaError>() {
                break diagnostic;
            }
            current = current.source().expect("typed native diagnostic retained");
        };
        assert_eq!(
            diagnostic.kind(),
            SchemaErrorKind::Unsupported,
            "normalized={normalized} {diagnostic:?}"
        );
        assert_eq!(
            diagnostic.to_string(),
            "default selected storage projection is unsupported"
        );
        assert_eq!(diagnostic.path(), ["model", "Project"]);
        let location = diagnostic.location().expect("winning declaration origin");
        assert!(location.source.ends_with(if normalized {
            "schema.json"
        } else {
            "schema.idea"
        }));
        if normalized {
            assert_eq!(location.bytes, None);
        } else {
            let (start, end) = location.bytes.unwrap();
            assert!(text[start..end].contains("Project"));
        }
        assert_eq!(
            std::fs::read(output.join("stores/project.rs")).unwrap(),
            before
        );
    }
}
