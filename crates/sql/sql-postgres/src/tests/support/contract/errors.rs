//! Shared public-trait errors assertions; native syntax differences stay in the suite inputs.
use super::{
    Database, DialectId, ErrorKind, QueryLimits, Result, SqlValue, integer, query, scalar,
};
use std::error::Error;

/// Assert pre-effect input validation, independent native arity, whole-result limits and causes.
pub(super) async fn check(database: &dyn Database) -> Result<()> {
    let dialect = database.dialect();
    let one = scalar(dialect, &SqlValue::Integer(1));
    for (sql, binds, expected, actual) in [
        (one, vec![], 1, 0),
        ("SELECT 1", vec![SqlValue::Integer(1)], 0, 1),
    ] {
        assert_eq!(
            database
                .run(query(dialect, sql, binds), QueryLimits::default())
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Arity { expected, actual }
        );
    }
    let mismatch = if dialect == DialectId::Sqlite {
        DialectId::Postgres
    } else {
        DialectId::Sqlite
    };
    assert_eq!(
        database
            .run(query(mismatch, "SELECT 1", []), QueryLimits::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    for limits in [
        QueryLimits {
            sql_bytes: 0,
            ..QueryLimits::default()
        },
        QueryLimits {
            bytes: 0,
            ..QueryLimits::default()
        },
        QueryLimits {
            rows: 0,
            ..QueryLimits::default()
        },
    ] {
        assert_eq!(
            database
                .run(query(dialect, "SELECT 'payload' AS value", []), limits)
                .await
                .unwrap_err()
                .kind(),
            ErrorKind::Limit
        );
    }
    let failure = database
        .run(
            query(
                dialect,
                "SELECT private_native_marker FROM contract_items",
                [],
            ),
            QueryLimits::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(failure.kind(), ErrorKind::Driver);
    assert!(failure.source().is_some());
    assert!(!format!("{failure:?} {failure}").contains("private_native_marker"));
    assert_eq!(integer(database, "SELECT 9").await?, 9);
    Ok(())
}
