//! Full shipped source graph and native value representation proofs, without Typed adapters.
#[cfg(feature = "codegen")]
mod normalization {
    use stackpress_framework_schema::{Schema, ValueType, codegen::*};
    use stackpress_idea_parser::AttributeValue;
    use stackpress_idea_transformer::{Composer, CompositionLimits, MemoryLoader, PackageLoader};
    use std::collections::BTreeSet;

    fn compose(api: bool) -> stackpress_idea_parser::SchemaConfig {
        let mut app = MemoryLoader::new("/app").unwrap();
        app.insert(
            "schema.idea",
            if api {
                "use \"stackpress-api/schema.idea\""
            } else {
                "use \"stackpress-session/schema.idea\""
            },
        );
        let loader = PackageLoader::new(app)
            .with_source(
                "stackpress-session/schema.idea",
                include_str!("../../framework-session/schema.idea"),
            )
            .unwrap()
            .with_source(
                "stackpress-api/schema.idea",
                include_str!("../../framework-api/schema.idea"),
            )
            .unwrap();
        Composer::new(loader, CompositionLimits::default())
            .compose("schema.idea")
            .unwrap()
            .config
    }
    fn value(value: &AttributeValue) -> serde_json::Value {
        match value {
            AttributeValue::Boolean(v) => (*v).into(),
            AttributeValue::Array(v) => v.clone().into(),
        }
    }
    #[test]
    fn complete_shipped_graph_keeps_all_51_attributes_exactly() {
        let config = compose(true);
        let before = serde_json::to_value(&config).unwrap();
        let schema = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
        let mut names = BTreeSet::new();
        let mut count = 0;
        assert_eq!(schema.models().len(), 4);
        for model in schema.models() {
            let original = &config.model[&model.name];
            assert_eq!(model.fields.len(), original.columns.len());
            for (name, original_value) in &original.attributes {
                names.insert(name.clone());
                count += 1;
                assert_eq!(
                    model
                        .attributes
                        .iter()
                        .find(|a| a.name == *name)
                        .unwrap()
                        .value,
                    value(original_value)
                );
            }
            assert_eq!(model.attributes.len(), original.attributes.len());
            assert_eq!(
                model
                    .attributes
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>(),
                original
                    .attributes
                    .keys()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            );
            for (field, column) in model.fields.iter().zip(&original.columns) {
                assert_eq!(
                    (&field.name, field.required, field.multiple),
                    (&column.name, column.required, column.multiple)
                );
                assert_eq!(field.attributes.len(), column.attributes.len());
                assert_eq!(
                    field
                        .attributes
                        .iter()
                        .map(|a| a.name.as_str())
                        .collect::<Vec<_>>(),
                    column
                        .attributes
                        .keys()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                );
                for (name, original_value) in &column.attributes {
                    names.insert(name.clone());
                    count += 1;
                    assert_eq!(
                        field
                            .attributes
                            .iter()
                            .find(|a| a.name == *name)
                            .unwrap()
                            .value,
                        value(original_value)
                    );
                }
            }
        }
        assert_eq!(names.len(), 51, "{names:?}");
        assert_eq!(count, 320);
        assert_eq!(serde_json::to_value(&config).unwrap(), before);
        let auth = schema.models().iter().find(|m| m.name == "Auth").unwrap();
        assert_eq!(auth.query, ["*", "profile.*"]);
        assert_eq!(
            auth.fields
                .iter()
                .find(|f| f.name == "token")
                .unwrap()
                .value_type,
            ValueType::String
        );
        assert!(
            auth.fields
                .iter()
                .find(|f| f.name == "secret")
                .unwrap()
                .access
                .sensitive
        );
        // Typed behavior still refuses the original graph; Source does not rewrite it into Typed input.
        assert!(
            Schema::normalize(
                &config,
                &DefinitionRegistry::standard(),
                NormalizeOptions::default()
            )
            .is_err()
        );
        #[cfg(all(feature = "source", not(target_arch = "wasm32")))]
        {
            use stackpress_framework_schema::{
                providers::BuiltinProviders,
                source::{as_date, default_value},
            };
            let profile = schema
                .models()
                .iter()
                .find(|m| m.name == "Profile")
                .unwrap();
            let field = |name: &str| profile.fields.iter().find(|f| f.name == name).unwrap();
            assert_eq!(
                as_date(
                    &default_value(field("created"), 42, &BuiltinProviders)
                        .unwrap()
                        .unwrap()
                ),
                Some(42)
            );
            let id = default_value(field("id"), 42, &BuiltinProviders)
                .unwrap()
                .unwrap();
            assert_eq!(id.as_str().unwrap().len(), 24);
            assert_eq!(
                default_value(field("active"), 42, &BuiltinProviders).unwrap(),
                Some(true.into())
            );
            assert_eq!(
                default_value(field("name"), 42, &BuiltinProviders).unwrap(),
                None
            );
        }
    }
    #[test]
    fn session_only_external_models_are_explicit_and_never_tables() {
        let config = compose(false);
        assert!(Schema::normalize_source(&config, NormalizeOptions::default(), &[]).is_err());
        let schema = Schema::normalize_source(
            &config,
            NormalizeOptions::default(),
            &["Application", "Session"],
        )
        .unwrap();
        assert_eq!(
            schema
                .models()
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>(),
            ["Profile", "Auth"]
        );
        assert_eq!(schema.external_models(), ["Application", "Session"]);
        assert!(
            schema.models()[0]
                .fields
                .iter()
                .any(|f| f.value_type == ValueType::Model("Application".into()))
        );
        assert!(
            Schema::normalize_source(
                &config,
                NormalizeOptions::default(),
                &["Application", "Application"]
            )
            .is_err()
        );
        assert!(
            Schema::normalize_source(
                &config,
                NormalizeOptions::default(),
                &["Profile", "Application", "Session"]
            )
            .is_err()
        );
    }
}

