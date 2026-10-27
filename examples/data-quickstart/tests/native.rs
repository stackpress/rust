//! Execute the new generated source's full callable lifecycle on real independent SQLite files.
#![cfg(feature = "native")]

use data_quickstart as generated_workspace_app;
use data_quickstart::generated::{
    contracts::{
        profile::CreateProfile,
        project::CreateProject,
        public::{PatchProfile, PatchProject, PatchTask, ProfileKey, ProjectKey, TaskKey},
        task::CreateTask,
    },
    stores::{profile::Profiles, project::Projects, task::Tasks},
};
use serde_json::json;
use stackpress_framework_schema::ValueContext;
use stackpress_framework_sql::DataSession;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

#[path = "../../generated-workspace/app/tests/support/native_calls.rs"]
mod native_calls;
mod support;
#[path = "../../generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Bound native acquisition, statements, transaction completion and close in a separate process.
#[test]
fn callable_lifecycle() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "callable_lifecycle",
        "quickstart",
        std::time::Duration::from_secs(25),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Exercise the existing contract harness on this example's newly generated types, not substitutes.
async fn prove() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("quickstart.sqlite");
    let engine = support::installed(&path).await;
    let mut db = DataSession::pooled(&engine);
    let context = ValueContext::default();
    native_calls::native_calls!(
        profile,
        Profiles,
        ProfileStore,
        CreateProfile,
        ProfileKey,
        PatchProfile,
        json!({"id":"u1","name":"Ada"}),
        json!({"id":"u1"}),
        json!({"name":"Updated"}),
        &mut db,
        &context
    );
    #[cfg(not(feature = "evolved"))]
    let (project, patch) = (
        json!({"id":"p1","ownerId":"u1","title":"Ship API"}),
        json!({"title":"Ship UI"}),
    );
    #[cfg(feature = "evolved")]
    let (project, patch) = (
        json!({"id":"p1","ownerId":"u1","heading":"Ship API"}),
        json!({"heading":"Ship UI","summary":null}),
    );
    native_calls::native_calls!(
        project,
        Projects,
        ProjectStore,
        CreateProject,
        ProjectKey,
        PatchProject,
        project,
        json!({"id":"p1"}),
        patch.clone(),
        &mut db,
        &context
    );
    native_calls::native_calls!(
        task,
        Tasks,
        TaskStore,
        CreateTask,
        TaskKey,
        PatchTask,
        json!({"id":"t1","projectId":"p1","title":"Task"}),
        json!({"id":"t1"}),
        json!({"title":"Reviewed","done":true}),
        &mut db,
        &context
    );
    #[cfg(feature = "expanded")]
    {
        use data_quickstart::generated::contracts::{
            link::CreateLink,
            public::{LinkKey, PatchLink},
        };
        native_calls::native_calls!(
            link,
            Links,
            LinkStore,
            CreateLink,
            LinkKey,
            PatchLink,
            json!({"id":"l1","ownerId":"u1"}),
            json!({"id":"l1"}),
            json!({"ownerId":"u1"}),
            &mut db,
            &context
        );
        assert!(
            data_quickstart::generated::stores::link::Links::delete(
                &mut db,
                &LinkKey { id: "l1".into() }
            )
            .await
            .unwrap()
            .is_some()
        );
    }
    // Read actual include shapes and observed statement execution before removing dependent rows.
    let report = data_quickstart::queries::read(&engine).await.unwrap();
    assert_eq!(report["task_count"], 1);
    let observed = data_quickstart::queries::open_and_read(&path)
        .await
        .unwrap();
    assert_eq!(observed["task_count"], 1);
    assert_eq!(
        observed["statements"].as_array().unwrap().len(),
        if cfg!(feature = "expanded") { 5 } else { 4 }
    );
    assert!(
        data_quickstart::generated::metadata::ui::schema()
            .unwrap()
            .models
            .len()
            >= 3
    );
    assert!(
        Tasks::delete(&mut db, &TaskKey { id: "t1".into() })
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        Projects::delete(&mut db, &ProjectKey { id: "p1".into() })
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        Profiles::delete(&mut db, &ProfileKey { id: "u1".into() })
            .await
            .unwrap()
            .is_some()
    );
    assert!(Profiles::get(&mut db, "u1").await.unwrap().is_none());
    engine.close().await.unwrap();
    // The app awaits rollback and close when pointed at a valid but uninstalled file.
    let empty = root.path().join("empty.sqlite");
    let database = SqliteDatabase::open(SqliteOptions::file(&empty).create_if_missing(true))
        .await
        .unwrap();
    use stackpress_sql::Database;
    database.close().await.unwrap();
    assert!(
        data_quickstart::queries::open_and_read(&empty)
            .await
            .is_err()
    );
    assert!(
        data_quickstart::queries::open_and_read(&root.path().join("missing.sqlite"))
            .await
            .is_err()
    );
    assert!(!root.path().join("missing.sqlite").exists());
}
