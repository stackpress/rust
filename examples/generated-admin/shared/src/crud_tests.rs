//! Submitted operation envelopes are frozen independently of subsequent editable controls.
use super::*;
use stackpress_framework_schema::Input;

/// All write actions share one snapshot rule; copy/update keep their actual generated semantics.
#[test]
fn every_editor_write_freezes_its_exact_operation_and_version() {
    let owner = Owner::new();
    owner.with(|| {
        for action in [
            Action::Create,
            Action::Update,
            Action::Copy,
            Action::Remove,
            Action::Restore,
        ] {
            let seed = Snapshot {
                transfer_limits: Default::default(),
                spec: ui::pages::spec("profile").unwrap(),
                action,
                allowed: vec![action],
                mount: "/admin/".into(),
                csrf: "fixture".into(),
                key: json!({"id":"existing"}),
                data: json!({}),
                query: Default::default(),
                parent: None,
            };
            let draft = Draft::new(
                &seed.spec.model.fields,
                &json!({"id":"new","name":"Submitted"}),
                Role::Field,
            )
            .unwrap();
            let confirmation = matches!(action, Action::Remove | Action::Restore);
            let (input, version) = prepare_submission(&seed, &draft, confirmation).unwrap();
            draft.get("name").unwrap().set(Input::Value(json!("Newer")));
            assert_eq!(version["name"], "Submitted");
            assert_eq!(draft.snapshot().unwrap()["name"], "Newer");
            if confirmation {
                assert_eq!(input, json!({"key":{"id":"existing"}}));
            } else if action == Action::Update {
                assert_eq!(input["key"], json!({"id":"existing"}));
                assert_eq!(input["patch"]["name"], "Submitted");
                assert!(input["patch"].get("id").is_none());
            } else {
                assert_eq!(input["input"]["id"], "new");
                assert_eq!(input["input"]["name"], "Submitted");
            }
        }
    });
    owner.cleanup();
}

/// Acknowledgements update status/issues without replacing a draft or inventing read writes.
#[test]
fn completion_keeps_public_errors_and_honest_states() {
    let owner = Owner::new();
    owner.with(|| {
        let data=RwSignal::new(json!({"original":true}));let status=RwSignal::new(WriteState::Idle);
        let issues=RwSignal::new(Vec::new());let write=complete(data,status,issues,true,None);
        write.run(Reply{status:400,envelope:json!({"code":400,"error":"input validation failed","errors":{"fields":[{"path":["name"],"code":"invalid","message":"field validation failed"}]}})});
        assert!(matches!(status.get_untracked(),WriteState::Failed(_)));assert_eq!(issues.get_untracked()[0].path,["name"]);assert_eq!(data.get_untracked(),json!({"original":true}));
        write.run(Reply{status:500,envelope:json!({"code":500,"errors":{"outcome":"unknown"}})});assert_eq!(status.get_untracked(),WriteState::Unknown);
        let read=complete(data,status,issues,false,None);read.run(Reply{status:500,envelope:json!({"code":500,"errors":{"outcome":"unknown"}})});assert!(matches!(status.get_untracked(),WriteState::Failed(_)));
        read.run(Reply{status:200,envelope:json!({"code":200,"results":{"rows":[],"total":0}})});assert_eq!(status.get_untracked(),WriteState::Idle);assert_eq!(data.get_untracked()["total"],0);
        write.run(Reply{status:200,envelope:json!({"code":200,"results":{"id":"created"}})});assert_eq!(status.get_untracked(),WriteState::Saved);assert!(issues.get_untracked().is_empty());
    });
    owner.cleanup();
}
