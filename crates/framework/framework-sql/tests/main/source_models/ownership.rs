//! Real SQLite + an explicit suspension barrier prove borrowing, Send and reentrant immutable state.

use super::{
    schema,
    support::{self, Query, Record, Value},
};
use stackpress_framework_schema::{
    Model,
    source::{Id, SourceField, SourceSchema},
};
use stackpress_framework_sql::{DataSession, source::*};
use stackpress_sql::{
    Database, DialectId, Engine, Query as SqlQuery, QueryLimits, QueryResult, SqlFuture,
    TransactionOptions, TransactionSession,
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

#[path = "../../../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Metadata is derived from a real composed app extension, without a handwritten field catalog.
struct ProfileSpec;
impl support::Spec for ProfileSpec {
    type Id = String;
    fn describe() -> Model {
        schema(true)
            .models()
            .iter()
            .find(|m| m.name == "Profile")
            .unwrap()
            .clone()
    }
}
type Profile = support::Model<ProfileSpec, ProfileRole>;

/// Exactly the first armed operation pauses before native submission; other calls can reenter.
#[derive(Default)]
struct Barrier {
    armed: AtomicBool,
    entered: Notify,
    release: Notify,
}
impl Barrier {
    /// No guard or borrowed state is held across this intentionally suspended provider.
    async fn wait(&self) {
        if self.armed.swap(false, Ordering::SeqCst) {
            self.entered.notify_one();
            self.release.notified().await;
        }
    }
}
/// Real adapter underneath; this finite seam adds suspension, not simulated SQL results.
struct Paused {
    native: SqliteDatabase,
    barrier: Arc<Barrier>,
}
impl Database for Paused {
    fn dialect(&self) -> DialectId {
        self.native.dialect()
    }
    fn run(&self, query: SqlQuery, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            self.barrier.wait().await;
            self.native.run(query, limits).await
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
/// The adapter's transaction remains exclusively owned by its original caller.
struct PausedTx {
    native: Box<dyn TransactionSession>,
    barrier: Arc<Barrier>,
}
impl TransactionSession for PausedTx {
    fn run(&mut self, query: SqlQuery, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            self.barrier.wait().await;
            self.native.run(query, limits).await
        })
    }
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.native.commit()
    }
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.native.rollback()
    }
}

/// Native Date and input insertion order survive ownership; neither is flattened to JSON text.
fn input(id: &str) -> Record {
    vec![
        ("id".into(), Value::Text(id.into())),
        ("nickname".into(), Value::Text("app column".into())),
        ("name".into(), Value::Text("Ada".into())),
        ("created".into(), Value::Date(42)),
        ("active".into(), Value::Bool(true)),
    ]
}

