//! Maintained generated app extension/collision fixture; source files are regenerated, never edited.
#![cfg(all(feature = "sql", not(target_arch = "wasm32")))]
#[allow(missing_docs)]
#[path = "extended/generated/mod.rs"]
pub mod extended;
use serde_json::json;
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
use stackpress_framework_sql::source::{ApiModels, SessionModels, SourceModel, StoreQuery};
use std::sync::Arc;
fn record(value: serde_json::Value) -> Record {
    serde_json::from_value(value).unwrap()
}

#[test]
fn extended_family_and_namespace_types_compile_and_retain_the_complete_catalog() {
    fn family<M: ApiModels>() {}
    family::<extended::stores::_family::Models>();
    fn session<M: SessionModels>() {}
    session::<extended::stores::_family::Models>();
    let _: Option<extended::stores::native_record::NativeRecordStore> = None;
    let _: Option<extended::stores::runtime::RuntimeStore> = None;
    let _: Option<extended::stores::source::SourceStore> = None;
    let _: Option<extended::stores::_stores::StoresStore> = None;
    let _: Option<extended::stores::family::FamilyStore> = None;
    let _: Option<extended::stores::native::NativeStore> = None;
    let catalog = extended::source::_catalog::describe();
    assert_eq!(catalog.models.len(), 14);
    assert_eq!(catalog.enums.len(), 1);
    assert_eq!(catalog.fieldsets.len(), 1);
    let schema = extended::source::profile::ProfileSchema::new("seed".into(), 0);
    let defaults = schema.defaults_at(0).unwrap();
    assert_eq!(defaults["nickname"], "application");
    assert_eq!(defaults["mood"], "on");
}

#[test]
fn fieldset_list_default_is_the_source_nested_object_and_scalar_lists_have_none() {
    let schema = extended::source::lists::ListsSchema::new(String::new(), 0);
    // Upstream column/index.ts:312–315 returns _fieldset.defaults even for Address[].
    // Its Address fields have no defaults, so the defined value is {}, never [].
    let defaults = schema.defaults_at(0).unwrap();
    assert_eq!(defaults, record(json!({"addresses": {}})));
}

