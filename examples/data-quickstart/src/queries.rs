//! Trusted domain queries over the exact caller-owned transaction; no transport exposure.

use crate::generated::stores::{
    profile::Profiles,
    project::{Projects, Query as ProjectQuery, View3},
    task::{Query as TaskQuery, Tasks},
};
use serde_json::{Value, json};
use stackpress_framework_sql::stores::Page;
use stackpress_framework_sql::{DataResult, DataSession, finish};
use stackpress_sql::Engine;

/// Read one profile, a deterministic project page with two relation includes, and task count.
///
/// # Errors
/// Returns checked query/decoding/SQL failures. All statements borrow one read transaction;
/// completion or rollback is awaited and a cleanup failure remains attached to the primary error.
pub async fn read(engine: &Engine) -> DataResult<Value> {
    let mut transaction = engine.begin().await?;
    let outcome = async {
        let mut db = DataSession::transaction(&mut transaction);
        let profile = Profiles::get(&mut db, "u1").await?;
        // Select a base page before loading its owner/tasks; includes never multiply the page.
        let projects = project_page(&mut db).await?;
        let tasks = Tasks::count(&mut db, TaskQuery::default()).await?;
        #[cfg(feature = "expanded")]
        let links = crate::generated::stores::link::Links::count(
            &mut db,
            crate::generated::stores::link::Query::default(),
        )
        .await?;
        // This executable displays only its synthetic demo data through a trusted local call.
        // Exposing the operation to users would require a separate app policy and projection.
        let report = json!({"profile":profile,"projects":projects,"task_count":tasks});
        #[cfg(feature = "expanded")]
        let report = {
            let mut report = report;
            report["link_count"] = links.into();
            report
        };
        Ok(report)
    }
    .await;
    finish(transaction, outcome).await
}

/// Search stored text and page base identities before eagerly loading the two explicit relations.
///
/// # Errors
/// Requires a transaction because total and includes span several SQL statements.
pub async fn project_page(db: &mut DataSession<'_>) -> DataResult<Page<View3>> {
    Projects::search(
        db,
        ProjectQuery {
            q: Some("Ship".into()),
            skip: 1,
            take: 1,
            total: true,
            includes: vec!["owner".into(), "tasks".into()],
            ..Default::default()
        },
    )
    .await
}

/// Open an existing file, read the example, and await pool close on success or failure.
///
/// # Errors
/// Never creates or installs a file. Returns the primary read failure with secondary close
/// context when applicable; a lone close failure is still an error.
pub async fn open_and_read(path: &std::path::Path) -> DataResult<Value> {
    let database = stackpress_sql_sqlite::SqliteDatabase::open(
        stackpress_sql_sqlite::SqliteOptions::file(path),
    )
    .await?;
    let statements = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let engine = Engine::new(crate::statements::Observed {
        database,
        statements: statements.clone(),
    });
    let outcome = read(&engine).await;
    let cleanup = engine.close().await;
    match (outcome, cleanup) {
        (Ok(mut report), Ok(())) => {
            report["statements"] = serde_json::to_value(&*statements.lock().map_err(|_| {
                stackpress_framework_sql::DataError::new("demo statement lock is poisoned")
            })?)
            .map_err(stackpress_framework_sql::codec_error)?;
            Ok(report)
        }
        (Err(error), Ok(())) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
        (Err(error), Err(cleanup)) => Err(error.with_cleanup(cleanup)),
    }
}
