//! Dev stylesheet reads are finite and never mutate shared environment or authored files.
use super::*;

/// Every request observes fresh bytes; missing dev input reports the real IO error.
#[test]
fn dev_styles_are_fresh_and_normal_styles_stay_embedded() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        stylesheet_at(None, "global.css", b"embedded").unwrap(),
        b"embedded"
    );
    std::fs::write(root.path().join("global.css"), b"first").unwrap();
    assert_eq!(
        stylesheet_at(Some(root.path()), "global.css", b"old").unwrap(),
        b"first"
    );
    std::fs::write(root.path().join("global.css"), b"second").unwrap();
    assert_eq!(
        stylesheet_at(Some(root.path()), "global.css", b"old").unwrap(),
        b"second"
    );
    assert!(stylesheet_at(Some(root.path()), "missing.css", b"old").is_err());
}
