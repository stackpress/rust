//! G51 named rule behavior and complete source extension mechanics.
#![cfg(all(feature = "source", feature = "codegen", not(target_arch = "wasm32")))]
use serde_json::json;
use stackpress_framework_schema::{
    Schema, ValueType,
    codegen::NormalizeOptions,
    source::{Column, Kind, SourceColumn, Value},
};
use stackpress_idea_parser::{CompileInputs, parse};

/// Use actual source normalization, preserving the dictionary's names, arguments and messages.
fn column(declaration: &str) -> SourceColumn {
    let input = format!("model Entry {{ {declaration} }}");
    let config = parse("source.idea", &input, &CompileInputs::default()).unwrap();
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    let field = graph.models()[0].fields[0].clone();
    let kind = if field.multiple {
        Kind::Strings
    } else {
        match field.value_type {
            ValueType::Boolean => Kind::Boolean,
            ValueType::Object => Kind::Object,
            ValueType::Date | ValueType::Time | ValueType::Datetime => Kind::Date,
            _ => Kind::String,
        }
    };
    SourceColumn::new(
        Column {
            kind,
            nullable: !field.required && !field.multiple,
        },
        field,
    )
    .with_catalog(std::sync::Arc::new(graph.catalog().unwrap()))
}

#[test]
fn g51_eq_neq_regex_weq_and_exclusive_utf16_boundaries() {
    for (declaration, good, bad, message) in [
        (
            "value String @is.eq(\"x\")",
            json!("x"),
            json!("y"),
            "Must be equal to x.",
        ),
        (
            "value String @is.neq(\"x\")",
            json!("y"),
            json!("x"),
            "Must not be equal to x.",
        ),
        (
            "value String @is.pattern(\"^a+$\")",
            json!("aaa"),
            json!("aba"),
            "Invalid format.",
        ),
        (
            "value String @is.weq(2)",
            json!("one two"),
            json!("one"),
            "Must be 2 words.",
        ),
        (
            "value String @is.cgt(2)",
            json!("abc"),
            json!("ab"),
            "Must be greater than 2 characters.",
        ),
        (
            "value String @is.clt(2)",
            json!("a"),
            json!("ab"),
            "Must be less than 2 characters.",
        ),
    ] {
        let column = column(declaration);
        assert_eq!(
            column.assert(Some(&Value::from(good))),
            None,
            "{declaration}"
        );
        assert_eq!(
            column.assert(Some(&Value::from(bad))),
            Some(message.into()),
            "{declaration}"
        );
    }
    let column = column("value String @is.cgt(2)");
    assert!(column.assert(Some(&Value::from("😀"))).is_some());
    assert_eq!(column.assert(Some(&Value::from("😀a"))), None);
}

#[test]
fn omitted_type_rules_use_original_messages_and_do_not_coerce() {
    for (declaration, value, message) in [
        ("value Hash @is.string", json!({}), "Must be a string."),
        ("value Hash @is.boolean", json!({}), "Must be a boolean."),
        ("value Hash @is.number", json!({}), "Must be a number."),
        (
            "value Hash @is.float",
            json!({}),
            "Must be a valid float number.",
        ),
        (
            "value Number @step(0.1) @is.integer",
            json!(1.5),
            "Must be a valid integer format.",
        ),
        (
            "value String @is.object",
            json!("object"),
            "Must be an object.",
        ),
    ] {
        assert_eq!(
            column(declaration).assert(Some(&Value::from(value))),
            Some(message.into()),
            "{declaration}"
        );
    }
    let unique = column("value String @is.unique");
    assert!(!unique.field.unique);
    assert!(unique.field.rules.iter().any(|rule| rule.code == "unique"));
    assert_eq!(unique.field.rules[0].message, "Already exists.");
    assert_eq!(
        unique.assert(Some(&Value::from("present"))),
        None,
        "unique is action-owned"
    );
}

#[test]
fn list_refinements_report_the_first_item_error_before_later_type_errors() {
    let list =
        column("value String[] @is.cgt(2 \"{{label}} {{arg}} {{value}}\") @label(\"Words\")");
    assert_eq!(
        list.assert(Some(&Value::from(json!(["abcd", "ab", false])))),
        Some("Words 2 ab".into())
    );
    assert_eq!(list.assert(Some(&Value::from(json!([])))), None);
    assert_eq!(
        list.assert(Some(&Value::from(json!(["abcd", "efg"])))),
        None
    );
}