#[cfg(all(feature = "source", not(target_arch = "wasm32")))]
mod native {
    use stackpress_framework_schema::source::*;
    #[test]
    fn shared_native_record_retains_dates_null_absence_and_nested_order() {
        let mut nested = Record::new();
        nested.insert("z".into(), date_with_display(42, "host spelling"));
        nested.insert("a".into(), Value::Null);
        let mut record = Record::new();
        record.insert("nested".into(), Value::Object(nested.clone()));
        record.insert("explicit".into(), Value::Null);
        assert!(!record.contains_key("missing"));
        assert_eq!(
            <Option<String> as NativeField>::read(&record["explicit"]),
            Some(None)
        );
        assert_eq!(<String as NativeField>::read(&record["explicit"]), None);
        let date = <Date as NativeField>::read(&nested["z"]).unwrap();
        assert_eq!(date.display.as_deref(), Some("host spelling"));
        assert_eq!(date.clone().value(), nested["z"]);
        assert_eq!(
            serde_json::to_string(&record).unwrap(),
            "{\"nested\":{\"z\":\"1970-01-01T00:00:00.042Z\",\"a\":null},\"explicit\":null}"
        );
        assert!(matches!(
            serde_json::from_str::<Value>("{\"millis\":42}").unwrap(),
            Value::Object(_)
        ));
        assert_eq!(
            <Record as NativeField>::read(&Value::Object(nested.clone())),
            Some(nested)
        );
        assert_eq!(
            <Vec<String> as NativeField>::read(&vec!["a".to_owned(), "b".to_owned()].value()),
            Some(vec!["a".into(), "b".into()])
        );
        assert_eq!(<bool as NativeField>::read(&true.value()), Some(true));
        assert_eq!(<i64 as NativeField>::read(&42_i64.value()), Some(42));
        assert_eq!(
            <Option<String> as NativeField>::read(&Some("x".to_owned()).value()),
            Some(Some("x".into()))
        );
    }
}
