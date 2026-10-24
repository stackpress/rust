//! Actual native output, lock and injected interruption/recovery proofs.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use stackpress_idea_parser::{CompileInputs, SchemaConfig, parse};

use super::*;
use crate::{GeneratedFile, Registry, merge_type};

/// Use a canonical temp parent so macOS's /var and /tmp symlinks are not output ancestors.
fn temporary() -> tempfile::TempDir {
    tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
}

/// Emit one owned text file whose bytes come from the parsed input.
fn emit(schema: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new(
        "data/a.txt",
        schema.prop["Input"]["text"].as_str().unwrap(),
    )])
}

/// Exercise the public registry and a genuine source parse rather than constructing writer state.
fn plan(text: &str) -> OutputPlan {
    let source = format!("plugin \"text\" {{ }} prop Input {{ text \"{text}\" }}");
    let schema = parse("s.idea", &source, &CompileInputs::default()).unwrap();
    let mut registry = Registry::new();
    registry.register("text", &["data"], emit).unwrap();
    registry
        .plan(&schema, vec![GeneratedFile::new("index.txt", "owned\n")])
        .unwrap()
}

/// Complete installs retain bytes on repeat, identify drift and remove stale owned paths.
#[test]
fn clean_repeat_check_and_removed_files() {
    let temp = temporary();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    assert!(!workspace.check(&plan("first")).unwrap().is_current());
    assert!(!workspace.install(plan("first")).unwrap().is_current());
    let marker = fs::read(root.join(MANIFEST)).unwrap();
    assert!(workspace.install(plan("first")).unwrap().is_current());
    assert_eq!(marker, fs::read(root.join(MANIFEST)).unwrap());
    assert!(workspace.check(&plan("first")).unwrap().is_current());
    assert!(!workspace.check(&plan("second")).unwrap().is_current());
    let mut removed = plan("first");
    removed.files.remove("data/a.txt");
    workspace.install(removed).unwrap();
    assert!(!root.join("data/a.txt").exists());
    assert!(!root.join("data").exists());
}

/// One injected interruption after backup publication keeps the prior tree recoverable.
#[test]
fn interruption_requires_explicit_recovery() {
    for initially_present in [false, true] {
        let temp = temporary();
        let root = temp.path().join("generated");
        let workspace = OutputWorkspace::new(&root).unwrap();
        if initially_present {
            workspace.install(plan("old")).unwrap();
        }
        let result = workspace.install_with(plan("new"), || {
            Err(TransformError::new("injected interruption after backup"))
        });
        assert!(result.is_err());
        assert!(!root.exists());
        assert!(workspace.install(plan("later")).is_err());
        workspace.recover().unwrap();
        assert_eq!(
            fs::read_to_string(root.join("data/a.txt")).unwrap(),
            if initially_present { "old" } else { "new" }
        );
        assert!(
            !workspace.stage.exists() && !workspace.backup.exists() && !workspace.journal.exists()
        );
        workspace.install(plan("final")).unwrap();
        assert!(workspace.recover().is_err());
    }
}

/// Manual content, unknown files/dirs and malformed recovery data are never silently purged.
#[test]
fn manual_drift_and_invalid_state_block_writes() {
    let temp = temporary();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    workspace.install(plan("owned")).unwrap();
    fs::write(root.join("data/a.txt"), "manual").unwrap();
    assert!(workspace.install(plan("next")).is_err());
    assert_eq!(
        fs::read_to_string(root.join("data/a.txt")).unwrap(),
        "manual"
    );
    fs::write(root.join("data/a.txt"), "owned").unwrap();
    fs::write(root.join("manual.txt"), "keep").unwrap();
    assert!(workspace.check(&plan("owned")).is_err());
    fs::remove_file(root.join("manual.txt")).unwrap();
    fs::create_dir(root.join("manual-directory")).unwrap();
    assert!(workspace.install(plan("next")).is_err());
    fs::remove_dir(root.join("manual-directory")).unwrap();
    fs::write(&workspace.journal, "unrecognized").unwrap();
    assert!(workspace.recover().is_err());
    assert_eq!(
        fs::read_to_string(root.join("data/a.txt")).unwrap(),
        "owned"
    );
}

