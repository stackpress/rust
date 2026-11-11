//! Pure adapter configuration, error-classification and query-boundary proofs.
use crate::{PostgresOptions as Options, PostgresTls as Tls, collect, execute};
use stackpress_sql::{DialectId, ErrorKind, QueryLimits, Render, Sql, SqlValue};
use std::error::Error;
use std::io;
use std::time::Duration;

/// Every native error category retains its concrete cause behind sanitized public context.
#[test]
fn native_error_mapping_preserves_category_and_cause() {
    let cause =
        || Box::new(io::Error::other("PRIVATE_NATIVE_CAUSE")) as Box<dyn Error + Send + Sync>;
    for (native, expected) in [
        (sqlx::Error::PoolTimedOut, ErrorKind::PoolTimedOut),
        (sqlx::Error::PoolClosed, ErrorKind::Closed),
        (sqlx::Error::WorkerCrashed, ErrorKind::Closed),
        (
            sqlx::Error::ColumnDecode {
                index: "PRIVATE_NATIVE_CAUSE".into(),
                source: cause(),
            },
            ErrorKind::Decode,
        ),
        (sqlx::Error::Decode(cause()), ErrorKind::Decode),
        (
            sqlx::Error::ColumnIndexOutOfBounds { index: 1, len: 0 },
            ErrorKind::Decode,
        ),
        (
            sqlx::Error::ColumnNotFound("PRIVATE_NATIVE_CAUSE".into()),
            ErrorKind::Decode,
        ),
        (
            sqlx::Error::Configuration(cause()),
            ErrorKind::Configuration,
        ),
        (sqlx::Error::Encode(cause()), ErrorKind::InvalidQuery),
        (
            sqlx::Error::InvalidArgument("PRIVATE_NATIVE_CAUSE".into()),
            ErrorKind::InvalidQuery,
        ),
        (
            sqlx::Error::Io(io::Error::other("PRIVATE_NATIVE_CAUSE")),
            ErrorKind::Driver,
        ),
    ] {
        let original = std::mem::discriminant(&native);
        let error = execute::driver_error(native);
        assert_eq!(error.kind(), expected);
        let native = error
            .source()
            .unwrap()
            .downcast_ref::<sqlx::Error>()
            .unwrap();
        assert_eq!(std::mem::discriminant(native), original);
        assert!(!format!("{error} {error:?}").contains("PRIVATE_NATIVE_CAUSE"));
    }
}

/// Serialized invalid fields are refused before native construction, including exact upper bounds.
#[test]
fn connection_fields_and_every_finite_budget_are_validated() {
    let valid =
        serde_json::to_value(Options::new("db.example.test", "application", "app", "")).unwrap();
    for (field, bad) in [
        ("host", serde_json::json!("")),
        ("host", serde_json::json!("/socket")),
        ("host", serde_json::json!("postgres://localhost")),
        ("host", serde_json::json!("host\n")),
        ("port", serde_json::json!(0)),
        ("database", serde_json::json!("")),
        ("user", serde_json::json!("user\n")),
        ("password", serde_json::json!("secret\0")),
        ("max_connections", serde_json::json!(129)),
        ("max_in_flight", serde_json::json!(0)),
        ("max_in_flight", serde_json::json!(4097)),
        ("acquire_timeout_ms", serde_json::json!(0)),
        ("acquire_timeout_ms", serde_json::json!(300001)),
        ("statement_timeout_ms", serde_json::json!(300001)),
        ("cleanup_timeout_ms", serde_json::json!(0)),
        ("cleanup_timeout_ms", serde_json::json!(300001)),
    ] {
        let mut value = valid.clone();
        value[field] = bad;
        let options: Options = serde_json::from_value(value).unwrap();
        assert_eq!(
            options.validate().unwrap_err().kind(),
            ErrorKind::Configuration,
            "{field}"
        );
    }
    let mut options = Options::new("db.example.test", "application", "app", "");
    options.max_connections = 128;
    options.max_in_flight = 4096;
    options.acquire_timeout_ms = 300000;
    options.statement_timeout_ms = 300000;
    options.cleanup_timeout_ms = 300000;
    options.validate().unwrap();
    assert_eq!(options.acquire_timeout(), Duration::from_millis(300000));
    assert_eq!(options.statement_timeout(), Duration::from_millis(300000));
    assert_eq!(options.cleanup_timeout(), Duration::from_millis(300000));
    for path in ["", "bad\npath"] {
        options.tls = Tls::CertificateAuthority { path: path.into() };
        assert_eq!(
            options.validate().unwrap_err().kind(),
            ErrorKind::Configuration
        );
    }
}

/// Explicit TLS policy maps to native options without opening sockets or reading certificate files.
#[test]
fn native_options_keep_explicit_identity_and_tls_policy() {
    let mut options = Options::new("db.example.test", "application", "app", "secret");
    for (tls, expected) in [
        (Tls::VerifyFull, "VerifyFull"),
        (
            Tls::CertificateAuthority {
                path: "nonexistent-ca.pem".into(),
            },
            "VerifyFull",
        ),
        (Tls::Plaintext, "Disable"),
    ] {
        options.tls = tls;
        options.validate().unwrap();
        let native = options.connect_options().unwrap();
        assert_eq!(native.get_host(), "db.example.test");
        assert_eq!(native.get_database(), Some("application"));
        assert_eq!(native.get_username(), "app");
        assert!(native.get_socket().is_none());
        assert_eq!(format!("{:?}", native.get_ssl_mode()), expected);
    }
}

/// Retained real values reject nonfinite numbers and charge the finite scalar exactly once.
#[test]
fn native_reals_keep_finite_values_and_budget_boundaries() {
    let mut budget = collect::OutputBudget::new(QueryLimits {
        bytes: 8,
        ..Default::default()
    });
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(budget.real(value).unwrap_err().kind(), ErrorKind::Decode);
    }
    assert_eq!(budget.real(-0.5).unwrap(), SqlValue::Real(-0.5));
    assert_eq!(budget.real(0.0).unwrap_err().kind(), ErrorKind::Limit);
}

/// Validation keeps data in binds, refuses other dialects and applies SQL budgets before IO.
#[test]
fn rendered_query_and_validation_preserve_the_driver_boundary() {
    let dialect = DialectId::Postgres;
    let sql = "SELECT $1::TEXT AS value";
    let query = Sql::literal(dialect, sql)
        .bind("quoted ' value")
        .render(dialect)
        .unwrap();
    assert_eq!(query.sql(), sql);
    assert_eq!(query.binds(), &[SqlValue::Text("quoted ' value".into())]);
    execute::validate(&query, QueryLimits::default()).unwrap();
    assert_eq!(
        execute::validate(
            &query,
            QueryLimits {
                sql_bytes: 1,
                ..Default::default()
            }
        )
        .unwrap_err()
        .kind(),
        ErrorKind::Limit
    );
    let other = Sql::literal(DialectId::Sqlite, "SELECT 1")
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(
        execute::validate(&other, QueryLimits::default())
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
}
