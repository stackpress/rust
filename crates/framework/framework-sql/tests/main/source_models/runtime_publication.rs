//! Generic native store/action contracts retain real SQLite state and executor affinity.
use super::{Profile, record};
use stackpress_framework_schema::source::SourceSchema;
use stackpress_framework_sql::{
    DataSession,
    source::{BatchMode, NativeActions, NativeStore, SourceActions, SourceStore, StoreQuery},
};
use stackpress_sql::{DialectId, Engine, Render};
use std::sync::Arc;

/// Generic calls exercise the public store contract, including native codecs and pure SQL shapes.
#[test]
fn generic_store_contract_preserves_metadata_queries_and_scalar_codecs() {
    type Store = NativeStore<Profile>;
    let store = <Store as SourceStore>::new("seed".into(), 0);
    assert_eq!(SourceStore::schema(&store).seed(), "seed");
    let query = SourceStore::query(&store, &record(serde_json::json!({"eq":{"id":"p"}}))).unwrap();
    let create = SourceStore::create(&store);
    assert!(
        !create
            .render(DialectId::Sqlite)
            .unwrap()
            .queries()
            .is_empty()
    );
    // Equal/absent targets retain an empty builder; the SQL core requires operations to execute.
    for target in [None, Some(&create)] {
        assert_eq!(
            SourceStore::alter(&store, target)
                .unwrap()
                .build()
                .unwrap_err()
                .kind(),
            stackpress_sql::ErrorKind::InvalidQuery
        );
    }
    // A changed adjacent definition produces an executable add-column plan, rather than
    // the empty alteration used for equal/absent targets above.
    let changed = create.clone().add_field(stackpress_sql::Field::new(
        "publication_extra",
        stackpress_sql::SqlType::Text,
    ));
    let plan = SourceStore::alter(&store, Some(&changed))
        .unwrap()
        .render(DialectId::Postgres)
        .unwrap();
    assert_eq!(plan.queries().len(), 1);
    assert!(
        plan.queries()[0]
            .sql()
            .contains("ADD COLUMN \"publication_extra\" TEXT")
    );
    assert!(plan.queries()[0].sql().contains("\"profile\""));
    assert!(plan.queries()[0].binds().is_empty());
    let encoded =
        SourceStore::scalarize(&store, &record(serde_json::json!({"id":"p","name":"Ada"})))
            .unwrap();
    assert!(
        encoded.iter().any(|(name, value)| name == "name"
            && value == &stackpress_sql::SqlValue::Text("Ada".into()))
    );
    let decoded = SourceStore::unscalarize(
        &store,
        &record(serde_json::json!({"name":"Ada","unknown":"ignored"})),
    )
    .unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded["name"], "Ada");
    let select = SourceStore::select(&store, &query, DialectId::Sqlite)
        .unwrap()
        .render(DialectId::Sqlite)
        .unwrap();
    assert!(select.sql().contains("\"profile\""));
    assert!(
        select
            .binds()
            .contains(&stackpress_sql::SqlValue::Text("p".into()))
    );
    let insert = SourceStore::insert(
        &store,
        &record(serde_json::json!({"id":"p","name":"Ada"})),
        DialectId::Sqlite,
    )
    .unwrap()
    .render(DialectId::Sqlite)
    .unwrap();
    assert!(insert.sql().contains("INSERT INTO \"profile\""));
    assert_eq!(
        insert.binds(),
        &[
            stackpress_sql::SqlValue::Text("p".into()),
            stackpress_sql::SqlValue::Text("Ada".into())
        ]
    );
    let mut patch = record(serde_json::json!({"name":"Grace"}));
    let update = SourceStore::update(&store, &query, &mut patch, DialectId::Sqlite)
        .unwrap()
        .render(DialectId::Sqlite)
        .unwrap();
    assert!(update.sql().contains("UPDATE \"profile\""));
    assert!(
        update
            .binds()
            .contains(&stackpress_sql::SqlValue::Text("Grace".into()))
    );
    assert!(
        update
            .binds()
            .contains(&stackpress_sql::SqlValue::Text("p".into()))
    );
    assert!(patch.contains_key("updated"));
    let delete = SourceStore::delete(&store, &query, DialectId::Sqlite)
        .unwrap()
        .render(DialectId::Sqlite)
        .unwrap();
    assert!(delete.sql().contains("DELETE FROM \"profile\""));
    assert!(
        delete
            .binds()
            .contains(&stackpress_sql::SqlValue::Text("p".into()))
    );
    // Computed projections have no declared column; source string interpolation still applies.
    assert_eq!(
        store.to_sql_value("computed", &super::Value::Null).unwrap(),
        stackpress_sql::SqlValue::Null
    );
    assert_eq!(
        store
            .to_sql_value(
                "computed",
                &super::Value::from(serde_json::json!({"label":"value"}))
            )
            .unwrap(),
        stackpress_sql::SqlValue::Text("[object Object]".into())
    );
}

