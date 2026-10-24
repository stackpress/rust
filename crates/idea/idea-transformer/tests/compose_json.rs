//! Normalized JSON composition with strict shape/duplicate/range and pre-recursion limits.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::json;
use stackpress_idea_parser::{CompileInputs, ParseLimits};
use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, ProjectLoader};

/// Compose JSON as a root data source without importing the native filesystem.
fn from_json(
    text: &str,
) -> stackpress_idea_transformer::Result<stackpress_idea_transformer::ComposedSchema> {
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("root.json", text);
    Composer::new(loader, CompositionLimits::default()).compose("root.json")
}

#[test]
/// Parsed Idea and genuinely loaded normalized JSON yield identical ordered merged config.
fn normalized_json_and_idea_imports_are_equivalent() {
    let idea = "prop Config { z [1 true null { nested \"literal\" }] a \"${Config}\" }
        enum Role { Z \"z\" A 18446744073709551615 }
        type Address { zip String? }
        model User @custom.flag @label(\"User\") { id String @id names String[]? }";
    let normalized = serde_json::to_string(
        &stackpress_idea_parser::parse("child.idea", idea, &CompileInputs::default()).unwrap(),
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("child.idea"), idea).unwrap();
    std::fs::write(directory.path().join("child.json"), &normalized).unwrap();
    for (root, imported) in [("idea.idea", "child.idea"), ("json.idea", "child.json")] {
        std::fs::write(
            directory.path().join(root),
            format!("use \"{imported}\" model User {{ local String }}"),
        )
        .unwrap();
    }
    let composer = Composer::new(
        ProjectLoader::new(directory.path()).unwrap(),
        CompositionLimits::default(),
    );
    let idea = composer.compose("idea.idea").unwrap();
    let imported_json = composer.compose("json.idea").unwrap();
    assert_eq!(
        serde_json::to_string(&idea.config).unwrap(),
        serde_json::to_string(&imported_json.config).unwrap()
    );
    assert_eq!(
        imported_json.config.prop["Config"]
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["z", "a"]
    );
    assert_eq!(
        imported_json.config.r#enum["Role"]
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Z", "A"]
    );
}

#[test]
/// JSON can import JSON/Idea transitively and root plugin order stays explicit.
fn json_root_transitive_uses_and_plugin_order() {
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert(
        "root.json",
        r#"{"use":["nested/child.json"],"plugin":{"z":{},"a":{}}}"#,
    );
    loader.insert(
        "nested/child.json",
        r#"{"use":["../base.idea"],
        "plugin":{"inactive":{}},"prop":{"Config":{"url":"${Config}"}}}"#,
    );
    loader.insert("base.idea", "model User { id String }");
    let result = Composer::new(loader, CompositionLimits::default())
        .compose("root.json")
        .unwrap();
    assert!(result.config.model.contains_key("User"));
    assert_eq!(
        result
            .config
            .plugin
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["z", "a"]
    );
    assert_eq!(result.config.prop["Config"]["url"], "${Config}");
    assert_eq!(result.sources.len(), 3);
    assert_eq!(result.imports.len(), 2);
}

#[test]
/// Malformed roots, duplicate keys at every level and invalid normalized fields never disappear.
fn malformed_json_and_duplicate_keys_are_errors() {
    for invalid in [
        "",
        "null",
        "[]",
        "{",
        "{} {}",
        "{\"unknown\":{}}",
        "{\"model\":null}",
        r#"{"prop":{},"prop":{}}"#,
        r#"{"prop":{"P":{"x":1,"x":2}}}"#,
        r#"{"prop":{"P":{"x":[{"a":1,"a":2}]}}}"#,
        r#"{"use":{}}"#,
        r#"{"use":[1]}"#,
        r#"{"plugin":{"p":1}}"#,
        r#"{"prop":{"lower":{}}}"#,
        r#"{"prop":{"P":{"bad-key":0}}}"#,
        r#"{"prop":{"P":{}},"enum":{"P":{}}}"#,
        r#"{"enum":{"E":{"lower":1}}}"#,
        r#"{"enum":{"E":{"A":{}}}}"#,
        r#"{"model":{"M":{}}}"#,
        r#"{"model":{"M":{"name":"Other","mutable":true,"attributes":{},"columns":[]}}}"#,
        r#"{"model":{"M":{"name":"M","mutable":1,"attributes":{},"columns":[]}}}"#,
        r#"{"model":{"M":{"name":"M","mutable":true,"attributes":{},"columns":{}}}}"#,
        r#"{"model":{"M":{"name":"M","mutable":true,"attributes":{"bad":1},"columns":[]}}}"#,
        r#"{"model":{"M":{"name":"M","mutable":true,"attributes":{"bad.Name":[]},"columns":[]}}}"#,
        r#"{"model":{"M":{"name":"M","mutable":true,"attributes":{},"columns":[],"extra":0}}}"#,
    ] {
        assert!(
            from_json(invalid).is_err(),
            "unexpectedly accepted: {invalid}"
        );
    }
}

#[test]
/// Every column modifier/shape is required, names stay unique and complete columns are retained.
fn normalized_columns_are_validated() {
    let valid_column = json!({"name":"id", "type":"String", "required":true,
        "multiple":false, "attributes":{"flag":false,"method":[[1],{"x":null}]}});
    let mut valid = json!({"model":{"M":{"name":"M","mutable":true,
        "attributes":{},"columns":[valid_column.clone()]}}});
    assert!(from_json(&valid.to_string()).is_ok());
    for field in ["name", "type", "required", "multiple", "attributes"] {
        let mut invalid = valid.clone();
        invalid["model"]["M"]["columns"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(from_json(&invalid.to_string()).is_err());
    }
    for (field, value) in [
        ("name", json!("Id")),
        ("name", json!("")),
        ("type", json!("String?")),
        ("type", json!("string")),
        ("multiple", json!("false")),
        ("required", json!(null)),
        ("attributes", json!([])),
        ("extra", json!(1)),
    ] {
        let mut invalid = valid.clone();
        invalid["model"]["M"]["columns"][0][field] = value;
        assert!(from_json(&invalid.to_string()).is_err());
    }
    valid["model"]["M"]["columns"]
        .as_array_mut()
        .unwrap()
        .push(valid_column);
    assert!(from_json(&valid.to_string()).is_err());
}

#[test]
/// Exact integer endpoints survive, overflow never rounds and quoted numbers stay literal.
fn exact_numbers_and_literal_strings() {
    let result = from_json(
        r#"{"prop":{"P":{"min":-9223372036854775808,
        "max":18446744073709551615,"fraction":-0.5,"exponent":1e3,
        "quoted":"18446744073709551616","escaped":"a\"18446744073709551616"}}}"#,
    )
    .unwrap();
    assert_eq!(result.config.prop["P"]["min"].as_i64(), Some(i64::MIN));
    assert_eq!(result.config.prop["P"]["max"].as_u64(), Some(u64::MAX));
    assert_eq!(result.config.prop["P"]["fraction"], -0.5);
    for number in [
        "-9223372036854775809",
        "18446744073709551616",
        "1e999",
        "NaN",
        "01",
    ] {
        assert!(
            from_json(&format!("{{\"prop\":{{\"P\":{{\"x\":{number}}}}}}}")).is_err(),
            "accepted {number}"
        );
    }
}

