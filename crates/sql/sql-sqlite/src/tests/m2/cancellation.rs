//! Safe native hooks and async ready/go barriers prove cancellation at owned transitions.

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use sqlx::Executor;
use sqlx::sqlite::SqlitePoolOptions;
use stackpress_sql::{
    Create, Database, DialectId, Engine, ErrorKind, Field, Index, QueryLimits, Render, SqlType,
    TransactionOptions,
};
use tokio::sync::{Notify, Semaphore, oneshot};

use super::super::transitions::{Hook, gate};
use super::{SqliteDatabase, TempDir, options, schema, sql};

/// Await actual detached cleanup relinquishing every admission slot, not an arbitrary delay.
async fn cleaned(db: &SqliteDatabase, permits: u32) {
    super::super::wait_for_cleanup(&db.admission, permits).await;
}

/// Cancel native streaming with rows in flight, both ordinarily and inside a dirty transaction.
pub(super) async fn fetch() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 1;
    let db = Arc::new(SqliteDatabase::open(opts).await.unwrap());
    let query = sql("WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<10000) SELECT x FROM n")
        .render(DialectId::Sqlite).unwrap();
    for _ in 0..3 {
        let (entered, release) = gate(&db, Hook::Costly).await;
        let shared = Arc::clone(&db);
        let query = query.clone();
        let task = tokio::spawn(async move { shared.run(query, QueryLimits::default()).await });
        entered.await.unwrap();
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(db.admit().err().unwrap().kind(), ErrorKind::Limit);
        release.send(()).unwrap();
        cleaned(&db, 1).await;
        assert_eq!(
            db.run(
                sql("SELECT 1 AS healthy")
                    .render(DialectId::Sqlite)
                    .unwrap(),
                QueryLimits::default()
            )
            .await
            .unwrap()
            .rows[0]
                .try_get::<i64>("healthy")
                .unwrap(),
            1
        );
    }
    db.run(
        sql("CREATE TABLE profile(name TEXT)")
            .render(DialectId::Sqlite)
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let (mut entered, release) = gate(&db, Hook::Costly).await;
    let mut transaction = db.begin(TransactionOptions::default()).await.unwrap();
    transaction
        .run(
            sql("INSERT INTO profile VALUES ('never committed')")
                .render(DialectId::Sqlite)
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
    let mut work = transaction.run(query, QueryLimits::default());
    tokio::select! {
        ready = &mut entered => ready.unwrap(),
        result = &mut work => panic!("fetch completed before native gate: {result:?}"),
    }
    drop(work);
    release.send(()).unwrap();
    assert_eq!(
        transaction
            .run(
                sql("SELECT 1").render(DialectId::Sqlite).unwrap(),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::RollbackOnly
    );
    transaction.rollback().await.unwrap();
    cleaned(&db, 1).await;
    assert_eq!(
        db.run(
            sql("SELECT count(*) AS n FROM profile")
                .render(DialectId::Sqlite)
                .unwrap(),
            QueryLimits::default()
        )
        .await
        .unwrap()
        .rows[0]
            .try_get::<i64>("n")
            .unwrap(),
        0
    );
    db.close().await.unwrap();
}

/// Rollback cancellation retains admission through the native acknowledgement and cannot commit.
pub(super) async fn rollback() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 1;
    let db = SqliteDatabase::open(opts).await.unwrap();
    db.run(
        sql("CREATE TABLE profile(name TEXT)")
            .render(DialectId::Sqlite)
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let (entered, release) = gate(&db, Hook::Rollback).await;
    let mut transaction = db.begin(TransactionOptions::default()).await.unwrap();
    transaction
        .run(
            sql("INSERT INTO profile VALUES ('rolled back')")
                .render(DialectId::Sqlite)
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
    let task = tokio::spawn(transaction.rollback());
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(db.admit().err().unwrap().kind(), ErrorKind::Limit);
    release.send(()).unwrap();
    cleaned(&db, 1).await;
    db.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(super::super::count(&reopened).await, 0);
    reopened.close().await.unwrap();
}

/// Aborting a partially executed DDL batch rolls it back; losing commit acknowledgement is unknown.
pub(super) async fn ddl() {
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let (mut entered_commit, release_commit) = gate(&db, Hook::Commit).await;
    let engine = Arc::new(Engine::new(db));
    let before = schema(&engine).await;
    let create = Create::new("partial")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("partial_index", ["id"]));
    let plan = create.render(DialectId::Sqlite).unwrap();
    let (ready, entered) = oneshot::channel();
    let (_go, proceed) = oneshot::channel::<()>();
    let shared = Arc::clone(&engine);
    let first = plan.queries()[0].clone();
    let task = tokio::spawn(async move {
        let mut transaction = shared.begin().await.unwrap();
        transaction.execute(&first).await.unwrap();
        ready.send(()).unwrap();
        proceed.await.unwrap();
        transaction.commit().await.unwrap();
    });
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(schema(&engine).await, before);

    // Retirement removed the first connection's hooks. Install a new commit gate on its replacement.
    // The old receiver is intentionally dropped: the aborted batch must not have attempted COMMIT.
    assert_eq!(
        entered_commit.try_recv(),
        Err(oneshot::error::TryRecvError::Closed)
    );
    drop(release_commit);
    engine.close().await.unwrap();
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let (entered, release) = gate(&db, Hook::Commit).await;
    let engine = Arc::new(Engine::new(db));
    let shared = Arc::clone(&engine);
    let task = tokio::spawn(async move { shared.execute_plan(&plan).await });
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    release.send(()).unwrap();
    let after = schema(&engine).await;
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].try_get::<String>("name").unwrap(), "partial_index");
    assert_eq!(after[1].try_get::<String>("name").unwrap(), "partial");
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(schema(&reopened).await, after);
    reopened.close().await.unwrap();
}

