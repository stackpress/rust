//! Real absent-root generation, deterministic regeneration and compile-selected plugin changes.

use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Always reap a child, including a test assertion or the parent watchdog expiring.
struct Guard(Child);
impl Drop for Guard {
    /// A completed process is harmless to kill; every path still waits for its child.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run generation in a child with a hard parent deadline and bounded retained output.
fn cli(args: &[&str], output: &Path) -> bool {
    let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let file = std::fs::File::create(log.path()).unwrap();
    let mut child = Guard(
        Command::new(env!("CARGO_BIN_EXE_generated-admin-tool"))
            .args(args)
            .arg("--output")
            .arg(output)
            .stdout(Stdio::from(file.try_clone().unwrap()))
            .stderr(Stdio::from(file))
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "generation exceeded parent watchdog"
        );
        assert!(
            log.as_file().metadata().unwrap().len() < 128 * 1024,
            "generation log exceeded bound"
        );
        if let Some(status) = child.0.try_wait().unwrap() {
            if !status.success() {
                eprintln!("{}", std::fs::read_to_string(log.path()).unwrap());
            }
            return status.success();
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Compare every writer-owned byte, including its manifest and all generated partitions.
fn snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut result = Vec::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            result.extend(
                snapshot(&path)
                    .into_iter()
                    .map(|(child, bytes)| (path.file_name().unwrap().into(), child, bytes))
                    .map(|(parent, child, bytes): (PathBuf, _, _)| (parent.join(child), bytes)),
            );
        } else {
            result.push((
                path.file_name().unwrap().into(),
                std::fs::read(path).unwrap(),
            ));
        }
    }
    result.sort();
    result
}

/// Removing/disabling a plugin regenerates its output; defaults return byte-for-byte.
#[test]
fn actual_three_model_writer_is_stable_and_plugin_is_removable() {
    let temporary = tempfile::tempdir_in("/private/tmp").unwrap();
    let output = temporary.path().join("generated");
    assert!(cli(&["emit", "generate"], &output));
    let baseline = snapshot(&output);
    assert!(output.join("contracts/public.rs").exists());
    assert!(output.join("stores/profile.rs").exists());
    assert!(output.join("ui/profile.rs").exists());
    assert!(cli(&["--check"], &output));
    assert!(cli(&[], &output));
    assert_eq!(baseline, snapshot(&output));
    assert!(cli(&["--plugin"], &output));
    let plugin = std::fs::read_to_string(output.join("ui/profile.rs")).unwrap();
    assert!(plugin.contains("crate::plugin::AccentField"));
    assert!(plugin.contains("Plugin field label"));
    assert!(plugin.contains("Plugin fixed hint"));
    let tags = std::fs::read_to_string(output.join("ui/project.rs")).unwrap();
    assert!(tags.contains("crate::plugin::AccentTags"));
    assert!(tags.contains("Plugin tags label"));
    assert!(!cli(&["--check"], &output));
    assert!(cli(&[], &output));
    assert_eq!(baseline, snapshot(&output));
}

/// Whole-root replacement removes stale models, and invalid selections preserve owned output.
#[test]
fn model_remove_rename_and_failed_regeneration_preserve_writer_contract() {
    let input_dir = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).unwrap();
    let temporary = tempfile::tempdir_in("/private/tmp").unwrap();
    let input = input_dir.path().join("schema.idea");
    let output = temporary.path().join("generated");
    let run = || cli(&["--input", input.to_str().unwrap()], &output);
    std::fs::write(&input, "plugin \"framework-schema\" {} plugin \"framework-admin\" {} model Profile { id String @id name String @field.input }").unwrap();
    assert!(run());
    assert!(output.join("ui/profile.rs").exists());
    std::fs::write(&input, "plugin \"framework-schema\" {} plugin \"framework-admin\" {} model Project { id String @id title String @field.input }").unwrap();
    assert!(run());
    assert!(!output.join("ui/profile.rs").exists());
    assert!(output.join("ui/project.rs").exists());
    let previous = snapshot(&output);
    for invalid in [
        "plugin \"framework-admin\" {} model Project { id String @id }",
        "plugin \"unknown\" {} model Project { id String @id }",
        "plugin \"framework-schema\" {} plugin \"framework-admin\" {} model Project { id String @id title String @field.code }",
    ] {
        std::fs::write(&input, invalid).unwrap();
        assert!(!run());
        assert_eq!(previous, snapshot(&output));
    }
    for invalid in [vec!["--unknown"], vec!["--input"], vec!["--output"]] {
        // The final --output argument is intentionally parsed as a missing option's value;
        // either parse/source refusal must remain a failed invocation.
        assert!(!cli(&invalid, &output));
    }
}
