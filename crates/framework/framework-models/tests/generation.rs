//! Official generator install/check controls over uniquely owned finite fixture directories.
#![cfg(all(feature = "generate", not(target_arch = "wasm32")))]
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

/// Own exactly the temporary parent created by this test; no generated repository root.
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Allocate an exclusive canonical parent; the managed writer owns only its child output.
fn fixture() -> Fixture {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "source-models-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&path).unwrap();
    Fixture(path)
}

/// Execute the real tool directly; it starts no runtime, listener or database.
fn generate(destination: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_framework-models-generate"))
        .arg("--output")
        .arg(destination)
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn official_tool_installs_checks_and_refuses_stale_output_without_writing() {
    let parent = fixture();
    let output = parent.0.join("generated");
    let installed = generate(&output, &[]);
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    assert!(generate(&output, &["--check"]).status.success());
    let path = output.join("source/profile.rs");
    let original = fs::read_to_string(&path).unwrap();
    fs::write(&path, format!("{original}\n// stale negative control\n")).unwrap();
    let stale = fs::read(&path).unwrap();
    let checked = generate(&output, &["--check"]);
    assert!(!checked.status.success());
    // The managed writer reports manual drift before the CLI's semantic-staleness branch.
    let error = String::from_utf8_lossy(&checked.stderr);
    assert!(error.contains("manually edited generated file"), "{error}");
    assert_eq!(
        fs::read(&path).unwrap(),
        stale,
        "check mode never fixes drift"
    );
}

#[test]
fn official_tool_accepts_application_extension_and_rejects_bad_arguments() {
    let parent = fixture();
    let output = parent.0.join("generated");
    let input = parent.0.join("schema.idea");
    fs::write(&input, "use \"stackpress-api/schema.idea\" model Profile { nickname String? @default(\"app\") @field.input({type \"text\"}) }").unwrap();
    let installed = generate(&output, &["--input", input.to_str().unwrap()]);
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    assert!(
        fs::read_to_string(output.join("source/profile.rs"))
            .unwrap()
            .contains("nickname")
    );
    assert!(
        generate(&output, &["--input", input.to_str().unwrap(), "--check"])
            .status
            .success()
    );
    let prior = fs::read(output.join("source/profile.rs")).unwrap();
    fs::write(&input, "use \"stackpress-api/schema.idea\" model Profile { nickname String? @default(\"new source default\") }").unwrap();
    let stale = generate(&output, &["--input", input.to_str().unwrap(), "--check"]);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("stale"));
    assert_eq!(fs::read(output.join("source/profile.rs")).unwrap(), prior);
    for args in [&["--input"][..], &["--unknown"][..], &["--output"][..]] {
        assert!(!generate(&output, args).status.success());
    }
}
