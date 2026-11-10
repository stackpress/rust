//! Decision and real-process unit tests; excluded from production coverage counts.

use super::*;
use std::ffi::OsString;

/// Default, relative and absolute target roots keep both renderers beside native output.
#[test]
fn targets_follow_the_native_override() {
    let root = tempfile::tempdir().unwrap();
    assert_eq!(
        target_parent(root.path(), None).unwrap(),
        root.path().join("target")
    );
    assert_eq!(
        target_parent(root.path(), Some(Path::new("out/native"))).unwrap(),
        root.path().join("out")
    );
    assert_eq!(
        target_parent(root.path(), Some(&root.path().join("custom/native"))).unwrap(),
        root.path().join("custom")
    );
    assert!(target_parent(root.path(), Some(Path::new("/"))).is_err());
}

/// Each bindgen call consumes its own renderer's WASM and writes only that renderer's glue.
#[test]
fn plans_separate_locked_offline_modes() {
    let root = tempfile::tempdir().unwrap();
    let mut calls = Vec::new();
    build_with(root.path(), None, |command, timeout| {
        assert_eq!(timeout, Duration::from_secs(600));
        assert_eq!(command.get_current_dir(), Some(root.path()));
        calls.push((
            command.get_program().to_owned(),
            command.get_args().map(OsString::from).collect::<Vec<_>>(),
            command
                .get_envs()
                .map(|(key, value)| (key.to_owned(), value.map(OsString::from)))
                .collect::<Vec<_>>(),
        ));
        Ok(())
    })
    .unwrap();
    assert_eq!(calls.len(), 4);
    for (index, mode) in ["hydrate", "csr"].iter().enumerate() {
        let (program, args, env) = &calls[index * 2];
        assert_eq!(program, "cargo");
        assert!(args.windows(2).any(|pair| pair == ["--features", mode]));
        assert!(args.ends_with(&["--locked".into(), "--offline".into()]));
        assert_eq!(
            env,
            &[(
                "CARGO_TARGET_DIR".into(),
                Some(root.path().join("target").join(mode).into_os_string())
            )]
        );
        let (program, args, _) = &calls[index * 2 + 1];
        assert_eq!(program, "wasm-bindgen");
        assert_eq!(
            args.last().unwrap(),
            root.path()
                .join("target")
                .join(mode)
                .join("wasm32-unknown-unknown/debug/blog_client.wasm")
                .as_os_str()
        );
        assert!(root.path().join(".build/browser").join(mode).is_dir());
    }
}

/// Neither a failed compile nor failed bindgen starts the next mode or hides the error.
#[test]
fn stops_at_the_first_failed_child() {
    for fail_at in 1..=4 {
        let root = tempfile::tempdir().unwrap();
        let mut count = 0;
        let result = build_with(root.path(), None, |_, _| {
            count += 1;
            if count == fail_at {
                Err("injected failure".into())
            } else {
                Ok(())
            }
        });
        assert!(result.unwrap_err().to_string().contains("injected failure"));
        assert_eq!(count, fail_at);
    }
}

/// Output filesystem errors abort publication without invoking bindgen.
#[test]
fn rejects_an_unwritable_output_location() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join(".build"), "file, not directory").unwrap();
    assert!(build_with(root.path(), None, |_, _| Ok(())).is_err());
}

/// Real child processes expose spawn/status errors and timeouts with no listener.
#[cfg(unix)]
#[test]
fn bounds_and_reaps_real_children() {
    assert!(
        bounded(
            Command::new("/bin/sh").args(["-c", "exit 0"]),
            Duration::from_secs(1)
        )
        .is_ok()
    );
    assert!(
        bounded(
            Command::new("/bin/sh").args(["-c", "exit 7"]),
            Duration::from_secs(1)
        )
        .unwrap_err()
        .to_string()
        .contains("failed")
    );
    assert!(
        bounded(
            &mut Command::new("/missing-blog-xtask-command"),
            Duration::from_secs(1)
        )
        .is_err()
    );
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("orphan");
    let error = bounded(
        Command::new("/bin/sh")
            .args(["-c", "(sleep 1; touch \"$1\") & wait", "sh"])
            .arg(&marker),
        Duration::from_millis(30),
    )
    .unwrap_err();
    assert!(error.to_string().contains("deadline"));
    std::thread::sleep(Duration::from_millis(1100));
    assert!(!marker.exists(), "deadline left a descendant running");
}

/// Cancellation is observed after a real child starts and kills nested work without waiting.
#[cfg(unix)]
#[test]
fn cancellation_reaps_the_active_build_group() {
    let root = tempfile::tempdir().unwrap();
    let marker = root.path().join("started");
    let error = bounded_until(
        Command::new("/bin/sh")
            .args(["-c", "touch \"$1\"; sleep 60 & wait", "sh"])
            .arg(&marker),
        Duration::from_secs(5),
        || marker.exists(),
    )
    .unwrap_err();
    assert!(error.to_string().contains("cancelled"));
}

/// Incremental rustc receives Cargo's flags directly; dependency invocations retain sccache.
#[test]
fn compiler_wrapper_preserves_flags_and_caches_dependencies() {
    let args = |values: &[&str]| values.iter().map(OsString::from).collect::<Vec<_>>();
    let local = compiler_command(
        std::ffi::OsStr::new("sccache"),
        &args(&["rustc", "-C", "incremental=/owned/cache"]),
        true,
    )
    .unwrap();
    assert_eq!(local.get_program(), "rustc");
    assert_eq!(
        local.get_args().collect::<Vec<_>>(),
        ["-C", "incremental=/owned/cache"]
    );
    assert!(
        local
            .get_envs()
            .any(|(k, v)| k == "CARGO_INCREMENTAL" && v.is_none())
    );
    let dependency = compiler_command(
        std::ffi::OsStr::new("sccache"),
        &args(&["rustc", "--crate-name", "dep"]),
        false,
    )
    .unwrap();
    assert_eq!(dependency.get_program(), "sccache");
    assert!(compiler_command(std::ffi::OsStr::new("sccache"), &[], false).is_err());
    #[cfg(unix)]
    {
        compiler(std::ffi::OsStr::new("/usr/bin/true"), &args(&["rustc"])).unwrap();
        assert!(compiler(std::ffi::OsStr::new("/usr/bin/false"), &args(&["rustc"])).is_err());
    }
}

/// Explicit incremental mode must not turn unwatched framework dependencies into slow local builds.
#[test]
fn strips_dependency_incremental_flags_before_sccache() {
    let args = [
        "rustc",
        "-C",
        "incremental=/owned/dep",
        "-Cincremental=/owned/other",
        "--crate-name",
        "dep",
    ]
    .map(OsString::from);
    let command = compiler_command(std::ffi::OsStr::new("sccache"), &args, false).unwrap();
    assert_eq!(command.get_program(), "sccache");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["rustc", "--crate-name", "dep"]
    );
    let command = compiler_command(std::ffi::OsStr::new(""), &args, false).unwrap();
    assert_eq!(command.get_program(), "rustc");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["--crate-name", "dep"]
    );
}
