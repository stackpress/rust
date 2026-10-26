//! Explicit setup, startup validation and source-preserving failures use isolated native files.

use std::error::Error;

use serde_json::json;
use stackpress_example_sql_profile::{build_application, database, profiles};
use stackpress_lib::Error as HostError;
use stackpress_server::{ConfigLayers, PluginSpec, ServerBuilder, StartupStage};
use stackpress_sql::{DialectId, Engine, ErrorKind, Select, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use tempfile::TempDir;

use super::{close, layers, writable};

/// Construct static fixture SQL through the same audited single-statement gate as consumers.
fn sql(text: &'static str) -> Sql {
    Sql::literal(DialectId::Sqlite, text)
}

/// Installation is an explicit atomic one-shot action, including rollback when a later index fails.
pub(crate) async fn setup() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    let database = writable(&path).await;
    let rows = database
        .fetch(&sql(
            "SELECT ordinal, id, name FROM profile ORDER BY ordinal",
        ))
        .await
        .unwrap();
    assert_eq!(rows.len(), 3);
    for (index, (id, name)) in [
        ("abc123", "Ada Lovelace"),
        ("abc456", "Grace Hopper"),
        ("abc789", "Margaret Hamilton"),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(rows[index].try_get::<i64>("ordinal").unwrap(), index as i64);
        assert_eq!(rows[index].try_get::<String>("id").unwrap(), id);
        assert_eq!(rows[index].try_get::<String>("name").unwrap(), name);
    }
    database.close().await.unwrap();
    let error = stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    let database = writable(&path).await;
    assert_eq!(
        database
            .fetch(&sql(
                "SELECT ordinal, id, name FROM profile ORDER BY ordinal"
            ))
            .await
            .unwrap(),
        rows
    );
    database.close().await.unwrap();

    // An existing unrelated index makes statement two fail after CREATE TABLE succeeded.
    let other = root.path().join("collision.sqlite");
    let database = Engine::new(
        SqliteDatabase::open(SqliteOptions::file(&other).create_if_missing(true))
            .await
            .unwrap(),
    );
    database
        .execute(&sql("CREATE TABLE unrelated(value TEXT)"))
        .await
        .unwrap();
    database
        .execute(&sql("CREATE INDEX profile_id ON unrelated(value)"))
        .await
        .unwrap();
    let inspect = sql("SELECT type, name, sql FROM sqlite_master ORDER BY type, name");
    let before = database.fetch(&inspect).await.unwrap();
    database.close().await.unwrap();
    let error = stackpress_example_sql_profile::setup::install_and_seed(&other)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(error.source().is_some());
    assert!(error.cleanup_error().is_none());
    let database = writable(&other).await;
    assert_eq!(database.fetch(&inspect).await.unwrap(), before);
    database.close().await.unwrap();
    assert!(
        stackpress_example_sql_profile::setup::install_and_seed(
            root.path().join("missing-parent/file.sqlite")
        )
        .await
        .is_err()
    );

    // Competing explicit installers must never merge/reseed partial installations. The
    // native schema transaction admits one winner; the loser retains its failure source.
    let concurrent = root.path().join("concurrent.sqlite");
    let (left, right) = tokio::join!(
        stackpress_example_sql_profile::setup::install_and_seed(&concurrent),
        stackpress_example_sql_profile::setup::install_and_seed(&concurrent)
    );
    let loser = match (left, right) {
        (Ok(()), Err(error)) | (Err(error), Ok(())) => error,
        results => panic!("expected exactly one installer: {results:?}"),
    };
    assert!(loser.source().is_some());
    let database = writable(&concurrent).await;
    assert_eq!(
        database
            .fetch(&sql(
                "SELECT ordinal, id, name FROM profile ORDER BY ordinal"
            ))
            .await
            .unwrap(),
        rows
    );
    database.close().await.unwrap();
}