/// Prepare/begin errors retain their causes; cancellation before any command retires the lease.
pub(super) async fn prepare() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 1;
    let db = SqliteDatabase::open(opts).await.unwrap();
    // Drop the actual production RAII owner after acquire, before queuing native SQL.
    let lease = crate::lease::Lease::new(db.pool.acquire().await.unwrap(), db.admit().unwrap());
    drop(lease);
    cleaned(&db, 1).await;
    let admission = Arc::clone(&db.admission);
    let engine = Engine::new(db);
    engine
        .execute(&sql("CREATE TABLE profile(name TEXT)"))
        .await
        .unwrap();
    let mut transaction = engine.begin().await.unwrap();
    transaction
        .execute(&sql("INSERT INTO profile VALUES ('must roll back')"))
        .await
        .unwrap();
    let error = transaction
        .execute(&sql("SELECT FROM profile"))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(error.source().is_some());
    assert!(
        transaction
            .rollback_after(error)
            .await
            .cleanup_error()
            .is_none()
    );
    // Explicit rollback acknowledges SQL, then lease retirement owns the last cleanup step.
    drop(admission.acquire_owned().await.unwrap());
    assert_eq!(super::super::count(&engine).await, 0);
    engine.close().await.unwrap();

    // Interrupt the counters/prepare phase before the requested write can execute.
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let mut connection = db.pool.acquire().await.unwrap();
    connection
        .lock_handle()
        .await
        .unwrap()
        .set_progress_handler(1, || false);
    drop(connection);
    let error = db
        .run(
            sql("INSERT INTO profile VALUES ('not executed')")
                .render(DialectId::Sqlite)
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    db.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(super::super::count(&reopened).await, 0);
    reopened.close().await.unwrap();
}

/// Acknowledge callback cancellation or native collation destruction without timer guesses.
struct Dropped(Arc<Notify>);

impl Drop for Dropped {
    /// Notify when the owned callback resource is actually destroyed.
    fn drop(&mut self) {
        self.0.notify_one();
    }
}

/// Cancel eager open after native initialization, while pool publication is still pending.
pub(super) async fn open() {
    let root = TempDir::new().unwrap();
    let initial = super::super::engine(&root, 1).await;
    initial.close().await.unwrap();
    let ready = Arc::new(Notify::new());
    let dropped = Arc::new(Notify::new());
    let closed = Arc::new(Notify::new());
    let barrier = Arc::new(Semaphore::new(0));
    for _ in 0..3 {
        let entered = Arc::clone(&ready);
        let released = Arc::clone(&dropped);
        let native_completion = Arc::clone(&closed);
        let pause = Arc::clone(&barrier);
        let pool = SqlitePoolOptions::new().after_connect(move |connection, _| {
            let entered = Arc::clone(&entered);
            let released = Arc::clone(&released);
            let closed = Arc::clone(&native_completion);
            let pause = Arc::clone(&pause);
            Box::pin(async move {
                let _guard = Dropped(released);
                // This uniquely named collation is never replaced or used by SQL. Its
                // captured guard is destroyed by SQLite close, after rollback and b-tree
                // closure release the writer lock (pinned libsqlite3-sys 0.37.0).
                let native_closed = Dropped(closed);
                connection.lock_handle().await?.create_collation(
                    "open_close_ack",
                    move |left, right| {
                        let _keep_until_close = &native_closed;
                        left.cmp(right)
                    },
                )?;
                // A dirty native transaction makes leaked init ownership observable as a writer lock.
                connection.execute("BEGIN").await?;
                connection
                    .execute("INSERT INTO profile(name) VALUES ('abandoned open')")
                    .await?;
                entered.notify_one();
                let _permit = pause.acquire().await.unwrap();
                Ok(())
            })
        });
        let task = tokio::spawn(SqliteDatabase::open_pool(options(&root, 1), pool));
        ready.notified().await;
        task.abort();
        assert!(task.await.err().unwrap().is_cancelled());
        dropped.notified().await;
        // Callback Drop precedes native worker destruction. Await its separate close
        // acknowledgement before recovery instead of spending a busy timeout on cleanup.
        closed.notified().await;
        let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
        // Native writer acquisition now proves no orphan writer survives acknowledged close.
        engine
            .execute(&sql("INSERT INTO profile(name) VALUES ('healthy')"))
            .await
            .unwrap();
        engine
            .execute(&sql("DELETE FROM profile WHERE name = 'healthy'"))
            .await
            .unwrap();
        assert_eq!(super::super::count(&engine).await, 0);
        engine.close().await.unwrap();
    }
    // Dropping an unpolled open future performs no filesystem effects.
    let absent = root.path().join("unpolled.sqlite");
    drop(SqliteDatabase::open(
        crate::SqliteOptions::file(&absent).create_if_missing(true),
    ));
    assert!(!absent.exists());
    // A directory cannot become a silently substituted database or publish a partial pool.
    let error = SqliteDatabase::open(crate::SqliteOptions::file(root.path()))
        .await
        .err()
        .unwrap();
    assert!(error.source().is_some());
    assert_eq!(
        crate::execute::driver_error(sqlx::Error::WorkerCrashed).kind(),
        ErrorKind::Closed
    );
    // The source-preserving worker-loss seam is synthetic; no unsafe forced worker crash is used.
}

