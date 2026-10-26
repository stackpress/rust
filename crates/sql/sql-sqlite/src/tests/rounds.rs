//! Safety follow-ups: real future suspension cuts, queued cancellation and runtime teardown.

use std::error::Error;
use std::future::{Future, poll_fn};
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;

use sqlx::Executor;
use stackpress_sql::{
    Create, DialectId, Engine, ErrorKind, Field, Index, QueryResult, SqlType, Transaction,
};
use tokio::sync::Semaphore;

use super::{SqliteDatabase, SqliteOptions, TempDir, count, options, sql, watchdog};

/// Drop a real Send future at its nth Pending response, or retain its complete result.
/// This sweeps observed suspension points; the native hook tests identify exact worker phases.
async fn cut<F: Future + Send>(future: F, cutoff: usize) -> Option<F::Output> {
    let mut work = Box::pin(future);
    let mut pending = 0;
    let result = poll_fn(|cx| match work.as_mut().poll(cx) {
        Poll::Ready(result) => Poll::Ready(Some(result)),
        Poll::Pending => {
            pending += 1;
            if pending == cutoff {
                Poll::Ready(None)
            } else {
                Poll::Pending
            }
        }
    })
    .await;
    // Drop before returning to the caller, including the future's owned transaction/session.
    drop(work);
    result
}

/// Wait for acknowledged lease retirement before inspecting persisted outcomes.
async fn settled(admission: &Arc<Semaphore>) {
    drop(admission.acquire().await.unwrap());
}

/// One lease/permit makes leaked or prematurely reused work observable on the next operation.
async fn fixture(root: &TempDir, permits: usize) -> (Engine, Arc<Semaphore>) {
    let mut opts = options(root, 1);
    opts.max_in_flight = permits;
    opts.acquire_timeout_ms = 1000;
    let db = SqliteDatabase::open(opts).await.unwrap();
    let admission = Arc::clone(&db.admission);
    let engine = Engine::new(db);
    engine
        .execute(&sql(
            "CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT)",
        ))
        .await
        .unwrap();
    (engine, admission)
}

/// Cut begin/run/completion/plan futures at successive real suspensions without injecting IO.
async fn suspensions(phase: &str) {
    let root = TempDir::new().unwrap();
    let (engine, admission) = fixture(&root, 1).await;
    let plan = Create::new("batch")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("batch_index", ["id"]))
        .render(DialectId::Sqlite)
        .unwrap();
    let mut completed = 0;
    let mut cancelled = 0;
    // Keep the sweep finite even if a driver regression produces excessive wakeups.
    for cutoff in 1..=64 {
        let finished = match phase {
            "begin" => match cut(engine.begin(), cutoff).await {
                Some(result) => {
                    result.unwrap().rollback().await.unwrap();
                    true
                }
                None => false,
            },
            "run" => {
                let mut tx = engine.begin().await.unwrap();
                tx.execute(&sql("INSERT INTO profile(name) VALUES ('prior write')"))
                    .await
                    .unwrap();
                let query = sql("INSERT INTO profile(name) VALUES ('cancelled run') RETURNING id");
                match cut(tx.execute(&query), cutoff).await {
                    Some(result) => {
                        assert_eq!(result.unwrap().affected, 1);
                        tx.rollback().await.unwrap();
                        true
                    }
                    None => {
                        // Every internal await occurs after run poisoned the session. Even
                        // cancellation during trailing counter IO cannot permit a later commit.
                        assert_eq!(
                            tx.commit().await.unwrap_err().kind(),
                            ErrorKind::RollbackOnly
                        );
                        false
                    }
                }
            }
            "commit" | "rollback" => {
                let mut tx = engine.begin().await.unwrap();
                tx.execute(&sql("INSERT INTO profile(name) VALUES ('left'), ('right')"))
                    .await
                    .unwrap();
                let result = if phase == "commit" {
                    cut(tx.commit(), cutoff).await
                } else {
                    cut(tx.rollback(), cutoff).await
                };
                if let Some(result) = &result {
                    assert!(result.is_ok(), "{result:?}");
                }
                result.is_some()
            }
            "plan" => {
                let result = cut(engine.execute_plan(&plan), cutoff).await;
                if let Some(result) = &result {
                    assert_eq!(result.as_ref().unwrap().len(), 2);
                }
                result.is_some()
            }
            "transaction_plan" => {
                let mut tx = engine.begin().await.unwrap();
                let result = cut(tx.execute_plan(&plan), cutoff).await;
                if let Some(result) = &result {
                    assert_eq!(result.as_ref().unwrap().len(), 2);
                    tx.rollback().await.unwrap();
                } else {
                    // Cancelling a borrowed plan leaves caller ownership and a poisoned
                    // active session, so consuming commit must retire rather than commit it.
                    assert_eq!(
                        tx.commit().await.unwrap_err().kind(),
                        ErrorKind::RollbackOnly
                    );
                }
                result.is_some()
            }
            _ => panic!("unknown suspension phase"),
        };
        settled(&admission).await;
        if matches!(phase, "plan" | "transaction_plan") {
            let rows = engine
                .fetch(&sql(
                    "SELECT name FROM sqlite_master WHERE tbl_name = 'batch'",
                ))
                .await
                .unwrap();
            // Lost commit observation may leave all effects; partial DDL is never allowed.
            assert!(rows.is_empty() || rows.len() == 2);
            if phase == "transaction_plan" {
                assert!(rows.is_empty());
            } else if finished {
                assert_eq!(rows.len(), 2);
            }
            engine
                .execute(&sql("DROP TABLE IF EXISTS batch"))
                .await
                .unwrap();
        } else {
            let rows = count(&engine).await;
            if phase == "commit" {
                assert!(rows == 0 || rows == 2);
                if finished {
                    assert_eq!(rows, 2);
                }
                engine.execute(&sql("DELETE FROM profile")).await.unwrap();
            } else {
                assert_eq!(rows, 0);
            }
        }
        completed += usize::from(finished);
        cancelled += usize::from(!finished);
    }
    eprintln!("{phase}: {cancelled} cancellation cuts, {completed} acknowledged completions");
    assert!(
        completed > 0,
        "no completion within the finite suspension sweep"
    );
    assert!(
        cancelled > 0,
        "suspension sweep did not exercise cancellation"
    );
    engine.close().await.unwrap();
    // Reopening and writing proves retirement left neither an orphan writer nor partial data.
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(count(&reopened).await, 0);
    reopened
        .execute(&sql("INSERT INTO profile(name) VALUES ('healthy')"))
        .await
        .unwrap();
    reopened.close().await.unwrap();
}

