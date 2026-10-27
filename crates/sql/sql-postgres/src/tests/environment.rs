//! Isolated environment regression cases never mutate the parallel test process.

use crate::PostgresOptions;
use stackpress_sql::ErrorKind;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Select one child assertion without changing the parent's process environment.
const CASE: &str = "STACKPRESS_SQL_POSTGRES_AMBIENT_CASE";
/// Every ambient field which cannot be cleared through SQLx's public options API.
const REJECTED: [&str; 4] = ["PGSSLROOTCERT", "PGSSLCERT", "PGSSLKEY", "PGOPTIONS"];
/// Synthetic private data must never enter a public rejection message.
const PRIVATE_VALUE: &str = "ambient-value-must-not-be-disclosed";

#[test]
/// Every actual TLS/startup variable is rejected, including an explicitly empty value.
fn ambient_configuration_is_rejected_in_isolated_processes() {
    // A child runs only this exact test, so no other test can observe its environment.
    if let Some(name) = std::env::var_os(CASE) {
        let name = name.to_str().expect("child case name is ASCII");
        assert!(
            REJECTED.contains(&name),
            "unknown isolated environment case"
        );
        assert_rejection(name);
        return;
    }

    // Run every variable before failing, retaining only names and empty-value flags.
    let mut failed = Vec::new();
    for name in REJECTED {
        for value in [PRIVATE_VALUE, ""] {
            if !isolated_case(name, value) {
                failed.push((name, value.is_empty()));
            }
        }
    }
    assert!(
        failed.is_empty(),
        "ambient rejection child failures (variable, empty): {failed:?}"
    );
}

/// Construct options without native IO and check classification plus public disclosure.
fn assert_rejection(name: &str) {
    let options = PostgresOptions::new("db.invalid", "application", "application", "");
    // Avoid expect_err: its unexpected-success panic would format secret-bearing options.
    let error = match options.connect_options() {
        Err(error) => error,
        Ok(_) => panic!("ambient configuration must fail before driver construction"),
    };
    assert_eq!(error.kind(), ErrorKind::Configuration);

    // The public context identifies the remedy without copying any variable value.
    let public = format!("{error} {error:?}");
    assert!(
        public.contains(name),
        "rejection must identify its variable"
    );
    assert!(
        !public.contains(PRIVATE_VALUE),
        "rejection must hide the ambient value"
    );
}

/// Spawn one sterile child and kill/reap it if its bounded assertion cannot complete.
fn isolated_case(name: &str, value: &str) -> bool {
    let mut command = Command::new(std::env::current_exe().expect("current test executable"));
    command
        .args([
            "--exact",
            "tests::environment::ambient_configuration_is_rejected_in_isolated_processes",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // Remove both actual variables and the old misspellings before selecting one case.
    for ambient in REJECTED
        .into_iter()
        .chain(["PGPgROOTCERT", "PGPgCERT", "PGPgKEY"])
    {
        command.env_remove(ambient);
    }
    command.env(CASE, name).env(name, value);
    let mut child = command.spawn().expect("spawn isolated environment test");
    let started = Instant::now();
    loop {
        match child.try_wait() {
            // try_wait reaps the completed child and reports only its exit status.
            Ok(Some(status)) => return status.success(),
            Ok(None) if started.elapsed() < Duration::from_secs(5) => {
                std::thread::sleep(Duration::from_millis(10));
            }
            // Neither a timeout nor an observation failure may leave a running child.
            Ok(None) | Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                panic!("isolated environment test timed out or could not be observed");
            }
        }
    }
}
