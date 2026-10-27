//! The exact generated contracts are exercised in both native and pure consumer packages.

use super::contracts::{
    address::Address,
    profile::{CreateProfile, Profile},
    project::{CreateProject, Project},
    project_state::ProjectState,
    task::{CreateTask, Task},
};

/// Check real missing/null/default/value/unknown/enum/embedded/collection behavior in each consumer.
pub fn prove_contracts() {
    let profile = serde_json::json!({"id":"u1","name":"Ada"});
    let project = serde_json::json!({"id":"p1","title":"Launch","ownerId":"u1"});
    let task = serde_json::json!({"id":"t1","title":"Draft","projectId":"p1"});
    for (kind, original, required, defaulted) in [
        (
            "profile",
            profile.clone(),
            vec!["id", "name"],
            vec!["active"],
        ),
        (
            "project",
            project.clone(),
            vec!["id", "title", "ownerId"],
            vec!["state", "tags"],
        ),
        (
            "task",
            task.clone(),
            vec!["id", "title", "projectId"],
            vec!["done"],
        ),
    ] {
        assert!(prepare(kind, original.clone()));
        // Required values reject both omission and explicit null. Defaulted fields accept
        // omission but reject null; the distinction is part of the transport contract.
        for field in required.iter().chain(&defaulted) {
            let mut missing = original.clone();
            missing.as_object_mut().unwrap().remove(*field);
            assert_eq!(prepare(kind, missing), defaulted.contains(field));
            let mut null = original.clone();
            null[*field] = serde_json::Value::Null;
            assert!(!prepare(kind, null));
        }
        let mut unknown = original.clone();
        unknown["private"] = serde_json::json!(true);
        assert!(!prepare(kind, unknown));
    }
    let input: CreateProfile = serde_json::from_value(
        serde_json::json!({"id":"u1","name":"Ada","active":false,"address":null}),
    )
    .unwrap();
    let value = input.prepare().unwrap();
    assert!(!value.active);
    assert!(value.address.is_none());
    let profile_json = serde_json::to_value(&value).unwrap();
    assert_eq!(
        serde_json::from_value::<Profile>(profile_json).unwrap(),
        value
    );
    let input: CreateTask = serde_json::from_value(task.clone()).unwrap();
    let value = input.prepare().unwrap();
    assert!(!value.done);
    assert_eq!(
        serde_json::from_value::<Task>(serde_json::to_value(&value).unwrap()).unwrap(),
        value
    );
    let input: CreateProject = serde_json::from_value(project.clone()).unwrap();
    let value = input.prepare().unwrap();
    assert_eq!(value.state, ProjectState::Open);
    assert!(value.tags.is_empty());
    assert_eq!(
        serde_json::from_value::<Project>(serde_json::to_value(&value).unwrap()).unwrap(),
        value
    );
    for state in [ProjectState::Open, ProjectState::Closed] {
        assert_eq!(
            serde_json::from_value::<ProjectState>(serde_json::to_value(&state).unwrap()).unwrap(),
            state
        );
    }
    let mut invalid = project.clone();
    invalid["state"] = serde_json::json!("unknown");
    assert!(!prepare("project", invalid));
    let mut nested = profile.clone();
    nested["address"] = serde_json::json!({"city":"Manila","country":"PH"});
    assert!(prepare("profile", nested.clone()));
    nested["address"]["city"] = serde_json::json!("");
    assert!(!prepare("profile", nested.clone()));
    nested["address"] = serde_json::json!({"city":"Manila","country":"x".repeat(1024 * 1024 + 1)});
    assert!(!prepare("profile", nested));
    let address = Address {
        city: "Manila".into(),
        country: "PH".into(),
    };
    assert_eq!(
        serde_json::from_value::<Address>(serde_json::to_value(&address).unwrap()).unwrap(),
        address
    );
    for (kind, mut original, text) in [
        ("profile", profile, "name"),
        ("project", project.clone(), "title"),
        ("task", task, "title"),
    ] {
        original[text] = serde_json::json!("");
        assert!(!prepare(kind, original.clone()));
        original[text] = serde_json::json!("x".repeat(1024 * 1024 + 1));
        assert!(!prepare(kind, original));
    }
    let mut oversized = project.clone();
    oversized["tags"] = serde_json::json!(vec!["x"; 1025]);
    assert!(!prepare("project", oversized));
    let mut large_tag = project;
    large_tag["tags"] = serde_json::json!(["x".repeat(1024 * 1024 + 1)]);
    assert!(!prepare("project", large_tag));
}

/// Use each concrete emitted input/preparation method; no handwritten substitute validator.
fn prepare(kind: &str, json: serde_json::Value) -> bool {
    match kind {
        "profile" => serde_json::from_value::<CreateProfile>(json)
            .ok()
            .is_some_and(|input| input.prepare().is_ok()),
        "project" => serde_json::from_value::<CreateProject>(json)
            .ok()
            .is_some_and(|input| input.prepare().is_ok()),
        "task" => serde_json::from_value::<CreateTask>(json)
            .ok()
            .is_some_and(|input| input.prepare().is_ok()),
        _ => panic!("unknown contract test family"),
    }
}