#[test]
fn nanoid_and_length_defaults_use_the_original_provider_calls() {
    for (declaration, length) in [
        ("value String @default(\"nanoid()\")", 21),
        ("value String @default(\"nanoid(0)\")", 0),
        ("value String @default(\"nanoid(8)\")", 8),
        ("value String @default(\"cuid(8)\")", 8),
        ("value String @default(\"cuid(0)\")", 2),
        ("value String @default(\"cuid(1)\")", 1),
    ] {
        let column = column(declaration);
        let value = column
            .defaults_at(
                0,
                &stackpress_framework_schema::providers::BuiltinProviders,
                0,
            )
            .unwrap()
            .unwrap();
        assert_eq!(value.as_str().unwrap().len(), length, "{declaration}");
    }
}

#[test]
fn enum_fieldset_scalar_and_all_list_families_retain_complete_metadata_and_native_values() {
    let input = "enum Mood { ACTIVE \"active\" AWAY \"away\" } type Address { label String @default(\"123\") zip Integer @default(7) } model Catalog { mood Mood @default(\"active\") address Address? numbers Number[] flags Boolean[] objects Hash[] dates Datetime[] moods Mood[] addresses Address[] }";
    let config = parse("source.idea", input, &CompileInputs::default()).unwrap();
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    assert_eq!(graph.enums().len(), 1);
    assert_eq!(graph.fieldsets().len(), 1);
    let catalog = std::sync::Arc::new(graph.catalog().unwrap());
    let schema = stackpress_framework_schema::source::SourceRecordSchema::from_metadata(
        &graph.models()[0],
        catalog,
        "seed".into(),
        0,
    );
    assert_eq!(
        schema.defaults_at(0).unwrap()["address"],
        json!({"label":"123","zip":7})
    );
    assert_eq!(
        schema
            .assert(
                &serde_json::from_value::<Value>(json!({"mood":"missing"}))
                    .unwrap()
                    .as_object()
                    .unwrap()
                    .clone(),
                false
            )
            .unwrap()["mood"],
        "Invalid option: expected one of \"active\"|\"away\""
    );
    let input=serde_json::from_value::<Value>(json!({"mood":"active","address":{"label":"123","zip":"8"},"numbers":["2",true,{},null],"flags":["false",false,0],"objects":[{"x":1},"{\"y\":2}"],"dates":[0],"moods":["active",4],"addresses":[{"label":"abc"}]})).unwrap();
    let serialized = schema.serialize(input.as_object().unwrap(), false).unwrap();
    assert_eq!(serialized["numbers"], "[2,1,0,0]");
    assert_eq!(serialized["flags"], "[false,false,false]");
    assert_eq!(serialized["dates"], "[\"1970-01-01 00:00:00\"]");
    assert_eq!(
        schema.unserialize(&serialized, false).unwrap()["address"],
        json!({}),
        "schema.ts double stringification is preserved pending a separate owner ruling"
    );
    let nested = schema.assert(input.as_object().unwrap(), false).unwrap();
    assert_eq!(
        nested["address"]["zip"],
        "Invalid input: expected number, received string"
    );
}

