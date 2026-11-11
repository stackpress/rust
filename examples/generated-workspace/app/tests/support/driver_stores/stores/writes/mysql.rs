//! Typed MySQL mutation refusals, independently observed before any executor entry.
use super::*;
use stackpress_sql::ErrorKind;

/// Every executing MySQL write is refused without entering its owner or borrowed session.
pub(super) async fn check(engine: &Engine, calls: &observe::Calls) -> TestResult<()> {
    let context = ValueContext::default();
    let mut db = DataSession::pooled(engine);
    macro_rules! refused {
        ($future:expr) => {{
            let before = calls.snapshot();
            preflight::refused($future.await, DataErrorKind::Unsupported);
            assert_eq!(calls.snapshot(), before);
        }};
    }
    // Exercise the complete three-model method families, not just the common capability helper.
    refused!(Profiles::create(
        &mut db,
        create_profile("blocked", "Owner")
    ));
    refused!(Profiles::create_with(
        &mut db,
        create_profile("blocked", "Owner"),
        &context
    ));
    refused!(Profiles::create_if_absent(
        &mut db,
        create_profile("blocked", "Owner"),
        &context
    ));
    refused!(Profiles::update(
        &mut db,
        "u00",
        patch_profile("Blocked"),
        &context
    ));
    refused!(Profiles::update_key(
        &mut db,
        &ProfileKey { id: "u00".into() },
        patch_profile("Blocked"),
        &context
    ));
    refused!(Profiles::upsert(
        &mut db,
        "u00",
        create_profile("u00", "Ignored"),
        patch_profile("Blocked"),
        &context
    ));
    refused!(Profiles::upsert_key(
        &mut db,
        &ProfileKey { id: "u00".into() },
        create_profile("u00", "Ignored"),
        patch_profile("Blocked"),
        &context
    ));
    refused!(Profiles::upsert_on(
        &mut db,
        profile::ConflictKey::Primary(ProfileKey { id: "u00".into() }),
        create_profile("u00", "Ignored"),
        patch_profile("Blocked"),
        &context
    ));
    refused!(Profiles::delete(&mut db, &ProfileKey { id: "u00".into() }));
    refused!(Profiles::remove(
        &mut db,
        &ProfileKey { id: "u00".into() },
        &context
    ));
    refused!(Profiles::restore(
        &mut db,
        &ProfileKey { id: "u00".into() },
        &context
    ));

    let project: CreateProject = serde_json::from_value(
        serde_json::json!({"id":"blocked_p","ownerId":"u00","title":"Blocked"}),
    )
    .unwrap();
    let patch: PatchProject =
        serde_json::from_value(serde_json::json!({"title":"Blocked"})).unwrap();
    refused!(Projects::create(&mut db, project.clone()));
    refused!(Projects::create_with(&mut db, project.clone(), &context));
    refused!(Projects::create_if_absent(
        &mut db,
        project.clone(),
        &context
    ));
    refused!(Projects::update(&mut db, "p00-0", patch.clone(), &context));
    refused!(Projects::update_key(
        &mut db,
        &ProjectKey { id: "p00-0".into() },
        patch.clone(),
        &context
    ));
    refused!(Projects::upsert(
        &mut db,
        "blocked_p",
        project.clone(),
        patch.clone(),
        &context
    ));
    refused!(Projects::upsert_key(
        &mut db,
        &ProjectKey {
            id: "blocked_p".into()
        },
        project.clone(),
        patch.clone(),
        &context
    ));
    refused!(Projects::upsert_on(
        &mut db,
        project::ConflictKey::Primary(ProjectKey {
            id: "blocked_p".into()
        }),
        project.clone(),
        patch.clone(),
        &context
    ));
    refused!(Projects::delete(
        &mut db,
        &ProjectKey { id: "p00-0".into() }
    ));

    let task: CreateTask = serde_json::from_value(
        serde_json::json!({"id":"blocked_t","projectId":"p00-0","title":"Blocked"}),
    )
    .unwrap();
    let patch: PatchTask = serde_json::from_value(serde_json::json!({"done":true})).unwrap();
    refused!(Tasks::create(&mut db, task.clone()));
    refused!(Tasks::create_with(&mut db, task.clone(), &context));
    refused!(Tasks::create_if_absent(&mut db, task.clone(), &context));
    refused!(Tasks::update(&mut db, "t00-0-0", patch.clone(), &context));
    refused!(Tasks::update_key(
        &mut db,
        &TaskKey {
            id: "t00-0-0".into()
        },
        patch.clone(),
        &context
    ));
    refused!(Tasks::upsert(
        &mut db,
        "blocked_t",
        task.clone(),
        patch.clone(),
        &context
    ));
    refused!(Tasks::upsert_key(
        &mut db,
        &TaskKey {
            id: "blocked_t".into()
        },
        task.clone(),
        patch.clone(),
        &context
    ));
    refused!(Tasks::upsert_on(
        &mut db,
        task::ConflictKey::Primary(TaskKey {
            id: "blocked_t".into()
        }),
        task.clone(),
        patch.clone(),
        &context
    ));
    refused!(Tasks::delete(
        &mut db,
        &TaskKey {
            id: "t00-0-0".into()
        }
    ));

    // The caller's BEGIN is outside each borrowed-batch boundary; no batch submits another call.
    let mut transaction = engine.begin().await?;
    let mut db = DataSession::transaction(&mut transaction);
    refused!(Profiles::batch(
        &mut db,
        vec![profile::BatchInput::Create {
            input: create_profile("blocked", "Owner")
        }],
        &context
    ));
    refused!(Projects::batch(
        &mut db,
        vec![project::BatchInput::Create {
            input: project.clone()
        }],
        &context
    ));
    refused!(Tasks::batch(
        &mut db,
        vec![task::BatchInput::Create {
            input: task.clone()
        }],
        &context
    ));
    transaction.rollback().await?;
    refused!(Profiles::batch_atomic(
        engine,
        vec![profile::BatchInput::Create {
            input: create_profile("blocked", "Owner")
        }],
        &context
    ));
    refused!(Projects::batch_atomic(
        engine,
        vec![project::BatchInput::Create { input: project }],
        &context
    ));
    refused!(Tasks::batch_atomic(
        engine,
        vec![task::BatchInput::Create { input: task }],
        &context
    ));

    // Core targeted conflicts are independently refused, without relying on a RETURNING clause.
    for conflict in [
        InsertConflict::DoNothing {
            target: vec!["id".into()],
        },
        InsertConflict::DoUpdate {
            target: vec!["id".into()],
            assignments: vec![("name".into(), SqlValue::Text("Blocked".into()).into())],
        },
    ] {
        let before = calls.snapshot();
        let query = Insert::new("profile")
            .columns(["id", "name", "active"])
            .values(vec![
                SqlValue::Text("u00".into()),
                SqlValue::Text("Blocked".into()),
                SqlValue::Bool(true),
            ])
            .on_conflict(conflict);
        match engine.execute(&query).await {
            Ok(_) => panic!("MySQL targeted conflict unexpectedly executed"),
            Err(error) => assert_eq!(error.kind(), ErrorKind::Unsupported),
        }
        assert_eq!(calls.snapshot(), before);
    }
    let mut db = DataSession::pooled(engine);
    assert_eq!(
        Profiles::get(&mut db, "u00")
            .await?
            .expect("unchanged owner")
            .name,
        "Owner"
    );
    assert!(Profiles::get(&mut db, "blocked").await?.is_none());
    Ok(())
}
