//! G48 source setup on owned live databases, under the existing external parent watchdog.
#[path = "live/cancellation.rs"]
mod cancellation;
#[path = "live/cascade.rs"]
mod cascade;
#[path = "../../framework-sql/tests/support/g54.rs"]
mod g54;
#[path = "../../framework-session/tests/live/support.rs"]
mod support;

use stackpress_framework::commands::sql::{self, History, Migration, Scripts};
use stackpress_framework_sql::{
    migrations::{MigrationPlanner, RenameMap, RevisionSnapshot},
    setup,
};
use stackpress_lib::{Error, Result};
use stackpress_sql::{Create, DialectId, Engine, ErrorKind, Field, Select, SqlType};
use std::{path::Path, sync::Arc};

/// A real catalog definition; source commands use its create/drop statements rather than fixture SQL.
fn table() -> stackpress_sql::TableDefinition {
    Create::new("g48_records")
        .add_field(Field::new("id", SqlType::Integer))
        .primary_key(["id"])
        .definition()
        .unwrap()
}

/// Native existence observation, without relying on MySQL/PostgreSQL catalog representation.
async fn exists(engine: &Engine, name: &str) -> bool {
    engine.fetch(&Select::new(["id"]).from(name)).await.is_ok()
}

/// Every backend uses the actual catalog compiler, including MySQL's G48 caller mapping.
fn scripts(engine: &Engine) -> Scripts {
    Scripts::from_tables(
        &[table()],
        engine.dialect(),
        serde_json::json!({"g48":true}),
    )
    .unwrap()
}

/// Reuse the caller-owned source statement inventory without adding a production Clone contract.
fn copy_scripts(scripts: &Scripts) -> Scripts {
    Scripts {
        install: scripts.install.clone(),
        purge: scripts.purge.clone(),
        uninstall: scripts.uninstall.clone(),
        schema: scripts.schema.clone(),
    }
}

/// Required history seam is intentionally unused by upgrade/push; no obsolete baseline is applied.
fn all(_: &str, _: &Engine) -> Result<Vec<Migration>> {
    panic!("upgrade must not request all history")
}
/// A real adjacent generated migration, exported as ordinary source command queries.
fn latest(root: &str, engine: &Engine) -> Result<Option<Migration>> {
    if !Path::new(root).join("pair").exists() {
        return Ok(None);
    }
    let original = table();
    let mut changed = original.clone();
    changed.fields.push(Field::new("score", SqlType::Integer));
    let from = RevisionSnapshot::new("g48", "001", vec![original]).map_err(Error::from_source)?;
    let to = RevisionSnapshot::new("g48", "002", vec![changed]).map_err(Error::from_source)?;
    let plan = MigrationPlanner::between(&from, &to, RenameMap::new(), engine.dialect())
        .map_err(Error::from_source)?;
    assert!(plan.blocked_reasons().is_empty());
    Ok(Some(Migration {
        epoch: 2,
        warning: Some("g48 source warning".into()),
        queries: plan.queries().to_vec(),
    }))
}
/// File-owned revision state avoids process-global callback state.
fn has_pair(root: &str) -> Result<bool> {
    Ok(Path::new(root).join("pair").exists())
}
/// Only successful complete source install may publish this exact owned marker.
fn insert(root: &str, _: &serde_json::Value) -> Result<()> {
    std::fs::write(Path::new(root).join("inserted"), b"complete").map_err(Error::from_source)
}

