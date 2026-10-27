//! Generated writes remain authoritative RETURNING operations; MySQL refusal is measured.
use super::*;
use stackpress_framework_sql::InsertOutcome;
use stackpress_sql::{Insert, InsertConflict, Sql, SqlValue};

mod mysql;

/// Construct one strict generated create input from fixture-owned data.
fn create_profile(id: &str, name: &str) -> CreateProfile {
    serde_json::from_value(serde_json::json!({"id":id,"name":name})).unwrap()
}
/// Patch spelling stays owned by the emitted public/trusted contract.
fn patch_profile(name: &str) -> PatchProfile {
    serde_json::from_value(serde_json::json!({"name":name})).unwrap()
}
/// Native three-model CRUD includes authoritative defaults, strict booleans and FK failure.
async fn crud(engine: &Engine) -> TestResult<()> {
    let context = ValueContext::default();
    let mut db = DataSession::pooled(engine);
    assert!(
        Profiles::create(&mut db, create_profile("crud_owner", "Create"))
            .await?
            .active
    );
    assert_eq!(
        Profiles::update(&mut db, "crud_owner", patch_profile("Updated"), &context)
            .await?
            .expect("updated owner")
            .name,
        "Updated"
    );
    assert!(
        Profiles::update(&mut db, "missing", patch_profile("Absent"), &context)
            .await?
            .is_none()
    );
    Projects::create(
        &mut db,
        serde_json::from_value(
            serde_json::json!({"id":"crud_project","ownerId":"crud_owner","title":"Create"}),
        )
        .unwrap(),
    )
    .await?;
    assert_eq!(
        Projects::update(
            &mut db,
            "crud_project",
            serde_json::from_value(serde_json::json!({"title":"Updated"})).unwrap(),
            &context
        )
        .await?
        .expect("project")
        .title,
        "Updated"
    );
    Tasks::create(
        &mut db,
        serde_json::from_value(
            serde_json::json!({"id":"crud_task","projectId":"crud_project","title":"Create"}),
        )
        .unwrap(),
    )
    .await?;
    assert!(
        Tasks::update(
            &mut db,
            "crud_task",
            serde_json::from_value(serde_json::json!({"done":true})).unwrap(),
            &context
        )
        .await?
        .expect("task")
        .done
    );
    // A submitted failed write stays conservative; never infer a portable unique/FK category.
    match Projects::delete(
        &mut db,
        &ProjectKey {
            id: "crud_project".into(),
        },
    )
    .await
    {
        Ok(_) => panic!("RESTRICT foreign key unexpectedly allowed delete"),
        Err(error) => assert_eq!(error.kind(), DataErrorKind::UnknownOutcome),
    }
    assert!(Projects::get(&mut db, "crud_project").await?.is_some());
    assert!(
        Tasks::delete(
            &mut db,
            &TaskKey {
                id: "crud_task".into()
            }
        )
        .await?
        .is_some()
    );
    assert!(
        Tasks::delete(
            &mut db,
            &TaskKey {
                id: "crud_task".into()
            }
        )
        .await?
        .is_none()
    );
    assert!(
        Projects::delete(
            &mut db,
            &ProjectKey {
                id: "crud_project".into()
            }
        )
        .await?
        .is_some()
    );
    assert!(
        Profiles::delete(
            &mut db,
            &ProfileKey {
                id: "crud_owner".into()
            }
        )
        .await?
        .is_some()
    );
    Ok(())
}
/// Conflict branches retain the actual key and do not fabricate an existing row on no-change.
async fn conflicts(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    let context = ValueContext::default();
    let mut db = DataSession::pooled(engine);
    assert!(matches!(
        Profiles::create_if_absent(&mut db, create_profile("up", "First"), &context).await?,
        InsertOutcome::Inserted(_)
    ));
    assert!(matches!(
        Profiles::create_if_absent(&mut db, create_profile("up", "Ignored"), &context).await?,
        InsertOutcome::AlreadyExists
    ));
    assert_eq!(
        Profiles::upsert(
            &mut db,
            "up",
            create_profile("up", "Ignored"),
            patch_profile("Updated"),
            &context
        )
        .await?
        .name,
        "Updated"
    );
    assert_eq!(
        Profiles::upsert_on(
            &mut db,
            profile::ConflictKey::Primary(ProfileKey { id: "up".into() }),
            create_profile("up", "Ignored"),
            patch_profile("Targeted"),
            &context
        )
        .await?
        .id,
        "up"
    );
    assert_eq!(
        Profiles::upsert(
            &mut db,
            "up_new",
            create_profile("up_new", "Inserted"),
            patch_profile("Ignored"),
            &context
        )
        .await?
        .name,
        "Inserted"
    );
    let before = calls.snapshot();
    preflight::refused(
        Profiles::upsert(
            &mut db,
            "wrong",
            create_profile("up", "Wrong"),
            patch_profile("Wrong"),
            &context,
        )
        .await,
        DataErrorKind::Validation,
    );
    assert_eq!(calls.snapshot(), before);

    // A focused core case verifies PostgreSQL's explicit non-primary unique target semantics.
    engine.execute(&Sql::literal(engine.dialect(),"CREATE TABLE store_conflicts (id BIGINT PRIMARY KEY, alias VARCHAR(191) NOT NULL UNIQUE, name TEXT NOT NULL)")).await?;
    let input = |id| {
        Insert::new("store_conflicts")
            .columns(["id", "alias", "name"])
            .values(vec![
                SqlValue::Integer(id),
                SqlValue::Text("same".into()),
                SqlValue::Text("First".into()),
            ])
    };
    engine.execute(&input(1)).await?;
    let result = engine
        .execute(
            &input(2)
                .on_conflict(InsertConflict::DoNothing {
                    target: vec!["alias".into()],
                })
                .returning(["id", "name"]),
        )
        .await?;
    assert!(result.rows.is_empty());
    let result = engine
        .execute(
            &input(2)
                .on_conflict(InsertConflict::DoUpdate {
                    target: vec!["alias".into()],
                    assignments: vec![(
                        "name".into(),
                        SqlValue::Text("Unique updated".into()).into(),
                    )],
                })
                .returning(["id", "name"]),
        )
        .await?;
    assert_eq!(result.rows[0].try_get::<i64>("id")?, 1);
    assert_eq!(result.rows[0].try_get::<String>("name")?, "Unique updated");
    Ok(())
}
/// Batches preflight completely, use one exact transaction and preserve ordered results.
async fn batches(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    let context = ValueContext::default();
    let before = calls.snapshot();
    let result = Profiles::batch_atomic(
        engine,
        vec![
            profile::BatchInput::Create {
                input: create_profile("batch", "First"),
            },
            profile::BatchInput::Update {
                key: ProfileKey { id: "batch".into() },
                patch: patch_profile("Second"),
            },
            profile::BatchInput::Upsert {
                key: ProfileKey { id: "batch".into() },
                input: create_profile("batch", "Ignored"),
                patch: patch_profile("Third"),
            },
        ],
        &context,
    )
    .await?;
    assert!(matches!(&result[0],profile::BatchOutcome::Created(row) if row.name=="First"));
    assert!(matches!(&result[1],profile::BatchOutcome::Updated(Some(row)) if row.name=="Second"));
    assert!(matches!(&result[2],profile::BatchOutcome::Upserted(row) if row.name=="Third"));
    assert_eq!(
        calls.snapshot().since(before),
        observe::Snapshot {
            begins: 1,
            runs: 3,
            commits: 1,
            rollbacks: 0
        }
    );
    match Profiles::batch_atomic(
        engine,
        vec![
            profile::BatchInput::Create {
                input: create_profile("must_rollback", "New"),
            },
            profile::BatchInput::Create {
                input: create_profile("batch", "Duplicate"),
            },
        ],
        &context,
    )
    .await
    {
        Ok(_) => panic!("duplicate batch unexpectedly succeeded"),
        Err(error) => assert_eq!(error.kind(), DataErrorKind::UnknownOutcome),
    }
    assert!(
        Profiles::get(&mut DataSession::pooled(engine), "must_rollback")
            .await?
            .is_none()
    );
    let mut transaction = engine.begin().await?;
    let before = calls.snapshot();
    preflight::refused(
        Profiles::batch(
            &mut DataSession::transaction(&mut transaction),
            vec![
                profile::BatchInput::Create {
                    input: create_profile("not_submitted", "Valid"),
                },
                profile::BatchInput::Create {
                    input: create_profile("invalid", ""),
                },
            ],
            &context,
        )
        .await,
        DataErrorKind::Validation,
    );
    assert_eq!(calls.snapshot(), before);
    transaction.rollback().await?;
    Ok(())
}
/// The caller's same session owns both store families across awaits, with explicit completion.
async fn transaction(engine: &Engine, commit: bool) -> TestResult<()> {
    let id = if commit { "committed" } else { "rolled" };
    let mut transaction = engine.begin().await?;
    let mut db = DataSession::transaction(&mut transaction);
    Profiles::create(&mut db, create_profile(id, "Owner")).await?;
    Projects::create(
        &mut db,
        serde_json::from_value(serde_json::json!({"id":id,"ownerId":id,"title":"Project"}))
            .unwrap(),
    )
    .await?;
    assert!(Profiles::get(&mut db, id).await?.is_some());
    if commit {
        transaction.commit().await?;
    } else {
        transaction.rollback().await?;
    }
    let mut db = DataSession::pooled(engine);
    assert_eq!(Profiles::get(&mut db, id).await?.is_some(), commit);
    assert_eq!(Projects::get(&mut db, id).await?.is_some(), commit);
    Ok(())
}
/// Active transitions are idempotent and independent of hard deletion or child existence.
async fn active(engine: &Engine) -> TestResult<()> {
    let context = ValueContext::default();
    let key = ProfileKey { id: "u00".into() };
    let mut db = DataSession::pooled(engine);
    for _ in 0..2 {
        assert!(
            !Profiles::remove(&mut db, &key, &context)
                .await?
                .expect("removed row")
                .active
        );
        assert!(Profiles::get(&mut db, "u00").await?.is_none());
        assert!(
            !Profiles::get_maintenance(
                &mut db,
                &key,
                stackpress_framework_sql::stores::Visibility::All
            )
            .await?
            .expect("maintenance row")
            .active
        );
    }
    for _ in 0..2 {
        assert!(
            Profiles::restore(&mut db, &key, &context)
                .await?
                .expect("restored row")
                .active
        );
        assert!(Profiles::get(&mut db, "u00").await?.is_some());
    }
    Ok(())
}
/// Run the same operations, with exact typed pre-IO refusal where MySQL lacks RETURNING.
pub(super) async fn check(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    if engine.dialect() == DialectId::Mysql {
        mysql::check(engine, calls).await?;
    } else {
        crud(engine).await?;
        conflicts(engine, calls).await?;
        batches(engine, calls).await?;
        transaction(engine, false).await?;
        transaction(engine, true).await?;
        active(engine).await?;
    }
    Ok(())
}