#[test]
/// Lowered JSON depth/node/byte budgets fail before unbounded data construction.
fn json_lowered_budgets() {
    let mut loader = MemoryLoader::new("/project").unwrap();
    loader.insert("root.json", r#"{"prop":{"P":{"x":[[1]]}}}"#);
    let limits = CompositionLimits {
        max_json_nodes: 4,
        ..CompositionLimits::default()
    };
    assert!(Composer::new(&loader, limits).compose("root.json").is_err());
    let inputs = CompileInputs {
        limits: ParseLimits {
            max_depth: 3,
            max_bytes: 1024,
        },
        ..CompileInputs::default()
    };
    assert!(
        Composer::new(&loader, CompositionLimits::default())
            .with_inputs(inputs)
            .compose("root.json")
            .is_err()
    );
    let inputs = CompileInputs {
        limits: ParseLimits {
            max_depth: 64,
            max_bytes: 2,
        },
        ..CompileInputs::default()
    };
    assert!(
        Composer::new(&loader, CompositionLimits::default())
            .with_inputs(inputs)
            .compose("root.json")
            .is_err()
    );
}

#[test]
/// The destructive-depth fixture runs only in a subprocess selected explicitly by its parent.
fn deep_json_child() {
    if std::env::var_os("STACKPRESS_COMPOSE_DEEP_CHILD").is_none() {
        return;
    }
    let text = format!(
        "{{\"prop\":{{\"P\":{{\"x\":{}0{}}}}}}}",
        "[".repeat(20_000),
        "]".repeat(20_000)
    );
    assert!(from_json(&text).is_err());
    // Include the maximum accepted native JSON depth and one above in the watchdog process.
    for (arrays, accepted) in [(61, true), (62, false)] {
        let text = format!(
            "{{\"prop\":{{\"P\":{{\"x\":{}0{}}}}}}}",
            "[".repeat(arrays),
            "]".repeat(arrays)
        );
        assert_eq!(from_json(&text).is_ok(), accepted);
    }
}

#[test]
/// A malformed 20,000-level JSON input returns an error and exits normally under a watchdog.
fn deep_json_fails_without_stack_abort() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "deep_json_child", "--nocapture"])
        .env("STACKPRESS_COMPOSE_DEEP_CHILD", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            let output = child.wait_with_output().unwrap();
            assert!(
                status.success(),
                "child failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("deep JSON child exceeded finite watchdog");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
