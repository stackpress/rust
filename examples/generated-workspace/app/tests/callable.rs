//! The original generated app supports the same detached/native calls as the expanded matrix.
#![cfg(not(any(feature = "matrix", feature = "names")))]
use generated_workspace_app::generated::contracts::{
    profile::CreateProfile,
    project::CreateProject,
    public::{PatchProfile, PatchProject, PatchTask, ProfileKey, ProjectKey, TaskKey},
    task::CreateTask,
};
use serde_json::json;
use stackpress_framework_schema::ValueContext;
use stackpress_framework_sql::DataSession;
use std::time::Duration;
#[path = "support/native_calls.rs"]
mod native_calls;
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;
/// Bound exact native sessions and explicit snapshot/atomic conveniences under an outer watchdog.
#[test]
fn callable_store_api() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) =
        watchdog::run("callable_store_api", "callable", Duration::from_secs(20));
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Preserve FK ownership while preparing, reading and updating all three native model identities.
async fn prove() {
    let fixture = support::fixture().await;
    let mut session = DataSession::pooled(&fixture.engine);
    let context = ValueContext::default();
    native_calls::native_calls!(
        profile,
        Profiles,
        ProfileStore,
        CreateProfile,
        ProfileKey,
        PatchProfile,
        json!({"id":"native-profile","name":"Profile"}),
        json!({"id":"native-profile"}),
        json!({"name":"Changed"}),
        &mut session,
        &context
    );
    native_calls::native_calls!(
        project,
        Projects,
        ProjectStore,
        CreateProject,
        ProjectKey,
        PatchProject,
        json!({"id":"native-project","ownerId":"native-profile","title":"Project"}),
        json!({"id":"native-project"}),
        json!({"title":"Changed","tags":["visible"]}),
        &mut session,
        &context
    );
    native_calls::native_calls!(
        task,
        Tasks,
        TaskStore,
        CreateTask,
        TaskKey,
        PatchTask,
        json!({"id":"native-task","projectId":"native-project","title":"Task"}),
        json!({"id":"native-task"}),
        json!({"title":"Changed","done":true}),
        &mut session,
        &context
    );
    let ui = generated_workspace_app::generated::metadata::ui::schema().unwrap();
    assert_eq!(ui.models.len(), 3);
    fixture.engine.close().await.unwrap();
}
