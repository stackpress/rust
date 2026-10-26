//! M0 production-type proofs. Async scenarios run only under an external watchdog.

mod host;
mod transitions;
mod watchdog;

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use stackpress_sql::{
    Database, DialectId, Engine, ErrorKind, QueryLimits, Render, Select, Sql, SqlValue, Transaction,
};
use tempfile::TempDir;

use super::{SqliteDatabase, SqliteOptions};

/// Build caller-authored SQLite SQL without obscuring its dialect tag.
fn sql(text: &'static str) -> Sql {
    Sql::literal(DialectId::Sqlite, text)
}

/// Isolate every scenario's files and use short but finite acquire/busy budgets.
fn options(root: &TempDir, connections: u32) -> SqliteOptions {
    let mut options = SqliteOptions::file(root.path().join("proof.sqlite")).create_if_missing(true);
    options.max_connections = connections;
    options.acquire_timeout_ms = 80;
    options.busy_timeout_ms = 80;
    options
}

/// Open one independent file; installation is an explicit test action after eager open.
async fn engine(root: &TempDir, connections: u32) -> Engine {
    let engine = Engine::new(
        SqliteDatabase::open(options(root, connections))
            .await
            .unwrap(),
    );
    engine
        .execute(&sql(
            "CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE)",
        ))
        .await
        .unwrap();
    engine
}

/// Count persisted rows through a new normal lease, independently of transaction state.
async fn count(engine: &Engine) -> i64 {
    engine
        .fetch(&sql("SELECT count(*) AS n FROM profile"))
        .await
        .unwrap()[0]
        .try_get("n")
        .unwrap()
}

/// Demonstrate domain composition takes the explicit guard rather than a captured pool.
async fn insert(transaction: &mut Transaction, name: &str) {
    transaction
        .execute(&sql("INSERT INTO profile(name) VALUES (?)").bind(name))
        .await
        .unwrap();
}

/// Bound parameters, prepared arity, strict native values and file reopen use real SQLite.
async fn statements() {
    let root = TempDir::new().unwrap();
    let engine = engine(&root, 2).await;
    let hostile = "O'Reilly'; DROP TABLE profile; -- λ";
    let result = engine
        .execute(&sql("INSERT INTO profile(name) VALUES (?) RETURNING id").bind(hostile))
        .await
        .unwrap();
    assert_eq!(result.rows[0].try_get::<i64>("id").unwrap(), 1);
    assert_eq!(result.affected, 1);
    assert_eq!(result.insert_id, None);
    assert_eq!(
        engine
            .execute(&sql("SELECT 1 AS n"))
            .await
            .unwrap()
            .affected,
        0
    );
    assert_eq!(
        engine
            .execute(&sql("CREATE TABLE unrelated(id INTEGER)"))
            .await
            .unwrap()
            .affected,
        0
    );
    assert_eq!(
        engine
            .execute(&sql("EXPLAIN UPDATE profile SET name = 'not executed'"))
            .await
            .unwrap()
            .affected,
        0
    );
    let query = Select::new(["name"]).from("profile").where_eq("id", 1_i64);
    assert_eq!(
        engine.fetch(&query).await.unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        hostile
    );

    // Missing/excess binds are rejected before effects; SQLite must not insert NULL.
    for (query, expected, actual) in [
        (sql("INSERT INTO profile(name) VALUES (?)"), 1, 0),
        (
            sql("INSERT INTO profile(name) VALUES (?)")
                .bind("x")
                .bind("extra"),
            1,
            2,
        ),
        (sql("SELECT 1").bind("extra"), 0, 1),
    ] {
        assert_eq!(
            engine.execute(&query).await.unwrap_err().kind(),
            ErrorKind::Arity { expected, actual }
        );
    }
    assert_eq!(count(&engine).await, 1);
    let values = sql("-- ? before\nSELECT '?' AS literal, ? AS value /* ? */;").bind("bound");
    let rows = engine.fetch(&values).await.unwrap();
    assert_eq!(rows[0].try_get::<String>("literal").unwrap(), "?");
    assert_eq!(rows[0].try_get::<String>("value").unwrap(), "bound");
    let values = sql("SELECT ? AS n, ? AS b, ? AS r, ? AS empty, ? AS blob, ? AS null_value")
        .bind(i64::MIN)
        .bind(false)
        .bind(SqlValue::Real(2.5))
        .bind("")
        .bind(SqlValue::Blob(vec![0, 255]))
        .bind(SqlValue::Null);
    let rows = engine.fetch(&values).await.unwrap();
    assert_eq!(rows[0].try_get::<i64>("n").unwrap(), i64::MIN);
    assert!(!rows[0].try_get::<bool>("b").unwrap());
    assert_eq!(rows[0].cells()[2].1, SqlValue::Real(2.5));
    assert_eq!(rows[0].cells()[4].1, SqlValue::Blob(vec![0, 255]));
    assert_eq!(rows[0].cells()[5].1, SqlValue::Null);
    assert!(
        engine.fetch(&sql("SELECT 1 AS id, 2 AS id")).await.unwrap()[0]
            .try_get::<i64>("id")
            .is_err()
    );
    assert_eq!(
        engine
            .execute(&sql("UPDATE profile SET name = name WHERE id = 999"))
            .await
            .unwrap()
            .affected,
        0
    );
    assert_eq!(
        engine
            .execute(&sql("WITH p AS (SELECT 1) SELECT * FROM p"))
            .await
            .unwrap()
            .rows
            .len(),
        1
    );
    assert_eq!(
        engine
            .execute(&sql("SELECT 1e999 AS infinite"))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Decode
    );
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(count(&reopened).await, 1);
    reopened.close().await.unwrap();
}

