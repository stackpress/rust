//! Private controller contract tests, separated from shipped implementation.
use super::*;
/// Selection, content and option changes cannot reuse preview permission, even between effects.
#[test]
fn preview_permission_is_bound_to_the_exact_current_file() {
    let file = FileDraft {
        csv: "id,name\na,First".into(),
        generation: 1,
        ..Default::default()
    };
    let selection = Selection::new(&file, false);
    let mut guard = PreviewGuard::default();
    assert!(!guard.permits(&selection));
    guard.approve(selection.clone());
    assert!(guard.permits(&selection));
    let next = FileDraft {
        generation: 2,
        ..file.clone()
    };
    assert!(!guard.permits(&Selection::new(&next, false)));
    let changed = FileDraft {
        csv: "id,name\na,New".into(),
        ..file.clone()
    };
    assert!(!guard.permits(&Selection::new(&changed, false)));
    assert!(!guard.permits(&Selection::new(&file, true)));
    guard.invalidate();
    assert!(
        !guard.permits(&selection),
        "toggling back still needs fresh preview"
    );
    guard.approve(selection);
    assert!(guard.permits(&Selection::new(&file, false)));
    assert!(import_message(&Phase::Imported(1), true).contains("newer unsaved"));
    assert!(!import_message(&Phase::Imported(1), true).contains("unknown"));
    assert!(import_message(&Phase::Unknown, true).contains("submitted file"));
    assert!(import_message(&Phase::Failed("Forbidden".into()), true).contains("Submitted file"));
    assert_eq!(row_count(1), "1 row");
    assert_eq!(row_count(0), "0 rows");
    assert_eq!(human_bytes(1_048_576), "1 MB");
    assert_eq!(human_bytes(65_536), "64 KB");
    assert_eq!(human_bytes(1536), "1.5 KB");
}
