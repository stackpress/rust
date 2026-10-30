//! Finite source SQL commands against real file SQLite and explicit revision providers.

use super::*;
use stackpress_framework::{
    CompiledClient,
    commands::sql::{self, History, Migration, Scripts},
    config::Environment,
};
use stackpress_sql::{Create, DialectId, Engine, Field, Render, Select, Sql, SqlType, TrustedSql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::path::PathBuf;

/// Exact owned temporary root; no shared target/dependency files are removed.
pub(super) struct Fixture(pub PathBuf);
impl Fixture {
    /// Allocate an isolated finite fixture directory under the effort's private root.
    pub(super) fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from("/private/tmp/slice11-m1/fixtures").join(format!(
            "{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    /// Remove only this test's own closed finite fixture.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

/// Open a real file; neither registration nor command selection opens it implicitly.
pub(super) async fn engine(root: &Fixture) -> Arc<Engine> {
    Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(root.0.join("database.sqlite")).create_if_missing(true),
        )
        .await
        .unwrap(),
    ))
}
/// Empty generated package is present, unlike a missing client module.
static CLIENT: CompiledClient = CompiledClient {
    models: &[],
    build: "m1-finite",
    describe: Vec::new,
};
/// Ordinary named callback remains usable with ServerBuilder::plugin.
fn register(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework::register_with(app, &CLIENT)
}

/// Real generated script shape over a simple owned table.
fn scripts() -> Scripts {
    let table = Create::new("records")
        .add_field(Field::new("id", SqlType::Integer))
        .primary_key(["id"])
        .definition()
        .unwrap();
    Scripts::from_tables(&[table], DialectId::Sqlite, json!({"source":"schema"})).unwrap()
}
/// Read finite plans from a provider-owned JSON file, keeping warning policy outside the provider.
fn plans(root: &str, engine: &Engine) -> Result<Vec<Migration>> {
    let values: Vec<Value> =
        serde_json::from_slice(&std::fs::read(root).map_err(stackpress_lib::Error::from_source)?)?;
    values
        .into_iter()
        .map(|value| {
            Ok(Migration {
                epoch: value["epoch"].as_i64().unwrap(),
                warning: value["warning"].as_str().map(str::to_owned),
                queries: value["queries"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|sql| {
                        Sql::trusted(
                            engine.dialect(),
                            TrustedSql::audited(sql.as_str().unwrap().to_owned()),
                        )
                        .render(engine.dialect())
                        .map_err(stackpress_lib::Error::from_source)
                    })
                    .collect::<Result<Vec<_>>>()?,
            })
        })
        .collect()
}
/// Finite insertion spy writes the exact client schema only after transaction completion.
fn insert(root: &str, schema: &Value) -> Result<()> {
    std::fs::write(format!("{root}.inserted"), serde_json::to_vec(schema)?)
        .map_err(stackpress_lib::Error::from_source)
}

/// Finite latest/has-pair seams remain distinct from all-plan migration generation.
fn latest(root: &str, engine: &Engine) -> Result<Option<Migration>> {
    let mut plans = plans(root, engine)?;
    Ok(if plans.len() >= 2 { plans.pop() } else { None })
}
/// Source push asks for revision presence without planning baseline SQL.
fn has_pair(root: &str) -> Result<bool> {
    let values: Vec<Value> =
        serde_json::from_slice(&std::fs::read(root).map_err(stackpress_lib::Error::from_source)?)?;
    Ok(values.len() >= 2)
}

/// Publish explicit Engine/scripts/history while keeping every command lazy.
fn app(config: Value, engine: Arc<Engine>) -> ServerBuilder {
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers.value("app", config).unwrap();
    app.config_layers(layers).unwrap();
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(Arc::new(scripts()))?;
            start.register(Arc::new(History {
                all: plans,
                latest,
                has_pair,
                insert,
            }))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.plugin("aggregate", register).unwrap();
    app
}

/// Install resets data, purge keeps tables, uninstall drops them; query consumes supplied argv.
#[tokio::test]
async fn real_sqlite_command_lifecycle_and_query() {
    let root = Fixture::new();
    let engine = engine(&root).await;
    let mut app = app(json!({}), engine.clone());
    commands::bind(
        &mut app,
        Environment {
            argv: ["node", "cli", "query", "SELECT \\value\\ AS answer"]
                .map(String::from)
                .into(),
            ..Default::default()
        },
    );
    let host = app.build().await.unwrap();
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("records"))
            .await
            .is_err()
    );
    assert_eq!(
        host.resolve("install", json!({}))
            .await
            .unwrap()
            .response
            .code,
        Some(200)
    );
    engine
        .execute(&engine.insert("records").columns(["id"]).values([1]))
        .await
        .unwrap();
    assert_eq!(
        engine
            .fetch(&Select::new(["id"]).from("records"))
            .await
            .unwrap()
            .len(),
        1
    );
    host.resolve("install", json!({})).await.unwrap();
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("records"))
            .await
            .unwrap()
            .is_empty()
    );
    let output = host
        .resolve("query", json!({}))
        .await
        .unwrap()
        .response
        .results
        .unwrap()
        .to_json()
        .unwrap();
    assert_eq!(output, "value");
    engine
        .execute(&engine.insert("records").columns(["id"]).values([2]))
        .await
        .unwrap();
    host.resolve("purge", json!({})).await.unwrap();
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("records"))
            .await
            .unwrap()
            .is_empty()
    );
    host.resolve("uninstall", json!({})).await.unwrap();
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("records"))
            .await
            .is_err()
    );
    engine.close().await.unwrap();
}

