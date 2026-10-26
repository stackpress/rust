//! Pure rendering, input validation, strict decoding and detached-data contracts.

use stackpress_sql::{
    DialectId, ErrorKind, FromSqlValue, QueryLimits, Render, Row, Select, Sql, SqlValue, TrustedSql,
};

/// Implicit Debug/Display must not disclose a native cause; source access stays explicit.
#[test]
fn errors_preserve_sources_without_implicit_disclosure() {
    let error = stackpress_sql::SqlError::with_source(
        ErrorKind::Driver,
        "SQL failed",
        std::io::Error::other("private-db-sentinel"),
    );
    assert_eq!(error.to_string(), "SQL failed");
    assert!(!format!("{error:?}").contains("private-db-sentinel"));
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .to_string()
            .contains("private-db-sentinel")
    );
}

/// Quote all identifier text while retaining malicious-looking values only in binds.
#[test]
fn detached_builder_separates_identifiers_and_values() {
    let base = Select::new(["na\"me"]).from("profile.table");
    let selected = base.clone().where_eq("id", "'; DROP TABLE profile; --");
    let query = selected.render(DialectId::Sqlite).unwrap();
    assert_eq!(
        query.sql(),
        "SELECT \"na\"\"me\" FROM \"profile.table\" WHERE \"id\" = ?"
    );
    assert_eq!(
        query.binds(),
        &[SqlValue::Text("'; DROP TABLE profile; --".into())]
    );
    assert!(base.render(DialectId::Sqlite).unwrap().binds().is_empty());
    assert!(query.render(DialectId::Postgres).is_err());
    assert!(base.render(DialectId::Mysql).is_ok());
    for name in ["", "a\0b", "a\nb"] {
        assert!(
            Select::new([name])
                .from("p")
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    assert!(
        Select::new(Vec::<String>::new())
            .from("p")
            .render(DialectId::Sqlite)
            .is_err()
    );
    assert!(Select::new(["id"]).render(DialectId::Sqlite).is_err());
}

/// Exercise quote/comment states rather than treating punctuation globally.
#[test]
fn lexical_gate_distinguishes_code_from_quoted_text() {
    for text in [
        "-- before\nSELECT '?;:x', \"?\", `?`, [?], 'it''s ?'; -- after",
        "/* before ? */ SELECT ? /* ; */; /* tail */",
        "WITH p AS (SELECT 1) SELECT * FROM p",
        "SELECT \"a\"\"?b\", `a``?b`",
    ] {
        assert!(
            Sql::trusted(DialectId::Sqlite, TrustedSql::audited(text))
                .render(DialectId::Sqlite)
                .is_ok(),
            "{text}"
        );
    }
    for text in [
        "",
        ";",
        "SELECT 1; SELECT 2",
        "SELECT 1;;",
        "SELECT '\0'",
        "SELECT 'unfinished",
        "SELECT \"unfinished",
        "SELECT `unfinished",
        "SELECT [unfinished",
        "SELECT 1 /* unfinished",
        "BEGIN",
        "COMMIT",
        "ROLLBACK",
        "SAVEPOINT a",
        "END",
        "PRAGMA foreign_keys=OFF",
        "ATTACH 'x' AS p",
        "SELECT ?1",
        "SELECT :name",
        "SELECT @name",
        "SELECT $name",
    ] {
        assert!(
            Sql::trusted(DialectId::Sqlite, TrustedSql::audited(text))
                .render(DialectId::Sqlite)
                .is_err(),
            "{text}"
        );
    }
    assert!(
        Sql::literal(DialectId::Postgres, "SELECT 1")
            .render(DialectId::Postgres)
            .is_ok()
    );
}

/// Enforce exact boundary values for SQL, binds and owned byte budgets.
#[test]
fn finite_input_budgets_and_real_values() {
    let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
        .bind("abc")
        .render(DialectId::Sqlite)
        .unwrap();
    let exact = QueryLimits {
        sql_bytes: 8,
        binds: 1,
        rows: 1,
        bytes: 3,
    };
    query.validate(exact).unwrap();
    for limits in [
        QueryLimits {
            sql_bytes: 7,
            ..exact
        },
        QueryLimits { binds: 0, ..exact },
        QueryLimits { bytes: 2, ..exact },
    ] {
        assert_eq!(query.validate(limits).unwrap_err().kind(), ErrorKind::Limit);
    }
    for real in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(
            Sql::literal(DialectId::Sqlite, "SELECT ?")
                .bind(SqlValue::Real(real))
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    assert!(
        Sql::literal(DialectId::Sqlite, "SELECT ?")
            .bind(SqlValue::Real(0.0))
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert_eq!(SqlValue::Blob(vec![1, 2]).retained_bytes(), 2);
    assert_eq!(SqlValue::Null.retained_bytes(), 0);
    assert_eq!(SqlValue::Bool(false).retained_bytes(), 1);
    assert_eq!(SqlValue::Integer(i64::MAX).retained_bytes(), 8);
}

/// Preserve duplicate columns, NULL, false and strict conversion failures.
#[test]
fn ordered_owned_rows_do_not_coerce_or_overwrite() {
    let row = Row::new(vec![
        ("name".into(), "Ada".into()),
        ("id".into(), 7_i64.into()),
    ]);
    assert_eq!(row.try_get::<String>("name").unwrap(), "Ada");
    assert_eq!(row.try_get::<i64>("id").unwrap(), 7);
    assert!(row.try_get::<String>("id").is_err());
    assert!(row.try_get::<i64>("name").is_err());
    assert!(row.try_get::<i64>("missing").is_err());
    let duplicate = Row::new(vec![
        ("id".into(), 1_i64.into()),
        ("id".into(), 2_i64.into()),
    ]);
    assert!(duplicate.try_get::<i64>("id").is_err());
    assert_eq!(duplicate.cells().len(), 2);
    assert!(!bool::from_sql(&SqlValue::Integer(0)).unwrap());
    assert!(bool::from_sql(&SqlValue::Integer(1)).unwrap());
    assert!(!bool::from_sql(&SqlValue::Bool(false)).unwrap());
    for bad in [
        SqlValue::Null,
        SqlValue::Integer(2),
        SqlValue::Text("false".into()),
    ] {
        assert!(bool::from_sql(&bad).is_err());
    }
}
