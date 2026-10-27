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

/// Wait for an atomically published PID; the independent parent bounds fixture startup.
fn ready_pid(path: &std::path::Path) -> String {
    loop {
        match std::fs::read_to_string(path) {
            Ok(pid) => return pid,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => panic!("formatter readiness failed: {error}"),
        }
    }
}

/// Start the production collector's deadline after the stalled child and pipe workers are ready.
fn deadline_failure(
    text: &str,
    command: &mut Command,
    pid_path: &std::path::Path,
    go_path: &std::path::Path,
) -> (crate::TransformError, Instant, String) {
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::scope(move |scope| {
        // The production owner must kill/reap before the production pipe workers join.
        let mut child = ChildOwner(child);
        let receiver = start_pipes(scope, text, &mut child.0).unwrap();
        let pid = ready_pid(pid_path);
        let started = Instant::now();
        std::fs::write(go_path, b"go").unwrap();
        let error =
            collect(&mut child.0, &receiver, started, Duration::from_millis(200)).unwrap_err();
        drop(child);
        (error, started, pid)
    })
}

/// Exercise the complete supervisor wiring after an explicit release triggers a pipe failure.
fn pipe_failure(
    text: &str,
    command: &mut Command,
    pid_path: &std::path::Path,
    go_path: &std::path::Path,
) -> (crate::TransformError, Instant, String) {
    std::thread::scope(|scope| {
        // Ordinary production supervision still owns spawn, pipes, kill and reap. Its
        // normal limit is not the property here: the released IO failure must end it.
        let supervisor = scope.spawn(|| supervise(text, command, FORMATTER_TIME));
        let pid = ready_pid(pid_path);
        let started = Instant::now();
        std::fs::write(go_path, b"go").unwrap();
        let error = supervisor.join().unwrap().unwrap_err();
        (error, started, pid)
    })
}

/// Observe termination and reaping, rather than merely receiving an error result.
fn assert_reaped_failure(script: &str, text: &str, kind: crate::TransformErrorKind) {
    let directory = tempfile::tempdir().unwrap();
    let pid_path = directory.path().join("pid");
    let go_path = directory.path().join("go");
    let measures_deadline = script.is_empty();
    // Publish readiness before any deliberate error and wait for the controlling test.
    // A missing kill cannot pass by waiting out a short sleep. If the outer watchdog
    // kills the Rust owner, reparenting also lets this fixture exit instead of leaking.
    let script = format!(
        "import os,sys,time\nowner=os.getppid()\n\
         with open(sys.argv[1]+'.tmp', 'w') as ready: ready.write(str(os.getpid()))\n\
         os.replace(sys.argv[1]+'.tmp', sys.argv[1])\n\
         while not os.path.exists(sys.argv[2]) and os.getppid()==owner: time.sleep(0.001)\n\
         {script}\n\
         while os.getppid()==owner: time.sleep(0.01)"
    );
    let mut command = Command::new("python3");
    command.args(["-c", &script]).args([&pid_path, &go_path]);
    let (error, started, pid) = if measures_deadline {
        deadline_failure(text, &mut command, &pid_path, &go_path)
    } else {
        pipe_failure(text, &mut command, &pid_path, &go_path)
    };
    assert_eq!(error.kind(), kind);
    // This includes native child termination, wait/reaping and all scoped pipe joins.
    assert!(started.elapsed() < Duration::from_secs(1));
    #[cfg(unix)]
    {
        // Even a zombie responds to kill -0: failure checks both termination and reaping.
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
    #[cfg(not(unix))]
    let _ = pid;
}

/// Runtime supervision must terminate/reap a child that never produces an artifact.
fn stalled_formatter_is_killed_and_reaped() {
    assert_reaped_failure("", "", crate::TransformErrorKind::Limit);
    for stream in ["stdout", "stderr"] {
        let script = format!(
            "try:\n sys.{stream}.write('x' * (1024 * 1024 + 1)); sys.{stream}.flush()\nexcept BrokenPipeError: pass"
        );
        assert_reaped_failure(&script, "", crate::TransformErrorKind::Limit);
    }
    // Stdin failure is supervised even while a child intentionally remains alive afterward.
    assert_reaped_failure(
        "os.close(0)",
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