/// Source skip modes return OK, warned migrate writes bytes, and upgrade force owns refusal.
#[tokio::test]
async fn revisions_skip_warning_migrate_upgrade_and_push() {
    let root = Fixture::new();
    let engine = engine(&root).await;
    let file = root.0.join("plans.json");
    std::fs::write(&file,r#"[{"epoch":10,"queries":["CREATE TABLE audit (id INTEGER)"],"warning":"baseline"},{"epoch":20,"queries":["INSERT INTO audit VALUES (7)"],"warning":"source warning"}]"#).unwrap();
    let history = History {
        all: plans,
        latest,
        has_pair,
        insert,
    };
    sql::migrate(&engine, None, None, &history).unwrap();
    sql::upgrade(&engine, None, &history, false).await.unwrap();
    let destination = root.0.join("migrations");
    sql::migrate(&engine, file.to_str(), destination.to_str(), &history).unwrap();
    assert_eq!(
        std::fs::read_to_string(destination.join("10.sql")).unwrap(),
        "CREATE TABLE audit (id INTEGER)"
    );
    assert_eq!(
        std::fs::read_to_string(destination.join("20.sql")).unwrap(),
        "INSERT INTO audit VALUES (7)"
    );
    let oracle: Value =
        serde_json::from_str(include_str!("../fixtures/source-scripts.json")).unwrap();
    let original = oracle["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["command"] == "migrate" && case["variant"] == "warned")
        .unwrap();
    for write in original["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry[0] == "write")
    {
        let name = std::path::Path::new(write[1].as_str().unwrap())
            .file_name()
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(destination.join(name)).unwrap(),
            write[2].as_str().unwrap()
        );
    }
    assert_eq!(
        sql::upgrade(&engine, file.to_str(), &history, false)
            .await
            .unwrap_err()
            .message(),
        "source warning"
    );
    engine
        .execute(&Sql::literal(
            DialectId::Sqlite,
            "CREATE TABLE audit (id INTEGER)",
        ))
        .await
        .unwrap();
    sql::upgrade(&engine, file.to_str(), &history, true)
        .await
        .unwrap();
    assert_eq!(
        engine
            .fetch(&Select::new(["id"]).from("audit"))
            .await
            .unwrap()[0]
            .try_get::<i64>("id")
            .unwrap(),
        7
    );
    let host = app(json!({}), engine.clone()).build().await.unwrap();
    for event in ["migrate", "push", "upgrade"] {
        assert_eq!(
            host.resolve(event, json!({})).await.unwrap().response.code,
            Some(200)
        );
    }
    std::fs::write(&file, "[]").unwrap();
    let host = app(
        json!({"client":{"revisions":file},"database":{"migrations":destination}}),
        engine.clone(),
    )
    .build()
    .await
    .unwrap();
    host.resolve("migrate", json!({})).await.unwrap();
    host.resolve("upgrade", json!({})).await.unwrap();
    host.resolve("push", json!({})).await.unwrap();
    assert_eq!(
        serde_json::from_slice::<Value>(
            &std::fs::read(format!("{}.inserted", file.display())).unwrap()
        )
        .unwrap(),
        json!({"source":"schema"})
    );
    engine.close().await.unwrap();
}

/// Push/upgrade must not plan obsolete revisions or require scripts on the two-revision path.
#[tokio::test]
async fn revision_operations_stay_lazy_and_direct_scripts_remain_independent() {
    fn all(_: &str, _: &Engine) -> Result<Vec<Migration>> {
        panic!("all history must not be read by push/upgrade")
    }
    fn latest(_: &str, _: &Engine) -> Result<Option<Migration>> {
        Ok(Some(Migration {
            epoch: 2,
            queries: vec![],
            warning: None,
        }))
    }
    fn has_pair(_: &str) -> Result<bool> {
        Ok(true)
    }
    let root = Fixture::new();
    let engine = engine(&root).await;
    let history = History {
        all,
        latest,
        has_pair,
        insert,
    };
    sql::push(&engine, None, &history, false, || {
        panic!("no revisions must skip client lookup")
    })
    .await
    .unwrap();
    sql::push(&engine, Some("finite"), &history, false, || {
        panic!("an adjacent plan needs no generated scripts")
    })
    .await
    .unwrap();
    sql::upgrade(&engine, Some("finite"), &history, false)
        .await
        .unwrap();
    let scripts = scripts();
    sql::install(&engine, &scripts, None).await.unwrap();
    sql::purge(&engine, &scripts).await.unwrap();
    sql::uninstall(&engine, &scripts).await.unwrap();
    engine.close().await.unwrap();
}

/// SQL failure rolls back the same transaction, retaining preexisting data and the original error.
#[tokio::test]
async fn transaction_rollback_and_required_client_failure() {
    let root = Fixture::new();
    let engine = engine(&root).await;
    sql::install(&engine, &scripts(), None).await.unwrap();
    engine
        .execute(&engine.insert("records").columns(["id"]).values([9]))
        .await
        .unwrap();
    let queries = [
        Sql::literal(DialectId::Sqlite, "DELETE FROM records")
            .render(DialectId::Sqlite)
            .unwrap(),
        Sql::literal(DialectId::Sqlite, "INSERT INTO absent VALUES (1)")
            .render(DialectId::Sqlite)
            .unwrap(),
    ];
    assert!(sql::transaction(&engine, &queries).await.is_err());
    assert_eq!(
        engine
            .fetch(&Select::new(["id"]).from("records"))
            .await
            .unwrap()
            .len(),
        1
    );
    let mut app = application(json!({}));
    let cloned = engine.clone();
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = cloned.clone();
        Box::pin(async move {
            start.register(engine)?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let host = app.build().await.unwrap();
    assert!(host.resolve("install", json!({})).await.is_err());
    engine.close().await.unwrap();
}

/// Populate ignores individual response status and requires neither database nor compiled client.
#[tokio::test]
async fn populate_reuses_events_without_an_engine_or_global_gate() {
    let mut app = application(
        json!({"database":{"populate":[{"event":"first","data":{"value":1}},{"event":"second","data":{"value":2}}]}}),
    );
    for event in ["first", "second"] {
        app.on(
            event,
            stackpress_lib::handler(|req: &mut Request, res: &mut Response, _: Invocation<'_>| {
                Box::pin(async move {
                    assert!(req.data.has("value"));
                    res.status_code(409, None)?;
                    Ok(Flow::Abort)
                })
            }),
        )
        .unwrap();
    }
    let host = app.build().await.unwrap();
    assert_eq!(
        host.resolve("populate", json!({}))
            .await
            .unwrap()
            .response
            .code,
        Some(200)
    );
    assert!(host.host().service::<Engine>().is_err());
}
