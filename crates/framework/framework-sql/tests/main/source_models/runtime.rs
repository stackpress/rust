//! Exercise the shared source runtime before migrating plugin model bindings.
use stackpress_framework_schema::{
    Model, SchemaResult, SourceCatalog,
    source::{Record, SourceRecordSchema, SourceSchema, Value},
};
use stackpress_framework_sql::source::{NativeActions, NativeStore, StoreQuery};
use stackpress_sql::{DialectId, Render};
use std::sync::Arc;

#[path = "../../support/g54.rs"]
mod g54;

/// A real composed schema delegates all mechanics to the same descriptor owner as emitted code.
macro_rules! schema {
    ($name:ident, $model:literal) => {
        struct $name(SourceRecordSchema);
        impl SourceSchema for $name {
            type Value = Value;
            type Record = Record;
            fn new(seed: String, offset: i32) -> Self {
                Self(SourceRecordSchema::from_metadata(
                    &Self::describe(),
                    Arc::new(Self::catalog()),
                    seed,
                    offset,
                ))
            }
            fn describe() -> Model {
                Self::catalog()
                    .models
                    .into_iter()
                    .find(|m| m.name == $model)
                    .unwrap()
            }
            fn catalog() -> SourceCatalog {
                super::source_schema(true).catalog().unwrap()
            }
            fn seed(&self) -> &str {
                self.0.seed()
            }
            fn offset_minutes(&self) -> i32 {
                self.0.offset_minutes()
            }
            fn defaults_at(&self, now: i64) -> SchemaResult<Record> {
                self.0.defaults_at(now)
            }
            fn defaults(&self) -> SchemaResult<Record> {
                self.0.defaults()
            }
            fn filter(&self, input: &Record, populate: bool) -> SchemaResult<Record> {
                self.0.filter(input, populate)
            }
            fn populate(&self, input: &Record) -> SchemaResult<Record> {
                self.0.populate(input)
            }
            fn assert(&self, input: &Record, required: bool) -> Option<Record> {
                self.0.assert(input, required)
            }
            fn serialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
                self.0.serialize(input, storage)
            }
            fn unserialize(&self, input: &Record, storage: bool) -> SchemaResult<Record> {
                self.0.unserialize(input, storage)
            }
        }
    };
}
schema!(Profile, "Profile");
schema!(Auth, "Auth");

/// Convert authored JSON without losing the native record's insertion order.
fn record(value: serde_json::Value) -> Record {
    Value::from(value).as_object().unwrap().clone()
}

/// Cross-model queries retain native Dates, insertion order and absent versus null q.
#[test]
fn source_query_family_transfer_preserves_native_values() {
    for q in [None, Some(Value::Null), Some("Ada".into())] {
        let mut query = StoreQuery {
            q: q.clone(),
            ..Default::default()
        };
        query.eq.insert(
            "updated".into(),
            stackpress_framework_schema::source::date(1234),
        );
        query.eq.insert("name".into(), "Ada".into());
        let input = query.to_record();
        assert_eq!(input.contains_key("q"), q.is_some());
        let decoded = StoreQuery::from_record(&input).unwrap();
        assert_eq!(decoded.q, q);
        assert_eq!(
            decoded.eq.keys().map(String::as_str).collect::<Vec<_>>(),
            ["updated", "name"]
        );
        assert!(matches!(
            decoded.eq["updated"],
            Value::Date { millis: 1234, .. }
        ));
    }
}

