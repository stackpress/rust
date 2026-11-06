//! Durable source JSON history, exact warnings/rename rules, and native failure preservation.

use blog_web::history::{binding, insert_at, snapshot};
use serde_json::{Value, json};
use stackpress_framework::commands::sql;
use stackpress_sql::{Create, Engine, Field, Select, SqlType};

/// One simple generated-shape definition keeps migration decisions inspectable in real SQLite.
fn table(fields: &[&str]) -> stackpress_sql::TableDefinition {
    let mut create = Create::new("note")
        .add_field(Field::new("id", SqlType::Text).not_null())
        .primary_key(["id"]);
    for name in fields {
        create = create.add_field(Field::new(*name, SqlType::Text));
    }
    create.definition().unwrap()
}

/// Encode real SQL definitions rather than canned query lists.
fn schema(fields: &[&str], version: u32) -> Value {
    snapshot(
        json!({"version":version,"tables":[{"name":"note","model":"Note"}]}),
        &[table(fields)],
    )
    .unwrap()
}

/// Open only a private fixture, with its connection shutdown explicitly awaited by each caller.
async fn engine(root: &std::path::Path) -> std::sync::Arc<Engine> {
    blog_web::plugins::store::open(&root.join("history.sqlite"), true)
        .await
        .unwrap()
}

/// Read live native columns; this proves DDL effects independently of an app model's field list.
async fn columns(engine: &Engine) -> Vec<String> {
    engine
        .fetch(&stackpress_sql::Sql::trusted(
            engine.dialect(),
            stackpress_sql::TrustedSql::audited("SELECT name FROM pragma_table_info('note')"),
        ))
        .await
        .unwrap()
        .iter()
        .map(|row| row.try_get("name").unwrap())
        .collect()
}

