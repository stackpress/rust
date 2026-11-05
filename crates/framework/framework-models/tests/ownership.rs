//! Generated actions reenter and cancel against a real adapter under a parent process deadline.
#![cfg(all(feature = "sql", not(target_arch = "wasm32")))]
use stackpress_framework_models::ProfileModel;
use stackpress_framework_schema::source::Record;
use stackpress_framework_sql::{
    DataSession,
    source::{SourceModel, StoreQuery},
};
use stackpress_sql::{
    Database, DialectId, Engine, Query, QueryLimits, QueryResult, SqlFuture, TransactionOptions,
    TransactionSession,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::Notify;
#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;
/// Suspension happens before native submission, with no guard held by the seam.
#[derive(Default)]
struct Barrier {
    armed: AtomicBool,
    entered: Notify,
    release: Notify,
}
impl Barrier {
    async fn wait(&self) {
        if self.armed.swap(false, Ordering::SeqCst) {
            self.entered.notify_one();
            self.release.notified().await;
        }
    }
}
struct Paused {
    native: SqliteDatabase,
    barrier: Arc<Barrier>,
}
impl Database for Paused {
    fn dialect(&self) -> DialectId {
        self.native.dialect()
    }
    fn run(&self, q: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            self.barrier.wait().await;
            self.native.run(q, limits).await
        })
    }
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            Ok(Box::new(PausedTx {
                native: self.native.begin(options).await?,
                barrier: self.barrier.clone(),
            }) as Box<dyn TransactionSession>)
        })
    }
    fn close(&self) -> SqlFuture<'_, ()> {
        self.native.close()
    }
}
struct PausedTx {
    native: Box<dyn TransactionSession>,
    barrier: Arc<Barrier>,
}
impl TransactionSession for PausedTx {
    fn run(&mut self, q: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            self.barrier.wait().await;
            self.native.run(q, limits).await
        })
    }
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.native.commit()
    }
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.native.rollback()
    }
}
#[test]
fn generated_ownership_watchdog() {
    if let Ok(mode) = std::env::var("STACKPRESS_S6_SCENARIO") {
        if mode == "forced-kill" {
            println!("forced child ready");
            loop {
                std::thread::park();
            }
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(exercise());
        println!("generated ownership complete");
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "generated_ownership_watchdog",
        "ownership",
        Duration::from_secs(20),
    );
    assert!(
        ok && !expired && output.contains("generated ownership complete"),
        "{output}"
    );
    let (ok, expired, output) = watchdog::run(
        "generated_ownership_watchdog",
        "forced-kill",
        Duration::from_secs(1),
    );
    assert!(
        !ok && expired && output.contains("forced child ready"),
        "{output}"
    );
}
fn send<T: Send>(future: T) -> T {
    future
}
async fn exercise() {
    let dir = tempfile::tempdir_in("/private/tmp").unwrap();
    let native = SqliteDatabase::open(
        SqliteOptions::file(dir.path().join("owned.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let barrier = Arc::new(Barrier::default());
    let engine = Arc::new(Engine::new(Paused {
        native,
        barrier: barrier.clone(),
    }));
    let actions = ProfileModel::actions(engine.clone(), "seed".into(), 0);
    actions.install().await.unwrap();
    let input = Record::from_iter([("id".into(), "owned".into()), ("name".into(), "Ada".into())]);
    send(actions.create(&input)).await.unwrap();
    let query = StoreQuery::eq("id", "owned");
    // A second read completes while the first call is suspended in the provider.
    barrier.armed.store(true, Ordering::SeqCst);
    {
        let first = send(actions.find(&query));
        tokio::pin!(first);
        tokio::select! { _=barrier.entered.notified()=>{},_= &mut first=>panic!("not suspended") }
        assert_eq!(actions.count(&query).await.unwrap(), 1);
        barrier.release.notify_one();
        assert!(first.await.unwrap().is_some());
    }
    // Cancelling a borrowed call releases the exclusive executor; the caller still finishes it.
    let mut transaction = engine.begin().await.unwrap();
    {
        let mut db = DataSession::transaction(&mut transaction);
        barrier.armed.store(true, Ordering::SeqCst);
        {
            let first = send(actions.find_in(&mut db, &query));
            tokio::pin!(first);
            tokio::select! { _=barrier.entered.notified()=>{},_= &mut first=>panic!("not suspended") }
        }
        assert_eq!(send(actions.count_in(&mut db, &query)).await.unwrap(), 1);
    }
    transaction.rollback().await.unwrap();
    actions.uninstall().await.unwrap();
    engine.close().await.unwrap();
}
