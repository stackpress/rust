//! M1 Expr/JSON/value/row proofs: helper encoding, typed paths and SQLite JSON selectors.

use stackpress_sql::*;

/// Render one scalar extraction with a caller-selected typed path.
fn extract(path: JsonPath) -> Query {
    Select::new([Selector::expression(Expr::json("data", path)).alias("selected")])
        .from("p")
        .render(DialectId::Sqlite)
        .unwrap()
}

/// Typed paths preserve key/index distinction, hostile punctuation and root selection.
#[test]
fn json_paths_are_bound_structural_data() {
    let path = JsonPath::new([
        JsonSegment::Key("01".into()),
        JsonSegment::Index(2),
        JsonSegment::Key("a\"b\\c\0.; -- 💡".into()),
    ]);
    assert_eq!(path.segments().len(), 3);
    let rendered = extract(path);
    assert_eq!(
        rendered.sql(),
        "SELECT json_extract(\"data\", ?) AS \"selected\" FROM \"p\""
    );
    assert_eq!(
        rendered.binds(),
        &[SqlValue::Text(
            "$.\"01\"[2].\"a\\\"b\\\\c\\u0000.; -- 💡\"".into()
        )]
    );
    assert_eq!(extract(JsonPath::default()).binds(), &["$".into()]);
    assert_eq!(
        extract(JsonPath::new([JsonSegment::Key(String::new())])).binds(),
        &["$.\"\"".into()]
    );
}