/// Cancel after admission but before a connection is available; release exactly the waiter slot.
async fn cancel_waiter<F: Future + Send>(future: F, admission: &Semaphore) {
    let mut work = Box::pin(future);
    poll_fn(|cx| {
        assert!(work.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    assert_eq!(admission.available_permits(), 0);
    drop(work);
    assert_eq!(admission.available_permits(), 1);
}

/// Public begin/run/plan acquisition futures all release admission when their waiter is dropped.
async fn acquisition() {
    let root = TempDir::new().unwrap();
    let (engine, admission) = fixture(&root, 2).await;
    let tx = engine.begin().await.unwrap();
    let query = sql("SELECT 1 AS n");
    let plan = Create::new("never_created")
        .add_field(Field::new("id", SqlType::Integer))
        .render(DialectId::Sqlite)
        .unwrap();
    cancel_waiter(engine.begin(), &admission).await;
    cancel_waiter(engine.execute(&query), &admission).await;
    cancel_waiter(engine.execute_plan(&plan), &admission).await;
    tx.rollback().await.unwrap();
    drop(admission.acquire_many(2).await.unwrap());
    assert!(
        engine
            .fetch(&sql(
                "SELECT name FROM sqlite_master WHERE name = 'never_created'"
            ))
            .await
            .unwrap()
            .is_empty()
    );
    engine.close().await.unwrap();
}

/// Real native sources remain available to diagnostics while default error text hides SQL names.
async fn disclosure() {
    let root = TempDir::new().unwrap();
    let (engine, _) = fixture(&root, 1).await;
    let error = engine
        .execute(&sql("SELECT * FROM private_sql_sentinel"))
        .await
        .unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("private_sql_sentinel")
    );
    for rendered in [error.to_string(), format!("{error:?}")] {
        assert!(!rendered.contains("private_sql_sentinel"));
        assert!(!rendered.contains(root.path().to_str().unwrap()));
        assert!(!rendered.contains("SELECT"));
    }
    engine.close().await.unwrap();
}

/// Abort the public Engine plan while its failure branch is awaiting explicit rollback.
async fn failed_plan_cleanup() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 1;
    let db = SqliteDatabase::open(opts).await.unwrap();
    let admission = Arc::clone(&db.admission);
    let mut connection = db.pool.acquire().await.unwrap();
    connection
        .execute("CREATE TABLE original(id INTEGER)")
        .await
        .unwrap();
    connection
        .execute("CREATE INDEX occupied ON original(id)")
        .await
        .unwrap();
    drop(connection);
    let (entered, release) =
        super::transitions::gate(&db, super::transitions::Hook::Rollback).await;
    let engine = Arc::new(Engine::new(db));
    let before = super::m2::schema(&engine).await;
    let plan = Create::new("abandoned")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("occupied", ["id"]))
        .render(DialectId::Sqlite)
        .unwrap();
    let shared = Arc::clone(&engine);
    let task = tokio::spawn(async move { shared.execute_plan(&plan).await });
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(admission.available_permits(), 0);
    release.send(()).unwrap();
    settled(&admission).await;
    assert_eq!(super::m2::schema(&engine).await, before);
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(super::m2::schema(&reopened).await, before);
    reopened.close().await.unwrap();
}