/// Formatting uses stdout, and invalid Rust never changes an installed tree.
#[test]
fn rust_formatting_and_failure_are_before_installation() {
    let temp = temporary();
    let root = temp.path().join("generated");
    let workspace = OutputWorkspace::new(&root).unwrap();
    let mut rust = plan("first");
    rust.files.insert(
        "data/a.rs".into(),
        ("text".into(), "pub struct X{pub id:String}".into()),
    );
    workspace.install(rust).unwrap();
    assert!(
        fs::read_to_string(root.join("data/a.rs"))
            .unwrap()
            .contains("pub id: String")
    );
    let marker = fs::read(root.join(MANIFEST)).unwrap();
    let mut invalid = plan("next");
    invalid
        .files
        .insert("data/a.rs".into(), ("text".into(), "not rust {{{".into()));
    assert!(workspace.install(invalid).is_err());
    assert_eq!(marker, fs::read(root.join(MANIFEST)).unwrap());
}

/// A separate OS process observes the same create-new writer exclusion.
#[test]
#[ignore]
fn lock_child() {
    let workspace = OutputWorkspace::new(std::env::var_os("IDEA_LOCK_TEST_ROOT").unwrap()).unwrap();
    assert!(
        workspace
            .install(plan("child"))
            .unwrap_err()
            .to_string()
            .contains("writer lock")
    );
}

/// Competing writers fail promptly, and the acquiring invocation alone releases its lock.
#[test]
fn native_writer_lock_is_exclusive() {
    let temp = temporary();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    let guard = workspace.acquire().unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "output::tests::lock_child"])
        .env("IDEA_LOCK_TEST_ROOT", &workspace.root)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("writer lock child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(workspace.lock.exists());
    drop(guard);
    workspace.install(plan("parent")).unwrap();
}

/// Output ancestry and nested output entries never follow symbolic links.
#[cfg(unix)]
#[test]
fn symlinks_and_unmarked_roots_are_rejected() {
    let temp = temporary();
    let root = temp.path().join("generated");
    fs::create_dir(&root).unwrap();
    fs::write(root.join("manual"), "preserve").unwrap();
    let workspace = OutputWorkspace::new(&root).unwrap();
    assert!(workspace.install(plan("next")).is_err());
    let alias = temp.path().join("alias");
    std::os::unix::fs::symlink(&root, &alias).unwrap();
    assert!(OutputWorkspace::new(alias).is_err());
    assert!(OutputWorkspace::new(temp.path().join("../escape")).is_err());
}

/// G19's immutable replacement and the remaining mutable merge rules are explicit contracts.
#[test]
fn imported_child_replaces_immutable_local_without_mutating_child() {
    let inputs = CompileInputs::default();
    let child = parse("child", "type X @child { a String b String }", &inputs).unwrap();
    let child = &child.r#type["X"];
    let before = serde_json::to_value(child).unwrap();
    for immutable in [false, true] {
        let marker = if immutable { "!" } else { "" };
        let source = format!("type X{marker} @local {{ b Number c Boolean }}");
        let mut local = parse("local", &source, &inputs)
            .unwrap()
            .r#type
            .swap_remove("X")
            .unwrap();
        merge_type(&mut local, child);
        let names: Vec<_> = local
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect();
        assert_eq!(
            names,
            if immutable {
                vec!["a", "b"]
            } else {
                vec!["a", "b", "c"]
            }
        );
        assert_eq!(
            local.columns[1].column_type,
            if immutable { "String" } else { "Number" }
        );
        assert_eq!(local.attributes.contains_key("local"), !immutable);
        assert!(local.attributes.contains_key("child"));
    }
    assert_eq!(before, serde_json::to_value(child).unwrap());
}
