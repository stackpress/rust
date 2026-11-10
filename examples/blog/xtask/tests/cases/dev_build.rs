//! Real filesystem staging with substituted finite compiler commands.
use super::*;

/// Generation precedes both renderers and native compile; runtime watch never enters builds.
#[test]
fn prepares_only_selected_builds_and_stages_all_output() {
    let root = tempfile::tempdir().unwrap();
    let mut builder = Builder::new(root.path(), None).unwrap();
    std::fs::create_dir_all(root.path().join("target/native/debug")).unwrap();
    std::fs::write(root.path().join("target/native/debug/serve"), "binary").unwrap();
    let work = Work {
        generate: true,
        browser: true,
        native: true,
        ..Default::default()
    };
    let mut calls = Vec::new();
    let prepared = builder
        .prepare(&work, |command, _| {
            assert_eq!(command.get_current_dir(), Some(root.path()));
            let args = command
                .get_args()
                .map(|s| s.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            assert!(
                command
                    .get_envs()
                    .any(|(k, v)| k == "LEPTOS_WATCH" && v == Some(std::ffi::OsStr::new("1")))
            );
            calls.push(args);
            Ok(())
        })
        .unwrap();
    assert_eq!(calls.len(), 6);
    assert!(calls[0].ends_with(&["--".into(), "generate-app".into()]));
    assert!(calls[5].iter().any(|s| s == "serve"));
    assert_eq!(
        std::fs::read(prepared.executable.unwrap()).unwrap(),
        b"binary"
    );
    assert!(!root.path().join(".build/browser").exists());
    let empty = builder
        .prepare(&Work::default(), |_, _| panic!("unexpected build"))
        .unwrap();
    empty.publish(root.path()).unwrap();
    let custom = Builder::new(root.path(), Some(Path::new("custom/native"))).unwrap();
    assert_eq!(custom.native, root.path().join("custom/native"));
    assert!(Builder::new(root.path(), Some(Path::new("/"))).is_err());
}

/// Any compile/bindgen failure preserves the last published browser products.
#[test]
fn failed_batches_never_publish_partial_bundles() {
    for fail_at in 1..=6 {
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(root.path().join(".build/browser")).unwrap();
        std::fs::write(root.path().join(".build/browser/old"), "last good").unwrap();
        let mut builder = Builder::new(root.path(), None).unwrap();
        let mut calls = 0;
        assert!(
            builder
                .prepare(
                    &Work {
                        generate: true,
                        browser: true,
                        native: true,
                        ..Default::default()
                    },
                    |_, _| {
                        calls += 1;
                        if calls == fail_at {
                            Err("build error".into())
                        } else {
                            Ok(())
                        }
                    }
                )
                .is_err()
        );
        assert_eq!(calls, fail_at);
        assert_eq!(
            std::fs::read(root.path().join(".build/browser/old")).unwrap(),
            b"last good"
        );
    }
}

/// Publication replaces a whole directory and restores the old bytes on a missing stage.
#[test]
fn publishes_or_rolls_back_both_modes() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join(".build/dev/stage")).unwrap();
    std::fs::write(root.path().join(".build/dev/stage/new"), "new").unwrap();
    let prepared = Prepared {
        browser: Some(root.path().join(".build/dev/stage")),
        executable: None,
    };
    prepared.publish(root.path()).unwrap();
    assert!(root.path().join(".build/browser/new").exists());
    assert!(prepared.publish(root.path()).is_err());
    assert!(root.path().join(".build/browser/new").exists());
    std::fs::create_dir_all(root.path().join(".build/dev/stage")).unwrap();
    std::fs::write(root.path().join(".build/dev/stage/next"), "next").unwrap();
    prepared.publish(root.path()).unwrap();
    assert!(root.path().join(".build/browser/next").exists());
    assert!(!root.path().join(".build/dev/previous-browser").exists());
}

/// Runtime dev environment never changes the plain serve command's defaults.
#[test]
fn configures_only_the_owned_dev_child() {
    let root = tempfile::tempdir().unwrap();
    let options = super::super::Options {
        port: 3300,
        reload_port: 3301,
    };
    let command = server_command(root.path(), Path::new("/owned/serve"), &options);
    assert_eq!(command.get_program(), "/owned/serve");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["3300", "config/develop.json"]
    );
    assert_eq!(command.get_envs().filter(|(_, v)| v.is_some()).count(), 3);
}