#[tokio::test]
async fn emitted_app_extension_persists_and_unique_rule_keeps_its_original_message() {
    use extended::stores::profile::ProfileModel;
    let dir = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("extended.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(db));
    let profile = ProfileModel::actions(engine, "seed".into(), 0);
    let ddl = profile.store.create().definition().unwrap();
    assert!(
        !ddl.indexes
            .iter()
            .any(|index| index.unique && index.columns == ["nickname"]),
        "is.unique does not add a source @unique SQL constraint"
    );
    profile.install().await.unwrap();
    profile
        .create(&record(json!({"id":"p","name":"Ada"})))
        .await
        .unwrap();
    let error = profile
        .create(&record(json!({"id":"q","name":"Bob"})))
        .await
        .unwrap_err();
    assert_eq!(error.code, 400);
    assert_eq!(error.errors["nickname"], "Already exists.");
    let mut patch = record(json!({"nickname":"changed"}));
    profile
        .update(&StoreQuery::eq("id", "p"), &mut patch)
        .await
        .unwrap();
    assert_eq!(
        profile
            .find(&StoreQuery::eq("id", "p"))
            .await
            .unwrap()
            .unwrap()["nickname"],
        "changed"
    );
    // Wrong enum input is never silently repaired into a source choice.
    let error = profile
        .create(&record(
            json!({"id":"r","name":"Other","nickname":"other","mood":"bad"}),
        ))
        .await
        .unwrap_err();
    assert_eq!(
        error.errors["mood"],
        "Invalid option: expected one of \"on\"|\"off\""
    );
}

#[test]
fn source_unknown_and_every_list_element_family_emit_without_hidden_storage() {
    use extended::source::catalog::CatalogSchema;
    use extended::source::lists::ListsSchema;
    let schema = CatalogSchema::new(String::new(), 0);
    let input = record(json!({"unknowns":[1,null,{"a":true},[2]]}));
    let encoded = schema.serialize(&input, false).unwrap();
    assert_eq!(encoded["unknowns"], "[1,{\"a\":true},[2]]");
    let store = extended::stores::catalog::CatalogStore::new(String::new(), 0);
    assert!(
        !store
            .create()
            .definition()
            .unwrap()
            .fields
            .iter()
            .any(|f| f.name == "unknowns")
    );
    let ddl = extended::stores::lists::ListsStore::new(String::new(), 0)
        .create()
        .definition()
        .unwrap();
    assert_eq!(
        ddl.fields
            .iter()
            .find(|f| f.name == "whole")
            .unwrap()
            .data_type,
        stackpress_sql::SqlType::Integer
    );
    assert_eq!(
        ddl.fields
            .iter()
            .find(|f| f.name == "fractional")
            .unwrap()
            .data_type,
        stackpress_sql::SqlType::Real
    );
    let schema = ListsSchema::new(String::new(), 0);
    let input = record(
        json!({"numbers":[1,2],"floats":[1,2],"integers":[1,2],"flags":[false,true],"objects":[{"x":1}],"dates":[0],"moods":["on"],"addresses":[{"label":"ok","zip":1}]}),
    );
    let encoded = schema.serialize(&input, false).unwrap();
    let decoded = schema.unserialize(&encoded, false).unwrap();
    assert_eq!(decoded["numbers"], json!([1, 2]));
    assert_eq!(decoded["flags"], json!([false, true]));
    assert_eq!(
        decoded["addresses"],
        json!([]),
        "original schema wraps fieldset lists twice"
    );
    assert!(schema.assert(&decoded, false).is_none());
    let _ = Value::Null;
}

#[tokio::test]
async fn source_no_active_model_removal_is_physical_and_borrowed_records_are_owned() {
    use extended::stores::native::NativeModel;
    use stackpress_framework_sql::{DataSession, source::SourceActions};
    let dir = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("borrowed.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(db));
    let actions = NativeModel::actions(engine.clone(), String::new(), 0);
    actions.install().await.unwrap();
    let mut transaction = engine.begin().await.unwrap();
    let row = {
        let mut db = DataSession::transaction(&mut transaction);
        let input = record(json!({"id":"borrowed"}));
        fn send<T: Send>(future: T) -> T {
            future
        }
        send(SourceActions::create_in(&actions, &mut db, &input))
            .await
            .unwrap()
    };
    transaction.commit().await.unwrap();
    assert_eq!(row["id"], "borrowed");
    assert_eq!(
        actions
            .remove(&StoreQuery::eq("id", "borrowed"))
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(actions.count(&StoreQuery::default()).await.unwrap(), 0);
}

#[test]
fn direct_source_store_query_uses_native_paging_errors_and_exact_q_values() {
    use stackpress_framework_sql::source::SourceStore;
    let store = extended::stores::profile::ProfileStore::new(String::new(), 0);
    for value in [json!(-1), json!(0.5), json!("5")] {
        assert_eq!(
            store
                .query(&record(json!({"take":value})))
                .unwrap_err()
                .error,
            "paging cannot be represented as a native nonnegative integer"
        );
    }
    let query = store
        .query(&record(json!({"q":[],"take":5.0,"skip":null})))
        .unwrap();
    assert_eq!(query.take, 5);
    assert_eq!(query.skip, 0);
    assert_eq!(query.q, Some(Value::Array(Vec::new())));
}

#[tokio::test]
async fn batch_null_diagnostics_follow_the_actual_model_selectors() {
    use stackpress_framework_sql::source::BatchMode;
    let dir = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("null.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(db));
    let keyed = extended::stores::keyed::KeyedModel::actions(engine.clone(), String::new(), 0);
    let keyless = extended::stores::keyless::KeylessModel::actions(engine, String::new(), 0);
    keyed.install().await.unwrap();
    keyless.install().await.unwrap();
    for mode in [BatchMode::Update, BatchMode::Upsert] {
        let status = keyed.batch_values(&mut [Value::Null], mode).await;
        assert_eq!(
            status[0].error.as_deref(),
            Some("Cannot read properties of null (reading 'key')")
        );
    }
    let status = keyless
        .batch_values(&mut [Value::Null], BatchMode::Update)
        .await;
    assert_eq!(status[0].code, None);
    assert_eq!(
        status[0].error.as_deref(),
        Some("ID or unique field is required for update mode")
    );
    for mode in [BatchMode::Create, BatchMode::Upsert] {
        let status = keyless.batch_values(&mut [Value::Null], mode).await;
        assert_eq!(
            status[0].error.as_deref(),
            Some("Cannot convert undefined or null to object")
        );
    }
}

/// A finite non-returning provider isolates the original MySQL input-fallback branch.
struct NonReturning;
impl stackpress_sql::Database for NonReturning {
    fn dialect(&self) -> stackpress_sql::DialectId {
        stackpress_sql::DialectId::Mysql
    }
    fn run(
        &self,
        _: stackpress_sql::Query,
        _: stackpress_sql::QueryLimits,
    ) -> stackpress_sql::SqlFuture<'_, stackpress_sql::QueryResult> {
        Box::pin(async { Ok(stackpress_sql::QueryResult::default()) })
    }
    fn begin(
        &self,
        _: stackpress_sql::TransactionOptions,
    ) -> stackpress_sql::SqlFuture<'_, Box<dyn stackpress_sql::TransactionSession>> {
        Box::pin(async {
            Ok(Box::new(NonReturning) as Box<dyn stackpress_sql::TransactionSession>)
        })
    }
    fn close(&self) -> stackpress_sql::SqlFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
impl stackpress_sql::TransactionSession for NonReturning {
    fn run(
        &mut self,
        _: stackpress_sql::Query,
        _: stackpress_sql::QueryLimits,
    ) -> stackpress_sql::SqlFuture<'_, stackpress_sql::QueryResult> {
        Box::pin(async { Ok(stackpress_sql::QueryResult::default()) })
    }
    fn commit(self: Box<Self>) -> stackpress_sql::SqlFuture<'static, ()> {
        Box::pin(async { Ok(()) })
    }
    fn rollback(self: Box<Self>) -> stackpress_sql::SqlFuture<'static, ()> {
        Box::pin(async { Ok(()) })
    }
}
#[tokio::test]
async fn dynamic_batch_mysql_fallback_retains_original_primitive_values() {
    use stackpress_framework_sql::source::BatchMode;
    let actions = extended::stores::keyless::KeylessModel::actions(
        Arc::new(stackpress_sql::Engine::new(NonReturning)),
        String::new(),
        0,
    );
    for mode in [BatchMode::Create, BatchMode::Upsert] {
        let original = vec![
            Value::from("boxed"),
            Value::from(42),
            Value::from(false),
            stackpress_framework_schema::source::date(42),
        ];
        let mut inputs = original.clone();
        let statuses = actions.batch_values(&mut inputs, mode).await;
        assert_eq!(inputs, original);
        for (status, value) in statuses.iter().zip(original) {
            assert_eq!(status.code, Some(200));
            assert_eq!(status.results, Some(value));
        }
    }
}
