//! Prepared-type encoding boundaries without a native server or synthetic output rows.

/// Supported prepared scalar widths accept values/nulls and produce exactly one native argument.
#[test]
fn supported_scalar_pairs_encode_one_argument() {
    use super::bind;
    use sqlx::{
        Arguments, Execute,
        postgres::{PgTypeInfo, types::Oid},
    };
    use stackpress_sql::SqlValue;
    for (oid, value) in [
        (16, SqlValue::Bool(true)),
        (21, SqlValue::Integer(i16::MAX.into())),
        (23, SqlValue::Integer(i32::MIN.into())),
        (20, SqlValue::Integer(i64::MAX)),
        (700, SqlValue::Real(1.5)),
        (701, SqlValue::Real(-2.5)),
        (25, SqlValue::Text("text".into())),
        (1043, SqlValue::Text("varchar".into())),
        (17, SqlValue::Blob(vec![0, 255])),
        (114, SqlValue::Text("{\"key\":null}".into())),
        (3802, SqlValue::Text("[1,true]".into())),
    ] {
        for input in [value, SqlValue::Null] {
            let mut query = bind(
                sqlx::query("SELECT $1"),
                &input,
                &PgTypeInfo::with_oid(Oid(oid)),
            )
            .unwrap();
            let arguments = query.take_arguments().unwrap().unwrap();
            assert_eq!(arguments.len(), 1, "OID {oid}");
        }
    }
}

/// Width overflow, incompatible types and invalid JSON are refused before native execution.
#[test]
fn unsupported_or_unrepresentable_binds_return_typed_errors() {
    use super::{bind, check_arity};
    use sqlx::postgres::{PgTypeInfo, types::Oid};
    use stackpress_sql::{ErrorKind, SqlValue};
    for (oid, value, expected) in [
        (21, SqlValue::Integer(i64::MAX), ErrorKind::InvalidQuery),
        (23, SqlValue::Integer(i64::MIN), ErrorKind::InvalidQuery),
        (700, SqlValue::Real(f64::MAX), ErrorKind::InvalidQuery),
        (
            114,
            SqlValue::Text("invalid JSON".into()),
            ErrorKind::InvalidQuery,
        ),
        (
            3802,
            SqlValue::Text("{\"a\":1,\"a\":2}".into()),
            ErrorKind::InvalidQuery,
        ),
        (16, SqlValue::Integer(1), ErrorKind::Unsupported),
        (1700, SqlValue::Null, ErrorKind::Unsupported),
    ] {
        let error = bind(
            sqlx::query("SELECT $1"),
            &value,
            &PgTypeInfo::with_oid(Oid(oid)),
        )
        .err()
        .unwrap();
        assert_eq!(error.kind(), expected, "OID {oid}");
    }
    assert!(
        bind(
            sqlx::query("SELECT $1"),
            &SqlValue::Null,
            &PgTypeInfo::with_name("INT8")
        )
        .is_err()
    );
    check_arity(1, 1).unwrap();
    assert_eq!(
        check_arity(2, 1).unwrap_err().kind(),
        ErrorKind::Arity {
            expected: 2,
            actual: 1
        }
    );
}
