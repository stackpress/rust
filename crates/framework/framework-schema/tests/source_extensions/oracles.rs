//! Original-source conversion oracles, kept distinct from G51 rule repair assertions.
use super::*;
#[test]
fn original_numeric_conversion_and_implicit_bound_oracles() {
    let oracle: serde_json::Value =
        serde_json::from_str(include_str!("../../src/source/number-oracles.json")).unwrap();
    let column = column("value Number");
    for row in oracle["converted"].as_array().unwrap() {
        let input = if let Some(millis) = row["input"]["nativeDate"].as_i64() {
            stackpress_framework_schema::source::date(millis)
        } else {
            Value::from(row["input"].clone())
        };
        let encoded = column
            .serialize(Some(&input), "", false, 0)
            .unwrap()
            .unwrap();
        let decoded = column
            .unserialize(Some(&input), "", false, 0)
            .unwrap()
            .unwrap();
        assert_eq!(encoded, row["serialize"], "serialize {:?}", row["input"]);
        assert_eq!(
            decoded, row["unserialize"],
            "unserialize {:?}",
            row["input"]
        );
        for (actual, tag) in [
            (&encoded, &row["serializeNonFinite"]),
            (&decoded, &row["unserializeNonFinite"]),
        ] {
            if let Some(expected) = tag.as_str() {
                assert!(!actual.is_null());
                assert_eq!(
                    stackpress_framework_schema::source::string(actual),
                    expected
                );
            }
        }
    }
    for shape in oracle["shapes"].as_array().unwrap() {
        let mut declaration = format!("value {}", shape["type"].as_str().unwrap());
        for (name, value) in shape["attrs"].as_object().unwrap() {
            declaration.push_str(&format!(" @{name}({value})"));
        }
        for rule in shape["rules"].as_array().unwrap() {
            declaration.push_str(&format!(
                " @is.{}({})",
                rule["name"].as_str().unwrap(),
                rule["args"][0]
            ));
        }
        let column = column_for_oracle(&declaration);
        for row in shape["results"].as_array().unwrap() {
            if shape["rules"].as_array().is_some_and(|r| !r.is_empty()) {
                // The untouched Node oracle records the zero-filtering Infinity defect.
                // G51's named rule exception keeps the declared zero bound instead.
                assert_eq!(shape["min"], "Infinity");
                let actual = column.assert(Some(&Value::from(row["value"].clone())));
                if matches!(row["value"].as_i64(), Some(1 | 2)) {
                    assert_eq!(actual, None);
                }
                if row["value"] == 0 {
                    assert_eq!(actual, Some("Must be greater than 0.".into()));
                }
                continue;
            }
            assert_eq!(
                column.assert(Some(&Value::from(row["value"].clone()))),
                row["error"].as_str().map(str::to_owned),
                "{declaration}: {:?}",
                row["value"]
            );
        }
    }
    let nullable = column_for_oracle("value Number?");
    let infinity = nullable
        .serialize(Some(&Value::from("Infinity")), "", false, 0)
        .unwrap()
        .unwrap();
    assert!(!infinity.is_null());
    assert_eq!(
        nullable.assert(Some(&infinity)),
        Some("Invalid input: expected number, received number".into())
    );
    assert_eq!(nullable.assert(Some(&Value::Null)), None);
}

/// Separate helper name keeps the local numeric column binding from hiding the fixture factory.
fn column_for_oracle(declaration: &str) -> SourceColumn {
    column(declaration)
}

#[test]
fn original_fieldset_and_enum_templates_pin_nested_and_schema_codec_order() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../framework-models/tests/extension-oracles.json"
    ))
    .unwrap();
    let input = "type Address { label String zip Integer } model Entry { address Address? addresses Address[] }";
    let parsed = parse("fieldset.idea", input, &CompileInputs::default()).unwrap();
    let graph = Schema::normalize_source(&parsed, NormalizeOptions::default(), &[]).unwrap();
    let schema = stackpress_framework_schema::source::SourceRecordSchema::from_metadata(
        &graph.models()[0],
        std::sync::Arc::new(graph.catalog().unwrap()),
        String::new(),
        0,
    );
    let scalar = &schema.columns()[0];
    let list = &schema.columns()[1];
    for row in fixture["rows"].as_array().unwrap() {
        let value = Value::from(row["value"].clone());
        for (column, prefix) in [(scalar, "scalar"), (list, "list")] {
            assert_eq!(
                column
                    .serialize(Some(&value), "", false, 0)
                    .unwrap()
                    .unwrap(),
                row[format!("{prefix}Serialize")],
                "{prefix} serialize {value}"
            );
            assert_eq!(
                column
                    .unserialize(Some(&value), "", false, 0)
                    .unwrap()
                    .unwrap(),
                row[format!("{prefix}Unserialize")],
                "{prefix} unserialize {value}"
            );
            assert_eq!(
                column
                    .assert_value_at(Some(&value), 0, 0)
                    .unwrap_or(Value::Null),
                row[format!("{prefix}Assert")],
                "{prefix} assert {value}"
            );
        }
        let data = stackpress_framework_schema::source::Record::from_iter([
            ("address".into(), value.clone()),
            ("addresses".into(), value),
        ]);
        let serialized = schema.serialize(&data, false).unwrap();
        assert_eq!(Value::Object(serialized.clone()), row["schemaSerialize"]);
        assert_eq!(
            Value::Object(schema.unserialize(&serialized, false).unwrap()),
            row["schemaRoundtrip"]
        );
    }
    for enumeration in fixture["enums"].as_array().unwrap() {
        let mut idea = String::from("enum Mood { ");
        for (name, value) in enumeration["options"].as_object().unwrap() {
            idea.push_str(&format!("{name} {value} "));
        }
        idea.push_str("} model Entry { value Mood }");
        let parsed = parse("enum.idea", &idea, &CompileInputs::default()).unwrap();
        let graph = Schema::normalize_source(&parsed, NormalizeOptions::default(), &[]).unwrap();
        let model = &graph.models()[0];
        let schema = stackpress_framework_schema::source::SourceRecordSchema::from_metadata(
            model,
            std::sync::Arc::new(graph.catalog().unwrap()),
            String::new(),
            0,
        );
        for row in enumeration["results"].as_array().unwrap() {
            assert_eq!(
                schema.columns()[0].assert(Some(&Value::from(row["value"].clone()))),
                row["error"].as_str().map(str::to_owned)
            );
        }
    }
}

#[test]
fn original_large_radix_and_numeric_spelling_oracles() {
    let rows: serde_json::Value =
        serde_json::from_str(include_str!("../../src/source/number-more.json")).unwrap();
    let col = column("value Number");
    for row in rows.as_array().unwrap() {
        let value = Value::from(row["input"].clone());
        let result = col.serialize(Some(&value), "", false, 0).unwrap().unwrap();
        assert_eq!(result, row["value"], "{}", row["input"]);
        if let Some(tag) = row["nonFinite"].as_str() {
            assert_eq!(
                stackpress_framework_schema::source::number_text(result.as_f64().unwrap()),
                tag
            );
        }
    }
}
