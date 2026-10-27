//! Validated file options, native connection policy and close/waiter behavior.

use std::error::Error;
use std::sync::Arc;
use std::task::Poll;

use sqlx::Connection;
use stackpress_sql::{Engine, ErrorKind};
use tempfile::TempDir;

use crate::tests::{expire, options, sql};
use crate::{SqliteDatabase, SqliteOptions};

/// Validate each supported bound independently and preserve serialized M0 configuration.
#[test]
fn option_bounds_and_serialized_compatibility() {
    let defaults = SqliteOptions::file("existing.sqlite");
    assert!(!defaults.read_only);
    assert!(!defaults.create_if_missing);
    assert_eq!(defaults.max_connections, 4);
    assert_eq!(defaults.acquire_timeout_ms, 5000);
    assert_eq!(defaults.busy_timeout_ms, 1000);
    assert_eq!(defaults.max_in_flight, 128);
    for max in [1, 128] {
        let mut value = defaults.clone();
        value.max_connections = max;
        value.validate().unwrap();
    }
    for max in [0, 129] {
        let mut value = defaults.clone();
        value.max_connections = max;
        assert_eq!(
            value.validate().unwrap_err().kind(),
            ErrorKind::Configuration
        );
    }
    for field in ["acquire_timeout_ms", "busy_timeout_ms", "max_in_flight"] {
        let upper = if field == "max_in_flight" {
            4096
        } else {
            300_000
        };
        for number in [0, 1, upper, upper + 1] {
            let mut value = serde_json::to_value(&defaults).unwrap();
            value[field] = serde_json::json!(number);
            let parsed: SqliteOptions = serde_json::from_value(value).unwrap();
            assert_eq!(
                parsed.validate().is_ok(),
                number > 0 && number <= upper,
                "{field}={number}"
            );
        }
    }
    let mut legacy = serde_json::to_value(&defaults).unwrap();
    legacy.as_object_mut().unwrap().remove("read_only");
    assert!(
        !serde_json::from_value::<SqliteOptions>(legacy)
            .unwrap()
            .read_only
    );
    let readonly = defaults.clone().read_only(true);
    assert!(readonly.validate().is_ok());
    assert!(readonly.create_if_missing(true).validate().is_err());
    for path in ["", ":memory:", "file:existing.sqlite?mode=ro", "path\nname"] {
        assert!(SqliteOptions::file(path).validate().is_err());
    }
    let mut unknown = serde_json::to_value(&defaults).unwrap();
    unknown["unrecognized"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SqliteOptions>(unknown).is_err());
    // The native driver requires UTF-8; reject an invalid platform path before any worker starts.
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        assert!(
            SqliteOptions::file(std::ffi::OsString::from_vec(vec![0xff]))
                .validate()
                .is_err()
        );
    }
}

/// Stable typed classifications retain the original native cause without default disclosure.
#[test]
fn native_error_categories_keep_sources() {
    for (native, kind) in [
        (sqlx::Error::WorkerCrashed, ErrorKind::Closed),
        (sqlx::Error::PoolClosed, ErrorKind::Closed),
        (sqlx::Error::PoolTimedOut, ErrorKind::PoolTimedOut),
        (
            sqlx::Error::InvalidArgument("private-sentinel".into()),
            ErrorKind::InvalidQuery,
        ),
        (
            sqlx::Error::Encode(Box::new(std::io::Error::other("private-sentinel"))),
            ErrorKind::InvalidQuery,
        ),
        (
            sqlx::Error::Configuration(Box::new(std::io::Error::other("private-sentinel"))),
            ErrorKind::Configuration,
        ),
        (
            sqlx::Error::Decode(Box::new(std::io::Error::other("private-sentinel"))),
            ErrorKind::Decode,
        ),
        (
            sqlx::Error::ColumnDecode {
                index: "private-sentinel".into(),
                source: Box::new(std::io::Error::other("private-sentinel")),
            },
            ErrorKind::Decode,
        ),
        (
            sqlx::Error::ColumnNotFound("private-sentinel".into()),
            ErrorKind::Decode,
        ),
        (
            sqlx::Error::ColumnIndexOutOfBounds { index: 1, len: 0 },
            ErrorKind::Decode,
        ),
        (
            sqlx::Error::Protocol("private-sentinel".into()),
            ErrorKind::Driver,
        ),
    ] {
        let error = crate::execute::driver_error(native);
        assert_eq!(error.kind(), kind);
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<sqlx::Error>()
                .is_some()
        );
        assert!(!error.to_string().contains("private-sentinel"));
        assert!(!format!("{error:?}").contains("private-sentinel"));
    }
}

