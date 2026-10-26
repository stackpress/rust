//! Real setup and main_with binaries use native argv, isolated files and bounded child lifetimes.

mod support;

use std::ffi::OsStr;
use std::process::Command;
use std::time::Duration;

use serde_json::json;
use stackpress_cli::{CliSinks, Terminal};
use stackpress_lib::ParseLimits;
use stackpress_server::{ConfigLayers, Server};
use tempfile::TempDir;

/// Spawn one Cargo-built executable without a shell, retaining exact stdout/stderr framing.
fn run(binary: &str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> support::Completion {
    let mut command = Command::new(binary);
    command.args(args);
    let result = support::run(&mut command, Duration::from_secs(10)).unwrap();
    assert!(
        !result.expired,
        "executable deadline: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    result
}

/// Capture the real original app's CLI envelope using the same argument parser/runner.
async fn reference(app: &Server, args: &[&str]) -> Vec<u8> {
    let args = args.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let terminal = Terminal::parse(&args, ParseLimits::default()).unwrap();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    stackpress_cli::run(app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
        .await
        .unwrap();
    stdout
}

/// Setup is separate; the actual main_with executable returns the reference's complete wire bytes.
#[tokio::test(flavor = "current_thread")]
async fn setup_and_cli_are_explicit_and_match_reference_bytes() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("profiles with spaces.sqlite");
    let setup = run(env!("CARGO_BIN_EXE_sql-profile-setup"), [&path]);
    assert!(setup.status.success());
    assert!(setup.stdout.is_empty() && setup.stderr.is_empty());
    let config = root.path().join("application with spaces.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&json!({"sqlite":{"path":path},"profiles":{"limit":2}})).unwrap(),
    )
    .unwrap();
    let mut layers = ConfigLayers::new();
    layers
        .value("test", json!({"profiles":{"limit":2}}))
        .unwrap();
    let original = stackpress_example_capability_host::build_application(layers)
        .await
        .unwrap();
    for id in ["abc", "abc4", "%", "_", "\\", "'", "λ", "ABC"] {
        let result = run(
            env!("CARGO_BIN_EXE_sql-profile"),
            [
                OsStr::new("-b"),
                config.as_os_str(),
                OsStr::new("emit"),
                OsStr::new("profile-search"),
                OsStr::new("--id"),
                OsStr::new(id),
            ],
        );
        assert_eq!(
            result.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(result.stderr.is_empty());
        assert_eq!(
            result.stdout,
            reference(&original, &["emit", "profile-search", "--id", id]).await
        );
    }
    for args in [
        vec!["emit", "profile-search"],
        vec!["emit", "profile-search", "--id", ""],
    ] {
        let native_args = [OsStr::new("-b"), config.as_os_str()]
            .into_iter()
            .chain(args.iter().map(OsStr::new));
        let rejected = run(env!("CARGO_BIN_EXE_sql-profile"), native_args);
        assert_eq!(rejected.status.code(), Some(1));
        assert_eq!(rejected.stdout, reference(&original, &args).await);
        assert!(!rejected.stderr.is_empty());
    }
    let unknown = run(
        env!("CARGO_BIN_EXE_sql-profile"),
        [
            OsStr::new("-b"),
            config.as_os_str(),
            OsStr::new("emit"),
            OsStr::new("unknown-event"),
        ],
    );
    assert_eq!(unknown.status.code(), Some(1));
    assert_eq!(
        unknown.stdout,
        reference(&original, &["emit", "unknown-event"]).await
    );
    assert!(!unknown.stderr.is_empty());
    // Refuse a second setup, then prove normal startup/search still sees the unchanged seed.
    let replay = run(env!("CARGO_BIN_EXE_sql-profile-setup"), [&path]);
    assert_eq!(replay.status.code(), Some(1));
    assert!(replay.stdout.is_empty() && !replay.stderr.is_empty());
    let result = run(
        env!("CARGO_BIN_EXE_sql-profile"),
        [
            OsStr::new("-b"),
            config.as_os_str(),
            OsStr::new("emit"),
            OsStr::new("profile-search"),
            OsStr::new("--id"),
            OsStr::new("abc"),
        ],
    );
    assert_eq!(
        result.stdout,
        reference(&original, &["emit", "profile-search", "--id", "abc"]).await
    );
}

/// Usage and startup failures have no machine envelope and cannot silently create a database.
#[test]
fn invalid_executable_input_and_missing_install_fail_without_effects() {
    let root = TempDir::new().unwrap();
    let unused = root.path().join("unused.sqlite");
    for args in [
        Vec::new(),
        vec![unused.as_os_str(), OsStr::new("unexpected")],
    ] {
        let result = run(env!("CARGO_BIN_EXE_sql-profile-setup"), args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty() && !result.stderr.is_empty());
        assert!(!unused.exists());
    }
    for args in [vec![], vec!["emit"]] {
        let result = run(env!("CARGO_BIN_EXE_sql-profile"), args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty() && !result.stderr.is_empty());
    }
    let missing = root.path().join("not-installed.sqlite");
    let config = root.path().join("application.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&json!({"sqlite":{"path":missing}})).unwrap(),
    )
    .unwrap();
    let result = run(
        env!("CARGO_BIN_EXE_sql-profile"),
        [
            OsStr::new("-b"),
            config.as_os_str(),
            OsStr::new("emit"),
            OsStr::new("profile-search"),
            OsStr::new("--id"),
            OsStr::new("abc"),
        ],
    );
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(!missing.exists());
    assert!(String::from_utf8_lossy(&result.stderr).contains("application startup failed"));
}
