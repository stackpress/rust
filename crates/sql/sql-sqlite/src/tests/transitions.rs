//! Test-only safe native hooks establish exact worker transitions; no production hook API.

use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::Duration;

use sqlx::{Connection, Executor};
use stackpress_sql::{
    Database, Engine, ErrorKind, QueryLimits, Render, TransactionOptions, TransactionSession,
};
use tokio::sync::oneshot;

use super::{SqliteDatabase, TempDir, count, options, sql};

/// Native phase at which the SQLite worker pauses behind a ready/go handshake.
enum Hook {
    Progress,
    Costly,
    Update,
    Commit,
}

/// Install a one-shot safe SQLx hook on the size-one pool's only connection.
async fn gate(db: &SqliteDatabase, hook: Hook) -> (oneshot::Receiver<()>, mpsc::Sender<()>) {
    let (ready, entered) = oneshot::channel();
    let (release, go) = mpsc::channel();
    let mut ready = Some(ready);
    let mut pause = move || {
        if let Some(ready) = ready.take() {
            ready.send(()).unwrap();
            // A missing release fails on the worker too; the parent still enforces 10s.
            go.recv_timeout(Duration::from_secs(3)).unwrap();
        }
    };
    let mut connection = db.pool.acquire().await.unwrap();
    let mut handle = connection.lock_handle().await.unwrap();
    match hook {
        Hook::Progress => handle.set_progress_handler(1, move || {
            pause();
            true
        }),
        // Tiny metadata queries execute fewer than 1000 VM instructions. This gate
        // therefore belongs to the costly recursive query, not to arity/accounting IO.
        Hook::Costly => handle.set_progress_handler(1000, move || {
            pause();
            true
        }),
        Hook::Update => handle.set_update_hook(move |_| pause()),
        Hook::Commit => handle.set_commit_hook(move || {
            pause();
            true
        }),
    }
    drop(handle);
    drop(connection);
    (entered, release)
}

