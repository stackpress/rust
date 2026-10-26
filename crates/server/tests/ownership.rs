//! Compiler-rejection proofs using Cargo-resolved production server/lib dependencies.

use std::process::Command;
use std::time::Duration;

#[path = "../../lib/tests/support/watchdog.rs"]
mod watchdog;

/// Preserve each service/startup Send, host lifetime, and unique-owner Clone diagnostic.
#[test]
#[ignore = "invoked by the compiler watchdog test"]
fn compiler_rejections_child() {
    // Cargo resolves server and lib together instead of independently guessing their artifacts.
    // Exact snapshots reject incidental missing-method or crate-version errors.
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}

/// Bound the complete compiler proof, including a cold Cargo build, by an external deadline.
#[test]
fn host_ownership_and_send_contracts_are_compiler_enforced() {
    // Run in this crate's directory so direct executable replays resolve the same fixture paths.
    // The external watchdog also bounds synchronous Cargo/rustc stalls.
    let mut command = Command::new(std::env::current_exe().expect("integration-test executable"));
    command.current_dir(env!("CARGO_MANIFEST_DIR")).args([
        "--ignored",
        "--exact",
        "compiler_rejections_child",
        "--nocapture",
    ]);
    let completion = watchdog::run(&mut command, "host-compile", Duration::from_secs(180))
        .expect("launch/reap compiler rejection proof");
    assert!(!completion.timed_out, "{}", completion.output);
    assert!(completion.status.success(), "{}", completion.output);
}