/// Automatic native rollback and commit veto preserve failures and never reuse dirty sessions.
pub(super) async fn native_failures() {
    let root = TempDir::new().unwrap();
    let engine = super::super::engine(&root, 1).await;
    engine
        .execute(&sql("INSERT INTO profile(name) VALUES ('existing')"))
        .await
        .unwrap();
    let mut transaction = engine.begin().await.unwrap();
    transaction
        .execute(&sql("INSERT INTO profile(name) VALUES ('uncommitted')"))
        .await
        .unwrap();
    let primary = transaction
        .execute(&sql(
            "INSERT OR ROLLBACK INTO profile(name) VALUES ('existing')",
        ))
        .await
        .unwrap_err();
    assert!(primary.source().is_some());
    let error = transaction.rollback_after(primary).await;
    // SQLite already rolled back the transaction; explicit cleanup now has a separate cause.
    assert!(error.source().is_some());
    assert!(error.cleanup_error().unwrap().source().is_some());
    assert_eq!(super::super::count(&engine).await, 1);
    engine.close().await.unwrap();

    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let mut connection = db.pool.acquire().await.unwrap();
    connection
        .lock_handle()
        .await
        .unwrap()
        .set_commit_hook(|| false);
    drop(connection);
    let engine = Engine::new(db);
    let mut transaction = engine.begin().await.unwrap();
    transaction
        .execute(&sql("INSERT INTO profile(name) VALUES ('vetoed commit')"))
        .await
        .unwrap();
    let error = transaction.commit().await.unwrap_err();
    assert!(error.source().is_some());
    assert_eq!(super::super::count(&engine).await, 1);
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(super::super::count(&reopened).await, 1);
    reopened.close().await.unwrap();
}

/// A current-thread timer progresses during native work, and a real writer lock reports Busy.
pub(super) async fn contention() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 2);
    opts.max_in_flight = 2;
    opts.busy_timeout_ms = 300;
    let db = Arc::new(SqliteDatabase::open(opts).await.unwrap());
    db.run(
        sql("CREATE TABLE profile(name TEXT)")
            .render(DialectId::Sqlite)
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let mut owner = db.begin(TransactionOptions::default()).await.unwrap();
    owner
        .run(
            sql("INSERT INTO profile VALUES ('held writer')")
                .render(DialectId::Sqlite)
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
    // Gate the other worker before it proceeds to the contending statement.
    let (entered, release) = gate(&db, Hook::Progress).await;
    let shared = Arc::clone(&db);
    let task = tokio::spawn(async move {
        shared
            .run(
                sql("INSERT INTO profile VALUES ('contender')")
                    .render(DialectId::Sqlite)
                    .unwrap(),
                QueryLimits::default(),
            )
            .await
    });
    entered.await.unwrap();
    // Keep the native worker at its acknowledged gate until the timer advances. The
    // contender cannot finish first merely because this executor was descheduled.
    // The parent watchdog bounds a blocked executor independently.
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(!task.is_finished());
    release.send(()).unwrap();
    // The writer remains locked after the responsiveness observation, so native
    // contention must still produce Busy with its original driver cause.
    let error = task.await.unwrap().unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Busy);
    assert!(error.source().is_some());
    owner.rollback().await.unwrap();
    // Busy retires the contender, and rollback retires the owner. SQL acknowledgement
    // precedes detached worker shutdown: wait for both admission slots to return
    // before starting a fresh acquisition's independent configured budget.
    cleaned(&db, 2).await;
    assert_eq!(db.pool.size(), 0);
    let tx = db.begin(TransactionOptions::default()).await.unwrap();
    tx.rollback().await.unwrap();
    db.close().await.unwrap();
}
