//! Real filesystem staging with substituted finite compiler commands and explicit barriers.
use super::*;
use std::sync::{Barrier, Mutex};

/// Make a mock native artifact without executing an app or opening a port.
fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("target/native/debug")).unwrap();
    std::fs::write(root.path().join("target/native/debug/serve"), "binary").unwrap();
    root
}

/// Generation precedes concurrent hydration/native builds; CSR is absent from the critical path.
#[test]
fn prepares_only_selected_builds_and_stages_all_output() {
    let root = fixture();
    let mut builder = Builder::new(root.path(), None).unwrap();
    let barrier = Barrier::new(2);
    let calls = Mutex::new(Vec::new());
    let prepared = builder
        .prepare(
            &Work {
                generate: true,
                browser: true,
                native: true,
                ..Default::default()
            },
            |command, _| {
                assert_eq!(command.get_current_dir(), Some(root.path()));
                let args = command
                    .get_args()
                    .map(|s| s.to_string_lossy().into_owned())
                    .collect::<Vec<_>>();
                let env = command
                    .get_envs()
                    .collect::<std::collections::BTreeMap<_, _>>();
                for key in ["LEPTOS_WATCH", "CARGO_INCREMENTAL"] {
                    assert_eq!(
                        env[std::ffi::OsStr::new(key)],
                        Some(std::ffi::OsStr::new("1"))
                    );
                }
                let jobs = if args.first().is_some_and(|s| s == "run") {
                    "2"
                } else {
                    "1"
                };
                assert_eq!(
                    env[std::ffi::OsStr::new("CARGO_BUILD_JOBS")],
                    Some(std::ffi::OsStr::new(jobs))
                );
                calls.lock().unwrap().push(args.clone());
                // Neither compiler can reach this point unless the other has started.
                if args.first().is_some_and(|s| s == "build") {
                    barrier.wait();
                }
                Ok(())
            },
        )
        .unwrap();
    let calls = calls.into_inner().unwrap();
    assert_eq!(calls.len(), 4);
    assert!(calls[0].ends_with(&["--".into(), "generate-app".into()]));
    assert!(!calls.iter().flatten().any(|s| s == "csr"));
    assert_eq!(
        std::fs::read(prepared.executable.unwrap()).unwrap(),
        b"binary"
    );
    assert!(!root.path().join(".build/browser").exists());
    builder
        .prepare(&Work::default(), |_, _| panic!("unexpected build"))
        .unwrap()
        .publish(root.path())
        .unwrap();
    assert_eq!(
        Builder::new(root.path(), Some(Path::new("custom/native")))
            .unwrap()
            .native,
        root.path().join("custom/native")
    );
    assert!(Builder::new(root.path(), Some(Path::new("/"))).is_err());
}

/// Every critical failure waits for both scoped workers and leaves public products unchanged.
#[test]
fn failed_batches_never_publish_partial_bundles() {
    for failure in ["generate-app", "serve", "hydrate", "bindgen"] {
        let root = fixture();
        std::fs::create_dir_all(root.path().join(".build/browser/hydrate")).unwrap();
        std::fs::write(root.path().join(".build/browser/hydrate/old"), "last good").unwrap();
        let mut builder = Builder::new(root.path(), None).unwrap();
        assert!(
            builder
                .prepare(
                    &Work {
                        generate: true,
                        browser: true,
                        native: true,
                        ..Default::default()
                    },
                    |c, _| {
                        let args = c
                            .get_args()
                            .map(|s| s.to_string_lossy())
                            .collect::<Vec<_>>();
                        if args.iter().any(|s| s == failure)
                            || (failure == "bindgen" && c.get_program() == "wasm-bindgen")
                        {
                            Err("build error".into())
                        } else {
                            Ok(())
                        }
                    }
                )
                .is_err()
        );
        assert_eq!(
            std::fs::read(root.path().join(".build/browser/hydrate/old")).unwrap(),
            b"last good"
        );
    }
}

/// Publishing one mode preserves the other mode; a missing stage restores the prior bytes.
#[test]
fn publishes_or_rolls_back_each_mode() {
    let root = fixture();
    for mode in ["hydrate", "csr"] {
        let source = root.path().join(".build/dev/browser").join(mode);
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("bundle"), mode).unwrap();
        let prepared = Prepared {
            browser: Some((source.clone(), mode)),
            executable: None,
        };
        prepared.publish(root.path()).unwrap();
        assert!(prepared.publish(root.path()).is_err());
        assert_eq!(
            std::fs::read_to_string(root.path().join(".build/browser").join(mode).join("bundle"))
                .unwrap(),
            mode
        );
        std::fs::create_dir_all(&source).unwrap();
        std::fs::write(source.join("bundle"), "next").unwrap();
        prepared.publish(root.path()).unwrap();
    }
    assert!(root.path().join(".build/browser/hydrate/bundle").exists());
    assert!(root.path().join(".build/browser/csr/bundle").exists());
}

/// Background output is invisible until joined/published, and cancellation joins the worker.
#[test]
fn defers_csr_and_cancels_superseded_work() {
    let root = fixture();
    let (started, receive) = std::sync::mpsc::channel();
    let mut worker = Csr::start_with(
        Builder::new(root.path(), None).unwrap(),
        move |_, _, cancel| {
            started.send(()).unwrap();
            while !cancel.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            Err("canceled".into())
        },
    );
    receive.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(!worker.finished());
    assert!(!root.path().join(".build/browser/csr").exists());
    // Drop sets the flag and joins; a later worker can reuse this exact staging directory.
    drop(worker);
    worker = Csr::start_with(Builder::new(root.path(), None).unwrap(), |_, _, _| Ok(()));
    let prepared = worker.finish().unwrap();
    assert!(worker.finish().is_err());
    prepared.publish(root.path()).unwrap();
    assert!(root.path().join(".build/browser/csr").is_dir());
    let mut failed = Csr::start_with(Builder::new(root.path(), None).unwrap(), |_, _, _| {
        Err("CSR failed".into())
    });
    assert!(failed.finish().is_err());
}

/// Runtime dev environment never changes the plain serve command's defaults.
#[test]
fn configures_only_the_owned_dev_child() {
    let root = fixture();
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