/// Assert intentional public sharing/moving guarantees without requiring a transaction to be Sync.
#[test]
fn public_thread_contracts() {
    /// Require a shareable public value at compile time.
    fn shared<T: Send + Sync>() {}
    /// Require an exclusively movable owner at compile time.
    fn movable<T: Send>() {}

    shared::<SqliteDatabase>();
    shared::<SqliteOptions>();
    shared::<Engine>();
    shared::<QueryResult>();
    movable::<Transaction>();
    movable::<super::super::SqliteSession>();
}

/// Bound every additional native safety case independently, including a blocked executor.
#[test]
fn safety_under_watchdog() {
    for scenario in [
        "begin",
        "run",
        "commit",
        "rollback",
        "plan",
        "transaction_plan",
        "failed_plan_cleanup",
        "acquire",
        "disclosure",
    ] {
        let (success, expired, output) = watchdog::run(
            "tests::rounds::native_child",
            scenario,
            Duration::from_secs(10),
        );
        assert!(
            success && !expired,
            "safety {scenario}: timeout={expired}\n{output}"
        );
        // Retain the cut/completion observations when this parent runs with --nocapture.
        eprintln!("{output}");
    }
}

/// Native hooks elsewhere establish exact phases; these cuts exercise actual await ownership.
#[tokio::test(flavor = "current_thread")]
#[ignore = "launched only by safety watchdog parent"]
async fn native_child() {
    let scenario = std::env::var("STACKPRESS_SQL_M0_SCENARIO").unwrap();
    let work = async {
        match scenario.as_str() {
            "acquire" => acquisition().await,
            "disclosure" => disclosure().await,
            "failed_plan_cleanup" => failed_plan_cleanup().await,
            phase => suspensions(phase).await,
        }
    };
    tokio::time::timeout(Duration::from_secs(8), work)
        .await
        .expect("safety deadline");
}

/// Cancel an unpolled retirement task at runtime shutdown, then drop the pool outside any runtime.
#[test]
#[ignore = "launched only by shutdown watchdog parent"]
fn shutdown_child() {
    for mode in ["retire", "close", "idle"] {
        let root = TempDir::new().unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let engine = runtime.block_on(async {
            let (engine, admission) = fixture(&root, 1).await;
            let mut tx = engine.begin().await.unwrap();
            tx.execute(&sql("INSERT INTO profile(name) VALUES ('abandoned')"))
                .await
                .unwrap();
            if mode == "close" {
                let mut close = Box::pin(engine.close());
                poll_fn(|cx| {
                    assert!(close.as_mut().poll(cx).is_pending());
                    Poll::Ready(())
                })
                .await;
                drop(close);
            }
            if mode == "idle" {
                tx.rollback().await.unwrap();
                settled(&admission).await;
                // An acknowledged ordinary statement follows retirement, so the pool's
                // final drop also covers reusable connection ownership outside a runtime.
                engine.execute(&sql("SELECT 1 AS n")).await.unwrap();
            } else {
                // No yield follows: cleanup cannot be polled before runtime shutdown.
                drop(tx);
            }
            engine
        });
        drop(runtime);
        // Constructing/dropping an unpolled close future performs no native IO.
        drop(engine.close());
        drop(engine);
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
            assert_eq!(count(&reopened).await, 0);
            reopened
                .execute(&sql("INSERT INTO profile(name) VALUES ('fresh writer')"))
                .await
                .unwrap();
            reopened.close().await.unwrap();
        });
    }
}

/// Bound runtime destruction itself, not just work executed before shutdown.
#[test]
fn shutdown_under_watchdog() {
    let (success, expired, output) = watchdog::run(
        "tests::rounds::shutdown_child",
        "shutdown",
        Duration::from_secs(10),
    );
    assert!(success && !expired, "shutdown: timeout={expired}\n{output}");
}
