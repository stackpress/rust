//! Strict runtime codecs retain null/type/precision boundaries and concrete failures.

use stackpress_framework_sql::{
    DataError, DataErrorKind, StorageCodec, StorageColumn, decode, encode,
};
use stackpress_sql::{ErrorKind, Row, SqlError, SqlValue};
use std::error::Error;

/// A reusable small codec descriptor; tests vary the meaningful scalar/null policies.
fn column(codec: StorageCodec) -> StorageColumn {
    StorageColumn {
        wire: "value",
        column: "stored",
        codec,
        required: true,
        nonempty: false,
    }
}

/// Scalars round-trip without bool/numeric/string coercion; null remains SQL NULL.
#[test]
fn scalar_codecs_are_strict() {
    let cases = [
        (
            StorageCodec::Text,
            serde_json::json!("O'Reilly"),
            SqlValue::Text("O'Reilly".into()),
        ),
        (
            StorageCodec::Integer,
            serde_json::json!(i64::MIN),
            SqlValue::Integer(i64::MIN),
        ),
        (
            StorageCodec::Boolean,
            serde_json::json!(false),
            SqlValue::Bool(false),
        ),
        (
            StorageCodec::Real,
            serde_json::json!(1.5),
            SqlValue::Real(1.5),
        ),
        (
            StorageCodec::Json,
            serde_json::json!({"nested":[false,null,0]}),
            SqlValue::Text("{\"nested\":[false,null,0]}".into()),
        ),
    ];
    for (codec, json, sql) in cases {
        let descriptor = column(codec);
        let encoded = encode(&serde_json::json!({ "value": json }), &[descriptor]).unwrap();
        assert_eq!(encoded, vec![sql.clone()]);
        let decoded: serde_json::Value =
            decode(Row::new(vec![("stored".into(), sql)]), &[descriptor]).unwrap();
        assert_eq!(decoded, serde_json::json!({ "value": json }));
    }
    assert!(
        encode(
            &serde_json::json!({"value":9007199254740993_i64}),
            &[column(StorageCodec::Real)]
        )
        .is_err()
    );
    let descriptor = column(StorageCodec::Text);
    assert!(encode(&serde_json::json!({ "value": null }), &[descriptor]).is_err());
    assert!(encode(&serde_json::json!({ "value": 1 }), &[descriptor]).is_err());
    assert!(encode(&serde_json::json!({}), &[descriptor]).is_err());
    assert!(encode(&serde_json::json!({ "other": "x" }), &[descriptor]).is_err());
    assert!(encode(&serde_json::json!([]), &[descriptor]).is_err());
    assert!(decode::<serde_json::Value>(Row::new(vec![]), &[descriptor]).is_err());
    assert!(
        decode::<serde_json::Value>(
            Row::new(vec![("stored".into(), SqlValue::Null)]),
            &[descriptor]
        )
        .is_err()
    );
    let nullable = StorageColumn {
        required: false,
        ..descriptor
    };
    assert_eq!(
        encode(&serde_json::json!({"value":null}), &[nullable]).unwrap(),
        vec![SqlValue::Null]
    );
    assert_eq!(
        decode::<serde_json::Value>(
            Row::new(vec![("stored".into(), SqlValue::Null)]),
            &[nullable]
        )
        .unwrap(),
        serde_json::json!({"value":null})
    );
    let nonempty = StorageColumn {
        nonempty: true,
        ..descriptor
    };
    assert!(encode(&serde_json::json!({"value":""}), &[nonempty]).is_err());
    assert!(
        encode(
            &serde_json::json!({"value":"x".repeat(1024 * 1024 + 1)}),
            &[descriptor]
        )
        .is_err()
    );
    for (codec, value) in [
        (StorageCodec::Integer, serde_json::json!(1.5)),
        (StorageCodec::Integer, serde_json::json!(u64::MAX)),
        (StorageCodec::Boolean, serde_json::json!(0)),
        (StorageCodec::Real, serde_json::json!("1")),
    ] {
        assert!(encode(&serde_json::json!({"value":value}), &[column(codec)]).is_err());
    }
    assert!(
        decode::<serde_json::Value>(
            Row::new(vec![("stored".into(), SqlValue::Text("true".into()))]),
            &[column(StorageCodec::Boolean)]
        )
        .is_err()
    );
    assert!(
        decode::<serde_json::Value>(
            Row::new(vec![("stored".into(), SqlValue::Real(f64::INFINITY))]),
            &[column(StorageCodec::Real)]
        )
        .is_err()
    );
    assert!(
        decode::<serde_json::Value>(
            Row::new(vec![(
                "stored".into(),
                SqlValue::Text("invalid JSON".into())
            )]),
            &[column(StorageCodec::Json)]
        )
        .is_err()
    );
}

/// Each wrapper explains its own boundary and retains primary/secondary typed causes separately.
#[test]
fn error_context_and_cleanup_are_separate() {
    let sql = SqlError::new(ErrorKind::Driver, "private driver context");
    let error =
        DataError::from(sql).with_cleanup(SqlError::new(ErrorKind::Closed, "cleanup failed"));
    assert_eq!(error.kind(), DataErrorKind::Sql);
    assert_eq!(error.to_string(), "data storage operation failed");
    assert!(error.source().unwrap().downcast_ref::<SqlError>().is_some());
    assert_eq!(error.cleanup_error().unwrap().kind(), ErrorKind::Closed);
}

/// Oversized records must stop serialization before a later codec can run or all data is cloned.
#[test]
fn record_byte_budget_precedes_later_codec_work() {
    struct Late;
    impl serde::Serialize for Late {
        /// Reaching this field means earlier oversized data was eagerly consumed.
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("late codec should not run"))
        }
    }
    #[derive(serde::Serialize)]
    struct Record<'a> {
        first: &'a str,
        second: &'a str,
        last: Late,
    }
    let text = "x".repeat(600_000);
    let record = Record {
        first: &text,
        second: &text,
        last: Late,
    };
    let columns = ["first", "second", "last"].map(|name| StorageColumn {
        wire: name,
        column: name,
        ..column(StorageCodec::Text)
    });
    let error = encode(&record, &columns).unwrap_err();
    assert_eq!(error.kind(), DataErrorKind::Validation);
    assert_eq!(error.to_string(), "record byte limit exceeded");
}
