//! Explicit bounded configuration never performs database setup or environment reads.

use stackpress_example_profile_ui_web::backend::configuration;

/// Accept ordinary JSON sections, while malformed/missing/oversize files fail early.
#[test]
fn explicit_file_and_input_failures() {
    assert!(configuration(None).unwrap().entries().next().is_none());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.json");
    assert!(configuration(Some(directory.path())).is_err());
    assert!(configuration(Some(&path)).is_err());
    for bytes in [b"not JSON".as_slice(), b"[]", &[255]] {
        std::fs::write(&path, bytes).unwrap();
        assert!(configuration(Some(&path)).is_err());
    }
    std::fs::write(
        &path,
        br#"{"sqlite":{"path":"existing.sqlite"},"profiles":{"limit":3}}"#,
    )
    .unwrap();
    assert_eq!(configuration(Some(&path)).unwrap().entries().count(), 1);
    std::fs::write(
        &path,
        vec![b' '; stackpress_lib::ParseLimits::default().max_bytes + 1],
    )
    .unwrap();
    assert!(configuration(Some(&path)).is_err());
    assert!(!directory.path().join("existing.sqlite").exists());
}
