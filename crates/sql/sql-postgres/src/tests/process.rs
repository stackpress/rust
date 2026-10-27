//! Independent success/timeout controls for the shared native-runner watchdog.
#[path = "support/process.rs"]
mod implementation;
use implementation::watchdog;

#[cfg(test)]
mod checks {
    use super::watchdog;
    use std::process::{Command, Stdio};
    use std::time::Duration;

    /// Child-only mode; environment selection belongs to Command, never the parallel parent.
    const MODE: &str = "STACKPRESS_SQL_WATCHDOG_MODE";

    #[test]
    #[ignore = "invoked only by the watchdog controls"]
    /// A successful exit or long pending computation, without a database or listener.
    fn watchdog_child() {
        match std::env::var(MODE).as_deref() {
            Ok("exit") => {}
            Ok("wait") => std::thread::sleep(Duration::from_secs(30)),
            _ => panic!("watchdog child requires an explicit mode"),
        }
    }

    /// Spawn only the exact ignored assertion; suppress unrelated process output.
    fn child(mode: &str) -> std::process::Child {
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "tests::process::checks::watchdog_child",
                "--ignored",
            ])
            .env(MODE, mode)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap()
    }

    #[test]
    /// Ordinary completion is reaped and retains its exact status.
    fn acknowledged_exit_is_successful() {
        assert!(
            watchdog(child("exit"), Duration::from_secs(5))
                .unwrap()
                .success()
        );
    }

    #[test]
    /// Expiration kills/reaps a deliberately pending child and remains an error.
    fn timeout_is_a_failure() {
        let error = watchdog(child("wait"), Duration::from_millis(20)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "native scenario process watchdog timed out; child killed and reaped"
        );
    }
}
