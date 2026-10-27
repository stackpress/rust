//! Bounded semantic normalization and pure missing/null/error contracts for M0.

use stackpress_framework_schema::{Input, SchemaError, SchemaErrorKind};
use std::error::Error;

/// Pure contracts preserve omission/null/supplied values and keep causes out of Display.
#[test]
fn pure_input_and_errors() {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct Form {
        #[serde(default, skip_serializing_if = "Input::is_missing")]
        value: Input<bool>,
    }
    let missing: Form = serde_json::from_str("{}").unwrap();
    let null: Form = serde_json::from_str("{\"value\":null}").unwrap();
    let supplied: Form = serde_json::from_str("{\"value\":false}").unwrap();
    assert_eq!(missing.value, Input::Missing);
    assert_eq!(null.value, Input::Null);
    assert_eq!(supplied.value, Input::Value(false));
    assert_eq!(serde_json::to_string(&missing).unwrap(), "{}");
    assert_eq!(serde_json::to_string(&null).unwrap(), "{\"value\":null}");
    assert!(serde_json::to_string(&Input::<bool>::Missing).is_err());
    let error = SchemaError::new(SchemaErrorKind::Codec, "decode failed")
        .with_cause(std::io::Error::other("secret sentinel"));
    assert_eq!(error.kind(), SchemaErrorKind::Codec);
    assert_eq!(error.to_string(), "decode failed");
    assert!(
        error
            .source()
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
}

#[cfg(feature = "codegen")]
mod normalization {
    use stackpress_framework_schema::{
        Schema,
        codegen::{DefinitionRegistry, DiagnosticMap, NormalizeOptions, portable_name},
    };
    use stackpress_idea_parser::{CompileInputs, parse};
    use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};

    /// Use the real existing parser to construct semantic test inputs.
    fn normalize(source: &str) -> stackpress_framework_schema::SchemaResult<Schema> {
        let config = parse("test.idea", source, &CompileInputs::default()).unwrap();
        Schema::normalize(
            &config,
            &DefinitionRegistry::standard(),
            NormalizeOptions::default(),
        )
    }

    /// Keyword/long-name handling is deterministic; collisions/traversal/nonportable names refuse.
    #[test]
    fn checked_names_and_registry() {
        assert_eq!(portable_name("ownerId").unwrap(), "owner_id");
        assert_eq!(portable_name("type").unwrap(), "type_");
        assert_eq!(portable_name("HTTPServer").unwrap(), "http_server");
        let long = "VeryLong".repeat(20);
        assert_eq!(portable_name(&long).unwrap().len(), 63);
        assert_eq!(portable_name(&long).unwrap(), portable_name(&long).unwrap());
        for invalid in ["", "../escape", "a__b", "über", "a\"b", "1value"] {
            assert!(portable_name(invalid).is_err());
        }
        assert!(
            DefinitionRegistry::checked(vec![
                ("is.ne".into(), "1".into()),
                ("is.ne".into(), "2".into())
            ])
            .is_err()
        );
        assert!(DefinitionRegistry::checked(vec![("".into(), "1".into())]).is_err());
        assert!(DefinitionRegistry::checked(vec![("is.ne".into(), "1".into())]).is_ok());
        assert!(
            normalize("model Profile { id String @id } model PROFILE { id String @id }").is_err()
        );
        assert!(normalize("model X { id String @id ownerId String owner_id String }").is_err());
    }

    /// Ordinary supported values/defaults/enums/fieldsets normalize with stable independent bytes.
    #[test]
    fn supported_and_unsupported_semantics() {
        let active = normalize("model Active { id String @id active Boolean @active }").unwrap();
        assert_eq!(
            active.models()[0].fields[1].default,
            Some(serde_json::json!(true))
        );
        let inactive =
            normalize("model Active { id String @id active Boolean @active @default(false) }")
                .unwrap();
        assert_eq!(
            inactive.models()[0].fields[1].default,
            Some(serde_json::json!(false))
        );
        let schema = normalize("enum State { OPEN \"open\" CLOSED \"closed\" } type Address { city String @is.ne } model Example { id Integer @id title String @is.ne @default(\"${literal}\") active Boolean @active @default(true) state State @default(\"open\") tags String[] @default([]) address Address? }").unwrap();
        assert_eq!(schema.models().len(), 1);
        assert_eq!(schema.fieldsets().len(), 1);
        assert_eq!(schema.enums().len(), 1);
        assert_eq!(
            schema.models()[0].fields[1].default.as_ref().unwrap(),
            "${literal}"
        );
        assert_eq!(
            schema.fingerprint().unwrap(),
            schema.clone().fingerprint().unwrap()
        );
        let cases = [
            "model X { id String }",
            "model X { id String? @id }",
            "model X { id String[] @id }",
            "model X { id String @id active String @active }",
            "model X { id String @id title String @is.typo }",
            "model X { id String @id title String @hashed }",
            "model X { id String @id title String @default(false) }",
            "model X { id String @id title String @default(null) }",
            "model X { id String @id active Boolean @default(0) }",
            "model X { id String @id count Integer @default(1.5) }",
            "model X { id String @id tags String[] @default([false]) }",
            "model X { id String @id title Missing }",
            "model X { id String @id title String @is.ne @default(\"\") }",
            "type A { child B? } type B { child A? } model X { id String @id value A? }",
            "enum State { A \"same\" B \"same\" } model X { id String @id state State }",
            "model X @unknown { id String @id }",
            "model X { id String @id label String @searchable(false) }",
        ];
        for source in cases {
            assert!(normalize(source).is_err(), "unexpected support: {source}");
        }
        let config = parse(
            "direct.idea",
            "model X { id String @id }",
            &CompileInputs::default(),
        )
        .unwrap();
        assert!(
            Schema::normalize(
                &config,
                &DefinitionRegistry::standard(),
                NormalizeOptions {
                    declarations: 0,
                    fields: 128
                }
            )
            .is_err()
        );
        assert!(
            Schema::normalize(
                &config,
                &DefinitionRegistry::standard(),
                NormalizeOptions {
                    declarations: 128,
                    fields: 0
                }
            )
            .is_err()
        );
        let mut mismatch = config.clone();
        mismatch.model["X"].name = "Other".into();
        assert!(
            Schema::normalize(
                &mismatch,
                &DefinitionRegistry::standard(),
                NormalizeOptions::default()
            )
            .is_err()
        );
        let mut unresolved = config;
        unresolved.r#use.push("unread.idea".into());
        assert!(
            Schema::normalize(
                &unresolved,
                &DefinitionRegistry::standard(),
                NormalizeOptions::default()
            )
            .is_err()
        );
    }

    /// Known provider expressions cannot silently become literal IDs when no provider exists.
    #[test]
    fn unavailable_providers_do_not_fall_back_to_literals() {
        for provider in ["cuid()", "nano()", "random()", "nanoid(12)"] {
            let source =
                format!("model Example {{ id String @id token String @default({provider:?}) }}");
            assert_eq!(
                normalize(&source).err().unwrap().kind(),
                stackpress_framework_schema::SchemaErrorKind::Unsupported
            );
        }
        // A provider-looking string inside a literal array remains ordinary element data.
        assert!(
            normalize("model Example { id String @id tags String[] @default([\"cuid()\"]) }")
                .is_ok()
        );
    }

    /// Owning/reverse mappings validate actual scalar keys and never choose an ambiguous inverse.
    #[test]
    fn relation_contracts() {
        assert!(normalize("model Invalid { id String @id foreign String @relation({ local \"foreign\" foreign \"id\" }) }").is_err());
        let valid = "model Parent { id String @id children Child[] } model Child { id String @id parentId String parent Parent @relation({ local \"parentId\" foreign \"id\" name \"children\" }) }";
        assert_eq!(normalize(valid).unwrap().models().len(), 2);
        for source in [
            valid.replace("local \"parentId\"", "local \"missing\""),
            valid.replace("foreign \"id\"", "foreign \"missing\""),
            valid.replace("parentId String", "parentId Integer"),
            valid.replace("parentId String", "parentId String?"),
            valid.replace("name \"children\"", "name \"other\""),
            valid.replace("parent Parent", "parent Parent[]"),
            valid.replace("children Child[]", "children Child"),
            valid.replace("name \"children\"", "unknown \"children\""),
        ] {
            assert!(normalize(&source).is_err());
        }
    }

    /// Imported winning attribute coordinates come from composer provenance and the checked AST.
    #[test]
    fn diagnostic_map_keeps_imported_winning_attribute() {
        let mut loader = MemoryLoader::new("/project").unwrap();
        loader.insert(
            "root.idea",
            "use \"child\" model Example { id String @id local String }",
        );
        let child = "// Unicode prefix: Café\nmodel Example { imported String @is.ne }";
        loader.insert("child.idea", child);
        let composed = Composer::new(loader, CompositionLimits::default())
            .compose("root.idea")
            .unwrap();
        let map = DiagnosticMap::from_composed(&composed).unwrap();
        let entry = map
            .entries
            .iter()
            .find(|entry| {
                entry.path
                    == [
                        "model",
                        "Example",
                        "columns",
                        "imported",
                        "attributes",
                        "is.ne",
                    ]
            })
            .unwrap();
        assert!(entry.source.ends_with("child.idea"));
        let (start, end) = entry.bytes.unwrap();
        assert!(child[start..end].contains("is.ne"));
        let root = map
            .entries
            .iter()
            .find(|entry| entry.path == ["model", "Example", "columns", "local"])
            .unwrap();
        assert!(root.source.ends_with("root.idea"));
    }
}