/// Startup opens only existing valid schema, without creating files, installing tables or seeding rows.
pub(crate) async fn startup() {
    let root = TempDir::new().unwrap();
    let missing = root.path().join("missing.sqlite");
    let error = build_application(layers(&missing, 3)).await.err().unwrap();
    assert_eq!(error.stage, StartupStage::Config);
    assert!(error.error.source().is_some());
    assert!(!missing.exists());
    let empty = root.path().join("empty.sqlite");
    let database = Engine::new(
        SqliteDatabase::open(SqliteOptions::file(&empty).create_if_missing(true))
            .await
            .unwrap(),
    );
    database.close().await.unwrap();
    assert!(build_application(layers(&empty, 3)).await.is_err());
    let database = writable(&empty).await;
    assert!(
        database
            .fetch(&sql("SELECT name FROM sqlite_master WHERE type = 'table'"))
            .await
            .unwrap()
            .is_empty()
    );
    database
        .execute(&sql("CREATE TABLE profile(id TEXT)"))
        .await
        .unwrap();
    database.close().await.unwrap();
    // Qualified startup selectors reject missing ordinal/name instead of SQLite string fallback.
    assert!(build_application(layers(&empty, 3)).await.is_err());

    let valid = root.path().join("valid.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&valid)
        .await
        .unwrap();
    for value in [
        json!({"profiles":{"limit":0}}),
        json!({"profiles":{"limit":101}}),
        json!({"profiles":{"limit":-1}}),
        json!({"profiles":{"limit":"two"}}),
        json!({"profiles":{"limti":1}}),
        json!({"sqlite":{"path":""}}),
        json!({"sqlite":{"create_if_missing":true}}),
        json!({"sqlite":{"read_only":false}}),
        json!({"sqlite":{"path":"file:unsafe"}}),
        json!({"unknown-plugin":{}}),
    ] {
        let mut config = layers(&valid, 3);
        config.value("invalid", value).unwrap();
        assert!(build_application(config).await.is_err());
    }
    // A profiles selection cannot silently create its own missing database prerequisite.
    let mut builder = ServerBuilder::new();
    builder
        .plugin_with(
            PluginSpec::new("profiles").requires(&["database"]),
            profiles::register,
        )
        .unwrap();
    assert_eq!(
        builder.build().await.err().unwrap().stage,
        StartupStage::Registration
    );
    // The database plugin alone registers a service but exposes no business or SQL capability.
    let mut builder = ServerBuilder::new();
    builder.config_layers(layers_database(&valid)).unwrap();
    builder.plugin("database", database::register).unwrap();
    let app = builder.build().await.unwrap();
    let reply = app
        .call("profile-search", json!({"id":"abc"}))
        .await
        .unwrap();
    assert_eq!(reply.response.try_to_json().unwrap()["code"], 404);
    close(app).await;
}

/// Supply only the section selected by a database-only host composition.
fn layers_database(path: &std::path::Path) -> ConfigLayers {
    let mut config = ConfigLayers::new();
    config
        .value("database-only", json!({"sqlite":{"path":path}}))
        .unwrap();
    config
}

/// Invalid native rows and closed resources become failures; startup remains read-only.
pub(crate) async fn failures() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    let app = build_application(layers(&path, 3)).await.unwrap();
    let database = app.host().shared_service::<Engine>().unwrap();
    let error = database
        .execute(&sql(
            "INSERT INTO profile VALUES (99, 'forbidden', 'write')",
        ))
        .await
        .unwrap_err();
    assert!(error.source().is_some());
    database.close().await.unwrap();
    let reply = app
        .call("profile-search", json!({"id":"abc"}))
        .await
        .unwrap();
    assert_eq!(reply.response.try_to_json().unwrap()["code"], 500);
    assert!(!reply.report.failures.is_empty());
    close(app).await;
    // Reopening never reseeds or modifies existing data: retain a deliberate external rename.
    let database = writable(&path).await;
    database
        .execute(&sql(
            "UPDATE profile SET name = 'operator edit' WHERE id = 'abc123'",
        ))
        .await
        .unwrap();
    database
        .execute(&sql("UPDATE profile SET name = X'01' WHERE id = 'abc456'"))
        .await
        .unwrap();
    database.close().await.unwrap();
    let app = build_application(layers(&path, 3)).await.unwrap();
    let reply = app
        .call("profile-search", json!({"id":"abc123"}))
        .await
        .unwrap();
    assert_eq!(
        reply.response.try_to_json().unwrap()["results"][0]["name"],
        "operator edit"
    );
    let reply = app
        .call("profile-search", json!({"id":"abc456"}))
        .await
        .unwrap();
    let value = reply.response.try_to_json().unwrap();
    assert_eq!(value["code"], 500);
    assert!(value.get("results").is_none());
    let error: &HostError = reply.report.failures[0].error.as_ref();
    assert!(error.source().is_some());
    close(app).await;
    // No test has smuggled new rows through startup or the failed read-only write.
    let database = writable(&path).await;
    assert_eq!(
        database
            .fetch(&Select::new(["id"]).from("profile"))
            .await
            .unwrap()
            .len(),
        3
    );
    database.close().await.unwrap();
}
