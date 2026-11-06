//! Real registered source SQL commands, including actual CLI argv/output/force boundaries.

use serde_json::{Value, json};
use stackpress_framework::commands::sql::Scripts;
use stackpress_lib::{Request, Response};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, StartupContext};
use stackpress_sql::{Engine, Field, Sql, SqlType, TrustedSql};
use std::{
    path::Path,
    process::{Command, Output},
    sync::Arc,
};

/// Bind every mutable fixture path explicitly, above the shared application defaults.
fn layers(root: &Path) -> ConfigLayers {
    let mut layers = ConfigLayers::new();
    layers.value("owned-fixture", json!({"sqlite":{"path":root.join("blog.sqlite")},
        "client":{"revisions":root.join("revisions")},"database":{"migrations":root.join("migrations")}})).unwrap();
    layers
}

/// Dispatch registered commands directly; the proof runner supplies the external deadline.
async fn emit(server: &Server, event: &str) -> stackpress_lib::Result<Value> {
    let mut req = Request::new();
    let mut res = Response::new();
    server.emit(event, &mut req, &mut res).await?;
    res.try_to_json()
}

/// Open an empty existing file so cold push is the explicit schema-changing operation.
async fn empty(root: &Path) {
    blog_web::plugins::store::open(&root.join("blog.sqlite"), true)
        .await
        .unwrap()
        .close()
        .await
        .unwrap();
}

/// Inspect actual schema ordinals rather than the compiled model's original field list.
async fn article_columns(engine: &Engine) -> Vec<String> {
    engine
        .fetch(&Sql::trusted(
            engine.dialect(),
            TrustedSql::audited("SELECT name FROM pragma_table_info('article')"),
        ))
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get("name").unwrap())
        .collect()
}

#[tokio::test(flavor = "current_thread")]
async fn cold_push_records_source_then_nullable_upgrade_preserves_fifteen_seed_records() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path();
    empty(root).await;
    let server = blog_web::build_storage(layers(root), "abc123")
        .await
        .unwrap();
    assert!(
        !root.join("revisions").exists(),
        "startup must not read/write revisions"
    );
    assert_eq!(emit(&server, "push").await.unwrap()["code"], 200);
    assert_eq!(
        std::fs::read_dir(root.join("revisions")).unwrap().count(),
        1
    );
    assert_eq!(emit(&server, "populate").await.unwrap()["code"], 200);
    let engine = server.host().shared_service::<Engine>().unwrap();
    let mut tables = blog_web::plugins::store::tables().unwrap();
    tables
        .iter_mut()
        .find(|table| table.name == "article")
        .unwrap()
        .fields
        .push(Field::new("subtitle", SqlType::Text));
    let current = blog_web::plugins::commands::current().unwrap();
    let next = blog_web::history::snapshot(current["source"].clone(), &tables).unwrap();
    let revisions = root.join("revisions");
    blog_web::history::insert_at(
        revisions.to_str().unwrap(),
        &next,
        chrono::Utc::now().timestamp_millis() + 1000,
    )
    .unwrap();
    assert_eq!(emit(&server, "migrate").await.unwrap()["code"], 200);
    assert!(
        !article_columns(&engine).await.contains(&"subtitle".into()),
        "migrate writes SQL only"
    );
    assert_eq!(emit(&server, "push").await.unwrap()["code"], 200);
    assert!(article_columns(&engine).await.contains(&"subtitle".into()));
    // Existing rows keep their identities and acquire NULL for the appended nullable column.
    let articles = engine
        .fetch(&stackpress_sql::Select::new(["id", "subtitle"]).from("article"))
        .await
        .unwrap();
    assert_eq!(articles.len(), 2);
    assert!(
        articles
            .iter()
            .all(|row| row.try_get::<String>("id").is_ok()
                && row.try_get::<Option<String>>("subtitle").unwrap().is_none())
    );
    for (event, count) in [
        ("profile-search", 1),
        ("auth-search", 2),
        ("application-search", 2),
        ("category-search", 2),
        ("article-search", 2),
        ("catalog-search", 2),
        ("comment-search", 4),
        ("session-search", 0),
    ] {
        assert_eq!(emit(&server, event).await.unwrap()["total"], count);
    }
    // The source has no applied ledger. Replaying the latest ADD reports SQLite's duplicate column.
    assert!(emit(&server, "push").await.is_err());
    assert_eq!(emit(&server, "article-search").await.unwrap()["total"], 2);
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn one_revision_push_reinstalls_source_schema_and_missing_provider_is_not_silent() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path();
    empty(root).await;
    let server = blog_web::build_storage(layers(root), "abc123")
        .await
        .unwrap();
    emit(&server, "push").await.unwrap();
    emit(&server, "populate").await.unwrap();
    assert_eq!(emit(&server, "article-search").await.unwrap()["total"], 2);
    emit(&server, "push").await.unwrap();
    assert_eq!(emit(&server, "article-search").await.unwrap()["total"], 0);
    assert_eq!(
        std::fs::read_dir(root.join("revisions")).unwrap().count(),
        1
    );
    let engine = server.host().shared_service::<Engine>().unwrap();
    let mut app = ServerBuilder::new();
    let mut provider_layers = ConfigLayers::new();
    provider_layers
        .value(
            "missing-history",
            json!({"client":{"revisions":root.join("revisions")},
        "database":{"migrations":root.join("migrations")}}),
        )
        .unwrap();
    app.config_layers(provider_layers).unwrap();
    stackpress_framework::config::register(&mut app).unwrap();
    blog_web::plugins::commands::register(&mut app).unwrap();
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut StartupContext| {
            let engine = engine.clone();
            Box::pin(async move {
                start.register(engine)?;
                Ok(stackpress_lib::Flow::Continue)
            })
        },
    ))
    .unwrap();
    let missing = app.build().await.unwrap();
    assert!(
        emit(&missing, "migrate").await.is_err(),
        "configured history cannot silently disappear"
    );
    missing
        .host()
        .shared_service::<Engine>()
        .unwrap()
        .close()
        .await
        .unwrap();
}

