//! Schema/contract matrix: actual checked types, business outcomes, diagnostics and disclosure.
use serde_json::{Value, json};
use stackpress_framework_schema::*;

/// Canonical values reject malformed inputs and retain exact portable semantics.
#[test]
fn canonical_value_matrix() {
    assert_eq!(
        DateValue::parse("2024-02-29").unwrap().as_str(),
        "2024-02-29"
    );
    for value in ["2023-02-29", "0000-01-01", "2024-2-03", "2024-13-01", ""] {
        assert!(DateValue::parse(value).is_err());
    }
    assert_eq!(
        TimeValue::parse("12:03:04.120000000").unwrap().as_str(),
        "12:03:04.12"
    );
    for value in [
        "24:00:00",
        "12:00",
        "12:03:60",
        "12:00:00.1234567890",
        "12:00:00Z",
    ] {
        assert!(TimeValue::parse(value).is_err());
    }
    assert_eq!(
        DatetimeValue::parse("2024-03-01T01:00:00+01:00")
            .unwrap()
            .as_str(),
        "2024-03-01T00:00:00Z"
    );
    assert!(DatetimeValue::parse("2024-01-01T00:00:00").is_err());
    assert!(DatetimeValue::parse("2024-01-01T00:00:60Z").is_err());
    assert!(FiniteNumber::new(f64::NAN).is_err());
    assert!(FiniteNumber::new(f64::INFINITY).is_err());
    assert!(FiniteNumber::from_json(&json!(9_007_199_254_740_993_i64)).is_err());
    assert_eq!(
        FiniteNumber::from_json(&json!(9_007_199_254_740_992_i64))
            .unwrap()
            .get(),
        9_007_199_254_740_992.0
    );
    assert!(FiniteNumber::from_json(&json!("2")).is_err());
    let object: JsonObject = serde_json::from_value(json!({"nested":[null,1,true]})).unwrap();
    assert_eq!(object.members()["nested"][0], Value::Null);
    assert!(serde_json::from_value::<JsonObject>(Value::Null).is_err());
    assert!(JsonObject::new(json!([])).is_err());
    assert!(browser_integer(9_007_199_254_740_992).is_err());
    assert!(browser_integer(i64::MIN).is_err());
    assert_eq!(browser_integer(0).unwrap(), 0);
    assert!(check_json(&json!("x".repeat(1024 * 1024 + 1))).is_err());
    let mut deep = Value::Null;
    for _ in 0..34 {
        deep = json!([deep]);
    }
    assert!(check_json(&deep).is_err());
    assert!(check_json(&json!((0..10_001).collect::<Vec<_>>())).is_err());
    assert!(serde_json::from_value::<DateValue>(json!("2023-02-29")).is_err());
}
/// Patch wire semantics distinguish omitted, null and supplied false/zero/empty.
#[test]
fn patch_and_bounded_issues() {
    #[derive(serde::Serialize, serde::Deserialize, Default)]
    struct Change {
        #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
        flag: Patch<bool>,
    }
    let missing: Change = serde_json::from_value(json!({})).unwrap();
    assert!(missing.flag.is_unchanged());
    assert_eq!(serde_json::to_value(missing).unwrap(), json!({}));
    let null: Change = serde_json::from_value(json!({"flag":null})).unwrap();
    assert_eq!(null.flag, Patch::Null);
    let value: Change = serde_json::from_value(json!({"flag":false})).unwrap();
    assert_eq!(value.flag, Patch::Set(false));
    assert_eq!(serde_json::to_value(value).unwrap(), json!({"flag":false}));
    assert!(serde_json::to_value(Patch::<bool>::Unchanged).is_err());
    let mut errors = ValidationErrors::default();
    for _ in 0..100 {
        errors.push(&["private".repeat(100)], "type");
    }
    assert_eq!(errors.issues.len(), 64);
    assert!(errors.issues[0].path[0].len() <= 512);
    assert!(errors.into_result().is_err());
}
#[cfg(feature = "codegen")]
mod generation {
    use super::*;
    use stackpress_framework_schema::codegen::*;
    use stackpress_idea_parser::{CompileInputs, parse};
    fn normalize(source: &str, registry: &DefinitionRegistry) -> SchemaResult<Schema> {
        let config = parse("matrix.idea", source, &CompileInputs::default()).unwrap();
        Schema::normalize(&config, registry, NormalizeOptions::default())
    }
    fn standard(source: &str) -> Schema {
        normalize(source, &DefinitionRegistry::standard()).unwrap()
    }
    fn record(schema: &Schema) -> RecordSchema {
        RecordSchema {
            model: schema.models()[0].clone(),
            fieldsets: schema.fieldsets().to_vec(),
            enums: schema.enums().to_vec(),
        }
    }
    /// Every accepted scalar, enum family, optional/list and fieldset appears in real normalization.
    #[test]
    fn supported_types_defaults_and_keys() {
        let schema = standard(
            "enum Count { ZERO 0 ONE 1 } enum Flag { NO false YES true } type Address { city String @is.ne } model Example { id String @id second Integer @id text Text @default(\"\") count Integer @default(0) real Number @default(0.25) float Float? truth Boolean @default(false) day Date @default(\"2024-02-29\") time Time @default(\"12:00:00\") stamp Datetime @default(\"2024-01-01T00:00:00Z\") data Json @default({ nested [null true] }) object Object? hash Hash? choice Count @default(0) flag Flag @default(false) items Integer[] @default([]) address Address? }",
        );
        assert_eq!(
            schema.models()[0].fields.iter().filter(|v| v.key).count(),
            2
        );
        assert_eq!(schema.enums().len(), 2);
        assert_eq!(schema.models()[0].fields[2].default, Some(json!("")));
        assert_eq!(schema.models()[0].fields[3].default, Some(json!(0)));
        for source in [
            "enum E { A 1 B true } model X { id String @id e E }",
            "enum E { A null } model X { id String @id e E }",
            "enum E { A 1.5 } model X { id String @id e E }",
            "model X { id String @id x Integer @default(1.5) }",
            "type A { city String @is.ne } model X { id String @id a A @default({ city \"\" }) }",
            "model X { id String @id active Boolean? @active }",
            "model X { id String @id stamp Date @timestamp }",
            "model X { id String @id data Object @unique }",
            "model X { id String @id data Json @sortable }",
            "model X { id String @id x String @generated }",
            "model X { id String @id x String @default(\"now()\") }",
        ] {
            assert!(
                normalize(source, &DefinitionRegistry::standard()).is_err(),
                "{source}"
            );
        }
    }
    /// Canonical aliases, Unicode character/word counts, finite comparisons and safe issues execute.
    #[test]
    fn assertions_execute_and_reject_incompatible_arguments() {
        let schema = standard(
            "model X { id String @id title String @is.notempty @is.cgte(2) @is.clte(20) @is.wgte(1) @is.wlte(4) @is.pattern(\"^[a-zé ]+$\") @is.lowercase count Integer @is.gte(2) @is.lte(10) @step(2) code String @is.hex slug String @is.slug url String @is.url pick String @is.option(\"a\" \"b\") }",
        );
        let descriptor = record(&schema);
        let ctx = ValueContext::default();
        let valid = json!({"id":"1","title":"éé","count":4,"code":"aF02","slug":"my-slug2","url":"https://example.test/path","pick":"a"});
        assert!(
            descriptor
                .validate(&valid, &ctx, false, false)
                .unwrap()
                .is_empty()
        );
        let invalid = json!({"id":"1","title":"PRIVATE SENTINEL","count":3,"code":"xyz","slug":"Bad-","url":"missing","pick":"c"});
        let errors = descriptor.validate(&invalid, &ctx, false, false).unwrap();
        assert!(errors.issues.len() >= 6);
        assert!(!serde_json::to_string(&errors).unwrap().contains("PRIVATE"));
        for (rule, good, bad) in [
            ("is.ceq(2)", json!("éé"), json!("é")),
            ("is.cgt(1)", json!("éé"), json!("é")),
            ("is.clt(3)", json!("ab"), json!("abc")),
            ("is.weq(2)", json!("a\tb"), json!("a")),
            ("is.wgt(1)", json!("a b"), json!("a")),
            ("is.wlt(3)", json!("a b"), json!("a b c")),
            ("is.uppercase", json!("ABC"), json!("AbC")),
            ("is.eq(\"a\")", json!("a"), json!("b")),
            ("is.neq(\"a\")", json!("b"), json!("a")),
        ] {
            let s = standard(&format!("model X {{ id String @id v String @{rule} }}"));
            let r = record(&s);
            assert!(
                r.validate(&json!({"id":"1","v":good}), &ctx, false, false)
                    .unwrap()
                    .is_empty()
            );
            assert!(
                !r.validate(&json!({"id":"1","v":bad}), &ctx, false, false)
                    .unwrap()
                    .is_empty()
            );
        }
        for source in [
            "model X { id String @id x String @is.gte(1) }",
            "model X { id String @id x Integer @is.regex(\"a\") }",
            "model X { id String @id x String @is.regex(\"[\") }",
            "model X { id String @id x String @is.eq(false) }",
            "model X { id String @id x String @min(false) }",
            "model X { id String @id x Number @step(0) }",
            "model X { id String @id x String @is.email }",
            "model X { id String @id x String @is.cc }",
            "model X { id String @id x String @is.color }",
            "model X { id String @id x Number @is.price }",
            "model X { id String @id x String @is.typo }",
            "model X { id String @id x String @hashed }",
        ] {
            assert!(
                normalize(source, &DefinitionRegistry::standard()).is_err(),
                "{source}"
            );
        }
    }
    /// Providers use explicit clocks and typed outputs; relative input rules never corrupt aged rows.
    #[test]
    fn clock_providers_and_host_validation() {
        let registry = DefinitionRegistry::standard()
            .with_provider(ProviderDefinition {
                name: "nanoid".into(),
                revision: "test-1".into(),
                output: ValueType::String,
            })
            .unwrap()
            .with_validator(ValidatorDefinition {
                name: "email".into(),
                revision: "test-2".into(),
                types: vec![ValueType::String],
            })
            .unwrap();
        assert!(
            registry
                .clone()
                .with_provider(ProviderDefinition {
                    name: "nanoid".into(),
                    revision: "2".into(),
                    output: ValueType::String
                })
                .is_err()
        );
        let schema=normalize("model X { id String @id @generated @default(\"nanoid(8)\") updated Datetime @timestamp stamp Datetime @is.future @default(\"now()\") email String @is.email }",&registry).unwrap();
        let now = DatetimeValue::parse("2024-01-01T00:00:00Z").unwrap();
        struct Providers;
        impl ValueProviders for Providers {
            fn generate(&self, name: &str, args: &[Value]) -> SchemaResult<Value> {
                assert_eq!(name, "nanoid");
                assert_eq!(args, &[json!(8)]);
                Ok(json!("fixture1"))
            }
            fn validate(&self, name: &str, value: &Value, _: &[Value]) -> SchemaResult<bool> {
                assert_eq!(name, "email");
                Ok(value.as_str() == Some("approved@example.test"))
            }
        }
        let host = Providers;
        let context = ValueContext {
            now: Some(&now),
            providers: Some(&host),
        };
        assert_eq!(
            schema.models()[0].fields[0]
                .value_rule
                .as_ref()
                .unwrap()
                .resolve(&context)
                .unwrap(),
            json!("fixture1")
        );
        assert_eq!(
            schema.models()[0].fields[1]
                .value_rule
                .as_ref()
                .unwrap()
                .resolve(&context)
                .unwrap(),
            json!(now.as_str())
        );
        assert!(
            schema.models()[0].fields[0]
                .value_rule
                .as_ref()
                .unwrap()
                .resolve(&ValueContext::default())
                .is_err()
        );
        let policy = record(&schema);
        let future = json!({"id":"1","updated":now.as_str(),"stamp":"2024-01-01T00:00:00.000000001Z","email":"approved@example.test"});
        assert!(
            policy
                .validate(&future, &context, false, false)
                .unwrap()
                .is_empty()
        );
        let past =
            json!({"id":"1","updated":now.as_str(),"stamp":"2023-01-01T00:00:00Z","email":"wrong"});
        assert_eq!(
            policy
                .validate(&past, &context, false, false)
                .unwrap()
                .issues
                .len(),
            2
        );
        assert!(
            policy
                .validate(&past, &ValueContext::default(), false, true)
                .unwrap()
                .is_empty()
        );
        for (name, stamp, valid) in [
            ("past", "2023-01-01T00:00:00Z", true),
            ("present", now.as_str(), true),
            ("present", "2024-01-01T00:00:00.1Z", false),
        ] {
            let schema = standard(&format!(
                "model X {{ id String @id stamp Datetime @is.{name} }}"
            ));
            assert_eq!(
                record(&schema)
                    .validate(&json!({"id":"1","stamp":stamp}), &context, false, false)
                    .unwrap()
                    .is_empty(),
                valid
            );
        }
        for source in [
            "model X { id String @id x Number @default(\"nanoid(8)\") }",
            "model X { id String @id x String @default(\"nanoid(0)\") }",
            "model X { id String @id x String @default(\"nanoid(no)\") }",
            "model X { id String @id x Integer @is.email }",
        ] {
            assert!(normalize(source, &registry).is_err());
        }
    }
    /// Every alias and derived role is canonical; app prop types, precedence and access are explicit.
    #[test]
    fn roles_props_aliases_projection_and_private_catalog() {
        use std::collections::BTreeMap;
        let definition = ComponentDefinition {
            name: "field.input".into(),
            key: "app.input".into(),
            revision: "2".into(),
            props: vec![
                PropDefinition {
                    name: "label".into(),
                    value_type: PropType::String,
                    required: true,
                    public: true,
                },
                PropDefinition {
                    name: "private".into(),
                    value_type: PropType::String,
                    required: false,
                    public: false,
                },
            ],
            fixed: BTreeMap::from([("label".into(), json!("Fixed"))]),
            available: true,
        };
        let registry = DefinitionRegistry::standard()
            .with_component(
                definition.clone(),
                Some(ComponentImport {
                    component: "app.input".into(),
                    module: "private_crate::controls".into(),
                    symbol: "Input".into(),
                    default: false,
                }),
            )
            .unwrap()
            .with_field_policy(
                "X",
                "title",
                FieldAccess {
                    readable: true,
                    writable: true,
                    sensitive: false,
                    publish_default: true,
                },
            )
            .unwrap()
            .with_field_policy(
                "X",
                "secret",
                FieldAccess {
                    readable: false,
                    writable: true,
                    sensitive: true,
                    publish_default: false,
                },
            )
            .unwrap();
        let source = "model X { id String @id title String @default(\"ok\") @field.string({ label \"Idea\" private \"PRIVATE_PROP\" }) @filter.input @span.string @view.string @list.string secret String @default(\"PRIVATE_DEFAULT\") hidden String @foreign.private(\"PRIVATE_CONFIG\") }";
        let schema = normalize(source, &registry).unwrap();
        let title = &schema.models()[0].fields[1];
        assert_eq!(title.roles.len(), 5);
        for role in title
            .roles
            .iter()
            .filter(|r| matches!(r.role, Role::Field | Role::Filter | Role::Span))
        {
            assert_eq!(role.component, "app.input");
            assert_eq!(role.props["label"], json!("Fixed"));
            assert!(!role.props.contains_key("private"));
        }
        let ui = schema.project_ui().unwrap();
        assert_eq!(ui.models[0].fields.len(), 2);
        assert!(ui.models[0].key.is_empty());
        let encoded = serde_json::to_string(&ui).unwrap();
        for sentinel in [
            "PRIVATE_PROP",
            "PRIVATE_DEFAULT",
            "PRIVATE_CONFIG",
            "private_crate",
        ] {
            assert!(!encoded.contains(sentinel));
        }
        assert_eq!(registry.imports()[0].module, "private_crate::controls");
        assert!(schema.require_available_roles().is_err());
        assert!(registry.clone().with_component(definition, None).is_err());
        assert!(
            DefinitionRegistry::standard()
                .with_field_policy(
                    "X",
                    "secret",
                    FieldAccess {
                        readable: true,
                        writable: true,
                        sensitive: true,
                        publish_default: false
                    }
                )
                .is_err()
        );
        for (alias, canonical) in [
            ("taglist", "tags"),
            ("string", "input"),
            ("text", "textarea"),
            ("float", "number"),
            ("boolean", "switch"),
            ("object", "json"),
            ("hash", "json"),
            ("strings", "stringlist"),
            ("texts", "stringlist"),
            ("dates", "datelist"),
            ("datetimes", "datetimelist"),
            ("times", "timelist"),
            ("integerlist", "numberlist"),
            ("integers", "numberlist"),
            ("floatlist", "numberlist"),
            ("floats", "numberlist"),
            ("numbers", "numberlist"),
        ] {
            for prefix in ["field", "filter", "span"] {
                let schema = standard(&format!(
                    "model X {{ id String @id x String @{prefix}.{alias} }}"
                ));
                assert_eq!(
                    schema.models()[0].fields[1].roles[0].component,
                    format!("stackpress.field.{canonical}")
                );
            }
        }
        for (alias, canonical) in [
            ("table", "tabular"),
            ("taglist", "tags"),
            ("clip", "overflow"),
            ("string", "text"),
            ("float", "number"),
            ("integer", "number"),
            ("boolean", "yesno"),
            ("datetime", "date"),
            ("object", "json"),
            ("hash", "json"),
            ("strings", "list"),
            ("numbers", "list"),
            ("datetimelist", "list"),
        ] {
            for prefix in ["view", "list"] {
                let schema = standard(&format!(
                    "model X {{ id String @id x String @{prefix}.{alias} }}"
                ));
                assert_eq!(
                    schema.models()[0].fields[1].roles[0].component,
                    format!("stackpress.view.{canonical}")
                );
            }
        }
        for suffix in [
            "@field.unknown",
            "@field.string @field.input",
            "@field.input({ unknown true })",
            "@field.input(false)",
        ] {
            assert!(
                normalize(
                    &format!("model X {{ id String @id x String {suffix} }}"),
                    &registry
                )
                .is_err()
            );
        }
        let named = registry
            .clone()
            .with_name("X", "title", "headline")
            .unwrap();
        let renamed = normalize("model X { id String @id title String }", &named).unwrap();
        assert_eq!(renamed.models()[0].fields[1].column, "headline");
        assert_ne!(
            renamed.fingerprint().unwrap(),
            schema.fingerprint().unwrap()
        );
    }
    /// Public metadata refuses integer literals the browser cannot preserve exactly.
    #[test]
    fn public_literals_and_explicit_model_names() {
        let registry = DefinitionRegistry::standard()
            .with_model_name("X", "records")
            .unwrap()
            .with_field_policy(
                "X",
                "id",
                FieldAccess {
                    readable: true,
                    writable: true,
                    sensitive: false,
                    publish_default: true,
                },
            )
            .unwrap();
        let schema = normalize(
            "model X { id Integer @id @default(9007199254740992) }",
            &registry,
        )
        .unwrap();
        assert_eq!(schema.models()[0].module, "records");
        assert_eq!(schema.models()[0].rust_name, "X");
        assert_eq!(schema.models()[0].names().source, "X");
        let error = schema.project_ui().unwrap_err();
        assert_eq!(error.path(), ["model", "X", "columns", "id"]);
        let schema = normalize(
            "enum E { BIG 9007199254740992 } model X { id E @id }",
            &registry,
        )
        .unwrap();
        assert!(schema.project_ui().is_err());
    }
    /// Semantic diagnostics retain imported winning UTF-8 spans; normalized JSON never invents positions.
    #[test]
    fn semantic_diagnostics_use_winning_source() {
        use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};
        let mut loader = MemoryLoader::new("/project").unwrap();
        loader.insert("root.idea", "use \"child\" model X { id String @id }");
        let child = "// Café\nmodel X { bad String @is.typo }";
        loader.insert("child.idea", child);
        let composed = Composer::new(loader, CompositionLimits::default())
            .compose("root.idea")
            .unwrap();
        let map = DiagnosticMap::from_composed(&composed).unwrap();
        let error = Schema::normalize_with_diagnostics(
            &composed.config,
            &DefinitionRegistry::standard(),
            NormalizeOptions::default(),
            &map,
        )
        .unwrap_err();
        assert_eq!(error.path().last().unwrap(), "is.typo");
        let location = error.location().unwrap();
        assert!(location.source.ends_with("child.idea"));
        let (start, end) = location.bytes.unwrap();
        assert!(child[start..end].contains("is.typo"));
        let mut loader = MemoryLoader::new("/project").unwrap();
        let config = parse(
            "json-source.idea",
            "model X { id String @id bad String @is.typo }",
            &CompileInputs::default(),
        )
        .unwrap();
        loader.insert("root.json", serde_json::to_string(&config).unwrap());
        let composed = Composer::new(loader, CompositionLimits::default())
            .compose("root.json")
            .unwrap();
        let map = DiagnosticMap::from_composed(&composed).unwrap();
        let error = Schema::normalize_with_diagnostics(
            &composed.config,
            &DefinitionRegistry::standard(),
            NormalizeOptions::default(),
            &map,
        )
        .unwrap_err();
        assert!(error.location().unwrap().bytes.is_none());
        assert!(!error.path().is_empty());
    }
    /// Canonical temporal metadata, nullable presence, exact integer steps and extension contracts execute.
    #[test]
    fn final_semantic_edges() {
        for value in [
            "2024-01-01T00:00:00.1234567890Z",
            "2024-01-01T00:00:00.1234567891+01:00",
        ] {
            assert!(DatetimeValue::parse(value).is_err());
        }
        let schema = standard(
            "model X { id String @id time Time @default(\"12:00:00.1200\") stamp Datetime @default(\"2024-03-01T01:00:00+01:00\") title String? @is.required(\"Please supply a title\") count Integer @step(2) }",
        );
        assert_eq!(
            schema.models()[0].fields[1].default,
            Some(json!("12:00:00.12"))
        );
        assert_eq!(
            schema.models()[0].fields[2].default,
            Some(json!("2024-03-01T00:00:00Z"))
        );
        assert!(schema.models()[0].fields[3].input_required);
        let error=record(&schema).validate(&json!({"id":"a","time":"12:00:00","stamp":"2024-01-01T00:00:00Z","title":null,"count":9_007_199_254_740_993_i64}),&ValueContext::default(),false,false).unwrap();
        assert_eq!(error.issues.len(), 2);
        let value = json!({"id":"a","time":"12:00:00","stamp":"2024-01-01T00:00:00Z","title":"yes","count":i64::MIN});
        assert!(
            record(&schema)
                .validate(&value, &ValueContext::default(), false, false)
                .unwrap()
                .is_empty()
        );
        let alias = standard("model X { id String @id title String @is.notempty }");
        let canonical = standard("model X { id String @id title String @is.ne }");
        assert_eq!(
            alias.fingerprint().unwrap(),
            canonical.fingerprint().unwrap()
        );
        assert!(
            normalize(
                "model X @query(\"id.title\") { id String @id title String }",
                &DefinitionRegistry::standard()
            )
            .is_err()
        );
        assert!(
            normalize(
                "model X { id String @id x String @generated @default(\"literal\") }",
                &DefinitionRegistry::standard()
            )
            .is_err()
        );
        let one = standard(
            "model Parent { id String @id child Child? } model Child { id String @id parentId String @unique parent Parent @relation({ local \"parentId\" foreign \"id\" name \"child\" }) }",
        );
        assert_eq!(one.models().len(), 2);
        for source in [
            "model X { id String @id x String @is.string }",
            "model X { id String @id x Integer @is.integer @is.number }",
            "model X { id String @id x Number @is.float }",
            "model X { id String @id x Boolean @is.boolean }",
            "model X { id String @id x Object @is.object }",
        ] {
            assert!(normalize(source, &DefinitionRegistry::standard()).is_ok());
        }
        for source in [
            "model X { id String @id x Integer @is.string }",
            "model X { id String @id x String @is.ne(false) }",
            "model X @labels(\"one\") { id String @id }",
        ] {
            assert!(normalize(source, &DefinitionRegistry::standard()).is_err());
        }
        let numbers = standard(
            "model X { id String @id value Number @default(1) @is.eq(1) @is.option(1 2) }",
        );
        assert!(
            record(&numbers)
                .validate(
                    &json!({"id":"1","value":1.0}),
                    &ValueContext::default(),
                    false,
                    false
                )
                .unwrap()
                .is_empty()
        );
        assert!(
            normalize(
                "model X { id String @id value Integer @step(0.5) }",
                &DefinitionRegistry::standard()
            )
            .is_err()
        );
        for source in [
            "model X { id String @id } model CreateX { id String @id }",
            "model X { id String @id } type PatchX { title String }",
        ] {
            let error = normalize(source, &DefinitionRegistry::standard()).unwrap_err();
            assert_eq!(error.to_string(), "generated public type name collision");
            assert!(!error.path().is_empty());
        }
        // Mixed numeric comparisons preserve adjacent large i64 values and fractional bounds.
        for (value, bound, code, expected) in [
            (
                json!(9_007_199_254_740_993_i64),
                json!(9_007_199_254_740_992.0),
                "gt",
                true,
            ),
            (
                json!(i64::MAX),
                json!(9_223_372_036_854_775_808.0),
                "lt",
                true,
            ),
            (
                json!(9_007_199_254_740_992.0),
                json!(9_007_199_254_740_993_i64),
                "lt",
                true,
            ),
            (json!(-1), json!(-1.5), "gt", true),
            (json!(1), json!(1.5), "ge", false),
        ] {
            let rule = ValidationRule {
                code: code.into(),
                args: vec![bound],
                client_checkable: true,
                message: "field validation failed".into(),
                definition_revision: "1".into(),
            };
            assert_eq!(
                check_rule(&rule, &value, &ValueContext::default(), false).unwrap(),
                expected
            );
        }
        let rule = ValidationRule {
            code: "step".into(),
            args: vec![json!(0.25)],
            client_checkable: true,
            message: "field validation failed".into(),
            definition_revision: "1".into(),
        };
        assert!(check_rule(&rule, &json!(0.5), &ValueContext::default(), false).unwrap());
        assert!(!check_rule(&rule, &json!(0.3), &ValueContext::default(), false).unwrap());
        let now = DatetimeValue::parse("2024-01-01T12:00:00Z").unwrap();
        let context = ValueContext {
            now: Some(&now),
            providers: None,
        };
        for (kind, value) in [("Date", "2024-01-02"), ("Time", "12:00:00.000000001")] {
            let schema = standard(&format!("model X {{ id String @id x {kind} @is.future }}"));
            assert!(
                record(&schema)
                    .validate(&json!({"id":"1","x":value}), &context, false, false)
                    .unwrap()
                    .is_empty()
            );
        }
        let definition = ComponentDefinition {
            name: "field.custom".into(),
            key: "app.custom".into(),
            revision: "1".into(),
            props: Vec::new(),
            fixed: Default::default(),
            available: false,
        };
        let registry = DefinitionRegistry::standard()
            .with_component(definition.clone(), None)
            .unwrap();
        assert!(registry.clone().with_component(definition, None).is_err());
        assert!(
            normalize(
                "model X { id String @id x String @field.custom @span.custom }",
                &registry
            )
            .is_ok()
        );
    }
    /// Every semantic category has a structural source path and cycles retain their complete trace.
    #[test]
    fn diagnostic_categories() {
        use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader};
        for source in [
            "enum E { A 1 B true } model X { id String @id e E }",
            "model X { id String? @id }",
            "model X { id String @id active String @active }",
            "model X { id String @id x String @field.typo }",
            "type A { b B } type B { a A } model X { id String @id a A }",
            "type Address { city String @is.ne @default(\"\") } model X { id String @id address Address }",
        ] {
            let mut loader = MemoryLoader::new("/project").unwrap();
            loader.insert("root.idea", source);
            let composed = Composer::new(loader, CompositionLimits::default())
                .compose("root.idea")
                .unwrap();
            let map = DiagnosticMap::from_composed(&composed).unwrap();
            let error = Schema::normalize_with_diagnostics(
                &composed.config,
                &DefinitionRegistry::standard(),
                NormalizeOptions::default(),
                &map,
            )
            .unwrap_err();
            assert!(!error.path().is_empty(), "{error:?}");
            assert!(error.location().is_some(), "{error:?}");
            if source.starts_with("type Address") {
                assert_eq!(
                    error.path(),
                    [
                        "type",
                        "Address",
                        "columns",
                        "city",
                        "attributes",
                        "default"
                    ]
                );
                assert!(error.location().unwrap().bytes.is_some());
            }
            if source.starts_with("type A {") {
                assert!(error.path().iter().any(|part| part == "A"));
                assert!(error.path().iter().any(|part| part == "B"));
            }
        }
    }
}
