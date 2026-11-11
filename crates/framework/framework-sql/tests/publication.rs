//! Public projection contracts at missing, nullable, nested and bounded input paths.
use serde::Serialize;
use serde_json::json;
use stackpress_framework_sql::{DataErrorKind, projection};

/// Base fallback and an intentionally empty selection retain different public shapes.
#[test]
fn base_and_empty_selection_keep_their_declared_shape() {
    let view = json!({"row":{"id":"one","nested":{"row":"ordinary data"}}});
    assert_eq!(projection::select(&view, &[], true).unwrap(), view["row"]);
    assert_eq!(projection::select(&view, &[], false).unwrap(), json!({}));
    assert_eq!(
        projection::select(&json!({}), &[], true)
            .unwrap_err()
            .kind(),
        DataErrorKind::Decode
    );
}

/// Whole related views flatten loaded edges, preserve null/list cardinality and omit unloaded ones.
#[test]
fn full_relations_flatten_only_loaded_public_edges() {
    let view = json!({
        "row":{"id":"root"},
        "children":{"state":"loaded","value":[
            {"row":{"id":"first","object":{"row":"data","state":"data"}},
             "__row":{"state":"loaded","value":{"row":{"id":"nested"}}},
             "hidden":{"state":"unloaded"}},
            null
        ]},
        "optional":{"state":"loaded","value":null}
    });
    assert_eq!(
        projection::select(&view, &["children", "optional"], false).unwrap(),
        json!({"children":[{"id":"first","object":{"row":"data","state":"data"},
                            "row":{"id":"nested"}},null],"optional":null})
    );
    // Selecting both a complete record and a narrower base field retains its approved siblings.
    assert_eq!(
        projection::select(&view, &["children", "children.object.row"], false).unwrap()["children"]
            [0]["id"],
        "first"
    );
}

/// Field arrays remain user data even when their keys resemble the eager-wrapper protocol.
#[test]
fn nested_field_arrays_and_reserved_relation_names_remain_distinct() {
    let view = json!({"row":{"items":[{"row":{"state":"data"}},{"row":null}]},
                      "__row":{"state":"loaded","value":{"row":{"name":"relation"}}}});
    assert_eq!(
        projection::select(&view, &["items.row.state", "row.name"], false).unwrap(),
        json!({"items":[{"row":{"state":"data"}},{"row":null}],"row":{"name":"relation"}})
    );
}

/// A missing member, unloaded relation or malformed wrapper never becomes a fabricated null.
#[test]
fn invalid_projection_states_return_decode_errors() {
    for (view, path) in [
        (json!({"row":{}}), "absent"),
        (json!({"row":{},"edge":{"state":"unloaded"}}), "edge"),
        (json!({"row":{},"edge":{"state":"loaded"}}), "edge"),
        (
            json!({"row":{},"edge":{"state":"loaded","value":{"id":1}}}),
            "edge",
        ),
        (json!({"row":{"object":{}}}), "object.missing"),
        (json!({"row":{}}), "field..child"),
        (json!({"row":{}}), ".field"),
    ] {
        assert_eq!(
            projection::select(&view, &[path], false)
                .unwrap_err()
                .kind(),
            DataErrorKind::Decode
        );
    }
    // Both independent path budgets are checked before any lookup occurs.
    for path in ["x".repeat(1025), vec!["x"; 33].join(".")] {
        assert_eq!(
            projection::select(&json!({"row":{}}), &[&path], false)
                .unwrap_err()
                .kind(),
            DataErrorKind::Decode
        );
    }
}

/// Nested list traversal is bounded even when no additional dotted path is requested.
#[test]
fn excessive_view_nesting_is_rejected_without_truncation() {
    let mut nested = json!({"row":{"id":"leaf"}});
    for _ in 0..42 {
        nested = json!([nested]);
    }
    let view = json!({"row":{},"edge":{"state":"loaded","value":nested}});
    for paths in [&["edge"][..], &["edge.id"][..]] {
        assert_eq!(
            projection::select(&view, paths, false).unwrap_err().kind(),
            DataErrorKind::Decode
        );
    }
}

/// A serializer failure propagates as a decode error rather than an empty public projection.
#[test]
fn projection_preserves_serialization_failures() {
    struct Rejected;
    impl Serialize for Rejected {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("fixture cannot serialize"))
        }
    }
    assert_eq!(
        projection::select(&Rejected, &["id"], false)
            .unwrap_err()
            .kind(),
        DataErrorKind::Decode
    );
}

/// Source SQL scalars preserve nullable, Boolean, Date and unknown-projection semantics.
#[cfg(feature = "source")]
#[test]
fn source_scalar_conversion_uses_column_metadata() {
    use stackpress_framework_schema::source::{Column, Kind, Value as SourceValue, date};
    use stackpress_framework_sql::source::to_sql_value;
    use stackpress_sql::SqlValue;
    let column = |kind: Kind, nullable: bool| Column { kind, nullable };
    assert_eq!(
        to_sql_value(None, &SourceValue::Null, 0).unwrap(),
        SqlValue::Null
    );
    assert_eq!(
        to_sql_value(None, &SourceValue::from(json!({"x":1})), 0).unwrap(),
        SqlValue::Text("{\"x\":1}".into())
    );
    assert_eq!(
        to_sql_value(Some(&column(Kind::String, true)), &SourceValue::Null, 0).unwrap(),
        SqlValue::Null
    );
    assert_eq!(
        to_sql_value(Some(&column(Kind::String, false)), &SourceValue::Null, 0).unwrap(),
        SqlValue::Text(String::new())
    );
    for (value, expected) in [
        (SourceValue::Null, false),
        (SourceValue::from("false"), true),
    ] {
        assert_eq!(
            to_sql_value(Some(&column(Kind::Boolean, false)), &value, 0).unwrap(),
            SqlValue::Bool(expected)
        );
    }
    assert_eq!(
        to_sql_value(Some(&column(Kind::Date, false)), &date(0), 0).unwrap(),
        SqlValue::Text("1970-01-01T00:00:00.000Z".into())
    );
    // Keep ordinary scalar text on the source string conversion path.
    assert_eq!(
        to_sql_value(
            Some(&column(Kind::String, false)),
            &SourceValue::from("text"),
            0
        )
        .unwrap(),
        SqlValue::Text("text".into())
    );
}