/// Real duplicate-table failure proves durable MySQL prefix vs PostgreSQL rollback.
async fn partial_state(engine: &Engine) -> support::TestResult<()> {
    let build = |name| Create::new(name).add_field(Field::new("id", SqlType::Integer));
    let a = build("g48_prefix").render(engine.dialect())?.queries()[0].clone();
    let c = build("g48_later").render(engine.dialect())?.queries()[0].clone();
    let error = setup::execute(engine, &[a.clone(), a, c])
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert_eq!(
        exists(engine, "g48_prefix").await,
        engine.dialect() == DialectId::Mysql
    );
    assert!(!exists(engine, "g48_later").await);
    if engine.dialect() == DialectId::Mysql {
        assert_eq!(error.plan_progress().unwrap().acknowledged, 1);
        assert_eq!(error.plan_progress().unwrap().failed_index, 1);
        assert_eq!(error.plan_progress().unwrap().total, 3);
        // Ordinary atomic APIs still refuse MySQL DDL; only the explicit setup path opts in.
        let plan = build("g48_atomic").render(engine.dialect())?;
        assert_eq!(
            engine.execute_plan(&plan).await.unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        let mut transaction = engine.begin().await?;
        assert_eq!(
            transaction
                .execute(&plan.queries()[0])
                .await
                .unwrap_err()
                .to_string(),
            "MySQL transaction DDL would implicitly commit"
        );
        transaction.rollback().await?;
        assert!(!exists(engine, "g48_atomic").await);
    } else {
        assert!(error.plan_progress().is_none());
    }
    Ok(())
}

/// Full source command sequencing, adjacent generated upgrade and revision publication timing.
async fn commands(engine: &Engine, root: &str) -> support::TestResult<()> {
    let scripts = scripts(engine);
    let history = History {
        all,
        latest,
        has_pair,
        insert,
    };
    sql::upgrade(engine, None, &history, false).await?;
    sql::upgrade(engine, Some(root), &history, false).await?;
    sql::push(engine, None, &history, false, || {
        panic!("missing root must skip client")
    })
    .await?;
    sql::install(engine, &scripts, Some((root, &history))).await?;
    assert!(Path::new(root).join("inserted").exists());
    engine
        .execute(&engine.insert("g48_records").columns(["id"]).values([7]))
        .await?;
    // Fewer than two revisions selects install and clears the old row before recreating the table.
    sql::push(engine, Some(root), &history, false, || {
        Ok(Arc::new(copy_scripts(&scripts)))
    })
    .await?;
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("g48_records"))
            .await?
            .is_empty()
    );
    std::fs::write(Path::new(root).join("pair"), b"adjacent")?;
    assert_eq!(
        sql::upgrade(engine, Some(root), &history, false)
            .await
            .unwrap_err()
            .message(),
        "g48 source warning"
    );
    // Push shares the actual upgrade function; no scripts/client lookup is needed for this branch.
    sql::push(engine, Some(root), &history, true, || {
        panic!("adjacent upgrade needs no scripts")
    })
    .await?;
    engine
        .execute(
            &engine
                .insert("g48_records")
                .columns(["id", "score"])
                .values([9, 42]),
        )
        .await?;
    assert_eq!(
        engine
            .fetch(&Select::new(["score"]).from("g48_records"))
            .await?[0]
            .try_get::<i64>("score")?,
        42
    );
    println!("adjacent generated migration applied through aggregate upgrade");
    sql::purge(engine, &scripts).await?;
    engine
        .execute(&engine.insert("g48_records").columns(["id"]).values([10]))
        .await?;
    std::fs::remove_file(Path::new(root).join("inserted"))?;
    let mut failed = copy_scripts(&scripts);
    failed.install.extend(scripts.install.clone());
    assert!(
        sql::install(engine, &failed, Some((root, &history)))
            .await
            .is_err()
    );
    assert!(!Path::new(root).join("inserted").exists());
    let rows = engine
        .fetch(&Select::new(["id"]).from("g48_records"))
        .await?;
    assert_eq!(
        rows.len(),
        if engine.dialect() == DialectId::Mysql {
            0
        } else {
            1
        }
    );
    sql::uninstall(engine, &scripts).await?;
    assert!(!exists(engine, "g48_records").await);
    Ok(())
}

/// Remove only this child-owned marker directory, including unwind; DB ownership is parent-owned.
struct Markers(std::path::PathBuf);
impl Drop for Markers {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
/// No listener, startup IO or generated migration runner is involved in these explicit commands.
async fn scenario(engine: Arc<Engine>) -> support::TestResult<()> {
    let root =
        std::path::PathBuf::from("/private/tmp/g48/markers").join(std::process::id().to_string());
    std::fs::create_dir_all(&root)?;
    let markers = Markers(root);
    partial_state(&engine).await?;
    commands(&engine, markers.0.to_str().unwrap()).await?;
    cascade::prove(&engine).await?;
    cancellation::prove(engine.clone()).await?;
    g54::prove(stackpress_framework_session::storage::ProfileActions::new(
        engine,
        stackpress_framework_session::storage::ProfileStore::new(String::new(), 0),
    ))
    .await?;
    Ok(())
}
fn main() {
    support::main(scenario);
}