/// Nested source processes retain Date objects; ordinary JSON events keep their existing wire form.
#[cfg(feature = "server")]
#[tokio::test]
async fn source_event_response_boundary_is_explicit_and_keeps_json_default_unchanged() {
    use stackpress_framework_sql::source::{ResponseValues, SourceModel};
    use stackpress_lib::{Flow, Request, Response};
    use stackpress_server::{ServerBuilder, StartupContext};
    struct Model;
    impl SourceModel for Model {
        type Role = ();
        type Schema = Profile;
        type Store = NativeStore<Profile>;
        type Actions = NativeActions<Profile>;
    }
    let directory = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(directory.path().join("response.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(database));
    let actions = Model::actions(engine.clone(), String::new(), 0);
    actions.install().await.unwrap();
    let mut input = record(serde_json::json!({"id":"p","name":"Ada"}));
    input.insert(
        "created".into(),
        stackpress_framework_schema::source::date(0),
    );
    actions.create(&input).await.unwrap();
    for native in [false, true] {
        let mut app = ServerBuilder::new();
        let owned = engine.clone();
        app.on_config(stackpress_lib::queue::task(
            move |start: &mut StartupContext| {
                let engine = owned.clone();
                Box::pin(async move {
                    start.register(engine)?;
                    Ok(Flow::Continue)
                })
            },
        ))
        .unwrap();
        if native {
            stackpress_framework_sql::source::register_with::<Model>(
                &mut app,
                ResponseValues::Native,
            )
            .unwrap();
        } else {
            // The existing API remains JSON without a new opt-in option or hidden default.
            stackpress_framework_sql::source::register::<Model>(&mut app).unwrap();
        }
        let host = app.build().await.unwrap();
        let mut request = Request::builder()
            .data(serde_json::json!({"id":"p"}))
            .build()
            .unwrap();
        let mut response = Response::new();
        host.emit("profile-detail", &mut request, &mut response)
            .await
            .unwrap();
        let created = &response.result().unwrap().as_object().unwrap()["created"];
        if native {
            assert!(matches!(created, stackpress_lib::Value::DateTime(_)));
            assert!(
                response.try_to_json().is_err(),
                "the host must select Date.toJSON explicitly"
            );
        } else {
            assert_eq!(created.as_str(), Some("1970-01-01T00:00:00.000Z"));
            assert!(response.try_to_json().is_ok());
        }
    }
    engine.close().await.unwrap();
}

#[test]
fn source_query_boundary_sort_and_terminal_paths() {
    let store = NativeStore::<Profile>::new("seed".into(), 0);
    let mut query = StoreQuery::from_record(&record(serde_json::json!({
        "columns": [], "skip": 2.0, "take": false,
        "sort": {"references:score":"dEsC", "name":"ASC", "id":"sideways", "image":3},
        "eq": {"active":-1, "missing":"ignored", "name":""},
        "ne": {"nickname":null}, "ge":{"references:score":2}, "le":{"references:score":9}
    })))
    .unwrap();
    assert!(query.columns.is_empty());
    assert_eq!((query.skip, query.take), (2, 0));
    // The carrier preserves an explicit empty selection; the SQL owner rejects it.
    assert!(
        store
            .select(&query, DialectId::Sqlite)
            .unwrap()
            .render(DialectId::Sqlite)
            .is_err()
    );
    query.columns = vec!["name".into()];
    let sql = store
        .select(&query, DialectId::Sqlite)
        .unwrap()
        .render(DialectId::Sqlite)
        .unwrap();
    assert!(sql.sql().contains("DESC"));
    assert!(sql.sql().contains("ASC"));
    assert!(!sql.sql().contains("active"));
    assert!(!sql.sql().contains("missing"));
    assert_eq!(StoreQuery::from_record(&Record::new()).unwrap().take, 50);
    for bad in [
        serde_json::json!(-1),
        serde_json::json!(0.5),
        serde_json::json!("2"),
        serde_json::json!(true),
    ] {
        let error = StoreQuery::from_record(&record(serde_json::json!({"skip":bad}))).unwrap_err();
        assert_eq!(error.code, 500);
    }
    let auth = NativeStore::<Auth>::new("seed".into(), 0);
    assert_eq!(auth.paths("profile.name")[0].model, "Profile");
    assert_eq!(auth.paths("profile.unknown")[0].field, "profile");
    assert_eq!(auth.paths("profile")[0].field, "profile");
    assert!(
        store.paths("auth.*").is_empty(),
        "reverse lists do not synthesize stored joins"
    );
    assert_eq!(store.selectors(&["name".into(), "name".into()]).len(), 1);
}

#[tokio::test]
async fn source_runtime_real_sqlite_ordered_json_queries_and_mutations() {
    let dir = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("source.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let actions = NativeActions::new(
        Arc::new(stackpress_sql::Engine::new(database)),
        NativeStore::<Profile>::new("seed".into(), 0),
    );
    actions.install().await.unwrap();
    let input = record(
        serde_json::json!({"id":"p1","name":"Ada","nickname":"Lovelace","references":{"z":2,"a":1}}),
    );
    actions.create(&input).await.unwrap();
    let mut query = StoreQuery::default();
    query.like.insert("nickname".into(), "LOVE".into());
    query.sort.insert("references:z".into(), "desc".into());
    let rows = actions.find_all(&query).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["nickname"], "Lovelace");
    assert_eq!(
        rows[0]["references"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["z", "a"]
    );
    let mut changed = record(serde_json::json!({"nickname":"Countess"}));
    assert_eq!(
        actions
            .update(&StoreQuery::eq("id", "p1"), &mut changed)
            .await
            .unwrap()
            .len(),
        1
    );
    actions.remove(&StoreQuery::eq("id", "p1")).await.unwrap();
    assert_eq!(actions.count(&StoreQuery::default()).await.unwrap(), 0);
    assert_eq!(
        actions.count(&StoreQuery::default().all()).await.unwrap(),
        1
    );
    actions
        .restore(&StoreQuery::eq("id", "p1").all())
        .await
        .unwrap();
    assert_eq!(actions.count(&StoreQuery::default()).await.unwrap(), 1);
    assert!(
        actions
            .upsert(&mut record(
                serde_json::json!({"id":"missing","name":"Grace"})
            ))
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        actions
            .delete(&StoreQuery::eq("id", "p1"))
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        actions
            .find(&StoreQuery::eq("id", "p1"))
            .await
            .unwrap()
            .is_none()
    );
    actions.uninstall().await.unwrap();
}

#[test]
fn source_scalar_codecs_keep_missing_null_and_invalid_shapes_distinct() {
    let store = NativeStore::<Profile>::new("seed".into(), 0);
    assert!(
        store
            .scalarize(&record(serde_json::json!({"notDeclared":"ignored"})))
            .unwrap()
            .is_empty()
    );
    let row = record(serde_json::json!({"id":"p", "references":"{\"b\":2,\"a\":1}", "image":null}));
    let decoded = store.unscalarize(&row).unwrap();
    assert_eq!(decoded["image"], Value::Null);
    assert!(!decoded.contains_key("name"));
    assert_eq!(
        decoded["references"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["b", "a"]
    );
}

#[test]
fn source_unique_indexes_precede_ordinary_indexes_for_every_table() {
    let store = NativeStore::<Auth>::new("seed".into(), 0);
    let definition = store.create().definition().unwrap();
    assert_eq!(
        definition.indexes.first().unwrap().name,
        "auth_token_unique"
    );
    let mut ordinary = false;
    for index in &definition.indexes {
        if !index.unique {
            ordinary = true;
        }
        assert!(
            !ordinary || !index.unique,
            "source unique grouping precedes ordinary indexes"
        );
    }
}

/// G53 keeps scalar has SQL unchanged while requiring every requested list member.
#[tokio::test]
async fn g53_has_all_values_preserves_single_value_sql_on_every_dialect() {
    let store = NativeStore::<Profile>::new(String::new(), 0);
    let query = |value| StoreQuery {
        has: record(serde_json::json!({"tags":value})),
        ..Default::default()
    };
    let single = query(serde_json::json!("red"));
    let one = query(serde_json::json!(["red"]));
    let both = query(serde_json::json!(["red", "blue"]));
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        let scalar = store
            .select(&single, dialect)
            .unwrap()
            .render(dialect)
            .unwrap();
        let array = store
            .select(&one, dialect)
            .unwrap()
            .render(dialect)
            .unwrap();
        let all = store
            .select(&both, dialect)
            .unwrap()
            .render(dialect)
            .unwrap();
        assert_eq!(scalar.sql(), array.sql());
        assert_eq!(scalar.binds(), array.binds());
        assert!(!all.sql().contains(" OR "), "{}", all.sql());
        assert!(!all.sql().contains("\"tags\".\"0\"") && !all.sql().contains("`tags`.`0`"));
        assert_eq!(all.binds().iter().filter(|v| matches!(v, stackpress_sql::SqlValue::Text(s) if s.contains("red") || s.contains("blue"))).count(), 2);
        println!(
            "G53 {dialect:?} single: {}; all: {}",
            scalar.sql(),
            all.sql()
        );
    }
    let directory = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(directory.path().join("has.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(database));
    let actions = NativeActions::new(engine.clone(), store);
    actions.install().await.unwrap();
    for (id, tags) in [
        ("red", serde_json::json!(["red"])),
        ("blue", serde_json::json!(["blue"])),
        ("both", serde_json::json!(["red", "blue"])),
    ] {
        actions
            .create(&record(serde_json::json!({"id":id,"name":id,"tags":tags})))
            .await
            .unwrap();
    }
    assert_eq!(actions.count(&single).await.unwrap(), 2);
    assert_eq!(actions.count(&one).await.unwrap(), 2);
    assert_eq!(actions.find_all(&both).await.unwrap()[0]["id"], "both");
    assert_eq!(actions.count(&both).await.unwrap(), 1);
    assert_eq!(
        actions
            .count(&query(serde_json::json!(["red", "missing"])))
            .await
            .unwrap(),
        0
    );
    engine.close().await.unwrap();
}

/// G54 hasnt uses one negative member predicate per value, retaining NULL/absent paths.
#[tokio::test]
async fn g54_hasnt_none_values_and_absence_on_real_sqlite() {
    let store = NativeStore::<Profile>::new(String::new(), 0);
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        let compile = |value| {
            store
                .select(
                    &StoreQuery {
                        hasnt: record(serde_json::json!({"tags":value})),
                        ..Default::default()
                    },
                    dialect,
                )
                .unwrap()
                .render(dialect)
                .unwrap()
        };
        let single = compile(serde_json::json!("red"));
        let one = compile(serde_json::json!(["red"]));
        assert_eq!(single.sql(), one.sql());
        assert_eq!(single.binds(), one.binds());
        let both = compile(serde_json::json!(["red", "blue"]));
        assert_eq!(single.sql().matches("NOT (").count(), 1);
        assert_eq!(both.sql().matches("NOT (").count(), 2);
        assert_eq!(both.sql().matches("IS NULL").count(), 2);
        assert!(!both.sql().contains("\"tags\".\"0\"") && !both.sql().contains("`tags`.`0`"));
        assert!(both.sql().contains(" AND "));
        println!("G54 {dialect:?}: {}", both.sql());
    }
    let directory = tempfile::tempdir().unwrap();
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(directory.path().join("hasnt.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(database));
    g54::prove(NativeActions::new(engine.clone(), store))
        .await
        .unwrap();
    engine.close().await.unwrap();
}