/// Borrowed generic actions stay on the caller's transaction and pooled defaults keep source results.
#[tokio::test]
async fn generic_actions_keep_transaction_affinity_and_source_lifecycle() {
    type Actions = NativeActions<Profile>;
    let directory = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(directory.path().join("generic.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(Engine::new(database));
    let actions =
        <Actions as SourceActions>::new(engine.clone(), NativeStore::new("seed".into(), 0));
    assert!(std::ptr::eq(
        SourceActions::engine(&actions),
        engine.as_ref()
    ));
    assert_eq!(SourceActions::store(&actions).table(), "profile");
    SourceActions::install(&actions).await.unwrap();
    SourceActions::upgrade(&actions, None).await.unwrap();
    SourceActions::upgrade(&actions, Some(&actions.store.create()))
        .await
        .unwrap();
    let original = SourceActions::create(
        &actions,
        &record(serde_json::json!({"id":"p","name":"Ada"})),
    )
    .await
    .unwrap();
    assert_eq!(original["id"], "p");
    let query = StoreQuery::eq("id", "p");
    assert_eq!(
        SourceActions::find(&actions, &query)
            .await
            .unwrap()
            .unwrap()["name"],
        "Ada"
    );
    let mut transaction = engine.begin().await.unwrap();
    {
        let mut db = DataSession::transaction(&mut transaction);
        SourceActions::create_in(
            &actions,
            &mut db,
            &record(serde_json::json!({"id":"rolled-back","name":"Transient"})),
        )
        .await
        .unwrap();
        assert_eq!(
            SourceActions::count_in(&actions, &mut db, &StoreQuery::default())
                .await
                .unwrap(),
            2
        );
        assert_eq!(
            SourceActions::find_all_in(&actions, &mut db, &StoreQuery::default())
                .await
                .unwrap()
                .len(),
            2
        );
    }
    transaction.rollback().await.unwrap();
    assert_eq!(
        SourceActions::count(&actions, &StoreQuery::default())
            .await
            .unwrap(),
        1
    );
    let mut patch = record(serde_json::json!({"name":"Grace"}));
    assert_eq!(
        SourceActions::update(&actions, &query, &mut patch)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        SourceActions::find_all(&actions, &query).await.unwrap()[0]["name"],
        "Grace"
    );
    assert_eq!(
        SourceActions::remove(&actions, &query).await.unwrap().len(),
        1
    );
    assert_eq!(
        SourceActions::count(&actions, &StoreQuery::default())
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        SourceActions::restore(&actions, &query.clone().all())
            .await
            .unwrap()
            .len(),
        1
    );
    let mut missing = record(serde_json::json!({"id":"nonexistent","name":"No creation"}));
    assert!(
        SourceActions::upsert(&actions, &mut missing)
            .await
            .unwrap()
            .is_none()
    );
    let statuses = SourceActions::batch(
        &actions,
        &mut [record(serde_json::json!({"id":"batch","name":"Batch"}))],
        BatchMode::Create,
    )
    .await;
    assert_eq!(statuses[0].code, Some(200));
    assert_eq!(
        SourceActions::delete(&actions, &StoreQuery::eq("id", "batch"))
            .await
            .unwrap()
            .len(),
        1
    );
    SourceActions::purge(&actions, false).await.unwrap();
    assert_eq!(
        SourceActions::count(&actions, &StoreQuery::default().all())
            .await
            .unwrap(),
        0
    );
    SourceActions::uninstall(&actions).await.unwrap();
    engine.close().await.unwrap();
}
