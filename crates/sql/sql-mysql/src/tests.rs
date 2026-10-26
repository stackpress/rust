//! Offline configuration, strict budgets, and diagnostic ownership checks.

use super::*;

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
