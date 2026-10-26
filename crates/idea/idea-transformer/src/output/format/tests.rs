//! Controlled subprocesses prove stream budgets, cleanup and accepted artifact compatibility.
use super::*;

/// A controlled command emits an excessive pipe without formatter/config dependencies.
fn oversized_formatter_stdout_is_rejected_during_acquisition() {
    let result = run_formatter(
        "",
        Command::new("python3").args([
            "-c",
            "import sys; sys.stdout.write('x' * (1024 * 1024 + 1))",
        ]),
    );
    assert!(result.is_err(), "oversized stdout was retained");
}

/// Diagnostics have their own bound independently of the accepted artifact bytes.
fn oversized_formatter_stderr_is_rejected_during_acquisition() {
    let result = run_formatter(
        "",
        Command::new("python3").args(["-c", "import sys; sys.stderr.write('x' * (64 * 1024 + 1))"]),
    );
    assert!(result.is_err(), "oversized stderr was retained");
}

/// Observe termination and reaping, rather than merely receiving an error result.
fn assert_reaped_failure(script: &str, text: &str, kind: crate::TransformErrorKind) {
    let directory = tempfile::tempdir().unwrap();
    let pid_path = directory.path().join("pid");
    let script =
        format!("import os,sys,time\nopen(sys.argv[1], 'w').write(str(os.getpid()))\n{script}");
    let started = Instant::now();
    let error = supervise(
        text,
        Command::new("python3").args(["-c", &script]).arg(&pid_path),
        Duration::from_millis(200),
    )
    .unwrap_err();
    assert_eq!(error.kind(), kind);
    // A missing kill would wait for the two-second fixture stall while joining pipe workers.
    assert!(started.elapsed() < Duration::from_secs(1));
    #[cfg(unix)]
    {
        let pid = std::fs::read_to_string(&pid_path).unwrap();
        // Even a zombie still responds to kill -0, so this checks reaping as well as termination.
        assert!(
            !Command::new("kill")
                .args(["-0", pid.trim()])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success()
        );
    }
}

/// Runtime supervision must terminate/reap a child that never produces an artifact.
fn stalled_formatter_is_killed_and_reaped() {
    assert_reaped_failure("time.sleep(2)", "", crate::TransformErrorKind::Limit);
    for stream in ["stdout", "stderr"] {
        let script = format!(
            "try:\n sys.{stream}.write('x' * (1024 * 1024 + 1)); sys.{stream}.flush()\nexcept BrokenPipeError: pass\ntime.sleep(2)"
        );
        assert_reaped_failure(&script, "", crate::TransformErrorKind::Limit);
    }
    // Stdin failure is supervised even while a child intentionally remains alive afterward.
    assert_reaped_failure(
        "os.close(0); time.sleep(2)",
        &"x".repeat(STDOUT_BYTES),
        crate::TransformErrorKind::Io,
    );
}

/// Pipe error, exit and encoding paths retain diagnostics without accepting failed output.
fn formatter_boundaries_and_failures() {
    let exact = run_formatter("", Command::new("python3").args(["-c", "import sys; sys.stdout.write('x' * (1024 * 1024)); sys.stderr.write('x' * (64 * 1024))"])).unwrap();
    assert_eq!(exact.len(), STDOUT_BYTES);
    for script in [
        "import sys; sys.exit(1)",
        "import sys; sys.stdout.buffer.write(bytes([255]))",
    ] {
        assert!(run_formatter("", Command::new("python3").args(["-c", script])).is_err());
    }
    struct BrokenReader;
    impl Read for BrokenReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::PermissionDenied.into())
        }
    }
    assert_eq!(
        read_capped(BrokenReader, 1).unwrap_err().kind(),
        crate::TransformErrorKind::Io
    );
}

/// All real subprocess checks run only under the independent parent watchdog.
#[test]
#[ignore = "run by formatter watchdog"]
fn formatter_child() {
    oversized_formatter_stdout_is_rejected_during_acquisition();
    oversized_formatter_stderr_is_rejected_during_acquisition();
    stalled_formatter_is_killed_and_reaped();
    formatter_boundaries_and_failures();
}

/// Kill/reap even if a broken implementation blocks a pipe worker or its scope join.
#[test]
fn formatter_contracts_are_watchdog_bounded() {
    let result = crate::watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "output::format::tests::formatter_child",
            "--nocapture",
        ]),
        "formatter",
        Duration::from_secs(5),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
