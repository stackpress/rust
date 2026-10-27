//! Generated DDL on SQLite/PostgreSQL; explicitly harness-owned widths on MySQL.
use super::*;
use stackpress_framework_sql::StorageColumn;
use stackpress_sql::{ErrorKind, Insert, Sql, SqlValue};

/// Install the exact generated tables unless the backend deliberately refuses their TEXT keys.
pub(super) async fn install(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    let definitions = generated_workspace_app::generated::stores::schema::definition()?;
    if engine.dialect() == DialectId::Mysql {
        let before = calls.snapshot();
        for table in definitions {
            match table.create().render(DialectId::Mysql) {
                Ok(_) => panic!("generated MySQL TEXT-key DDL unexpectedly rendered"),
                Err(error) => assert_eq!(error.kind(), ErrorKind::Unsupported),
            }
        }
        assert_eq!(calls.snapshot(), before);
        // These physical widths belong only to this fixture, not generated schema policy.
        println!("generated stores Mysql: installing harness-owned VARCHAR(191) / InnoDB schema");
        for sql in [
            "CREATE TABLE profile (id VARCHAR(191) NOT NULL PRIMARY KEY, name VARCHAR(191) NOT NULL, active BOOLEAN NOT NULL, address TEXT NULL, INDEX profile_name_index (name)) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin",
            "CREATE TABLE project (id VARCHAR(191) NOT NULL PRIMARY KEY, title VARCHAR(191) NOT NULL, owner_id VARCHAR(191) NOT NULL, state TEXT NOT NULL, tags TEXT NOT NULL, INDEX project_title_index (title), CONSTRAINT project_owner_id_fk FOREIGN KEY (owner_id) REFERENCES profile (id) ON DELETE RESTRICT) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin",
            "CREATE TABLE task (id VARCHAR(191) NOT NULL PRIMARY KEY, project_id VARCHAR(191) NOT NULL, title TEXT NOT NULL, done BOOLEAN NOT NULL, CONSTRAINT task_project_id_fk FOREIGN KEY (project_id) REFERENCES project (id) ON DELETE RESTRICT) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_bin",
        ] {
            engine.execute(&Sql::literal(DialectId::Mysql, sql)).await?;
        }
    } else {
        // Use generated metadata and plans without rewriting any type or constraint.
        for table in definitions {
            engine
                .execute_plan(&table.create().render(engine.dialect())?)
                .await?;
        }
    }
    Ok(())
}
/// Bind prepared generated values into a seed-only insert, without claiming RETURNING support.
async fn insert(
    db: &mut DataSession<'_>,
    table: &'static str,
    columns: &[StorageColumn],
    values: Vec<SqlValue>,
) -> DataResult<()> {
    db.execute(
        &Insert::new(table)
            .columns(columns.iter().map(|column| column.column))
            .values(values),
    )
    .await?;
    Ok(())
}
/// MySQL seeds use generated preparation/codecs; generated executing writes stay refused.
async fn profile(
    db: &mut DataSession<'_>,
    input: CreateProfile,
    context: &ValueContext<'_>,
) -> DataResult<()> {
    if db.dialect() == DialectId::Mysql {
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        insert(
            db,
            "profile",
            profile::COLUMNS,
            ProfileStore::encode(&value)?,
        )
        .await
    } else {
        Profiles::create_with(db, input, context).await.map(|_| ())
    }
}
/// Seed project enum/list codecs using the same declared column order.
async fn project(
    db: &mut DataSession<'_>,
    input: CreateProject,
    context: &ValueContext<'_>,
) -> DataResult<()> {
    if db.dialect() == DialectId::Mysql {
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        insert(
            db,
            "project",
            project::COLUMNS,
            ProjectStore::encode(&value)?,
        )
        .await
    } else {
        Projects::create_with(db, input, context).await.map(|_| ())
    }
}
/// Seed both Boolean values, preserving the native MySQL integer-to-bool decode boundary.
async fn task(
    db: &mut DataSession<'_>,
    input: CreateTask,
    context: &ValueContext<'_>,
) -> DataResult<()> {
    if db.dialect() == DialectId::Mysql {
        let value = input
            .prepare_with(context)
            .map_err(stackpress_framework_sql::stores::preparation_error)?;
        insert(db, "task", task::COLUMNS, TaskStore::encode(&value)?).await
    } else {
        Tasks::create_with(db, input, context).await.map(|_| ())
    }
}
/// Populate one and twenty-parent cases on the caller's exact exclusive transaction.
async fn rows(db: &mut DataSession<'_>) -> DataResult<()> {
    let context = ValueContext::default();
    for owner in 0..20 {
        profile(
            db,
            serde_json::from_value(serde_json::json!({
                "id":format!("u{owner:02}"), "name":"Owner",
                "address":{"city":"Manila","country":"PH"}
            }))
            .unwrap(),
            &context,
        )
        .await?;
        for project_index in 0..2 {
            let id = format!("p{owner:02}-{project_index}");
            project(
                db,
                serde_json::from_value(serde_json::json!({
                    "id":id, "ownerId":format!("u{owner:02}"),
                    "title":if owner==0 {"Literal%_"}else{"literal"},
                    "tags":if project_index==0 {vec!["rust","sql"]}else{vec!["rust"]}
                }))
                .unwrap(),
                &context,
            )
            .await?;
            for task_index in 0..2 {
                task(
                    db,
                    serde_json::from_value(serde_json::json!({
                        "id":format!("t{owner:02}-{project_index}-{task_index}"),
                        "projectId":id, "title":"Task", "done":task_index==1
                    }))
                    .unwrap(),
                    &context,
                )
                .await?;
            }
        }
    }
    // Empty and hidden parents distinguish unloaded, loaded-empty and active visibility.
    profile(
        db,
        serde_json::from_value(serde_json::json!({"id":"empty","name":"Empty"})).unwrap(),
        &context,
    )
    .await?;
    profile(
        db,
        serde_json::from_value(serde_json::json!({"id":"hidden","name":"Hidden","active":false}))
            .unwrap(),
        &context,
    )
    .await?;
    Ok(())
}
/// Atomic seeding is explicit; error/commit handling uses the real framework completion helper.
pub(super) async fn seed(engine: &Engine) -> TestResult<()> {
    let mut transaction = engine.begin().await?;
    let result = rows(&mut DataSession::transaction(&mut transaction)).await;
    stackpress_framework_sql::finish(transaction, result).await?;
    Ok(())
}
