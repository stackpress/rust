//! Session publication, startup readiness and retirement use finite dummy processes only.
use super::super::change::classify;
use super::*;
use std::collections::BTreeSet;

/// Build errors leave a real child alive; replacement and Drop reap its entire group.
#[cfg(unix)]
#[test]
fn retains_last_good_child_and_reaps_replacements() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    std::fs::write(&first, "owned executable").unwrap();
    std::fs::write(&second, "owned executable").unwrap();
    let mut session = Session::default();
    session
        .replace(
            Command::new("/bin/sh").args(["-c", "sleep 60 & wait"]),
            first.clone(),
        )
        .unwrap();
    let first_id = session.child.as_ref().unwrap().child.id();
    let mut builder = Builder::new(root.path(), None).unwrap();
    assert!(
        session
            .update(
                root.path(),
                &Options::parse(&[]).unwrap(),
                &mut builder,
                &Work {
                    native: true,
                    restart: true,
                    ..Default::default()
                },
                |_, _| Err("compiler failure".into())
            )
            .is_err()
    );
    assert_eq!(session.child.as_ref().unwrap().child.id(), first_id);
    assert!(
        session
            .child
            .as_mut()
            .unwrap()
            .child
            .try_wait()
            .unwrap()
            .is_none()
    );
    let notices = session
        .update(
            root.path(),
            &Options::parse(&[]).unwrap(),
            &mut builder,
            &classify(&BTreeSet::from([PathBuf::from(
                "plugins/assets/assets/global.css",
            )])),
            |_, _| panic!("CSS compiled"),
        )
        .unwrap();
    assert_eq!(notices, vec![Notice::Css("styles/global.css".into())]);
    session
        .replace(
            Command::new("/bin/sh").args(["-c", "exit 0"]),
            second.clone(),
        )
        .unwrap();
    assert!(!first.exists());
    let deadline = Instant::now() + Duration::from_secs(3);
    while session.child.is_some() && Instant::now() < deadline {
        session.check_exit().unwrap();
        std::thread::yield_now();
    }
    assert!(session.child.is_none());
    drop(session);
    assert!(!second.exists());
    let mut empty = Session::default();
    assert!(
        empty
            .update(
                root.path(),
                &Options::parse(&[]).unwrap(),
                &mut builder,
                &Work {
                    restart: true,
                    ..Default::default()
                },
                |_, _| Ok(())
            )
            .is_err()
    );
    assert!(
        empty
            .replace(
                &mut Command::new("/no-such-dev-command"),
                root.path().join("missing")
            )
            .is_err()
    );
}

/// Readiness comes from the owned child's stdout, without opening or probing a socket.
#[cfg(unix)]
#[test]
fn waits_for_owned_readiness_and_reports_startup_errors() {
    let root = tempfile::tempdir().unwrap();
    let executable = root.path().join("dummy");
    std::fs::write(&executable, "owned").unwrap();
    let mut session = Session::default();
    session
        .replace(
            Command::new("/bin/sh")
                .args(["-c", "echo 'ready http://localhost:3000/'; sleep 60 & wait"])
                .env("BLOG_DEV_RELOAD_PORT", "3001"),
            executable.clone(),
        )
        .unwrap();
    assert!(session.output.is_some());
    assert!(
        session
            .child
            .as_mut()
            .unwrap()
            .child
            .try_wait()
            .unwrap()
            .is_none()
    );
    assert!(
        session
            .replace(
                Command::new("/bin/sh")
                    .args(["-c", "exit 3"])
                    .env("BLOG_DEV_RELOAD_PORT", "3001"),
                executable
            )
            .unwrap_err()
            .to_string()
            .contains("did not become ready")
    );
}

/// Successful CSR publication advances only its renderer; failures remain retryable.
#[test]
fn publishes_deferred_csr_or_retains_the_last_good_bundle() {
    let root = tempfile::tempdir().unwrap();
    let mut builder = Builder::new(root.path(), None).unwrap();
    let mut session = Session::default();
    assert!(session.finish_csr(root.path()).is_none());
    for fail in [false, true] {
        session.csr_due = true;
        session.csr = Some(Csr::start_with(builder.clone(), move |_, _, _| {
            if fail {
                Err("CSR compile failure".into())
            } else {
                Ok(())
            }
        }));
        let deadline = Instant::now() + Duration::from_secs(5);
        while !session.csr.as_ref().unwrap().finished() {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        assert_eq!(
            session.finish_csr(root.path()),
            if fail { None } else { Some(Notice::Csr) }
        );
        assert_eq!(session.csr_due, fail);
        assert!(root.path().join(".build/browser/csr").is_dir());
    }
    // A newer failed critical batch cancels the old worker but cannot publish its staging.
    let (send, receive) = std::sync::mpsc::channel();
    session.csr = Some(Csr::start_with(builder.clone(), move |_, _, cancel| {
        send.send(()).unwrap();
        while !cancel.load(Ordering::Relaxed) {
            std::thread::yield_now();
        }
        Err("canceled".into())
    }));
    receive.recv_timeout(Duration::from_secs(2)).unwrap();
    assert!(
        session
            .update(
                root.path(),
                &Options::parse(&[]).unwrap(),
                &mut builder,
                &Work {
                    browser: true,
                    restart: true,
                    ..Default::default()
                },
                |_, _| Err("critical failed".into())
            )
            .is_err()
    );
    assert!(session.csr.is_none());
    assert!(session.csr_due);
    assert_eq!(session.notice(), Notice::Hydrate);
}

/// Ctrl-C interrupts a child that never prints readiness, and its process group is reaped.
#[cfg(unix)]
#[test]
fn cancels_a_pending_readiness_wait() {
    let root = tempfile::tempdir().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let mut session = Session::new(stop.clone());
    let marker = root.path().join("started");
    let observed = marker.clone();
    let cancel = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !observed.exists() {
            assert!(Instant::now() < deadline);
            std::thread::yield_now();
        }
        stop.store(true, Ordering::Relaxed);
    });
    let started = Instant::now();
    assert!(
        session
            .replace(
                Command::new("/bin/sh")
                    .args(["-c", "sleep 60 & echo started > \"$1\"; wait", "sh"])
                    .arg(&marker)
                    .env("BLOG_DEV_RELOAD_PORT", "3001"),
                root.path().join("candidate")
            )
            .unwrap_err()
            .to_string()
            .contains("canceled")
    );
    cancel.join().unwrap();
    assert!(started.elapsed() < Duration::from_secs(3));
    assert!(session.child.is_none());
}