/// Parse options belong to each path; ambiguous convenience syntax is rejected.
#[test]
fn checked_json_path_options_and_limits() {
    let dot = JsonPath::parse("info.name", JsonPathOptions::default()).unwrap();
    let slash = JsonPath::parse("info/name", JsonPathOptions { separator: '/' }).unwrap();
    assert_eq!(dot, slash);
    assert_eq!(
        JsonPath::parse("01", JsonPathOptions::default())
            .unwrap()
            .segments(),
        &[JsonSegment::Key("01".into())]
    );
    for path in ["", ".a", "a.", "a..b"] {
        assert!(JsonPath::parse(path, JsonPathOptions::default()).is_err());
    }
    for separator in ['\0', 'a', '1', '"', '\\', '[', ']', '$'] {
        assert!(JsonPath::parse("a", JsonPathOptions { separator }).is_err());
    }
    for path in [
        JsonPath::new(vec![JsonSegment::Index(0); 65]),
        JsonPath::new([JsonSegment::Index(u32::MAX)]),
        JsonPath::new([JsonSegment::Key("x".repeat(16 * 1024))]),
    ] {
        assert!(
            Select::new([Selector::expression(Expr::json("data", path))])
                .from("p")
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    assert!(
        extract(JsonPath::new([JsonSegment::Index(i32::MAX as u32)])).binds()[0]
            == SqlValue::Text(format!("$[{}]", i32::MAX))
    );
}

/// JSON equality binds the exact type; JSON null does not match an absent path.
#[test]
fn json_equality_preserves_scalar_types_and_null() {
    for (value, kind) in [
        (JsonScalar::Null, "null"),
        (JsonScalar::Bool(true), "true"),
        (JsonScalar::Bool(false), "false"),
        (JsonScalar::Integer(1), "integer"),
        (JsonScalar::Real(1.0), "real"),
        (JsonScalar::String("1".into()), "text"),
    ] {
        let path = JsonPath::parse("item", JsonPathOptions::default()).unwrap();
        let rendered = Select::default()
            .from("p")
            .where_json("data", path, value.clone())
            .render(DialectId::Sqlite)
            .unwrap();
        assert_eq!(rendered.binds()[1], SqlValue::Text(kind.into()));
        assert!(rendered.sql().contains("json_type(\"data\", ?) = ?"));
        if value == JsonScalar::Null {
            assert_eq!(rendered.binds().len(), 2);
        } else {
            assert_eq!(rendered.binds().len(), 4);
            assert!(rendered.sql().contains("json_extract(\"data\", ?) = ?"));
        }
    }
}

/// Direct membership is array-gated and type-checked rather than recursive containment.
#[test]
fn json_membership_and_dml_share_one_expression_contract() {
    let path = JsonPath::parse("tags", JsonPathOptions::default()).unwrap();
    let query = Select::default().from("p").where_json_contains(
        "data",
        path.clone(),
        JsonScalar::String("'admin';--".into()),
    );
    let rendered = query.render(DialectId::Sqlite).unwrap();
    assert!(rendered.sql().contains("= 'array' AND EXISTS"));
    assert!(rendered.sql().contains("json_each(\"data\", ?)"));
    assert_eq!(
        rendered.binds(),
        &[
            "$.\"tags\"".into(),
            "$.\"tags\"".into(),
            "text".into(),
            "'admin';--".into()
        ]
    );
    let null = Select::default()
        .from("p")
        .where_json_contains("data", path.clone(), JsonScalar::Null)
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(null.binds().len(), 3);
    assert!(!null.sql().contains("\"value\" ="));
    let update = Update::new("p")
        .set("x", 1)
        .where_json("data", path.clone(), JsonScalar::Bool(false))
        .where_json_contains("data", path.clone(), JsonScalar::Integer(7));
    let delete = Delete::new("p")
        .where_json("data", path.clone(), JsonScalar::Bool(false))
        .where_json_contains("data", path, JsonScalar::Integer(7));
    assert_eq!(update.build().filters, delete.build().filters);
    assert_eq!(
        &update.render(DialectId::Sqlite).unwrap().binds()[1..],
        delete.render(DialectId::Sqlite).unwrap().binds()
    );
    for value in [JsonScalar::Real(f64::NAN), JsonScalar::Real(f64::INFINITY)] {
        assert!(
            Select::default()
                .from("p")
                .where_json("data", JsonPath::default(), value.clone())
                .render(DialectId::Sqlite)
                .is_err()
        );
        assert!(
            Select::default()
                .from("p")
                .where_json_contains("data", JsonPath::default(), value)
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
}

/// Explicit JSON encoding preserves scalar types, order and standard escaping.
#[test]
fn bounded_explicit_json_encoding() {
    let document = JsonValue::Object(vec![
        (
            "\"\\\0💡".into(),
            JsonValue::Array(vec![
                JsonValue::Scalar(JsonScalar::Null),
                JsonValue::Scalar(JsonScalar::Bool(false)),
                JsonValue::Scalar(JsonScalar::Integer(i64::MAX)),
                JsonValue::Scalar(JsonScalar::Real(1.0)),
                JsonValue::Scalar(JsonScalar::String("\n\r\t'".into())),
            ]),
        ),
        ("empty".into(), JsonValue::Object(vec![])),
    ]);
    let encoded = document.to_json().unwrap();
    assert_eq!(
        encoded,
        "{\"\\\"\\\\\\u0000💡\":[null,false,9223372036854775807,1.0,\"\\u000a\\u000d\\u0009'\"],\"empty\":{}}"
    );
    assert_eq!(SqlValue::json(&document).unwrap(), SqlValue::Text(encoded));
    assert_eq!(
        SqlValue::json(&JsonValue::Scalar(JsonScalar::Null)).unwrap(),
        SqlValue::Text("null".into())
    );
    for bad in [
        JsonValue::Scalar(JsonScalar::Real(f64::NAN)),
        JsonValue::Object(vec![
            ("x".into(), JsonValue::Array(vec![])),
            ("x".into(), JsonValue::Array(vec![])),
        ]),
    ] {
        assert!(bad.to_json().is_err());
    }
    let mut deep = JsonValue::Scalar(JsonScalar::Null);
    for _ in 0..66 {
        deep = JsonValue::Array(vec![deep]);
    }
    assert_eq!(deep.to_json().unwrap_err().kind(), ErrorKind::Limit);
    let wide = JsonValue::Array(vec![JsonValue::Scalar(JsonScalar::Null); 10_001]);
    assert_eq!(wide.to_json().unwrap_err().kind(), ErrorKind::Limit);
    for text in ["x".repeat(1024 * 1024 + 1), "\0".repeat(200_000)] {
        assert_eq!(
            JsonValue::Scalar(JsonScalar::String(text))
                .to_json()
                .unwrap_err()
                .kind(),
            ErrorKind::Limit
        );
    }
}

/// Dates require canonical UTC and a real calendar day; no epoch/default fallback exists.
#[test]
fn explicit_canonical_dates_validate_calendar_and_timezone() {
    for value in [
        "2024-02-29T23:59:59Z",
        "2000-02-29T00:00:00.123456789Z",
        "0001-01-01T00:00:00Z",
        "2026-10-26T12:00:00.1Z",
    ] {
        assert_eq!(
            SqlValue::utc_text(value).unwrap(),
            SqlValue::Text(value.into())
        );
    }
    for value in [
        "",
        "2023-02-29T00:00:00Z",
        "1900-02-29T00:00:00Z",
        "2026-04-31T00:00:00Z",
        "0000-01-01T00:00:00Z",
        "2026-00-01T00:00:00Z",
        "2026-13-01T00:00:00Z",
        "2026-01-00T00:00:00Z",
        "2026-01-01T24:00:00Z",
        "2026-01-01T00:60:00Z",
        "2026-01-01T00:00:60Z",
        "2026-01-01 00:00:00Z",
        "2026-01-01T00:00:00+00:00",
        "2026-01-01T00:00:00.Z",
        "2026-01-01T00:00:00.xZ",
        "2026-01-01T00:00:00.1234567890Z",
        "💡26-01-01T00:00:00Z",
    ] {
        assert!(SqlValue::utc_text(value).is_err(), "{value}");
    }
}

/// Strict native conversions enforce ranges, finite floats, NULL and actual storage types.
#[test]
fn value_conversions_are_exact_and_fallible() {
    assert_eq!(
        SqlValue::try_from(i64::MAX as u64).unwrap(),
        SqlValue::Integer(i64::MAX)
    );
    assert!(SqlValue::try_from(u64::MAX).is_err());
    for (value, expected) in [
        (SqlValue::from(1_i8), 1),
        (SqlValue::from(2_i16), 2),
        (SqlValue::from(3_i32), 3),
        (SqlValue::from(4_u8), 4),
        (SqlValue::from(5_u16), 5),
        (SqlValue::from(6_u32), 6),
    ] {
        assert_eq!(value, SqlValue::Integer(expected));
    }
    assert_eq!(SqlValue::from(1.5_f64), SqlValue::Real(1.5));
    assert_eq!(
        SqlValue::from(vec![0_u8, 255]),
        SqlValue::Blob(vec![0, 255])
    );
    assert_eq!(
        String::from_sql(&SqlValue::from(String::from(""))).unwrap(),
        ""
    );
    assert_eq!(Option::<bool>::from_sql(&SqlValue::Null).unwrap(), None);
    assert_eq!(
        Option::<bool>::from_sql(&SqlValue::Integer(0)).unwrap(),
        Some(false)
    );
    assert!(Option::<bool>::from_sql(&SqlValue::Integer(2)).is_err());
    assert_eq!(
        Vec::<u8>::from_sql(&SqlValue::Blob(vec![0, 255])).unwrap(),
        [0, 255]
    );
    assert!(Vec::<u8>::from_sql(&SqlValue::Text("bytes".into())).is_err());
    assert_eq!(f64::from_sql(&SqlValue::Real(1.5)).unwrap(), 1.5);
    for value in [
        SqlValue::Real(f64::INFINITY),
        SqlValue::Integer(1),
        SqlValue::Null,
    ] {
        assert!(f64::from_sql(&value).is_err());
    }
    assert_eq!(i8::from_sql(&SqlValue::Integer(127)).unwrap(), 127);
    assert!(i8::from_sql(&SqlValue::Integer(128)).is_err());
    assert!(u8::from_sql(&SqlValue::Integer(-1)).is_err());
    assert_eq!(u8::from_sql(&SqlValue::Integer(255)).unwrap(), 255);
    assert_eq!(i16::from_sql(&SqlValue::Integer(32_767)).unwrap(), 32_767);
    assert_eq!(u16::from_sql(&SqlValue::Integer(65_535)).unwrap(), 65_535);
    assert_eq!(
        i32::from_sql(&SqlValue::Integer(i32::MAX as i64)).unwrap(),
        i32::MAX
    );
    assert_eq!(
        u32::from_sql(&SqlValue::Integer(u32::MAX as i64)).unwrap(),
        u32::MAX
    );
    assert_eq!(
        u64::from_sql(&SqlValue::Integer(i64::MAX)).unwrap(),
        i64::MAX as u64
    );
    assert_eq!(usize::from_sql(&SqlValue::Integer(1)).unwrap(), 1);
}

/// Owned metadata/order expose both duplicate join columns and explicit native declarations.
#[test]
fn ordered_rows_metadata_decode_and_byte_accounting() {
    let row = Row::new(vec![
        ("id".into(), 1.into()),
        ("id".into(), 2.into()),
        ("nil".into(), SqlValue::Null),
    ]);
    assert!(row.try_get::<i64>("id").is_err());
    assert_eq!(row.try_get_at::<i64>(0).unwrap(), 1);
    assert_eq!(row.try_get_at::<i64>(1).unwrap(), 2);
    assert_eq!(row.try_get_at::<Option<String>>(2).unwrap(), None);
    assert!(row.try_get_at::<i64>(3).is_err());
    assert_eq!(row.retained_bytes().unwrap(), 2 + 8 + 2 + 8 + 3);
    let metadata = vec![
        ColumnMetadata {
            name: "id".into(),
            ordinal: 0,
            declared_type: Some("INTEGER".into()),
        },
        ColumnMetadata {
            name: "id".into(),
            ordinal: 1,
            declared_type: None,
        },
        ColumnMetadata {
            name: "nil".into(),
            ordinal: 2,
            declared_type: None,
        },
    ];
    let typed = row.clone().with_metadata(metadata.clone()).unwrap();
    assert_eq!(
        typed.columns().collect::<Vec<_>>()[0],
        ColumnMetadataRef {
            name: "id",
            ordinal: 0,
            declared_type: Some("INTEGER")
        }
    );
    assert_eq!(
        typed.retained_bytes().unwrap(),
        row.retained_bytes().unwrap() + 7
    );
    assert!(row.clone().with_metadata(vec![]).is_err());
    let mut wrong_ordinal = metadata.clone();
    wrong_ordinal[1].ordinal = 0;
    assert!(row.clone().with_metadata(wrong_ordinal).is_err());
    let mut wrong_name = metadata;
    wrong_name[1].name = "other".into();
    assert!(row.with_metadata(wrong_name).is_err());
}
