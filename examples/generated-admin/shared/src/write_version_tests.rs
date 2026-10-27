//! Private controller contract tests, separated from shipped implementation.
use super::*;
use serde_json::json;
/// Structural comparison covers newer, reverted, invalid and partial drafts without mutation.
#[test]
fn submitted_snapshot_feedback_preserves_later_edits() {
    let submitted = json!({"name":"Sent","zero":0,"flag":false});
    let newer = json!({"name":"Newer","zero":0,"flag":false});
    assert!(!changed(&submitted, Some(&submitted), false));
    assert!(changed(&submitted, Some(&newer), false));
    assert!(
        !changed(&submitted, Some(&submitted), false),
        "a reverted structural draft is exact again"
    );
    assert!(changed(&submitted, None, false));
    assert!(
        changed(&submitted, Some(&submitted), true),
        "raw partial formatting is newer too"
    );
    assert_eq!(message(&WriteState::Saved, false), "Saved");
    assert_eq!(
        message(&WriteState::Saved, true),
        "Saved the submitted version. You have newer unsaved changes."
    );
    for newer in [false, true] {
        let unknown = message(&WriteState::Unknown, newer);
        assert!(unknown.contains("submitted version"));
        assert!(unknown.contains("no automatic retry"));
        assert_eq!(unknown.contains("newer unsaved"), newer);
    }
    assert!(
        message(&WriteState::Failed("Forbidden".into()), true)
            .contains("Submitted version: Forbidden")
    );
    assert_eq!(message(&WriteState::Pending, true), "Submitting…");
}
/// The actual controller observes current signals at acknowledgement time without an effect race.
#[test]
fn live_versions_never_overwrite_newer_controls() {
    let owner = Owner::new();
    owner.with(|| {
        let spec = crate::ui::pages::spec("profile").unwrap();
        let draft = Draft::new(
            &spec.model.fields,
            &json!({"id":"p","name":"Sent"}),
            stackpress_framework_schema::Role::Field,
        )
        .unwrap();
        let pending = PendingEdits::default();
        let versions = WriteVersions::default();
        versions.attach(
            draft.clone(),
            pending.clone(),
            vec!["id".into(), "name".into()],
        );
        assert!(versions.message(&WriteState::Idle).is_empty());
        versions.begin(draft.snapshot().unwrap());
        assert_eq!(versions.message(&WriteState::Saved), "Saved");
        let name = draft.get("name").unwrap();
        name.set(stackpress_framework_schema::Input::Value(json!("Newer")));
        assert!(
            versions
                .message(&WriteState::Saved)
                .contains("newer unsaved")
        );
        assert!(
            versions
                .message(&WriteState::Unknown)
                .contains("newer unsaved")
        );
        assert_eq!(draft.snapshot().unwrap()["name"], "Newer");
        name.set(stackpress_framework_schema::Input::Value(json!("Sent")));
        assert_eq!(versions.message(&WriteState::Saved), "Saved");
        pending.0.update(|values| {
            values.insert("name".into(), true);
        });
        assert!(
            versions
                .message(&WriteState::Saved)
                .contains("newer unsaved")
        );
        assert_eq!(draft.snapshot().unwrap()["name"], "Sent");
    });
    owner.cleanup();
}
