//! Actual native output, lock and injected interruption/recovery proofs.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use stackpress_idea_parser::{CompileInputs, SchemaConfig, parse};

use super::*;
use crate::{GeneratedFile, Registry, merge_type};

impl OutputWorkspace {
    /// Run the M0 interruption seam against the same production transition runner.
    fn install_with(
        &self,
        plan: &OutputPlan,
        after_backup: impl FnOnce() -> Result<()>,
    ) -> Result<CheckReport> {
        let mut after_backup = Some(after_backup);
        self.install_at(plan, |phase| {
            if phase == Phase::BackupRetained {
                return after_backup.take().expect("one backup transition")();
            }
            Ok(())
        })
    }
}

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
        let result = workspace.install_with(&plan("new"), || {
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

/// Every install transition fails through the real runner, preserving a usable prior/new tree.
#[test]
fn complete_install_failure_matrix() {
    let phases = [
        Phase::StageWrite,
        Phase::StagePublish,
        Phase::IntentWrite,
        Phase::IntentPublished,
        Phase::BackupRename,
        Phase::BackupRetained,
        Phase::InstallRename,
        Phase::Installed,
        Phase::CleanupTree,
        Phase::ClearIntent,
    ];
    for initially_present in [false, true] {
        for phase in phases {
            // CleanupTree has no retained predecessor on a first install.
            if !initially_present && phase == Phase::CleanupTree {
                continue;
            }
            let temp = temporary();
            let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
            if initially_present {
                workspace.install(plan("old")).unwrap();
            }
            let error = workspace
                .install_at(&plan("new"), |current| {
                    if current == phase {
                        Err(std::io::Error::new(
                            match phase {
                                Phase::StageWrite => std::io::ErrorKind::StorageFull,
                                Phase::IntentWrite => std::io::ErrorKind::WriteZero,
                                _ => std::io::ErrorKind::PermissionDenied,
                            },
                            "injected native operation failure",
                        )
                        .into())
                    } else {
                        Ok(())
                    }
                })
                .unwrap_err();
            assert!(error.to_string().contains(&format!("{phase:?}")), "{error}");
            assert!(std::error::Error::source(&error).is_some());
            assert!(!workspace.lock.exists());
            if matches!(phase, Phase::StageWrite | Phase::StagePublish) {
                assert!(!workspace.stage.exists() && !workspace.journal.exists());
                assert_eq!(
                    fs::read_dir(temp.path()).unwrap().count(),
                    usize::from(initially_present)
                );
            } else if phase == Phase::IntentWrite {
                // Pre-intent stages are retained for explicit inspection, never guessed away.
                assert!(workspace.stage.exists() && !workspace.journal.exists());
                assert!(workspace.check(&plan("old")).is_err());
                assert!(workspace.recover().is_err());
                inspect_owned(&workspace.stage).unwrap();
                fs::remove_dir_all(&workspace.stage).unwrap();
            } else {
                assert!(workspace.check(&plan("later")).is_err());
                workspace.recover().unwrap();
                let installed = fs::read_to_string(workspace.root.join("data/a.txt")).unwrap();
                let new_wins = !initially_present
                    || matches!(
                        phase,
                        Phase::Installed | Phase::CleanupTree | Phase::ClearIntent
                    );
                assert_eq!(installed, if new_wins { "new" } else { "old" });
                assert!(
                    !workspace.stage.exists()
                        && !workspace.backup.exists()
                        && !workspace.journal.exists()
                );
            }
            workspace.install(plan("final")).unwrap();
            assert!(workspace.check(&plan("final")).unwrap().is_current());
        }
    }
}

/// Empty selection removes prior partitions; check performs no writes and receipts are portable.
#[test]
fn empty_selection_removes_partitions_and_reports_are_deterministic() {
    let mut receipts = Vec::new();
    for _ in 0..2 {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        let first = workspace.install_report(plan("first")).unwrap();
        assert_eq!(first.generated, ["data/a.txt", "index.txt"]);
        assert!(first.removed.is_empty());
        let before = fs::read(workspace.root.join(MANIFEST)).unwrap();
        let changed = workspace.check(&plan("second")).unwrap();
        assert_eq!(changed.differences, ["data/a.txt"]);
        assert_eq!(before, fs::read(workspace.root.join(MANIFEST)).unwrap());
        assert!(!workspace.stage.exists() && !workspace.lock.exists());
        let unchanged = workspace.install_report(plan("first")).unwrap();
        assert_eq!(unchanged.unchanged, ["data/a.txt", "index.txt"]);
        let empty = Registry::new()
            .plan(
                &SchemaConfig::default(),
                vec![GeneratedFile::new("index.txt", "owned\n")],
            )
            .unwrap();
        let removed = workspace.install_report(empty).unwrap();
        assert!(removed.selected.is_empty());
        assert_eq!(removed.removed, ["data/a.txt"]);
        assert!(!workspace.root.join("data").exists());
        let json = serde_json::to_string(&removed).unwrap();
        assert!(!json.contains(temp.path().to_str().unwrap()));
        receipts.push(json);
    }
    assert_eq!(receipts[0], receipts[1]);
}

/// Absent and existing-empty roots both gain a real ownership marker for an empty plan.
#[test]
fn empty_plans_require_and_install_the_marker() {
    for existing in [false, true] {
        let temp = temporary();
        let root = temp.path().join("generated");
        if existing {
            fs::create_dir(&root).unwrap();
        }
        let workspace = OutputWorkspace::new(&root).unwrap();
        let empty = || {
            Registry::new()
                .plan(&SchemaConfig::default(), vec![])
                .unwrap()
        };
        assert_eq!(workspace.check(&empty()).unwrap().differences, [MANIFEST]);
        workspace.install(empty()).unwrap();
        assert!(root.join(MANIFEST).is_file());
        assert!(workspace.check(&empty()).unwrap().is_current());
    }
}

/// Unknown or edited data in any retained tree blocks recovery before any other tree is purged.
#[test]
fn recovery_validates_all_trees_before_mutation() {
    for retained in ["stage", "backup"] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("old")).unwrap();
        assert!(
            workspace
                .install_at(&plan("new"), |phase| {
                    if phase == Phase::BackupRetained {
                        Err(TransformError::new("interrupt"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
        );
        let tree = if retained == "stage" {
            &workspace.stage
        } else {
            &workspace.backup
        };
        fs::write(tree.join("manual.txt"), "manual sentinel").unwrap();
        assert!(workspace.recover().is_err());
        assert!(!workspace.root.exists());
        assert!(workspace.stage.exists() && workspace.backup.exists());
        assert_eq!(
            fs::read_to_string(tree.join("manual.txt")).unwrap(),
            "manual sentinel"
        );
        fs::remove_file(tree.join("manual.txt")).unwrap();
        workspace.recover().unwrap();
        assert_eq!(
            fs::read_to_string(workspace.root.join("data/a.txt")).unwrap(),
            "old"
        );
    }
}

/// Cleanup intent distinguishes partial deletion from a candidate restore tree.
#[test]
fn recovery_resumes_partial_cleanup_without_accepting_manual_files() {
    for emptied in [false, true] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("old")).unwrap();
        assert!(
            workspace
                .install_at(&plan("new"), |phase| {
                    if phase == Phase::CleanupTree {
                        Err(TransformError::new("delete interrupted"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
        );
        fs::remove_file(workspace.backup.join("data/a.txt")).unwrap();
        if emptied {
            fs::remove_dir(workspace.backup.join("data")).unwrap();
            fs::remove_file(workspace.backup.join("index.txt")).unwrap();
            fs::remove_file(workspace.backup.join(MANIFEST)).unwrap();
        } else {
            fs::write(workspace.backup.join("manual.txt"), "preserve").unwrap();
            assert!(workspace.recover().is_err());
            assert_eq!(
                fs::read_to_string(workspace.backup.join("manual.txt")).unwrap(),
                "preserve"
            );
            fs::remove_file(workspace.backup.join("manual.txt")).unwrap();
        }
        workspace.recover().unwrap();
        assert!(!workspace.backup.exists());
        assert_eq!(
            fs::read_to_string(workspace.root.join("data/a.txt")).unwrap(),
            "new"
        );
    }
}

/// Real output replacement and every metadata/sibling surface reject symbolic links.
#[cfg(unix)]
#[test]
fn all_output_symlink_surfaces_are_rejected() {
    for target in [
        "file",
        "directory",
        "marker",
        "journal",
        "stage",
        "backup",
        "lock",
    ] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("old")).unwrap();
        let outside = temp.path().join("outside");
        fs::write(&outside, "manual sentinel").unwrap();
        let link = match target {
            "file" => {
                fs::remove_file(workspace.root.join("data/a.txt")).unwrap();
                workspace.root.join("data/a.txt")
            }
            "directory" => {
                fs::remove_dir_all(workspace.root.join("data")).unwrap();
                workspace.root.join("data")
            }
            "marker" => {
                fs::remove_file(workspace.root.join(MANIFEST)).unwrap();
                workspace.root.join(MANIFEST)
            }
            "journal" => workspace.journal.clone(),
            "stage" => workspace.stage.clone(),
            "backup" => workspace.backup.clone(),
            _ => workspace.lock.clone(),
        };
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        assert!(workspace.check(&plan("new")).is_err(), "followed {target}");
        assert!(workspace.install(plan("new")).is_err(), "followed {target}");
        if target == "journal" {
            assert!(workspace.recover().is_err());
        }
        assert_eq!(fs::read_to_string(outside).unwrap(), "manual sentinel");
        assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    }
}

/// Corrupt markers, digest/length drift, missing files and unknown versions never grant deletion.
#[test]
fn manifest_validation_and_legacy_length_compatibility() {
    for case in [
        "bad-json", "version", "length", "sha", "owner", "path", "missing",
    ] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("old")).unwrap();
        let marker = workspace.root.join(MANIFEST);
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        match case {
            "bad-json" => {
                fs::write(&marker, "{").unwrap();
            }
            "version" => {
                value["version"] = 999.into();
            }
            "length" => {
                value["files"]["data/a.txt"]["bytes"] = 999.into();
            }
            "sha" => {
                value["files"]["data/a.txt"]["sha256"] = "0".repeat(64).into();
            }
            "owner" => {
                value["files"]["data/a.txt"]["owner"] = "".into();
            }
            "path" => {
                let entry = value["files"]["data/a.txt"].clone();
                value["files"]["../escape"] = entry;
            }
            _ => {
                fs::remove_file(workspace.root.join("data/a.txt")).unwrap();
            }
        }
        if case != "bad-json" {
            fs::write(&marker, serde_json::to_vec(&value).unwrap()).unwrap();
        }
        let before = fs::read(&marker).unwrap();
        assert!(workspace.install(plan("new")).is_err(), "accepted {case}");
        assert_eq!(before, fs::read(marker).unwrap());
        assert!(workspace.root.join("index.txt").exists());
    }
    let temp = temporary();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    workspace.install(plan("old")).unwrap();
    let marker = workspace.root.join(MANIFEST);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
    for entry in value["files"].as_object_mut().unwrap().values_mut() {
        entry.as_object_mut().unwrap().remove("bytes");
    }
    fs::write(&marker, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(workspace.check(&plan("old")).unwrap().is_current());
}

/// The child holds a real install boundary until the watchdog parent kills and reaps it.
#[test]
#[ignore]
fn interrupted_install_child() {
    let workspace = OutputWorkspace::new(std::env::var_os("IDEA_KILL_ROOT").unwrap()).unwrap();
    let phase = std::env::var("IDEA_KILL_PHASE").unwrap();
    workspace
        .install_at(&plan("new"), |current| {
            if format!("{current:?}") == phase {
                fs::write(workspace.root.parent().unwrap().join("ready"), b"ready").unwrap();
                let deadline = Instant::now() + Duration::from_secs(10);
                // Explicit ready/go barrier; the parent kills rather than sends go for this proof.
                while !workspace.root.parent().unwrap().join("go").exists() {
                    assert!(
                        Instant::now() < deadline,
                        "parent did not kill/release child"
                    );
                    std::thread::sleep(Duration::from_millis(5));
                }
            }
            Ok(())
        })
        .unwrap();
}

/// Process death retains a stale lock; explicit operator removal permits recognized recovery.
#[test]
fn killed_install_barriers_are_recoverable_after_explicit_lock_inspection() {
    for initially_present in [false, true] {
        for phase in [
            Phase::IntentPublished,
            Phase::BackupRetained,
            Phase::Installed,
            Phase::CleanupTree,
            Phase::CleanupEntry,
            Phase::ClearIntent,
        ] {
            if !initially_present && matches!(phase, Phase::CleanupTree | Phase::CleanupEntry) {
                continue;
            }
            let temp = temporary();
            let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
            if initially_present {
                workspace.install(plan("old")).unwrap();
            }
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--ignored",
                    "--exact",
                    "output::tests::interrupted_install_child",
                ])
                .env("IDEA_KILL_ROOT", &workspace.root)
                .env("IDEA_KILL_PHASE", format!("{phase:?}"))
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            while !temp.path().join("ready").exists() {
                if let Some(status) = child.try_wait().unwrap() {
                    panic!("install child exited before ready: {status}");
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("install child ready watchdog expired");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            let blocked = workspace.check(&plan("blocked")).is_err();
            child.kill().unwrap();
            assert!(!child.wait().unwrap().success());
            assert!(blocked);
            assert!(workspace.lock.exists());
            assert!(workspace.recover().is_err());
            // The child is reaped; this simulates the documented explicit operator action.
            fs::remove_file(&workspace.lock).unwrap();
            workspace.recover().unwrap();
            let text = fs::read_to_string(workspace.root.join("data/a.txt")).unwrap();
            let new_wins = !initially_present
                || matches!(
                    phase,
                    Phase::Installed
                        | Phase::CleanupTree
                        | Phase::CleanupEntry
                        | Phase::ClearIntent
                );
            assert_eq!(text, if new_wins { "new" } else { "old" });
            workspace.install(plan("final")).unwrap();
        }
    }
}

/// Unexpected retained state and broken journals refuse recovery without deleting sentinels.
#[test]
fn orphan_and_invalid_recovery_states_require_inspection() {
    for state in [
        "stage",
        "backup",
        "missing",
        "partial-journal",
        "unmarked-stage",
    ] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        match state {
            "stage" => fs::create_dir(&workspace.stage).unwrap(),
            "backup" => fs::create_dir(&workspace.backup).unwrap(),
            "unmarked-stage" => {
                fs::create_dir(&workspace.stage).unwrap();
                fs::write(&workspace.journal, "stackpress-install-v1\n").unwrap();
            }
            "partial-journal" => fs::write(&workspace.journal, "stackpress-install-").unwrap(),
            _ => fs::write(&workspace.journal, "stackpress-install-v1\n").unwrap(),
        }
        assert!(workspace.check(&plan("new")).is_err(), "accepted {state}");
        assert!(workspace.recover().is_err(), "recovered {state}");
        assert!(!workspace.root.exists());
        assert!(!workspace.lock.exists());
    }
}

/// Ownership manifests reject duplicate JSON keys before maps can hide conflicting claims.
#[test]
fn duplicate_manifest_paths_and_unknown_fields_are_rejected() {
    let temp = temporary();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    workspace.install(plan("old")).unwrap();
    let marker = workspace.root.join(MANIFEST);
    let original = fs::read_to_string(&marker).unwrap();
    let value: serde_json::Value = serde_json::from_str(&original).unwrap();
    let entry = serde_json::to_string(&value["files"]["data/a.txt"]).unwrap();
    let duplicate =
        format!("{{\"version\":1,\"files\":{{\"data/a.txt\":{entry},\"data/a.txt\":{entry}}}}}");
    fs::write(&marker, &duplicate).unwrap();
    assert!(workspace.install(plan("new")).is_err());
    assert_eq!(fs::read_to_string(&marker).unwrap(), duplicate);
    let mut value = value;
    value["unexpected"] = true.into();
    fs::write(&marker, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(workspace.check(&plan("old")).is_err());
    assert_eq!(
        fs::read_to_string(workspace.root.join("data/a.txt")).unwrap(),
        "old"
    );
}

/// A genuine native delete denial keeps the complete new root and explicit cleanup intent.
#[cfg(unix)]
#[test]
fn native_delete_permission_failure_is_recoverable() {
    use std::os::unix::fs::PermissionsExt;
    let temp = temporary();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    workspace.install(plan("old")).unwrap();
    let result = workspace.install_at(&plan("new"), |phase| {
        if phase == Phase::CleanupTree {
            fs::set_permissions(
                workspace.backup.join("data"),
                fs::Permissions::from_mode(0o500),
            )?;
            fs::set_permissions(&workspace.backup, fs::Permissions::from_mode(0o500))?;
        }
        Ok(())
    });
    // Restore permissions before assertions so disposable roots remain removable on failure.
    if workspace.backup.exists() {
        fs::set_permissions(&workspace.backup, fs::Permissions::from_mode(0o700)).unwrap();
        if workspace.backup.join("data").exists() {
            fs::set_permissions(
                workspace.backup.join("data"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
        }
    }
    assert!(
        result.is_err(),
        "host privileges bypassed delete denial; no permission proof"
    );
    assert!(workspace.journal.exists());
    assert!(workspace.backup.join(MANIFEST).is_file());
    assert_eq!(
        fs::read_to_string(workspace.root.join("data/a.txt")).unwrap(),
        "new"
    );
    workspace.recover().unwrap();
    assert!(workspace.check(&plan("new")).unwrap().is_current());
}

/// A failed transform/panic runs before writer ownership or output installation begins.
#[test]
#[ignore]
fn panicking_transform_child() {
    use crate::{OutputPlan, TransformContext, TransformSpec};
    /// Panic deliberately to prove the installed root remains outside callback authority.
    fn panic_transform(_: &TransformContext<'_>, _: &mut OutputPlan) -> Result<()> {
        panic!("deliberate transform panic");
    }
    let mut registry = Registry::new();
    registry
        .add(TransformSpec::new("panic"), panic_transform)
        .unwrap();
    let plan = registry.plan(&SchemaConfig::default(), vec![]).unwrap();
    OutputWorkspace::new(std::env::var_os("IDEA_PANIC_ROOT").unwrap())
        .unwrap()
        .install(plan)
        .unwrap();
}

/// Callback panics are not converted into successful output or hidden by the test harness.
#[test]
fn transform_panic_leaves_prior_output_intact_under_watchdog() {
    let temp = temporary();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    workspace.install(plan("old")).unwrap();
    let marker = fs::read(workspace.root.join(MANIFEST)).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "output::tests::panicking_transform_child",
        ])
        .env("IDEA_PANIC_ROOT", &workspace.root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(!status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("panic child timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(marker, fs::read(workspace.root.join(MANIFEST)).unwrap());
    assert!(workspace.check(&plan("old")).unwrap().is_current());
    assert!(!workspace.lock.exists() && !workspace.stage.exists());
}

/// An adopted empty directory has no owned predecessor and recovers to the complete stage.
#[test]
fn initially_empty_root_recovers_across_install_boundaries() {
    for target in [
        Phase::IntentPublished,
        Phase::BackupRetained,
        Phase::Installed,
        Phase::ClearIntent,
    ] {
        let temp = temporary();
        let root = temp.path().join("generated");
        fs::create_dir(&root).unwrap();
        let workspace = OutputWorkspace::new(&root).unwrap();
        assert!(
            workspace
                .install_at(&plan("new"), |phase| {
                    if phase == target {
                        Err(TransformError::new("empty-root interruption"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
        );
        workspace.recover().unwrap();
        assert!(workspace.check(&plan("new")).unwrap().is_current());
        assert!(!workspace.backup.exists() && !workspace.stage.exists());
    }
}

/// FIFO metadata must be rejected before a bounded text reader can block opening it.
#[cfg(unix)]
#[test]
#[ignore]
fn special_file_child() {
    let workspace = OutputWorkspace::new(std::env::var_os("IDEA_SPECIAL_ROOT").unwrap()).unwrap();
    workspace.install(plan("owned")).unwrap();
    let case = std::env::var("IDEA_SPECIAL_CASE").unwrap();
    let path = match case.as_str() {
        "marker" => workspace.root.join(MANIFEST),
        "owned-file" => workspace.root.join("data/a.txt"),
        _ => workspace.journal.clone(),
    };
    if path.exists() {
        fs::remove_file(&path).unwrap();
    }
    assert!(
        Command::new("mkfifo")
            .arg(&path)
            .status()
            .unwrap()
            .success()
    );
    if case == "journal" {
        assert!(workspace.recover().is_err());
    } else {
        assert!(workspace.check(&plan("owned")).is_err());
        assert!(workspace.install(plan("replacement")).is_err());
        fs::write(&workspace.journal, INSTALL_INTENT).unwrap();
        assert!(workspace.recover().is_err());
    }
    assert!(!fs::symlink_metadata(path).unwrap().is_file());
}

/// Every special-file surface fails promptly in its own process under a kill/reap watchdog.
#[cfg(unix)]
#[test]
fn fifo_inspection_is_watchdog_bounded() {
    let mut failures = Vec::new();
    for case in ["marker", "owned-file", "journal"] {
        let temp = temporary();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "output::tests::special_file_child"])
            .env("IDEA_SPECIAL_ROOT", temp.path().join("generated"))
            .env("IDEA_SPECIAL_CASE", case)
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                if !status.success() {
                    failures.push(format!("{case}: {status}"));
                }
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                failures.push(format!("{case}: blocked until watchdog"));
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join(", "));
}

/// Every deletion boundary retains the root marker until the remaining owned entries are gone.
#[test]
fn cleanup_keeps_marker_until_last_entry_and_recovers_each_failure() {
    for stop_at in 1..=4 {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("old")).unwrap();
        let mut entries = 0;
        let result = workspace.install_at(&plan("new"), |phase| {
            if phase == Phase::CleanupEntry {
                entries += 1;
                if entries == stop_at {
                    return Err(TransformError::new("entry deletion failed"));
                }
            }
            Ok(())
        });
        assert!(result.is_err(), "no interruption at deletion {stop_at}");
        assert!(workspace.backup.exists());
        if !workspace.backup.join(MANIFEST).exists() {
            assert!(
                fs::read_dir(&workspace.backup).unwrap().next().is_none(),
                "ownership marker removed before remaining files"
            );
        }
        assert!(workspace.check(&plan("new")).is_err());
        workspace.recover().unwrap();
        assert!(workspace.check(&plan("new")).unwrap().is_current());
        assert!(!workspace.backup.exists() && !workspace.journal.exists());
    }
}

/// Invalid native names and text are rejected losslessly without granting cleanup authority.
#[cfg(unix)]
#[test]
fn non_utf8_names_and_owned_text_preserve_the_tree() {
    use std::os::unix::ffi::OsStringExt;
    for case in [
        "file-name",
        "directory-name",
        "owned-text",
        "marker-text",
        "root-name",
    ] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("owned")).unwrap();
        let invalid = std::ffi::OsString::from_vec(b"manual-\xff".to_vec());
        let unknown = workspace.root.join(&invalid);
        let created = match case {
            "file-name" => fs::write(&unknown, b"sentinel"),
            "directory-name" => fs::create_dir(&unknown),
            "owned-text" => fs::write(workspace.root.join("data/a.txt"), b"\xff"),
            "marker-text" => fs::write(workspace.root.join(MANIFEST), b"\xff"),
            _ => {
                assert!(OutputWorkspace::new(temp.path().join(invalid)).is_err());
                continue;
            }
        };
        if let Err(error) = created {
            // Some native filesystems/sandboxes cannot represent undecodable names at all.
            assert!(
                matches!(case, "file-name" | "directory-name"),
                "{case}: {error}"
            );
            assert!(
                // APFS can return EILSEQ as Uncategorized; the sandbox can return EPERM.
                matches!(
                    error.kind(),
                    std::io::ErrorKind::PermissionDenied
                        | std::io::ErrorKind::InvalidInput
                        | std::io::ErrorKind::InvalidFilename
                ) || (cfg!(target_os = "macos") && error.raw_os_error() == Some(92)),
                "{case}: {error}"
            );
            eprintln!("host refused non-UTF-8 {case} creation: {error}");
            assert!(!unknown.exists());
            continue;
        }
        let marker = fs::read(workspace.root.join(MANIFEST)).unwrap();
        assert!(workspace.check(&plan("owned")).is_err());
        assert!(workspace.install(plan("replacement")).is_err());
        assert_eq!(fs::read(workspace.root.join(MANIFEST)).unwrap(), marker);
        assert_eq!(
            fs::read_to_string(workspace.root.join("index.txt")).unwrap(),
            "owned\n"
        );
        if case == "file-name" {
            assert_eq!(fs::read(unknown).unwrap(), b"sentinel");
        }
    }
}

/// A forged ownership path never allows a marker to claim files outside the generated root.
#[test]
fn tampered_manifest_cannot_claim_external_paths() {
    for path in [
        "../outside.txt",
        "/outside.txt",
        "data/../../outside.txt",
        "data/x\n",
    ] {
        let temp = temporary();
        let outside = temp.path().join("outside.txt");
        fs::write(&outside, "manual sentinel").unwrap();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("owned")).unwrap();
        let marker = workspace.root.join(MANIFEST);
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&marker).unwrap()).unwrap();
        value["files"][path] = value["files"]["data/a.txt"].clone();
        fs::write(&marker, serde_json::to_vec(&value).unwrap()).unwrap();
        let before = fs::read(&marker).unwrap();
        assert!(workspace.check(&plan("owned")).is_err());
        assert!(workspace.install(plan("replacement")).is_err());
        fs::write(&workspace.journal, INSTALL_INTENT).unwrap();
        assert!(workspace.recover().is_err());
        assert_eq!(fs::read(&marker).unwrap(), before);
        assert_eq!(fs::read_to_string(&outside).unwrap(), "manual sentinel");
    }
}

/// Shared comparison returns identical differences for checking and installing each plan.
#[test]
fn check_and_install_share_the_same_differences() {
    let temp = temporary();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    for text in ["first", "first", "second"] {
        let checked = workspace.check(&plan(text)).unwrap();
        let installed = workspace.install(plan(text)).unwrap();
        assert_eq!(checked.differences, installed.differences);
    }
    let empty = || {
        Registry::new()
            .plan(&SchemaConfig::default(), vec![])
            .unwrap()
    };
    let checked = workspace.check(&empty()).unwrap();
    assert_eq!(
        checked.differences,
        workspace.install(empty()).unwrap().differences
    );
    assert!(workspace.check(&empty()).unwrap().is_current());
}

/// Stale recognized siblings block work; unrelated temporary files are never guessed away.
#[cfg(unix)]
#[test]
fn dangling_recovery_links_and_unrelated_temps_are_preserved() {
    for case in ["stage", "backup", "journal", "lock"] {
        let temp = temporary();
        let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
        workspace.install(plan("owned")).unwrap();
        let link = match case {
            "stage" => &workspace.stage,
            "backup" => &workspace.backup,
            "journal" => &workspace.journal,
            _ => &workspace.lock,
        };
        let absent = temp.path().join("absent");
        std::os::unix::fs::symlink(&absent, link).unwrap();
        assert!(workspace.check(&plan("replacement")).is_err());
        assert!(workspace.install(plan("replacement")).is_err());
        assert!(workspace.recover().is_err());
        assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
        assert!(!absent.exists());
        assert_eq!(
            fs::read_to_string(workspace.root.join("data/a.txt")).unwrap(),
            "owned"
        );
    }
    let temp = temporary();
    let orphan = temp.path().join(".tmp-unrecognized");
    fs::write(&orphan, "manual sentinel").unwrap();
    let workspace = OutputWorkspace::new(temp.path().join("generated")).unwrap();
    workspace.install(plan("new")).unwrap();
    assert_eq!(fs::read_to_string(orphan).unwrap(), "manual sentinel");
}