/// Commit and every ordinary unwind/drop path preserve transaction affinity and rollback.
async fn transactions() {
    let root = TempDir::new().unwrap();
    let engine = Arc::new(engine(&root, 2).await);
    let mut tx = engine.begin().await.unwrap();
    insert(&mut tx, "committed").await;
    tokio::task::yield_now().await;
    assert_eq!(
        tx.fetch(&sql("SELECT name FROM profile"))
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(count(&engine).await, 0);
    tx.commit().await.unwrap();
    assert_eq!(count(&engine).await, 1);
    let mut tx = engine.begin().await.unwrap();
    insert(&mut tx, "explicit").await;
    tx.rollback().await.unwrap();
    let mut tx = engine.begin().await.unwrap();
    insert(&mut tx, "dropped").await;
    drop(tx);
    let mut tx = engine.begin().await.unwrap();
    insert(&mut tx, "error").await;
    let primary = tx
        .execute(&sql("INSERT INTO profile(name) VALUES (?)").bind("committed"))
        .await
        .unwrap_err();
    assert!(primary.source().is_some());
    assert_eq!(
        tx.execute(&sql("SELECT 1")).await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );
    assert_eq!(
        tx.commit().await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );

    // Ready/go channels establish ownership of an inserted but uncommitted guard.
    for panic in [false, true] {
        let (ready, entered) = tokio::sync::oneshot::channel();
        let (go, proceed) = tokio::sync::oneshot::channel();
        let shared = Arc::clone(&engine);
        let task = tokio::spawn(async move {
            let mut tx = shared.begin().await.unwrap();
            insert(&mut tx, if panic { "panic" } else { "abort" }).await;
            ready.send(()).unwrap();
            proceed.await.unwrap();
            panic!("intentional application unwind with owned transaction");
        });
        entered.await.unwrap();
        if panic {
            go.send(()).unwrap();
        } else {
            task.abort();
        }
        let error = task.await.unwrap_err();
        assert_eq!(error.is_panic(), panic);
    }
    assert_eq!(count(&engine).await, 1);
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(count(&reopened).await, 1);
    reopened.close().await.unwrap();
}

/// Actual waiter cancellation, bounded admission and close/re-await work under a held lease.
async fn pool() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 2;
    let database = SqliteDatabase::open(opts).await.unwrap();
    let engine = Arc::new(Engine::new(database));
    let tx = engine.begin().await.unwrap();
    let query = sql("SELECT 1");
    let mut waiter = Box::pin(engine.execute(&query));
    std::future::poll_fn(|cx| {
        assert!(waiter.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert_eq!(
        engine.execute(&query).await.unwrap_err().kind(),
        ErrorKind::Limit
    );
    drop(waiter);
    assert_eq!(
        engine.execute(&query).await.unwrap_err().kind(),
        ErrorKind::PoolTimedOut
    );
    let mut close = Box::pin(engine.close());
    std::future::poll_fn(|cx| {
        assert!(close.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert_eq!(
        engine.execute(&query).await.unwrap_err().kind(),
        ErrorKind::Closed
    );
    drop(close);
    drop(tx);
    engine.close().await.unwrap();
    engine.close().await.unwrap();
}

/// Retained output budgets error explicitly, and a subsequent lease remains usable.
async fn limits() {
    let root = TempDir::new().unwrap();
    let native = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let engine = Engine::new(native).with_limits(QueryLimits {
        rows: 1,
        bytes: 3,
        ..QueryLimits::default()
    });
    let rows = engine.fetch(&sql("SELECT 'ab' AS x")).await.unwrap();
    assert_eq!(rows[0].try_get::<String>("x").unwrap(), "ab");
    for query in [
        sql("SELECT 'abc' AS x"),
        sql("SELECT 'a' AS x UNION ALL SELECT 'b'"),
        sql("SELECT 'a' AS xxxx"),
    ] {
        assert_eq!(
            engine.execute(&query).await.unwrap_err().kind(),
            ErrorKind::Limit
        );
    }
    assert!(engine.execute(&sql("SELECT 'ok' AS x")).await.is_ok());
    engine.close().await.unwrap();
}

/// Configuration fails before open, native errors retain causes and public text is sanitized.
async fn configuration() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.create_if_missing = false;
    let error = SqliteDatabase::open(opts.clone()).await.err().unwrap();
    assert!(error.source().is_some());
    assert!(!error.to_string().contains("proof.sqlite"));
    assert!(!format!("{error:?}").contains("proof.sqlite"));
    for invalid in [0, 129] {
        opts.max_connections = invalid;
        assert_eq!(
            opts.validate().unwrap_err().kind(),
            ErrorKind::Configuration
        );
    }
    assert!(SqliteOptions::file("").validate().is_err());
    assert!(SqliteOptions::file(":memory:").validate().is_err());
    let good = options(&root, 1);
    let mut value = serde_json::to_value(&good).unwrap();
    value["extra"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SqliteOptions>(value).is_err());
    for field in ["acquire_timeout_ms", "busy_timeout_ms", "max_in_flight"] {
        let mut value = serde_json::to_value(&good).unwrap();
        value[field] = serde_json::json!(0);
        assert!(
            serde_json::from_value::<SqliteOptions>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }
    let db = SqliteDatabase::open(good).await.unwrap();
    let invalid_dialect = sql("SELECT 1").render(DialectId::Sqlite).unwrap();
    db.run(invalid_dialect, QueryLimits::default())
        .await
        .unwrap();
    db.close().await.unwrap();
}

/// SQLx opens with URI support, so the file API must reject URI memory/policy overrides.
#[test]
fn file_options_reject_native_uri_escape_hatches() {
    for path in [
        "file::memory:?cache=shared",
        "file:proof.sqlite?mode=memory",
        "bad\0path",
    ] {
        assert_eq!(
            SqliteOptions::file(path).validate().unwrap_err().kind(),
            ErrorKind::Configuration
        );
    }
}

/// Run each real-IO scenario under a process deadline that survives executor blocking.
#[test]
fn m0_under_process_watchdog() {
    for scenario in [
        "statements",
        "transactions",
        "pool",
        "limits",
        "configuration",
        "host",
        "transitions",
        "responsive",
        "failures",
        "admission",
    ] {
        let (success, expired, output) =
            watchdog::run("tests::m0_child", scenario, Duration::from_secs(10));
        assert!(
            success && !expired,
            "{scenario}: timeout={expired}\n{output}"
        );
    }
}

/// Only the watchdog parent invokes this test, with a fixed scenario name.
#[tokio::test(flavor = "current_thread")]
#[ignore = "launched only by external watchdog parent"]
async fn m0_child() {
    let scenario = std::env::var("STACKPRESS_SQL_M0_SCENARIO").unwrap();
    let work = async {
        match scenario.as_str() {
            "statements" => statements().await,
            "transactions" => transactions().await,
            "pool" => pool().await,
            "limits" => limits().await,
            "configuration" => configuration().await,
            "host" => host::proof().await,
            "transitions" => transitions::cancellation().await,
            "responsive" => transitions::responsive().await,
            "failures" => transitions::failures().await,
            "admission" => transitions::admission_cleanup().await,
            _ => panic!("unknown proof scenario"),
        }
    };
    tokio::time::timeout(Duration::from_secs(8), work)
        .await
        .expect("cooperative scenario deadline");
}

/// A non-yielding child demonstrates the parent can detect, kill and reap a deadlock.
#[test]
#[ignore = "deliberately stuck watchdog negative control"]
fn stuck_child() {
    loop {
        std::thread::park();
    }
}

/// Validate the watchdog itself against synchronous blocking, not just an async timeout.
#[test]
fn watchdog_kills_negative_control() {
    let (success, expired, _) =
        watchdog::run("tests::stuck_child", "stuck", Duration::from_millis(150));
    assert!(!success && expired);
}

/// Compile failures get an independent finite compilation deadline too.
#[test]
fn ownership_under_process_watchdog() {
    let (success, expired, output) =
        watchdog::run("tests::compile_child", "compile", Duration::from_secs(180));
    assert!(success && !expired, "compile: timeout={expired}\n{output}");
}

/// Use actual public Engine/Transaction types for every negative compiler proof.
#[test]
#[ignore = "launched only by external watchdog parent"]
fn compile_child() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}

/// Drop a real uncommitted transaction after its runtime exits, then reopen the file.
#[test]
#[ignore = "launched only by external watchdog parent"]
fn runtime_child() {
    let root = TempDir::new().unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let (engine, tx) = runtime.block_on(async {
        let engine = engine(&root, 1).await;
        let mut tx = engine.begin().await.unwrap();
        insert(&mut tx, "teardown").await;
        (engine, tx)
    });
    drop(runtime);
    drop(tx);
    drop(engine);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
        assert_eq!(count(&engine).await, 0);
        engine.close().await.unwrap();
    });
}

/// The outer deadline covers runtime shutdown and Drop outside a runtime, too.
#[test]
fn runtime_teardown_under_process_watchdog() {
    let (success, expired, output) =
        watchdog::run("tests::runtime_child", "teardown", Duration::from_secs(10));
    assert!(success && !expired, "teardown: timeout={expired}\n{output}");
}