/// Existing files can be attached read-only; writes fail without fallback or data mutation.
pub(super) async fn read_only() {
    let root = TempDir::new().unwrap();
    let seeded = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    seeded
        .execute(&sql(
            "CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT)",
        ))
        .await
        .unwrap();
    seeded
        .execute(&sql("INSERT INTO profile VALUES (1, 'preserved')"))
        .await
        .unwrap();
    seeded.close().await.unwrap();
    let readonly = SqliteOptions::file(root.path().join("proof.sqlite")).read_only(true);
    let engine = Engine::new(SqliteDatabase::open(readonly).await.unwrap());
    assert_eq!(
        engine
            .fetch(&sql("SELECT name FROM profile"))
            .await
            .unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        "preserved"
    );
    let error = engine
        .execute(&sql("DELETE FROM profile"))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(error.source().is_some());
    assert_eq!(
        engine
            .fetch(&sql("SELECT count(*) AS n FROM profile"))
            .await
            .unwrap()[0]
            .try_get::<i64>("n")
            .unwrap(),
        1
    );
    let mut tx = engine.begin().await.unwrap();
    assert!(
        tx.execute(&sql("INSERT INTO profile VALUES (2, 'forbidden')"))
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
    engine.close().await.unwrap();
    assert!(
        SqliteDatabase::open(
            SqliteOptions::file(root.path().join("missing.sqlite")).read_only(true)
        )
        .await
        .is_err()
    );
    assert!(!root.path().join("missing.sqlite").exists());

    // A read-only attachment also works with an existing non-WAL file. The caller's
    // file mode must be preserved rather than changed by connection initialization.
    let delete_path = root.path().join("delete-mode.sqlite");
    let opts = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(&delete_path)
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Delete);
    let raw = sqlx::SqliteConnection::connect_with(&opts).await.unwrap();
    raw.close().await.unwrap();
    let db = SqliteDatabase::open(SqliteOptions::file(&delete_path).read_only(true))
        .await
        .unwrap();
    let mut connection = db.pool.acquire().await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
            .fetch_one(&mut *connection)
            .await
            .unwrap(),
        "delete"
    );
    drop(connection);
    stackpress_sql::Database::close(&db).await.unwrap();
}

/// Close wakes an already queued waiter while waiting for an existing transaction to finish.
pub(super) async fn close() {
    let root = TempDir::new().unwrap();
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(options(&root, 1)).await.unwrap(),
    ));
    let tx = engine.begin().await.unwrap();
    let query = sql("SELECT 1 AS n");
    let mut waiting = Box::pin(engine.execute(&query));
    std::future::poll_fn(|cx| {
        assert!(waiting.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    let mut closing = Box::pin(engine.close());
    std::future::poll_fn(|cx| {
        assert!(closing.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    // Closing was explicitly polled above, so closure—not a one-second race—wakes this waiter.
    assert_eq!(waiting.await.unwrap_err().kind(), ErrorKind::Closed);
    drop(closing);
    // Existing guards may still complete. A cancelled close observation neither reopens
    // the pool nor prevents re-awaiting shutdown once the guard releases its connection.
    tx.rollback().await.unwrap();
    engine.close().await.unwrap();
    engine.close().await.unwrap();
    assert_eq!(
        engine.execute(&query).await.unwrap_err().kind(),
        ErrorKind::Closed
    );
    assert_eq!(
        engine.begin().await.err().unwrap().kind(),
        ErrorKind::Closed
    );
}

/// Policy is applied on every actual connection, and native pool capacity has a finite bound.
pub(super) async fn policy() {
    let root = TempDir::new().unwrap();
    let opts = options(&root, 2);
    let timeout = opts.acquire_timeout();
    let db = SqliteDatabase::open(opts).await.unwrap();
    let mut first = db.pool.acquire().await.unwrap();
    let mut second = db.pool.acquire().await.unwrap();
    assert_eq!(db.pool.size(), 2);
    for connection in [&mut first, &mut second] {
        assert_eq!(
            sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
                .fetch_one(&mut **connection)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
                .fetch_one(&mut **connection)
                .await
                .unwrap(),
            "wal"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("PRAGMA synchronous")
                .fetch_one(&mut **connection)
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("PRAGMA busy_timeout")
                .fetch_one(&mut **connection)
                .await
                .unwrap(),
            80
        );
    }
    // Both native connections remain owned until the actual acquisition deadline fires.
    let error = expire(db.pool.acquire(), timeout).await.err().unwrap();
    assert!(matches!(error, sqlx::Error::PoolTimedOut));
    drop(first);
    drop(second);
    stackpress_sql::Database::close(&db).await.unwrap();
    assert_eq!(db.pool.size(), 0);
    assert_eq!(db.admission.available_permits(), 128);
}
