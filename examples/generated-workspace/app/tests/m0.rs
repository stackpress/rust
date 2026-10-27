//! Actual generated calls, current host services and borrow/Send contracts.
#![cfg(not(any(feature = "matrix", feature = "names")))]

use std::sync::Arc;
use std::time::Duration;

use generated_workspace_app::{
    ProfileReadPolicy,
    generated::{
        contracts::{profile::CreateProfile, project::CreateProject, task::CreateTask},
        stores::{profile::Profiles, project::Projects, task::Tasks},
    },
};
use stackpress_framework_schema::Input;
use stackpress_framework_sql::DataSession;
use stackpress_lib::queue::task;
use stackpress_lib::{Flow, Request, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::{Engine, Select};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Enforce an outer deadline even if nested acquisition blocks the child runtime.
#[test]
fn generated_store_contracts() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove_stores());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "generated_store_contracts",
        "stores",
        Duration::from_secs(10),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// The app explicitly authorizes this test's read operation; a model grants no access itself.
fn allow(_: &Request) -> Result<()> {
    Ok(())
}

/// Exercise the real three-model tables, JSON/enum/default codecs, transaction and named handler.
async fn prove_stores() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("workspace.sqlite")).create_if_missing(true);
    let engine = Arc::new(Engine::new(SqliteDatabase::open(options).await.unwrap()));
    for table in generated_workspace_app::generated::stores::schema::definition().unwrap() {
        engine
            .execute_plan(&table.create().render(engine.dialect()).unwrap())
            .await
            .unwrap();
    }
    let mut tx = engine.begin().await.unwrap();
    {
        let mut session = DataSession::transaction(&mut tx);
        // All calls below execute emitted stores, never handwritten substitute queries.
        let profile: CreateProfile = serde_json::from_value(serde_json::json!({ "id": "u1", "name": "Ada", "address": { "city": "Manila", "country": "PH" } })).unwrap();
        assert!(
            Profiles::create(&mut session, profile)
                .await
                .unwrap()
                .active
        );
        let inactive = CreateProfile {
            id: Input::Value("inactive".into()),
            name: Input::Value("Hidden".into()),
            active: Input::Value(false),
            ..Default::default()
        };
        assert!(
            !Profiles::create(&mut session, inactive)
                .await
                .unwrap()
                .active
        );
        assert!(
            Profiles::get(&mut session, "inactive")
                .await
                .unwrap()
                .is_none(),
            "ordinary get exposed an inactive record"
        );
        tokio::task::yield_now().await;
        let project: CreateProject = serde_json::from_value(
            serde_json::json!({ "id": "p1", "title": "Launch", "ownerId": "u1" }),
        )
        .unwrap();
        let project = Projects::create(&mut session, project).await.unwrap();
        assert_eq!(project.tags, Vec::<String>::new());
        assert_eq!(serde_json::to_value(project.state).unwrap(), "open");
        let task: CreateTask = serde_json::from_value(
            serde_json::json!({ "id": "t1", "projectId": "p1", "title": "Draft" }),
        )
        .unwrap();
        // A transaction-borrowing generated call must itself be Send, not just executable
        // on this current-thread test runtime. The trait object makes the compiler prove it.
        let create: std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = stackpress_framework_sql::DataResult<
                            generated_workspace_app::generated::contracts::task::Task,
                        >,
                    > + Send
                    + '_,
            >,
        > = Box::pin(Tasks::create(&mut session, task));
        assert!(!create.await.unwrap().done);
        assert_eq!(
            Profiles::get(&mut session, "u1")
                .await
                .unwrap()
                .unwrap()
                .name,
            "Ada"
        );
        assert_eq!(
            Projects::get(&mut session, "p1")
                .await
                .unwrap()
                .unwrap()
                .owner_id,
            "u1"
        );
        assert!(Tasks::get(&mut session, "t1").await.unwrap().is_some());
        // An ordinary external read has no visibility into these uncommitted writes.
        assert!(
            engine
                .fetch(&Select::default().from("profile"))
                .await
                .unwrap()
                .is_empty()
        );
    }
    tx.rollback().await.unwrap();
    for table in ["profile", "project", "task"] {
        assert!(
            engine
                .fetch(&Select::default().from(table))
                .await
                .unwrap()
                .is_empty()
        );
    }
    // A second explicit transaction is committed by the caller, never by Flow::Continue.
    let mut tx = engine.begin().await.unwrap();
    let profile = CreateProfile {
        id: Input::Value("u1".into()),
        name: Input::Value("Ada".into()),
        ..Default::default()
    };
    Profiles::create(&mut DataSession::transaction(&mut tx), profile)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut builder = ServerBuilder::new();
    let shared = Arc::clone(&engine);
    builder
        .on_config(task(move |startup: &mut StartupContext| {
            let engine = Arc::clone(&shared);
            Box::pin(async move {
                startup.register(engine)?;
                startup.register(Arc::new(ProfileReadPolicy(allow)))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    // Registering this named async function proves the real Handler future is Send.
    builder
        .on("profile-name", generated_workspace_app::profile_name)
        .unwrap();
    let app = builder.build().await.unwrap();
    let reply = app
        .call("profile-name", serde_json::json!({ "id": "u1" }))
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(reply.response.result().unwrap()).unwrap(),
        serde_json::json!({ "name": "Ada" })
    );
    let fingerprint = generated_workspace_app::generated::metadata::schema::FINGERPRINT;
    assert_eq!(
        fingerprint,
        generated_workspace_app::generated::stores::profile::FINGERPRINT
    );
    assert_eq!(
        fingerprint,
        generated_workspace_app::generated::stores::project::FINGERPRINT
    );
    assert_eq!(
        fingerprint,
        generated_workspace_app::generated::stores::task::FINGERPRINT
    );
    engine.close().await.unwrap();
}

/// Omission, explicit null, unknown fields and empty validation remain distinct.
#[test]
fn create_inputs_preserve_omission_and_null() {
    let missing: CreateProfile =
        serde_json::from_value(serde_json::json!({ "id": "u", "name": "Ada" })).unwrap();
    assert!(matches!(missing.active, Input::Missing));
    assert!(missing.prepare().unwrap().active);
    let null: CreateProfile =
        serde_json::from_value(serde_json::json!({ "id": "u", "name": "Ada", "active": null }))
            .unwrap();
    assert!(matches!(null.active, Input::Null));
    assert!(null.prepare().is_err());
    assert!(
        serde_json::from_value::<CreateProfile>(
            serde_json::json!({ "id": "u", "name": "Ada", "bogus": true })
        )
        .is_err()
    );
    assert!(CreateProfile::default().prepare().is_err());
    let value = CreateProfile {
        id: Input::Value("u".into()),
        name: Input::Value(String::new()),
        ..Default::default()
    }
    .prepare();
    assert!(value.is_err());
}

/// Borrow aliasing, Clone, lifetime escape and service registration must fail at compilation.
#[test]
fn exclusive_session_compile_failures() {
    if std::env::var("STACKPRESS_S6_SCENARIO").as_deref() == Ok("compile") {
        trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
        return;
    }
    // Fresh compiler fixtures get the same separate finite build budget as bootstrap Cargo
    // checks; the native transaction/locking cases retain their shorter ten-second bound.
    let (ok, expired, output) = watchdog::run(
        "exclusive_session_compile_failures",
        "compile",
        Duration::from_secs(180),
    );
    assert!(ok && !expired, "compile timeout={expired}\n{output}");
}
