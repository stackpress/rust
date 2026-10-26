//! Offline configuration, strict budgets, and diagnostic ownership checks.

use super::*;

#[test]
/// Invalid options fail before IO and Debug never reveals connection fields.
fn options_are_finite_and_redacted() {
    let mut options = PostgresOptions::new(
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
    let mut encoded = serde_json::to_value(PostgresOptions::new("host", "db", "user", "")).unwrap();
    encoded["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<PostgresOptions>(encoded).is_err());
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
    let options: PostgresOptions = serde_json::from_value(serde_json::json!({
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
/// A custom/unresolved name cannot masquerade as a built-in PostgreSQL bind or NULL type.
fn postgres_type_identity_is_not_guessed_from_names() {
    use sqlx::postgres::{PgTypeInfo, types::Oid};
    assert_eq!(
        execute::scalar_type(&PgTypeInfo::with_oid(Oid(23))),
        Some("INT4")
    );
    assert_eq!(execute::scalar_type(&PgTypeInfo::with_name("INT4")), None);
    assert_eq!(execute::scalar_type(&PgTypeInfo::with_oid(Oid(1700))), None);
}
