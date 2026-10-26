//! Offline configuration, strict budgets, and diagnostic ownership checks.

use super::*;
use stackpress_sql::{DialectId, ErrorKind, QueryLimits, Result, SqlError};

#[test]
/// Invalid options fail before IO and Debug never reveals connection fields.
fn options_are_finite_and_redacted() {
    let mut options = MysqlOptions::new(
        "secret-host",
        "secret-database",
        "secret-user",
        "secret-password",
    );
    options.validate().unwrap();
    let debug = format!("{options:?}");
    for secret in [
        "secret-host",
        "secret-database",
        "secret-user",
        "secret-password",
    ] {
        assert!(!debug.contains(secret));
    }
    options.max_connections = 0;
    assert_eq!(
        options.validate().unwrap_err().kind(),
        ErrorKind::Configuration
    );
    options.max_connections = 1;
    options.statement_timeout_ms = 0;
    assert_eq!(
        options.validate().unwrap_err().kind(),
        ErrorKind::Configuration
    );
    let mut encoded = serde_json::to_value(MysqlOptions::new("host", "db", "user", "")).unwrap();
    encoded["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<MysqlOptions>(encoded).is_err());
}

#[test]
/// A retained-value overrun leaves no wrapped arithmetic or partial reservation.
fn output_budget_boundaries() {
    let mut budget = collect::OutputBudget::new(QueryLimits {
        bytes: 4,
        ..QueryLimits::default()
    });
    budget.charge(4).unwrap();
    assert_eq!(budget.charge(1).unwrap_err().kind(), ErrorKind::Limit);
    assert_eq!(
        budget.charge(usize::MAX).unwrap_err().kind(),
        ErrorKind::Limit
    );
    budget.charge(0).unwrap();
}

#[test]
/// Native causes and cleanup errors remain distinct from sanitized public messages.
fn error_causes_are_preserved() {
    use std::error::Error;
    let primary = execute::driver_error(sqlx::Error::Protocol("secret native SQL".into()));
    assert!(!format!("{primary:?} {primary}").contains("secret native SQL"));
    assert!(
        primary
            .source()
            .unwrap()
            .downcast_ref::<sqlx::Error>()
            .is_some()
    );
    let result: Result<()> = lease::completed(
        Err(primary),
        Err(SqlError::new(ErrorKind::Closed, "cleanup failed")),
    );
    assert_eq!(result.as_ref().unwrap_err().kind(), ErrorKind::Driver);
    assert_eq!(
        result.unwrap_err().cleanup_error().unwrap().kind(),
        ErrorKind::Closed
    );
}

#[test]
/// Minimal serialized config applies finite budgets and preserves a caller-supplied secret.
fn serialized_defaults_are_finite() {
    let options: MysqlOptions = serde_json::from_value(serde_json::json!({
        "host": "localhost", "database": "app", "user": "app", "password": ""
    }))
    .unwrap();
    options.validate().unwrap();
    assert_eq!(options.max_connections, 4);
    assert_eq!(options.max_in_flight, 128);
    assert_eq!(options.acquire_timeout_ms, 5000);
    assert_eq!(options.statement_timeout_ms, 30000);
    assert_eq!(options.cleanup_timeout_ms, 5000);
}

#[test]
/// Token-aware transaction protection skips quoted/comment text and rejects hidden DDL.
fn transaction_gate_respects_lexical_boundaries() {
    use stackpress_sql::{Render, Sql};
    for sql in [
        "/* comment */ CREATE TABLE x (id BIGINT)",
        "EXPLAIN /* comment */ DROP TABLE x",
        "WITH x AS (SELECT 1) ALTER TABLE y ADD z BIGINT",
    ] {
        let query = Sql::literal(DialectId::Mysql, sql)
            .render(DialectId::Mysql)
            .unwrap();
        assert_eq!(
            execute::validate_transaction(&query).unwrap_err().kind(),
            ErrorKind::Unsupported
        );
    }
    for sql in [
        "SELECT 'CREATE' AS command",
        "/* DROP */ SELECT 1",
        "SELECT `ALTER` FROM items",
        "SELECT 'it''s DROP'",
    ] {
        let query = Sql::literal(DialectId::Mysql, sql)
            .render(DialectId::Mysql)
            .unwrap();
        execute::validate_transaction(&query).unwrap();
    }
}

#[test]
/// JSON decoding preserves SQL/JSON null separation, deterministic objects and numeric digits.
fn json_is_bounded_and_lossless() {
    use serde_json::value::RawValue;
    let raw = RawValue::from_string("{\"z\": 123456789012345678901234567890, \"a\":null}".into())
        .unwrap();
    assert_eq!(
        crate::json::canonical(&raw, 1000).unwrap(),
        "{\"a\":null,\"z\":123456789012345678901234567890}"
    );
    assert_eq!(
        crate::json::canonical(&raw, 1).unwrap_err().kind(),
        ErrorKind::Limit
    );
    let null = RawValue::from_string("null".into()).unwrap();
    assert_eq!(crate::json::canonical(&null, 4).unwrap(), "null");
    assert!(RawValue::from_string("invalid".into()).is_err());
}

#[test]
/// Reuse candidates exclude stateful/ambiguous syntax while ordinary DML remains reusable.
fn mysql_reuse_policy_is_conservative() {
    for sql in [
        "SELECT ? AS value",
        "/* lead */ SELECT COUNT(*) FROM items",
        "INSERT INTO items(id) VALUES (?)",
        "INSERT INTO `schema`.`items` (`id`) VALUES (?)",
        "UPDATE items SET id = ? WHERE id = ?",
        "DELETE FROM items WHERE id = ?",
        "SELECT 'GET_LOCK(@x)'",
        "SELECT 1; /* tail */",
    ] {
        assert!(crate::reuse::allowed_sql(sql), "must reuse: {sql}");
    }
    for sql in [
        "SET @x=1",
        "SELECT @x",
        "CREATE TEMPORARY TABLE t(id INT)",
        "WITH t AS (SELECT 1) SELECT * FROM t",
        "EXPLAIN SELECT 1",
        "SELECT GET_LOCK('x',0)",
        "SELECT GET_LOCK -- comment\n ('x',0)",
        "SELECT RELEASE_LOCK/* comment */('x')",
        "SELECT schema. /* comment */ ABS(1)",
        "SELECT `ABS`(1)",
        "SELECT unknown_function()",
        "LOCK TABLES t WRITE",
        "HANDLER t OPEN",
        "PREPARE s FROM 'SELECT 1'",
        "SELECT 1 INTO OUTFILE 'x'",
        "SELECT SQL_CALC_FOUND_ROWS * FROM t",
        "SELECT 1; SELECT 2",
        "SELECT 'unclosed",
        "SELECT /* unclosed",
        "SELECT /*! native */ 1",
    ] {
        assert!(!crate::reuse::allowed_sql(sql), "must retire: {sql}");
    }
}

#[test]
/// Duplicate keys cannot disappear during canonicalization, including nested objects.
fn duplicate_json_keys_are_rejected() {
    for source in [r#"{"a":1,"a":2}"#, r#"{"nested":{"a":1,"a":2}}"#] {
        let raw = serde_json::value::RawValue::from_string(source.into()).unwrap();
        assert_eq!(
            crate::json::canonical(&raw, 1000).unwrap_err().kind(),
            ErrorKind::Decode
        );
    }
}

#[test]
/// Raw traversal preserves huge/exponent tokens and JSON strings without global number features.
fn json_raw_token_and_depth_boundaries() {
    use serde_json::value::RawValue;
    let raw = RawValue::from_string(
        r#"{"z":[1e999,123456789012345678901234567890],"a":"\u0061"}"#.into(),
    )
    .unwrap();
    assert_eq!(
        crate::json::canonical(&raw, 1000).unwrap(),
        r#"{"a":"a","z":[1e999,123456789012345678901234567890]}"#
    );
    let duplicate = RawValue::from_string(r#"[{"a":1,"\u0061":2}]"#.into()).unwrap();
    assert_eq!(
        crate::json::canonical(&duplicate, 1000).unwrap_err().kind(),
        ErrorKind::Decode
    );
    let nested = format!("{}0{}", "[".repeat(128), "]".repeat(128));
    let raw = RawValue::from_string(nested).unwrap();
    assert_eq!(
        crate::json::canonical(&raw, 1000).unwrap_err().kind(),
        ErrorKind::Decode
    );
    let allowed = format!("{}0{}", "[".repeat(127), "]".repeat(127));
    let raw = RawValue::from_string(allowed.clone()).unwrap();
    assert_eq!(crate::json::canonical(&raw, 1000).unwrap(), allowed);
}
