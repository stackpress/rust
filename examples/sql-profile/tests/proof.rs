//! Native-file and actual-reference wire proofs. All SQLite work is in externally bounded children.

mod cases;
mod support;

use std::process::Command;
use std::time::Duration;

/// Every native scenario has a separate independent process deadline, even on synchronous deadlock.
#[test]
fn native_contract_under_watchdog() {
    for scenario in ["parity", "literals", "startup", "setup", "failures"] {
        let mut child = Command::new(std::env::current_exe().unwrap());
        child
            .args(["--exact", "native_child", "--ignored", "--nocapture"])
            .env("STACKPRESS_SQL_PROFILE_SCENARIO", scenario);
        let result = support::run(&mut child, Duration::from_secs(10)).unwrap();
        assert!(
            result.status.success() && !result.expired,
            "{scenario}: timeout={}\n{}\n{}",
            result.expired,
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

/// Only the process parent launches native tests; the executor timeout is a second line of defense.
#[tokio::test(flavor = "current_thread")]
#[ignore = "launched by native_contract_under_watchdog"]
async fn native_child() {
    let scenario = std::env::var("STACKPRESS_SQL_PROFILE_SCENARIO").unwrap();
    let work = async {
        match scenario.as_str() {
            "parity" => cases::parity().await,
            "literals" => cases::literals().await,
            "startup" => cases::startup().await,
            "setup" => cases::setup().await,
            "failures" => cases::failures().await,
            _ => panic!("unknown native scenario"),
        }
    };
    tokio::time::timeout(Duration::from_secs(8), work)
        .await
        .expect("cooperative native deadline");
}

/// A known synchronously stuck process validates this exact parent's kill/reap path.
#[test]
fn watchdog_catches_synchronous_stall() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--exact", "stuck_child", "--ignored"]);
    let result = support::run(&mut command, Duration::from_millis(150)).unwrap();
    assert!(result.expired && !result.status.success());
}

/// Intentional negative control, never invoked without the external parent deadline.
#[test]
#[ignore = "watchdog negative control"]
fn stuck_child() {
    loop {
        std::thread::park();
    }
}
