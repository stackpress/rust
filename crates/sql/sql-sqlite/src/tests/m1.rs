//! M1 native execution evidence; all real IO runs beneath an external process watchdog.

mod outcomes;
mod pool;
mod values;

use std::error::Error;
use std::time::Duration;

use stackpress_sql::{Engine, ErrorKind};
use tempfile::TempDir;

use super::{SqliteDatabase, engine, options, sql, watchdog};

/// Each insert reports only a native ID established by this statement, never a later SELECT.
async fn statement_ids() {
    let root = TempDir::new().unwrap();
    let engine = engine(&root, 1).await;
    let inserted = engine
        .execute(&sql("INSERT INTO profile(name) VALUES (?) RETURNING id").bind("first"))
        .await
        .unwrap();
    assert_eq!(inserted.insert_id, Some(1));
    assert_eq!(inserted.rows[0].try_get::<i64>("id").unwrap(), 1);
    let selected = engine
        .execute(&sql("SELECT id FROM profile"))
        .await
        .unwrap();
    assert_eq!(selected.insert_id, None);
    assert_eq!(selected.affected, 0);
    engine.close().await.unwrap();
}

/// Invalid SQLite text must return a typed source-preserving decode error, not empty rows.
async fn decoding_errors() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    let error = engine
        .fetch(&sql("SELECT CAST(X'80' AS TEXT) AS invalid_text"))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Decode);
    assert!(error.source().is_some());
    assert!(engine.fetch(&sql("SELECT 'healthy' AS x")).await.is_ok());
    engine.close().await.unwrap();
}

/// Parent enforces a real process deadline independently of executor or native progress.
#[test]
fn native_execution_under_watchdog() {
    for scenario in [
        "statement_ids",
        "decoding_errors",
        "crud",
        "metadata",
        "binding",
        "limits",
        "read_only",
        "pool_close",
        "policy",
    ] {
        let (success, expired, output) =
            watchdog::run("tests::m1::native_child", scenario, Duration::from_secs(10));
        assert!(
            success && !expired,
            "M1 {scenario}: timeout={expired}\n{output}"
        );
    }
}

/// This entrypoint is executed by the parent, never as an unbounded direct test.
#[tokio::test(flavor = "current_thread")]
#[ignore = "launched only by M1 watchdog parent"]
async fn native_child() {
    let scenario = std::env::var("STACKPRESS_SQL_M0_SCENARIO").unwrap();
    // The parent enforces a real deadline; controlled pool timers may advance Tokio time.
    match scenario.as_str() {
        "statement_ids" => statement_ids().await,
        "decoding_errors" => decoding_errors().await,
        "crud" => outcomes::crud().await,
        "metadata" => outcomes::metadata().await,
        "binding" => values::binding().await,
        "limits" => values::limits().await,
        "read_only" => pool::read_only().await,
        "pool_close" => pool::close().await,
        "policy" => pool::policy().await,
        _ => panic!("unknown M1 scenario"),
    }
}