/// All potentially blocking mechanics run under the external process owner, including its control.
#[test]
fn supervised_source_ownership() {
    if let Ok(scenario) = std::env::var("STACKPRESS_S6_SCENARIO") {
        if scenario == "forced-kill" {
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
        println!("source ownership complete");
        return;
    }
    let test = "source_models::ownership::supervised_source_ownership";
    let (ok, expired, output) = watchdog::run(test, "ownership", Duration::from_secs(20));
    assert!(
        ok && !expired && output.contains("source ownership complete"),
        "{output}"
    );
    let (ok, expired, output) = watchdog::run(test, "forced-kill", Duration::from_secs(1));
    assert!(
        !ok && expired && output.contains("forced child ready"),
        "{output}"
    );
}

/// Native IO proves real action signatures and exact transaction affinity; no process listener.
async fn exercise() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let native = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("source.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let barrier = Arc::new(Barrier::default());
    let engine = Arc::new(Engine::new(Paused {
        native,
        barrier: barrier.clone(),
    }));
    let actions = Profile::actions(engine.clone(), "native seed".into(), 90);
    assert_eq!(Profile::describe().name, "Profile");
    assert_eq!(
        Profile::schema("schema only".into(), 30).seed(),
        "schema only"
    );
    let description = actions.store().schema();
    assert_eq!(description.seed(), "native seed");
    assert_eq!(description.offset_minutes(), 90);
    assert_eq!(
        description
            .defaults_at(77)
            .unwrap()
            .iter()
            .find(|(n, _)| n == "created")
            .unwrap()
            .1,
        Value::Date(77)
    );
    assert!(description.assert(&Vec::new(), true).is_some());
    assert!(description.assert(&input("one"), true).is_none());
    assert_eq!(
        description.serialize(&input("one"), false).unwrap(),
        input("one")
    );
    assert_eq!(
        description.unserialize(&input("one"), false).unwrap(),
        input("one")
    );
    let mut record = Record::default();
    <support::Schema<ProfileSpec> as SourceField<Id>>::write(&mut record, "key".into());
    assert_eq!(
        <support::Schema<ProfileSpec> as SourceField<Id>>::read(&record),
        Some("key".into())
    );
    assert!(
        <support::Schema<ProfileSpec> as SourceField<Id>>::read(&vec![("id".into(), Value::Null)])
            .is_none()
    );
    actions.install().await.unwrap();
    let row = support::send(actions.create(&input("one"))).await.unwrap();
    assert_eq!(row, input("one"));
    let query = actions.store().query(&row).unwrap();
    assert_eq!(actions.store().unscalarize(&row).unwrap(), row);
    let read = support::send(actions.find(&query)).await.unwrap().unwrap();
    assert_eq!(
        support::get(&read, "nickname"),
        Some(&Value::Text("app column".into()))
    );
    assert_eq!(support::get(&read, "created"), Some(&Value::Date(42)));
    assert_eq!(support::send(actions.count(&query)).await.unwrap(), 1);
    assert_eq!(
        support::send(actions.find_all(&query)).await.unwrap().len(),
        1
    );
    let mut update = vec![
        ("id".into(), Value::Text("ignored write key".into())),
        ("nickname".into(), Value::Text("updated".into())),
    ];
    assert_eq!(
        support::send(actions.update(&query, &mut update))
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(support::get(&update, "id").is_none());
    assert_eq!(
        support::send(actions.remove(&query)).await.unwrap().len(),
        1
    );
    assert_eq!(
        support::send(actions.restore(&query)).await.unwrap().len(),
        1
    );
    let mut upsert = vec![
        ("id".into(), Value::Text("one".into())),
        ("nickname".into(), Value::Text("upsert".into())),
    ];
    assert!(
        support::send(actions.upsert(&mut upsert))
            .await
            .unwrap()
            .is_some()
    );

    // A returned record outlives both the exclusive DataSession and caller's transaction.
    let mut transaction = engine.begin().await.unwrap();
    let owned = {
        let mut db = DataSession::transaction(&mut transaction);
        // A real borrowed write must use this transaction, including mutable update input.
        support::send(actions.create_in(&mut db, &input("borrowed")))
            .await
            .unwrap();
        let borrowed = Query(Some("borrowed".into()));
        let mut changes = vec![("nickname".into(), Value::Text("transaction only".into()))];
        let changed = support::send(actions.update_in(&mut db, &borrowed, &mut changes))
            .await
            .unwrap();
        assert_eq!(
            support::get(&changed[0], "nickname"),
            Some(&Value::Text("transaction only".into()))
        );
        support::send(actions.find_in(&mut db, &query))
            .await
            .unwrap()
            .unwrap()
    };
    transaction.rollback().await.unwrap();
    assert_eq!(support::get(&owned, "created"), Some(&Value::Date(42)));
    assert!(
        actions
            .find(&Query(Some("borrowed".into())))
            .await
            .unwrap()
            .is_none()
    );

    // Reenter the same immutable action/schema while its first SQL provider is suspended.
    barrier.armed.store(true, Ordering::SeqCst);
    {
        let first = support::send(actions.find(&query));
        tokio::pin!(first);
        tokio::select! {
            _ = barrier.entered.notified() => {},
            _ = &mut first => panic!("first call did not suspend"),
        }
        assert_eq!(actions.store().schema().seed(), "native seed");
        assert_eq!(actions.count(&query).await.unwrap(), 1);
        barrier.release.notify_one();
        assert!(first.await.unwrap().is_some());
    }
    // Cancellation drops the first exclusive transaction borrow; the owner can reuse/finish it.
    let mut transaction = engine.begin().await.unwrap();
    {
        let mut db = DataSession::transaction(&mut transaction);
        barrier.armed.store(true, Ordering::SeqCst);
        {
            let first = support::send(actions.find_in(&mut db, &query));
            tokio::pin!(first);
            tokio::select! {
                _ = barrier.entered.notified() => {},
                _ = &mut first => panic!("borrowed call did not suspend"),
            }
        }
        assert_eq!(actions.count_in(&mut db, &query).await.unwrap(), 1);
    }
    transaction.rollback().await.unwrap();
    let mut inputs = [input("two")];
    assert!(support::send(actions.batch(&mut inputs, BatchMode::Create)).await[0].is_ok());
    actions.upgrade(None).await.unwrap();
    assert_eq!(
        support::send(actions.delete(&query)).await.unwrap().len(),
        1
    );
    actions.purge(false).await.unwrap();
    assert_eq!(actions.count(&Query::default()).await.unwrap(), 0);
    actions.uninstall().await.unwrap();
    engine.close().await.unwrap();
}