#[tokio::test(flavor = "current_thread")]
async fn history_reloads_noops_metadata_changes_and_source_epoch_overwrite() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path().join("revisions");
    let root = root.to_str().unwrap();
    let engine = engine(fixture.path()).await;
    let original = blog_web::plugins::commands::current().unwrap();
    insert_at(root, &original, 100).unwrap();
    let file = std::path::Path::new(root).join("100.json");
    let bytes = std::fs::read(&file).unwrap();
    let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();
    insert_at(root, &original, 200).unwrap();
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 1);
    assert_eq!(std::fs::read(&file).unwrap(), bytes);
    assert_eq!(std::fs::metadata(&file).unwrap().modified().unwrap(), mtime);
    assert!(!(binding().has_pair)(root).unwrap());
    assert!((binding().latest)(root, &engine).unwrap().is_none());
    let baseline = (binding().all)(root, &engine).unwrap();
    assert_eq!(baseline.len(), 1);
    assert!(
        baseline[0]
            .warning
            .as_ref()
            .unwrap()
            .contains("Dropped tables:")
    );
    let creates = baseline[0]
        .queries
        .iter()
        .filter(|query| query.sql().starts_with("CREATE TABLE"))
        .count();
    assert_eq!(creates, 8);
    assert!(
        baseline[0]
            .queries
            .iter()
            .any(|query| query.sql().contains("CURRENT_TIMESTAMP"))
    );

    let mut changed = original.clone();
    changed["source"]["models"][0]["documentation"]["description"] =
        "Changed documentation only".into();
    insert_at(root, &changed, 300).unwrap();
    assert!((binding().has_pair)(root).unwrap());
    let latest = (binding().latest)(root, &engine).unwrap().unwrap();
    assert_eq!(latest.epoch, 300);
    assert!(latest.queries.is_empty() && latest.warning.is_none());
    // Same millisecond retains source overwrite semantics, with no invented sequence suffix.
    changed["source"]["models"][0]["documentation"]["description"] =
        "Same epoch replacement".into();
    insert_at(root, &changed, 300).unwrap();
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 2);
    let stored: Value = serde_json::from_slice(
        &std::fs::read(std::path::Path::new(root).join("300.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(stored, changed);
    assert_eq!(original["source"]["models"].as_array().unwrap().len(), 8);
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn latest_is_lazy_but_all_and_selected_corrupt_revisions_fail() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path().join("revisions");
    let root = root.to_str().unwrap();
    let engine = engine(fixture.path()).await;
    assert!((binding().all)(root, &engine).unwrap().is_empty());
    for epoch in [10, 20, 30] {
        insert_at(root, &schema(&["body"], epoch as u32), epoch).unwrap();
    }
    std::fs::write(std::path::Path::new(root).join("10.json"), "{broken").unwrap();
    assert!((binding().has_pair)(root).unwrap());
    assert!(
        (binding().latest)(root, &engine)
            .unwrap()
            .unwrap()
            .queries
            .is_empty()
    );
    assert!((binding().all)(root, &engine).is_err());
    std::fs::write(std::path::Path::new(root).join("30.json"), "{broken").unwrap();
    assert!((binding().latest)(root, &engine).is_err());
    assert!((binding().insert)(root, &schema(&[], 4)).is_err());
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn source_zero_epoch_is_absent_without_promoting_its_successor_to_baseline() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path().join("revisions");
    let root = root.to_str().unwrap();
    let engine = engine(fixture.path()).await;
    insert_at(root, &schema(&[], 1), 0).unwrap();
    std::fs::write(
        std::path::Path::new(root).join("0.json"),
        "not read by source index(0)",
    )
    .unwrap();
    assert!((binding().all)(root, &engine).unwrap().is_empty());
    insert_at(root, &schema(&[], 2), 10).unwrap();
    assert!(!(binding().has_pair)(root).unwrap());
    assert!((binding().latest)(root, &engine).unwrap().is_none());
    assert!((binding().all)(root, &engine).unwrap().is_empty());
    insert_at(root, &schema(&[], 3), 20).unwrap();
    let plans = (binding().all)(root, &engine).unwrap();
    assert_eq!(plans.len(), 1);
    assert_eq!(plans[0].epoch, 20);
    assert!(plans[0].queries.is_empty());
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn indexed_rename_consumes_equivalent_replacements_and_keeps_uniqueness() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path().join("revisions");
    let root = root.to_str().unwrap();
    let engine = engine(fixture.path()).await;
    let mut before = table(&["body"]);
    before.indexes = vec![
        stackpress_sql::Index::new("note_body_unique", ["body"]).unique(),
        stackpress_sql::Index::new("note_body_index", ["body"]),
    ];
    let mut after = table(&["contents"]);
    after.indexes = vec![
        stackpress_sql::Index::new("note_contents_unique", ["contents"]).unique(),
        stackpress_sql::Index::new("note_contents_index", ["contents"]),
    ];
    engine
        .execute_plan(&before.create().render(engine.dialect()).unwrap())
        .await
        .unwrap();
    engine
        .execute(
            &engine
                .insert("note")
                .columns(["id", "body"])
                .values(["1", "same"]),
        )
        .await
        .unwrap();
    insert_at(root, &snapshot(json!({}), &[before]).unwrap(), 10).unwrap();
    insert_at(root, &snapshot(json!({}), &[after]).unwrap(), 20).unwrap();
    assert!(
        (binding().latest)(root, &engine)
            .unwrap()
            .unwrap()
            .warning
            .is_none()
    );
    sql::upgrade(&engine, Some(root), &binding(), false)
        .await
        .unwrap();
    let rows = engine
        .fetch(&Select::new(["contents"]).from("note"))
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>("contents").unwrap(), "same");
    assert!(
        engine
            .execute(
                &engine
                    .insert("note")
                    .columns(["id", "contents"])
                    .values(["2", "same"])
            )
            .await
            .is_err()
    );
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn exact_one_to_one_rename_preserves_rows_and_ambiguous_groups_warn() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = fixture.path().join("rename");
    let root = root.to_str().unwrap();
    let engine = engine(fixture.path()).await;
    engine
        .execute_plan(&table(&["body"]).create().render(engine.dialect()).unwrap())
        .await
        .unwrap();
    engine
        .execute(
            &engine
                .insert("note")
                .columns(["id", "body"])
                .values(["n1", "preserved"]),
        )
        .await
        .unwrap();
    insert_at(root, &schema(&["body"], 1), 10).unwrap();
    insert_at(root, &schema(&["contents"], 2), 20).unwrap();
    let latest = (binding().latest)(root, &engine).unwrap().unwrap();
    assert!(latest.warning.is_none());
    sql::upgrade(&engine, Some(root), &binding(), false)
        .await
        .unwrap();
    let rows = engine
        .fetch(&Select::new(["contents"]).from("note"))
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>("contents").unwrap(), "preserved");

    let ambiguous = fixture.path().join("ambiguous");
    let ambiguous = ambiguous.to_str().unwrap();
    insert_at(ambiguous, &schema(&["a", "b"], 1), 10).unwrap();
    insert_at(ambiguous, &schema(&["c", "d"], 2), 20).unwrap();
    let warning = (binding().latest)(ambiguous, &engine)
        .unwrap()
        .unwrap()
        .warning
        .unwrap();
    assert!(warning.contains("Ambiguous field rename detected.\nStackpress found multiple same-shape rename candidates and refused to guess."));
    assert!(warning.contains("- Note: a, b -> c, d"));
    assert!(warning.contains("- Removed fields: a, b"));
    assert!(warning.ends_with(
        "Re-run with `--force` only if you intentionally want to execute these changes."
    ));
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn migrate_writes_warned_sql_upgrade_requires_force_and_failed_upgrade_preserves_data() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = engine(fixture.path()).await;
    engine
        .execute_plan(&table(&["body"]).create().render(engine.dialect()).unwrap())
        .await
        .unwrap();
    engine
        .execute(
            &engine
                .insert("note")
                .columns(["id", "body"])
                .values(["n1", "original"]),
        )
        .await
        .unwrap();
    let root = fixture.path().join("revisions");
    let root = root.to_str().unwrap();
    insert_at(root, &schema(&["body"], 1), 10).unwrap();
    insert_at(root, &schema(&[], 2), 20).unwrap();
    let destination = fixture.path().join("migrations");
    sql::migrate(
        &engine,
        Some(root),
        Some(destination.to_str().unwrap()),
        &binding(),
    )
    .unwrap();
    assert!(destination.join("10.sql").exists() && destination.join("20.sql").exists());
    let before = columns(&engine).await;
    assert!(
        sql::upgrade(&engine, Some(root), &binding(), false)
            .await
            .unwrap_err()
            .to_string()
            .contains("Destructive schema changes detected.")
    );
    assert_eq!(columns(&engine).await, before);
    sql::upgrade(&engine, Some(root), &binding(), true)
        .await
        .unwrap();
    assert_eq!(columns(&engine).await, ["id"]);

    let rollback = fixture.path().join("rollback");
    let rollback = rollback.to_str().unwrap();
    let mut changed = table(&[]);
    changed.fields.extend([
        Field::new("optional", SqlType::Text),
        Field::new("required", SqlType::Text).not_null(),
    ]);
    insert_at(rollback, &schema(&[], 1), 10).unwrap();
    insert_at(rollback, &snapshot(json!({}), &[changed]).unwrap(), 20).unwrap();
    assert!(
        sql::upgrade(&engine, Some(rollback), &binding(), false)
            .await
            .is_err()
    );
    assert_eq!(columns(&engine).await, ["id"]);
    let rows = engine
        .fetch(&Select::new(["id"]).from("note"))
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>("id").unwrap(), "n1");
    engine.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn history_insert_failure_after_install_does_not_claim_database_rollback() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = engine(fixture.path()).await;
    let blocked = fixture.path().join("not-a-directory");
    std::fs::write(&blocked, "owned fixture file").unwrap();
    let table = table(&["body"]);
    let schema = snapshot(json!({}), std::slice::from_ref(&table)).unwrap();
    let scripts = sql::Scripts::from_tables(&[table], engine.dialect(), schema).unwrap();
    assert!(
        sql::install(
            &engine,
            &scripts,
            Some((blocked.to_str().unwrap(), &binding()))
        )
        .await
        .is_err()
    );
    // Source inserts the revision after the SQL transaction. The IO failure is visible,
    // while the already completed installation remains an observed database effect.
    assert!(columns(&engine).await.contains(&"body".into()));
    assert_eq!(
        std::fs::read_to_string(&blocked).unwrap(),
        "owned fixture file"
    );
    engine.close().await.unwrap();
}