#[test]
fn complete_canonical_dictionary_rule_matrix_and_authored_messages() {
    let cases = [
        ("required", "String", "", json!("good"), json!("")),
        ("ne", "String", "", json!("good"), json!("")),
        ("eq", "String", "\"yes\"", json!("yes"), json!("no")),
        ("neq", "String", "\"no\"", json!("yes"), json!("no")),
        ("option", "String", "[\"a\" \"b\"]", json!("a"), json!("c")),
        ("regex", "String", "\"^a+$\"", json!("aa"), json!("ba")),
        ("gt", "Number", "2", json!(3), json!(2)),
        ("ge", "Number", "2", json!(2), json!(1)),
        ("lt", "Number", "2", json!(1), json!(2)),
        ("le", "Number", "2", json!(2), json!(3)),
        ("ceq", "String", "2", json!("ab"), json!("a")),
        ("cgt", "String", "2", json!("abc"), json!("ab")),
        ("cge", "String", "2", json!("ab"), json!("a")),
        ("clt", "String", "2", json!("a"), json!("ab")),
        ("cle", "String", "2", json!("ab"), json!("abc")),
        ("weq", "String", "2", json!("a b"), json!("a")),
        ("wgt", "String", "2", json!("a b c"), json!("a b")),
        ("wge", "String", "2", json!("a b"), json!("a")),
        ("wlt", "String", "2", json!("a"), json!("a b")),
        ("wle", "String", "2", json!("a b"), json!("a b c")),
        ("lowercase", "String", "", json!("abc"), json!("Abc")),
        ("uppercase", "String", "", json!("ABC"), json!("Abc")),
        ("slug", "String", "", json!("a-b_1"), json!("a b")),
        ("cc", "String", "", json!("4111111111111111"), json!("bad")),
        ("color", "String", "", json!("#fff"), json!("red")),
        ("email", "String", "", json!("a@example.com"), json!("a@b")),
        ("hex", "String", "", json!("ab09"), json!("zz")),
        ("price", "String", "", json!("12.00"), json!("12.0")),
        (
            "url",
            "String",
            "",
            json!("https://example.com"),
            json!("no url"),
        ),
        (
            "date",
            "String",
            "",
            json!("1970-01-01"),
            json!("not a date"),
        ),
    ];
    let definitions: serde_json::Value =
        serde_json::from_str(include_str!("../src/codegen/source-assertions.json")).unwrap();
    let mut exercised = std::collections::BTreeSet::new();
    for (name, kind, args, good, bad) in cases {
        let extra = if kind == "Number" {
            " @min(-100) @max(100)"
        } else {
            ""
        };
        let declaration = format!("value {kind}{extra} @is.{name}({args})");
        let column = column(&declaration);
        assert_eq!(
            column.assert_at(Some(&Value::from(good)), 0, 0),
            None,
            "{declaration}"
        );
        assert!(
            column
                .assert_at(Some(&Value::from(bad.clone())), 0, 0)
                .is_some(),
            "{declaration}: {bad}"
        );
        // A second normalization pins the original dictionary message, not a made-up fallback.
        assert_eq!(
            column.field.rules[0].message,
            definitions
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["name"] == name)
                .unwrap()["message"]
        );
        exercised.insert(name.to_owned());
    }
    for (name, good, bad) in [
        ("future", 2000, 0),
        ("past", 0, 2000),
        ("present", 1000, 86_401_000),
    ] {
        let column = column(&format!("value Datetime @is.{name}"));
        assert_eq!(
            column.assert_at(
                Some(&stackpress_framework_schema::source::date(good)),
                1000,
                0
            ),
            None
        );
        assert!(
            column
                .assert_at(
                    Some(&stackpress_framework_schema::source::date(bad)),
                    1000,
                    0
                )
                .is_some()
        );
        exercised.insert(name.into());
    }
    // Type refinements are independently covered on compatible/incompatible base shapes;
    // uniqueness cannot be proved by schema validation and is covered through generated actions.
    exercised.extend(
        [
            "unique", "string", "boolean", "number", "float", "integer", "object",
        ]
        .map(str::to_owned),
    );
    let canonical = definitions
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["canonical"].as_str().unwrap().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        exercised, canonical,
        "new dictionary rules need an explicit semantic proof"
    );
    assert_eq!(
        column("value String").assert(Some(&Value::from(""))),
        Some("Value is required.".into())
    );
}

#[test]
fn g51_zero_valued_numeric_rules_remain_valid_bounds_and_epoch_dates_are_valid() {
    let eq = column("value Number @is.eq(0)");
    assert_eq!(eq.assert(Some(&Value::from(0))), None);
    assert!(eq.assert(Some(&Value::from(1))).is_some());
    let gt = column("value Number @is.gt(0)");
    assert_eq!(gt.assert(Some(&Value::from(1))), None);
    assert_eq!(
        gt.assert(Some(&Value::from(0))),
        Some("Must be greater than 0.".into())
    );
    let date = column("value String @is.date");
    assert_eq!(date.assert(Some(&Value::from("1970-01-01T00:00:00"))), None);
    assert!(date.assert(Some(&Value::from("not a date"))).is_some());
    let huge = parse(
        "provider.idea",
        "model Entry { value String @default(\"nanoid(18446744073709551616)\") }",
        &CompileInputs::default(),
    )
    .unwrap();
    assert!(Schema::normalize_source(&huge, NormalizeOptions::default(), &[]).is_err());
}

#[path = "source_extensions/oracles.rs"]
mod oracles;

#[test]
fn native_integral_float_text_and_shared_number_interpolation_match_source() {
    let col = column("value String");
    assert_eq!(
        col.serialize(Some(&Value::from(1.0)), "", false, 0)
            .unwrap(),
        Some(Value::from("1"))
    );
    for (number, text) in [(0.0, "0"), (1e21, "1e+21"), (1e-7, "1e-7"), (1.25, "1.25")] {
        assert_eq!(
            stackpress_framework_schema::source::number_text(number),
            text
        );
    }
}
