//! The default package executes the exact pure DTO/validation source used by the native app.

use data_quickstart::generated::{
    contracts::{profile::CreateProfile, project::CreateProject, task::CreateTask},
    metadata::ui,
};
use serde_json::json;

/// Distinguish required missing/null/empty values, false defaults and strict declared schema names.
#[test]
fn pure_contracts() {
    let valid: CreateProfile = serde_json::from_value(json!({"id":"u1","name":"Ada"})).unwrap();
    assert_eq!(valid.prepare().unwrap().name, "Ada");
    for invalid in [
        json!({}),
        json!({"id":null,"name":"Ada"}),
        json!({"id":"u1","name":""}),
    ] {
        assert!(
            serde_json::from_value::<CreateProfile>(invalid)
                .unwrap()
                .prepare()
                .is_err()
        );
    }
    #[cfg(not(feature = "evolved"))]
    let project = json!({"id":"p1","ownerId":"u1","title":"Ship"});
    #[cfg(feature = "evolved")]
    let project = json!({"id":"p1","ownerId":"u1","heading":"Ship"});
    let prepared = serde_json::from_value::<CreateProject>(project)
        .unwrap()
        .prepare()
        .unwrap();
    assert_eq!(prepared.owner_id, "u1");
    #[cfg(feature = "evolved")]
    assert_eq!(prepared.summary, None);
    assert!(serde_json::from_value::<CreateProject>(json!({"unknown":true})).is_err());
    let task: CreateTask =
        serde_json::from_value(json!({"id":"t1","projectId":"p1","title":"Review"})).unwrap();
    assert!(!task.prepare().unwrap().done);
    let schema = ui::schema().unwrap();
    assert_eq!(
        schema.models.len(),
        if cfg!(feature = "expanded") { 4 } else { 3 }
    );
}
