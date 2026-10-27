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
    // A fieldset can be prepared independently with the same required/null policy as embedding.
    let address_input: super::contracts::address::CreateAddress =
        serde_json::from_value(serde_json::json!({"city":"Manila","country":"PH"})).unwrap();
    let context = stackpress_framework_schema::ValueContext::default();
    assert!(address_input.validate_create(&context).unwrap().is_empty());
    let address = address_input.prepare_create(&context).unwrap();
    let prepared = super::contracts::address::CreateAddress {
        city: stackpress_framework_schema::Input::Value("Manila".into()),
        country: stackpress_framework_schema::Input::Value("PH".into()),
    }
    .prepare()
    .unwrap();
    assert_eq!(address, prepared);
    assert_eq!(address.country, "PH");
    for field in ["city", "country"] {
        let mut value = serde_json::json!({"city":"Manila","country":"PH"});
        value[field] = serde_json::Value::Null;
        let input: super::contracts::address::CreateAddress =
            serde_json::from_value(value).unwrap();
        assert!(input.prepare().is_err());
    }
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

/// Execute the published M1 DTO surface in both native and pure consumers.
pub fn prove_public_contracts() {
    use super::contracts::public::*;
    use stackpress_framework_schema::{NullOrder, Predicate, Sort, SortDirection, ValueContext};
    let context = ValueContext::default();
    let input: CreateProfile =
        serde_json::from_value(serde_json::json!({"id":"u1","name":"Ada"})).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    let profile = input.prepare_create(&context).unwrap();
    let projected = profile.project().unwrap();
    assert_eq!(projected.name, "Ada");
    assert!(projected.active);
    let input: PublicCreateProfile =
        serde_json::from_value(serde_json::json!({"id":"u1","name":"Ada"})).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert_eq!(input.prepare_create(&context).unwrap()["active"], true);
    let input: PublicCreateAddress =
        serde_json::from_value(serde_json::json!({"city":"Paris","country":"FR"})).unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert_eq!(input.prepare_create(&context).unwrap()["city"], "Paris");
    let patch: PatchProfile = serde_json::from_value(serde_json::json!({"active":false})).unwrap();
    patch.validate(&context).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    let patch: PublicPatchProfile =
        serde_json::from_value(serde_json::json!({"name":"New"})).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    assert!(patch.validate_client(&context).unwrap().is_empty());
    let mut query = ProfileQuery {
        q: Some("literal%_".into()),
        ..Default::default()
    };
    query
        .filters
        .push(ProfileFilter::Id(Predicate::Eq("u1".into())));
    query.sort.push(Sort {
        field: ProfileSort::Name,
        direction: SortDirection::Asc,
        nulls: NullOrder::Last,
    });
    query.includes.push("projects.tasks".into());
    query.validate().unwrap();
    assert_eq!(
        serde_json::to_value(ProfileKey { id: "u1".into() }).unwrap(),
        serde_json::json!({"id":"u1"})
    );
    let input: CreateProject =
        serde_json::from_value(serde_json::json!({"id":"p1","ownerId":"u1","title":"Launch"}))
            .unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    let project = input.prepare_create(&context).unwrap();
    assert_eq!(project.project().unwrap().title, "Launch");
    let input: PublicCreateProject =
        serde_json::from_value(serde_json::json!({"id":"p1","ownerId":"u1","title":"Launch"}))
            .unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert_eq!(
        input.prepare_create(&context).unwrap()["tags"],
        serde_json::json!([])
    );
    let patch: PatchProject = serde_json::from_value(serde_json::json!({"title":"Next"})).unwrap();
    patch.validate(&context).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    let patch: PublicPatchProject =
        serde_json::from_value(serde_json::json!({"title":"Next"})).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    assert!(patch.validate_client(&context).unwrap().is_empty());
    let mut query = ProjectQuery {
        q: Some("Launch".into()),
        ..Default::default()
    };
    query
        .filters
        .push(ProjectFilter::State(Predicate::Eq(ProjectState::Open)));
    query.sort.push(Sort {
        field: ProjectSort::Title,
        direction: SortDirection::Desc,
        nulls: NullOrder::First,
    });
    query.includes.push("owner".into());
    query.validate().unwrap();
    assert_eq!(
        serde_json::to_value(ProjectKey { id: "p1".into() }).unwrap(),
        serde_json::json!({"id":"p1"})
    );
    let input: CreateTask =
        serde_json::from_value(serde_json::json!({"id":"t1","projectId":"p1","title":"Draft"}))
            .unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    let task = input.prepare_create(&context).unwrap();
    assert_eq!(task.project().unwrap().title, "Draft");
    let input: PublicCreateTask =
        serde_json::from_value(serde_json::json!({"id":"t1","projectId":"p1","title":"Draft"}))
            .unwrap();
    assert!(input.validate_create(&context).unwrap().is_empty());
    assert!(input.validate_client(&context).unwrap().is_empty());
    assert_eq!(input.prepare_create(&context).unwrap()["done"], false);
    let patch: PatchTask = serde_json::from_value(serde_json::json!({"done":true})).unwrap();
    patch.validate(&context).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    let patch: PublicPatchTask = serde_json::from_value(serde_json::json!({"done":true})).unwrap();
    assert!(patch.validate_patch(&context).unwrap().is_empty());
    assert!(patch.validate_client(&context).unwrap().is_empty());
    let mut query = TaskQuery::default();
    query.filters.push(TaskFilter::Done(Predicate::Eq(false)));
    query.includes.push("project.owner".into());
    query.validate().unwrap();
    assert_eq!(
        serde_json::to_value(TaskKey { id: "t1".into() }).unwrap(),
        serde_json::json!({"id":"t1"})
    );
    query.q = Some("unavailable".into());
    assert!(query.validate().is_err());
}