/// Run the actual binary with the same file/config layer and raw final argv token.
fn cli(config: &Path, force: bool, event: &str, sql: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_blog-web"));
    command.arg("-b").arg(config);
    if force {
        command.arg("--force");
    }
    command.args(["emit", event]);
    if let Some(sql) = sql {
        command.arg(sql);
    }
    command.output().unwrap()
}

/// Decode only successful complete CLI JSON; retain stderr when a native command fails.
fn success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

#[tokio::test(flavor = "current_thread")]
async fn real_cli_query_warning_force_purge_and_constraints() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path();
    empty(root).await;
    let config = root.join("develop.json");
    let value = json!({"sqlite":{"path":root.join("blog.sqlite")},"client":{"revisions":root.join("revisions")},
        "database":{"migrations":root.join("migrations")}});
    std::fs::write(&config, serde_json::to_vec(&value).unwrap()).unwrap();
    success(cli(&config, false, "push", None));
    success(cli(&config, false, "populate", None));
    success(cli(&config, false, "revision", None));
    assert_eq!(
        std::fs::read_dir(root.join("revisions")).unwrap().count(),
        1
    );
    let queried = success(cli(
        &config,
        false,
        "query",
        Some("SELECT COUNT(*) AS total FROM article"),
    ));
    assert_eq!(queried["results"]["rows"][0]["total"], 2);
    let quoted = success(cli(
        &config,
        false,
        "query",
        Some("SELECT \\source\\ AS marker"),
    ));
    assert_eq!(quoted["results"]["rows"][0]["marker"], "source");
    assert!(
        !cli(&config, false, "query", Some("SELECT missing FROM article"))
            .status
            .success()
    );

    let mut tables = blog_web::plugins::store::tables().unwrap();
    tables
        .iter_mut()
        .find(|table| table.name == "article")
        .unwrap()
        .fields
        .retain(|field| field.name != "banner");
    let current = blog_web::plugins::commands::current().unwrap();
    let next = blog_web::history::snapshot(current["source"].clone(), &tables).unwrap();
    blog_web::history::insert_at(
        root.join("revisions").to_str().unwrap(),
        &next,
        chrono::Utc::now().timestamp_millis() + 1000,
    )
    .unwrap();
    success(cli(&config, false, "migrate", None));
    assert!(!cli(&config, false, "push", None).status.success());
    let engine = blog_web::plugins::store::open(&root.join("blog.sqlite"), false)
        .await
        .unwrap();
    assert!(article_columns(&engine).await.contains(&"banner".into()));
    success(cli(&config, true, "push", None));
    assert!(!article_columns(&engine).await.contains(&"banner".into()));
    success(cli(&config, false, "purge", None));
    let count = success(cli(
        &config,
        false,
        "query",
        Some("SELECT COUNT(*) AS total FROM profile"),
    ));
    assert_eq!(count["results"]["rows"][0]["total"], 0);
    let orphan = engine.execute(&Sql::trusted(engine.dialect(), TrustedSql::audited(
        "INSERT INTO comment (id, profile_id, article_id, comment) VALUES ('orphan', 'absent', 'absent', 'x')"))).await;
    let error = orphan.unwrap_err();
    let mut cause = std::error::Error::source(&error);
    let mut details = String::new();
    while let Some(source) = cause {
        details.push_str(&source.to_string());
        cause = source.source();
    }
    assert!(details.to_lowercase().contains("foreign key"), "{details}");
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn adjacent_push_does_not_evaluate_scripts_and_missing_paths_keep_source_skips() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path();
    let engine = blog_web::plugins::store::open(&root.join("blog.sqlite"), true)
        .await
        .unwrap();
    let table = stackpress_sql::Create::new("record")
        .add_field(Field::new("id", SqlType::Text))
        .definition()
        .unwrap();
    engine
        .execute_plan(&table.create().render(engine.dialect()).unwrap())
        .await
        .unwrap();
    let revisions = root.join("revisions");
    let revisions = revisions.to_str().unwrap();
    blog_web::history::insert_at(
        revisions,
        &blog_web::history::snapshot(json!({"v":1}), std::slice::from_ref(&table)).unwrap(),
        10,
    )
    .unwrap();
    blog_web::history::insert_at(
        revisions,
        &blog_web::history::snapshot(json!({"v":2}), &[table]).unwrap(),
        20,
    )
    .unwrap();
    let history = blog_web::history::binding();
    stackpress_framework::commands::sql::push(
        &engine,
        Some(revisions),
        &history,
        false,
        || -> stackpress_lib::Result<Arc<Scripts>> {
            panic!("adjacent push must not request install scripts")
        },
    )
    .await
    .unwrap();
    stackpress_framework::commands::sql::push(
        &engine,
        None,
        &history,
        false,
        || -> stackpress_lib::Result<Arc<Scripts>> { panic!("missing path must skip") },
    )
    .await
    .unwrap();
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn earlier_response_and_missing_engine_precede_invalid_revision_path() {
    let mut layers = ConfigLayers::new();
    layers
        .value("invalid-unused-path", json!({"client":{"revisions":42}}))
        .unwrap();
    let mut app = ServerBuilder::new();
    app.config_layers(layers).unwrap();
    stackpress_framework::config::register(&mut app).unwrap();
    blog_web::plugins::commands::register(&mut app).unwrap();
    let server = app.build().await.unwrap();
    let mut res = Response::new();
    res.status_code(404, None).unwrap();
    server
        .emit("push", &mut Request::new(), &mut res)
        .await
        .unwrap();
    assert_eq!(res.status().unwrap().code, 404);
    let reply = emit(&server, "push").await.unwrap();
    assert_eq!(reply["code"], 500);
    assert_eq!(reply["error"], "No database found");
}