/// Repeated real begin/active-query/commit cancellation retires sessions and preserves data.
pub(super) async fn cancellation() {
    let root = TempDir::new().unwrap();
    let db = Arc::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    db.run(
        sql("CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT)")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    for _ in 0..3 {
        // Progress is inside SQLite BEGIN, after the command was queued and before acknowledgement.
        let (entered, release) = gate(&db, Hook::Progress).await;
        let shared = Arc::clone(&db);
        let task = tokio::spawn(async move { shared.begin(TransactionOptions::default()).await });
        entered.await.unwrap();
        task.abort();
        assert!(task.await.err().unwrap().is_cancelled());
        release.send(()).unwrap();
        // A fresh acquisition executes cleanly after rollback/retirement of the abandoned BEGIN.
        let mut tx = db.begin(TransactionOptions::default()).await.unwrap();
        tx.run(
            sql("INSERT INTO profile(name) VALUES ('after begin')")
                .render(db.dialect())
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
        tx.rollback().await.unwrap();
    }

    // The update hook proves INSERT has reached its native mutation, not just prepare.
    let (mut entered, release) = gate(&db, Hook::Update).await;
    let mut tx = db.begin(TransactionOptions::default()).await.unwrap();
    let query = sql("INSERT INTO profile(name) VALUES ('cancelled statement')")
        .render(db.dialect())
        .unwrap();
    let future = tx.run(query, QueryLimits::default());
    let mut future = Box::pin(future);
    // Continue polling through prepare/bind until the worker reaches the actual write.
    // Polling only once could stop at prepare and never enqueue execution at all.
    tokio::select! {
        ready = &mut entered => ready.unwrap(),
        result = future.as_mut() => panic!("statement completed before its native gate: {result:?}"),
    }
    drop(future);
    release.send(()).unwrap();
    assert_eq!(
        tx.commit().await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );

    // Pause inside COMMIT before acknowledgement; cancellation must never replay it.
    let (entered, release) = gate(&db, Hook::Commit).await;
    let mut tx = db.begin(TransactionOptions::default()).await.unwrap();
    tx.run(
        sql("INSERT INTO profile(name) VALUES ('unknown commit')")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let task = tokio::spawn(tx.commit());
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    release.send(()).unwrap();
    // This controlled seam commits despite losing its acknowledgement. The caller's
    // general contract is unknown; persistence here is evidence that rollback cannot be promised.
    let result = db
        .run(
            sql("SELECT count(*) AS n FROM profile")
                .render(db.dialect())
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
    assert_eq!(result.rows[0].try_get::<i64>("n").unwrap(), 1);
    db.close().await.unwrap();
    drop(db);
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(count(&reopened).await, 1);
    reopened.close().await.unwrap();
}

/// Cancelled native work keeps its admission slot until a real worker-close acknowledgement.
pub(super) async fn admission_cleanup() {
    let root = TempDir::new().unwrap();
    let mut opts = options(&root, 1);
    opts.max_in_flight = 1;
    let db = Arc::new(SqliteDatabase::open(opts).await.unwrap());
    db.run(
        sql("CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT)")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let (entered, release) = gate(&db, Hook::Update).await;
    let shared = Arc::clone(&db);
    let task = tokio::spawn(async move {
        shared
            .run(
                sql("INSERT INTO profile(name) VALUES ('unobserved')")
                    .render(shared.dialect())
                    .unwrap(),
                QueryLimits::default(),
            )
            .await
    });
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    // The worker is still held inside mutation, so cancellation cannot free admission.
    assert_eq!(db.admit().err().unwrap().kind(), ErrorKind::Limit);
    release.send(()).unwrap();
    // Await the actual cleanup owner relinquishing its slot, without timing guesses.
    let available = db.admission.clone().acquire_owned().await.unwrap();
    drop(available);
    let result = db
        .run(
            sql("SELECT count(*) AS n FROM profile")
                .render(db.dialect())
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
    assert_eq!(result.rows[0].try_get::<i64>("n").unwrap(), 1);
    db.close().await.unwrap();
}

/// A real costly query leaves a current-thread timer runnable; cancellation remains bounded.
pub(super) async fn responsive() {
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let (entered, release) = gate(&db, Hook::Costly).await;
    let engine = Arc::new(Engine::new(db));
    let query = sql(
        "WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<2000000) SELECT sum(x) AS n FROM n",
    );
    let shared = Arc::clone(&engine);
    let mut task = tokio::spawn(async move { shared.execute(&query).await });
    entered.await.unwrap();
    // The worker is provably executing SQL while this current-thread runtime timer advances.
    tokio::time::timeout(
        Duration::from_millis(100),
        tokio::time::sleep(Duration::from_millis(10)),
    )
    .await
    .unwrap();
    assert!(!task.is_finished());
    release.send(()).unwrap();
    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(10)) => {},
        result = &mut task => panic!("costly query finished before responsiveness observation: {result:?}"),
    }
    assert_eq!(
        task.await.unwrap().unwrap().rows[0]
            .try_get::<i64>("n")
            .unwrap(),
        2_000_001_000_000
    );
    engine.close().await.unwrap();
}

/// Native lock, begin, commit and rollback failure seams retain causes and discard sessions.
pub(super) async fn failures() {
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 2)).await.unwrap();
    db.run(
        sql("CREATE TABLE parent(id INTEGER PRIMARY KEY)")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    db.run(
        sql("CREATE TABLE child(id INTEGER REFERENCES parent(id) DEFERRABLE INITIALLY DEFERRED)")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let mut tx = db.begin(TransactionOptions::default()).await.unwrap();
    tx.run(
        sql("INSERT INTO child(id) VALUES (9)")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let error = tx.commit().await.unwrap_err();
    assert!(error.source().is_some());
    let result = db
        .run(
            sql("SELECT count(*) AS n FROM child")
                .render(db.dialect())
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap();
    assert_eq!(result.rows[0].try_get::<i64>("n").unwrap(), 0);

    // Hold SQLite's writer lock on one real connection; another write must hit busy.
    let mut tx = db.begin(TransactionOptions::default()).await.unwrap();
    tx.run(
        sql("INSERT INTO parent VALUES (1)")
            .render(db.dialect())
            .unwrap(),
        QueryLimits::default(),
    )
    .await
    .unwrap();
    let error = db
        .run(
            sql("INSERT INTO parent VALUES (2)")
                .render(db.dialect())
                .unwrap(),
            QueryLimits::default(),
        )
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Busy);
    tx.rollback().await.unwrap();

    // Interrupt the actual BEGIN VM via a safe native hook; SQLx returns its source error.
    let single = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let mut connection = single.pool.acquire().await.unwrap();
    connection
        .lock_handle()
        .await
        .unwrap()
        .set_progress_handler(1, || false);
    drop(connection);
    let error = single
        .begin(TransactionOptions::default())
        .await
        .err()
        .unwrap();
    assert!(error.source().is_some());
    let tx = single.begin(TransactionOptions::default()).await.unwrap();
    tx.rollback().await.unwrap();

    // Exercise rollback failure on a real session whose transaction was ended by a
    // test-only control command. Public raw SQL cannot end a transaction this way.
    let mut connection = single.pool.acquire().await.unwrap();
    connection.execute("BEGIN").await.unwrap();
    connection.execute("ROLLBACK").await.unwrap();
    let permit = single.admit().unwrap();
    let session = super::super::SqliteSession {
        lease: super::super::lease::Lease::new(connection, permit),
        rollback_only: false,
    };
    let error = Box::new(session).rollback().await.unwrap_err();
    assert!(error.source().is_some());
    let tx = single.begin(TransactionOptions::default()).await.unwrap();
    tx.rollback().await.unwrap();

    // A closed connection cannot manufacture a successful empty result.
    let connection = single.pool.acquire().await.unwrap().detach();
    connection.close_hard().await.unwrap();
    single.close().await.unwrap();
    assert_eq!(
        single
            .begin(TransactionOptions::default())
            .await
            .err()
            .unwrap()
            .kind(),
        ErrorKind::Closed
    );
    db.close().await.unwrap();

    // Interrupt only explicit rollback after a genuine statement failure. The helper
    // must preserve both native sources rather than replacing the primary diagnosis.
    let cleanup_root = TempDir::new().unwrap();
    let native = SqliteDatabase::open(options(&cleanup_root, 1))
        .await
        .unwrap();
    let interrupt = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&interrupt);
    let mut connection = native.pool.acquire().await.unwrap();
    connection
        .lock_handle()
        .await
        .unwrap()
        .set_progress_handler(1, move || !flag.load(Ordering::SeqCst));
    drop(connection);
    let engine = Engine::new(native);
    engine
        .execute(&sql("CREATE TABLE profile(name TEXT UNIQUE)"))
        .await
        .unwrap();
    let mut tx = engine.begin().await.unwrap();
    tx.execute(&sql("INSERT INTO profile VALUES ('same')"))
        .await
        .unwrap();
    let primary = tx
        .execute(&sql("INSERT INTO profile VALUES ('same')"))
        .await
        .unwrap_err();
    interrupt.store(true, Ordering::SeqCst);
    let error = tx.rollback_after(primary).await;
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(error.source().is_some());
    assert!(error.cleanup_error().unwrap().source().is_some());
    assert_eq!(count(&engine).await, 0);
    engine.close().await.unwrap();

    // With healthy cleanup, the primary error still retains its original native cause.
    let native = SqliteDatabase::open(options(&cleanup_root, 1))
        .await
        .unwrap();
    let engine = Engine::new(native);
    let mut tx = engine.begin().await.unwrap();
    let primary = tx
        .execute(&sql("INSERT INTO missing VALUES (1)"))
        .await
        .unwrap_err();
    let error = tx.rollback_after(primary).await;
    assert!(error.source().is_some());
    assert!(error.cleanup_error().is_none());
    engine.close().await.unwrap();
}
