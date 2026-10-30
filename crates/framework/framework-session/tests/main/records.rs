//! Golden values are captured by executing the original generated TypeScript classes.
use serde_json::{Value as JsonValue, json};
use stackpress_framework_session::columns::Value;
use stackpress_framework_session::{
    auth::AuthSchema,
    columns::{self, Record},
    profile::ProfileSchema,
};

/// Translate the native Date marker to the original JSON Date spelling for oracle comparison.
fn wire(value: Value) -> JsonValue {
    columns::to_json(&value).unwrap()
}
#[test]
fn original_column_goldens() {
    let fixture: JsonValue = serde_json::from_str(include_str!("../records.json")).unwrap();
    let profile = ProfileSchema::new("database-seed");
    let auth = AuthSchema::new("database-seed");
    for case in fixture["columns"].as_array().unwrap() {
        let schema = if case["model"] == "Profile" {
            &*profile
        } else {
            &*auth
        };
        let column = schema.column(case["name"].as_str().unwrap()).unwrap();
        let mut input = case.get("input").cloned().map(Value::from);
        if case["nativeDate"] == true {
            input = Some(columns::date(1700000000123));
        }
        if case["nestedDate"] == true {
            input = Some(Value::Array(vec![
                columns::date(0),
                Value::Null,
                Value::Object(Record::new()),
            ]));
        }
        let input = input.as_ref();
        for (name, result) in [
            (
                "serialize",
                column.serialize(input, "database-seed", false, 0),
            ),
            (
                "unserialize",
                column.unserialize(input, "database-seed", false, 0),
            ),
        ] {
            let actual = match result {
                Ok(Some(value)) => json!({"value":wire(value)}),
                Ok(None) => json!({"undefined":true}),
                Err(_) => json!({"error":true}),
            };
            assert_eq!(
                actual, case[name],
                "{} {} {name} input={input:?}",
                schema.name, column.name
            );
        }
        assert_eq!(
            column
                .assert(input)
                .map(JsonValue::String)
                .unwrap_or(JsonValue::Null),
            case["assert"],
            "{} {} assertion input={input:?}",
            schema.name,
            column.name
        );
    }
}
#[test]
fn schema_defaults_missing_null_filter_and_date_contract() {
    let schema = ProfileSchema::new("");
    let first = schema.defaults_at(1700000000123).unwrap();
    let second = schema.defaults_at(1700000000123).unwrap();
    assert_ne!(first["id"], second["id"]);
    assert_eq!(first["id"].as_str().unwrap().len(), 24);
    assert_eq!(first["roles"], json!(["GUEST"]));
    assert!(!first.contains_key("image"));
    assert_eq!(
        Value::Object(schema.assert(&first, true).unwrap()),
        json!({"name":"Invalid input: expected string, received undefined"})
    );
    let input: Record =
        serde_json::from_value(json!({"name":"Ada","image":null,"unknown":42})).unwrap();
    assert_eq!(
        Value::Object(schema.filter(&input, false).unwrap()),
        json!({"name":"Ada","image":null})
    );
    assert!(schema.populate(&input).unwrap().contains_key("unknown"));
    let dates = Record::from_iter([("created".into(), columns::date(1700000000123))]);
    assert_eq!(
        schema.serialize(&dates, false).unwrap()["created"],
        "2023-11-14 22:13:20"
    );
    assert_eq!(
        schema
            .unserialize(&schema.serialize(&dates, false).unwrap(), false)
            .unwrap()["created"],
        columns::date(1700000000000)
    );
    // Source lists acquire [] even when omitted, including update preparation.
    assert_eq!(
        Value::Object(schema.serialize(&Record::new(), false).unwrap()),
        json!({"roles":"[]","tags":"[]"})
    );
    assert_eq!(
        schema
            .column("name")
            .unwrap()
            .serialize_date(0, 480)
            .unwrap(),
        "1970-01-01 08:00:00"
    );
}
#[cfg(feature = "sql")]
#[test]
fn original_email_goldens() {
    let fixture: JsonValue = serde_json::from_str(include_str!("../records.json")).unwrap();
    for case in fixture["emails"].as_array().unwrap() {
        assert_eq!(
            stackpress_framework_session::storage::is_email(case["input"].as_str().unwrap()),
            case["valid"].as_bool().unwrap(),
            "{}",
            case["input"]
        );
    }
}
#[test]
fn model_descriptor_is_seed_free_and_contains_api_profile_edges() {
    let schema = ProfileSchema::new("private-seed");
    let model = schema.model();
    let encoded = serde_json::to_string(&model).unwrap();
    assert!(!encoded.contains("private-seed"));
    assert_eq!(model.name, "Profile");
    assert_eq!(model.fields.len(), 13);
    for name in ["auth", "applications", "sessions"] {
        assert!(
            model
                .fields
                .iter()
                .any(|field| field.name == name && field.multiple)
        );
    }
    assert_eq!(
        model.fields[0].value_rule.as_ref().unwrap().provider,
        "cuid"
    );
    assert!(
        schema
            .column("image")
            .unwrap()
            .defaults()
            .unwrap()
            .is_none()
    );
    assert_eq!(
        schema.column("type").unwrap().defaults().unwrap(),
        Some(Value::String("person".into()))
    );
    let date = columns::date_with_display(
        0,
        "Thu Jan 01 1970 08:00:00 GMT+0800 (Philippine Standard Time)",
    );
    assert_eq!(
        schema
            .column("name")
            .unwrap()
            .serialize(Some(&Value::Array(vec![date])), "", false, 480)
            .unwrap()
            .unwrap(),
        "Thu Jan 01 1970 08:00:00 GMT+0800 (Philippine Standard Time)"
    );
}
#[test]
fn native_dates_cannot_collide_with_arbitrary_json_objects() {
    let object = json!({"$stackpressDate":123,"$stackpressDateString":"ordinary metadata"});
    let native = Value::from(object.clone());
    assert!(columns::as_date(&native).is_none());
    assert_eq!(columns::to_json(&native).unwrap(), object);
    let decoded: Value = serde_json::from_value(object.clone()).unwrap();
    assert_eq!(decoded, native);
    let date = columns::date(1700000000123);
    assert_eq!(columns::as_date(&date), Some(1700000000123));
    assert_eq!(
        serde_json::to_value(&date).unwrap(),
        json!("2023-11-14T22:13:20.123Z")
    );
    let schema = ProfileSchema::default();
    assert_eq!(
        schema
            .column("references")
            .unwrap()
            .unserialize(Some(&native), "", false, 0)
            .unwrap(),
        Some(native)
    );
    assert_eq!(
        schema
            .model()
            .fields
            .iter()
            .find(|field| field.name == "type")
            .unwrap()
            .rust_name,
        "type_"
    );
    assert!(columns::date_text(i64::MAX, 0).is_err());
    assert!(columns::date_text(0, i32::MAX).is_err());
}
