//! Controlled workers exercise cancellation independently of native OS scheduling.
use super::*;

#[path = "../../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Build the real input owner around a controlled stalled native-work substitute.
fn controlled(work: impl FnOnce() -> io::Result<()> + Send + 'static) -> TtyInput {
    let stop = Arc::new(AtomicBool::new(false));
    let (_, events) = mpsc::channel(1);
    let (worker, done) = start_worker(&stop, work).unwrap();
    TtyInput {
        stop,
        worker,
        done: Some(done),
        events,
    }
}

#[test]
#[ignore = "run by cleanup watchdog"]
fn dropping_input_transfers_cleanup_without_waiting_for_native_work() {
    let (release, held) = std::sync::mpsc::channel();
    let (joined, restored) = std::sync::mpsc::channel();
    let input = controlled(move || {
        held.recv().unwrap();
        joined.send(()).unwrap();
        Ok(())
    });
    let unblock = thread::spawn(move || {
        thread::sleep(Duration::from_millis(400));
        release.send(()).unwrap();
    });
    let started = std::time::Instant::now();
    drop(input);
    let elapsed = started.elapsed();
    unblock.join().unwrap();
    restored.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(
        elapsed < Duration::from_millis(100),
        "Drop waited {elapsed:?}"
    );
}

/// Retrying a cancelled close must await the same worker, then allow idempotent close.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by cleanup watchdog"]
async fn close_cancellation_failure_and_repeated_close() {
    for failure in [false, true] {
        let (release, held) = std::sync::mpsc::channel();
        let mut input = controlled(move || {
            held.recv().unwrap();
            if failure {
                Err(io::Error::other("restoration failed"))
            } else {
                Ok(())
            }
        });
        assert!(
            tokio::time::timeout(Duration::from_millis(10), input.close())
                .await
                .is_err()
        );
        release.send(()).unwrap();
        let result = input.close().await;
        assert_eq!(result.is_err(), failure);
        input.close().await.unwrap();
    }
    let mut input = controlled(|| panic!("PRIVATE worker panic"));
    let error = input.close().await.unwrap_err();
    assert_eq!(error.to_string(), "terminal input worker panicked");
    input.close().await.unwrap();
}

/// Failure retains native ownership and restoration intent until an explicit successful retry.
#[test]
fn restoration_failure_retains_the_terminal_lease() {
    TERMINAL_LEASED.store(true, Ordering::Release);
    let mut session = TtySession {
        restore_raw_mode: true,
        active: true,
    };
    assert!(
        session
            .close_with(|| Err(io::Error::other("restore failed")))
            .is_err()
    );
    assert!(session.active && session.restore_raw_mode);
    assert!(TERMINAL_LEASED.load(Ordering::Acquire));
    session.close_with(|| Ok(())).unwrap();
    assert!(!session.active && !session.restore_raw_mode);
    assert!(!TERMINAL_LEASED.load(Ordering::Acquire));
}

/// A hung native substitute is killed/reaped independently of the async test executor.
#[test]
fn cleanup_contracts_are_watchdog_bounded() {
    for name in [
        "tty::tests::dropping_input_transfers_cleanup_without_waiting_for_native_work",
        "tty::tests::close_cancellation_failure_and_repeated_close",
    ] {
        let result = watchdog::run(
            std::process::Command::new(std::env::current_exe().unwrap()).args([
                "--ignored",
                "--exact",
                name,
                "--nocapture",
            ]),
            "tty-cleanup",
            Duration::from_secs(3),
        )
        .unwrap();
        assert!(!result.timed_out, "{}", result.output);
        assert!(result.status.success(), "{}", result.output);
    }
}
