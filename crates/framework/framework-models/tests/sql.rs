//! Generated native stores execute on an actual in-process SQLite file.
#![cfg(all(feature = "sql", not(target_arch = "wasm32")))]
#[path = "support/scenario.rs"]
mod scenario;
use std::sync::Arc;
#[tokio::test]
async fn generated_four_model_sqlite_search_actions_setup_and_purge() {
    let dir = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("generated.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    scenario::scenario(Arc::new(stackpress_sql::Engine::new(db)))
        .await
        .unwrap();
}

#[tokio::test]
async fn generated_source_scalar_returning_and_query_oracles() {
    use stackpress_framework_models::generated::stores::{auth::AuthModel, profile::ProfileModel};
    use stackpress_framework_schema::source::{Record, Value, date};
    use stackpress_framework_sql::source::{SourceModel, StoreQuery};
    use stackpress_sql::{DialectId, Render, SqlValue};
    let dir = tempfile::tempdir().unwrap();
    let db = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(dir.path().join("oracles.sqlite"))
            .create_if_missing(true),
    )
    .await
    .unwrap();
    let engine = Arc::new(stackpress_sql::Engine::new(db));
    let profile = ProfileModel::actions(engine.clone(), "database-seed".into(), 0);
    let auth = AuthModel::actions(engine.clone(), "database-seed".into(), 0);
    profile.install().await.unwrap();
    auth.install().await.unwrap();
    profile
        .create(&scenario::record(
            serde_json::json!({"id":"profile-1","name":"Ada"}),
        ))
        .await
        .unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../framework-session/tests/records.json")).unwrap();
    let mut input = scenario::record(
        serde_json::json!({"id":"record-1","name":"Ada","profileId":"profile-1","token":"user@example.test","secret":"hello","type":"email","roles":["USER"],"tags":["tag"],"references":{"fbid":"abc"},"verified":true,"active":true}),
    );
    for name in ["created", "updated", "consumed"] {
        input.insert(name.into(), date(1700000000123));
    }
    let raw = |values: Vec<(String, SqlValue)>| {
        Value::Object(
            values
                .into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        match v {
                            SqlValue::Text(v) => Value::String(v),
                            SqlValue::Bool(v) => Value::Bool(v),
                            _ => panic!("golden uses text/bool native cells"),
                        },
                    )
                })
                .collect::<Record>(),
        )
    };
    assert_eq!(
        raw(profile.store.scalarize(&input).unwrap()),
        fixture["scalar"]["Profile"]
    );
    assert_eq!(
        raw(auth.store.scalarize(&input).unwrap()),
        fixture["scalar"]["Auth"]
    );
    let result = auth.create(&input).await.unwrap();
    assert_eq!(result["token"], fixture["authReturned"]["token"]);
    assert_eq!(result["secret"], fixture["authReturned"]["secret"]);
    assert!(!result.contains_key("profileId"));
    assert_eq!(
        auth.find(&StoreQuery::eq("id", "record-1"))
            .await
            .unwrap()
            .unwrap()["profileId"],
        "profile-1"
    );
    for name in ["search", "has", "hasnt"] {
        let query = match name {
            "search" => StoreQuery {
                q: Some("ad".into()),
                ..Default::default()
            },
            "has" => StoreQuery {
                has: scenario::record(serde_json::json!({"tags":["red","blue"]})),
                ..Default::default()
            },
            _ => StoreQuery {
                hasnt: scenario::record(serde_json::json!({"tags":["red","blue"]})),
                ..Default::default()
            },
        };
        let rendered = profile
            .store
            .select(&query, DialectId::Sqlite)
            .unwrap()
            .render(DialectId::Sqlite)
            .unwrap();
        let binds = rendered
            .binds()
            .iter()
            .map(|v| match v {
                SqlValue::Bool(v) => serde_json::json!(v),
                SqlValue::Text(v) => serde_json::json!(v),
                _ => panic!("source query oracle scalar"),
            })
            .collect::<Vec<_>>();
        let expected = fixture["sourceQueries"][name]["values"].as_array().unwrap();
        let expected = if name == "search" {
            expected.clone()
        } else {
            expected
                .iter()
                .take(1)
                .cloned()
                .chain(
                    expected
                        .iter()
                        .skip(1)
                        .flat_map(|value| [serde_json::json!("$"), value.clone()]),
                )
                .collect()
        };
        // The accepted Rust JSON AST binds the fixed path instead of interpolating '$'.
        assert_eq!(binds, expected);
        if name == "hasnt" {
            assert!(rendered.sql().contains("json_each(\"tags\".\"0\""));
            assert!(
                engine.fetch(&rendered).await.is_err(),
                "source flatten(...,true) retains the invalid indexed-column path"
            );
        } else if name == "has" {
            // G53 deliberately fixes has only; the original golden remains unchanged.
            assert!(!rendered.sql().contains("json_each(\"tags\".\"0\""));
            assert!(engine.fetch(&rendered).await.is_ok());
        }
    }
    // A relation-ending selector is emitted faithfully; it is not changed into a wildcard.
    let query = StoreQuery {
        columns: vec!["profile".into()],
        ..Default::default()
    };
    assert!(auth.find_all(&query).await.is_err());
}
