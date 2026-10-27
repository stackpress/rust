//! G38 declaration, serialization and character semantics, independent of a driver.
#![cfg(feature = "codegen")]
use serde_json::json;
use stackpress_framework_schema::codegen::{DefinitionRegistry, NormalizeOptions};
use stackpress_framework_schema::{Field, RecordSchema, Schema, ValueContext};
use stackpress_idea_parser::{CompileInputs, ParseLimits, SchemaConfig, parse};

/// Normalize through the real Idea parser; malformed argument spelling is retained.
fn normalize(source: &str) -> stackpress_framework_schema::SchemaResult<Schema> {
    let config = parse("length.idea", source, &CompileInputs::default()).unwrap();
    Schema::normalize(
        &config,
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    )
}

/// Only an explicit positive integer on a stored scalar String/Text declares a width.
#[test]
fn declarations_and_diagnostics() {
    for argument in [
        "0",
        "-1",
        "1.5",
        "2.0",
        "2147483648",
        "\"2\"",
        "null",
        "2 3",
    ] {
        let error = normalize(&format!(
            "model X {{ id String @id @store.length({argument}) }}"
        ))
        .unwrap_err();
        assert_eq!(
            error.path().last().map(String::as_str),
            Some("store.length")
        );
    }
    for source in [
        "model X { id String @id @store.length }",
        "model X { id String @id @store.lenght(2) }",
        "model X { id Integer @id @store.length(2) }",
        "model X { id String @id list String[] @store.length(2) }",
        "type Y { name String @store.length(2) } model X { id String @id y Y }",
        "enum Y { A \"a\" } model X { id Y @id @store.length(2) }",
        "model X { id String @id other X? @store.length(2) }",
    ] {
        assert!(normalize(source).is_err(), "{source}");
    }
    let schema = normalize(
        "model X { id String @id title Text @store.length(2147483647) other String @max(2) }",
    )
    .unwrap();
    assert_eq!(
        schema.models()[0].fields[1].storage_length,
        Some(i32::MAX as u32)
    );
    assert_eq!(schema.models()[0].fields[2].storage_length, None);
    assert_eq!(stackpress_framework_schema::CONTRACT_REVISION, 3);
}

/// Width survives semantic hashing and JSON but absent width remains byte compatible.
#[test]
fn json_and_fingerprint() {
    let source = "model X { id String @id @store.length(2) }";
    let config = parse("length.idea", source, &CompileInputs::default()).unwrap();
    let serialized = serde_json::to_value(&config).unwrap();
    assert_eq!(
        serialized["model"]["X"]["columns"][0]["attributes"]["store.length"],
        json!([2])
    );
    let config = SchemaConfig::from_json(&serialized.to_string(), ParseLimits::default()).unwrap();
    let schema = Schema::normalize(
        &config,
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    )
    .unwrap();
    let field = &schema.models()[0].fields[0];
    let value = serde_json::to_value(field).unwrap();
    assert_eq!(value["storage_length"], json!(2));
    assert_eq!(
        serde_json::from_value::<Field>(value.clone()).unwrap(),
        *field
    );
    for rejected in [
        json!(0),
        json!(2147483648_u64),
        json!(-1),
        json!(2.0),
        json!("2"),
    ] {
        let mut value = value.clone();
        value["storage_length"] = rejected;
        assert!(serde_json::from_value::<Field>(value).is_err());
    }
    let plain = normalize("model X { id String @id @max(2) }").unwrap();
    assert!(
        serde_json::to_value(&plain.models()[0].fields[0])
            .unwrap()
            .get("storage_length")
            .is_none()
    );
    assert_ne!(schema.fingerprint().unwrap(), plain.fingerprint().unwrap());
    assert_ne!(
        schema.fingerprint().unwrap(),
        normalize("model X { id String @id @store.length(3) }")
            .unwrap()
            .fingerprint()
            .unwrap()
    );
}

/// Character count includes combining scalars, embedded NUL and trailing spaces.
#[test]
fn rules_defaults_and_characters() {
    for attributes in [
        "@max(3)",
        "@min(3)",
        "@is.cgt(2)",
        "@is.ceq(3)",
        "@is.ceq(-1)",
        "@is.ceq(1.5)",
    ] {
        assert!(
            normalize(&format!(
                "model X {{ id String @id @store.length(2) {attributes} }}"
            ))
            .is_err()
        );
    }
    for attributes in ["@max(1)", "@min(2)", "@is.cgt(1)", "@is.ceq(2)"] {
        assert!(
            normalize(&format!(
                "model X {{ id String @id @store.length(2) {attributes} }}"
            ))
            .is_ok()
        );
    }
    let schema = normalize("model X { id String @id @store.length(2) @max(2) }").unwrap();
    assert_eq!(
        schema.models()[0].fields[0]
            .rules
            .iter()
            .filter(|r| r.code == "cle")
            .count(),
        1
    );
    let explicit =
        normalize("model X { id String @id @store.length(2) @is.cle(2 \"two characters\") }")
            .unwrap();
    assert_eq!(explicit.models()[0].fields[0].rules.len(), 1);
    assert_eq!(
        explicit.models()[0].fields[0].rules[0].message,
        "two characters"
    );
    let record = RecordSchema {
        model: schema.models()[0].clone(),
        fieldsets: vec![],
        enums: vec![],
    };
    for value in ["", "ab", "😀😀", "e\u{301}", "a\0", "a "] {
        assert!(
            record
                .validate(&json!({"id":value}), &ValueContext::default(), false, true)
                .unwrap()
                .is_empty()
        );
    }
    for value in ["abc", "😀😀😀", "e\u{301}x", "a\0x", "a  "] {
        assert!(
            !record
                .validate(&json!({"id":value}), &ValueContext::default(), false, true)
                .unwrap()
                .is_empty()
        );
    }
    assert!(normalize("model X { id String @id @store.length(2) @default(\"abc\") }").is_err());
    assert!(normalize("model X { id String @id @store.length(2) @default(\"😀😀\") }").is_ok());
}

/// A sensitive writable field publishes a host rule with no private bound arguments.
#[test]
fn sensitive_rule_projection() {
    let config = parse(
        "length.idea",
        "model X { id String @id secret String @store.length(5) }",
        &CompileInputs::default(),
    )
    .unwrap();
    let registry = DefinitionRegistry::standard()
        .with_field_policy(
            "X",
            "secret",
            stackpress_framework_schema::FieldAccess {
                readable: false,
                writable: true,
                sensitive: true,
                publish_default: false,
            },
        )
        .unwrap();
    let schema = Schema::normalize(&config, &registry, NormalizeOptions::default()).unwrap();
    let public = schema.project_ui().unwrap();
    let field = &public.models[0].fields[0];
    assert_eq!(field.wire_name, "secret");
    assert!(
        field
            .validation
            .iter()
            .all(|rule| rule.args.is_empty() && !rule.client_checkable)
    );
    assert_eq!(schema.models()[0].fields[1].storage_length, Some(5));
}
