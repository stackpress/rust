//! Disabled codegen contains no profile transform and owns only its aggregate registry.

#![cfg(not(feature = "profiles"))]

use stackpress_example_profile_codegen::generate;

/// Empty selections generate a compiling registry shape and reject uncompiled plugin blocks.
#[test]
fn disabled_selection_has_no_profile_outputs() {
    // Use a real independent root; the checked-in selected app output remains untouched.
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, "").unwrap();
    generate(&input, &output, false).unwrap();
    assert!(generate(&input, &output, true).unwrap().is_current());
    assert!(!output.join("contracts").exists());
    assert!(!output.join("server").exists());

    // Schema configuration cannot activate a callback removed at compilation.
    std::fs::write(&input, "plugin \"profile-demo\" { }").unwrap();
    assert!(generate(&input, &output, false).is_err());
}
