//! Real executable argv, exit codes and public output under a kill-and-reap watchdog.

use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

use serde_json::{Value as Json, json};
use stackpress_example_capability_host::build_application;
use stackpress_lib::{Error, Result};
use stackpress_server::ConfigLayers;
use stackpress_server::http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput};

#[path = "support/cli_watchdog.rs"]
mod watchdog;

/// Run the actual Cargo-built executable with native argument boundaries and a finite deadline.
fn run_cli(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<watchdog::Completion> {
    // No shell, PATH lookup, argv joining or inherited stdin can alter the proof inputs.
    let mut command = Command::new(env!("CARGO_BIN_EXE_stackpress"));
    command.args(args);
    let completion = watchdog::run(&mut command, Duration::from_secs(10))?;
    assert!(
        !completion.timed_out,
        "stackpress exceeded its ten-second deadline"
    );
    Ok(completion)
}

/// Require exactly one JSON object followed by one newline on machine stdout.
fn envelope(completion: &watchdog::Completion, exit_code: i32) -> Result<Json> {
    // A public application error still emits one envelope; usage errors are checked separately.
    assert_eq!(
        completion.status.code(),
        Some(exit_code),
        "stderr: {}",
        String::from_utf8_lossy(&completion.stderr)
    );
    assert_eq!(completion.stdout.last(), Some(&b'\n'));
    assert_eq!(
        completion
            .stdout
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count(),
        1
    );
    let value: Json = serde_json::from_slice(&completion.stdout)?;
    assert!(value.is_object());
    Ok(value)
}

/// One compiled capability has identical public envelopes through the CLI, HTTP and direct call.
#[tokio::test]
async fn binary_http_and_call_return_the_same_public_envelope() -> Result<()> {
    let completion = run_cli(["emit", "profile-search", "--id", "abc123"])?;
    let cli = envelope(&completion, 0)?;
    assert!(completion.stderr.is_empty());

    // Build the same profiles selection in process and compare the complete public values.
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    let called = app.call("profile-search", json!({"id": "abc123"})).await?;
    let mut request = BufferedRequest::new(Vec::new());
    *request.uri_mut() = "/profiles/abc123".parse().map_err(Error::from_source)?;
    let reply = HttpAdapter::new(HttpOptions::default())?
        .handle(&app, request)
        .await?;
    let HttpOutput::Response(response) = reply.output else {
        panic!("HTTP parity requires native output");
    };
    assert_eq!(response.status().as_u16(), 200);
    let http: Json = serde_json::from_slice(response.body())?;
    assert_eq!(cli, called.response.try_to_json()?);
    assert_eq!(cli, http);
    assert_eq!(cli["code"], 200);
    assert_eq!(cli["total"], 1);
    assert_eq!(
        cli["results"],
        json!([{"id": "abc123", "name": "Ada Lovelace"}])
    );
    Ok(())
}

/// Unknown events exit one while retaining an ordinary public error envelope on stdout.
#[test]
fn unknown_event_exits_one_with_a_public_envelope() -> Result<()> {
    let completion = run_cli(["emit", "unknown-event"])?;
    let value = envelope(&completion, 1)?;
    assert_eq!(value["code"], 404);
    assert!(value["error"].is_string());
    assert!(value.get("results").is_none());
    assert!(!completion.stderr.is_empty());
    Ok(())
}

/// Usage failures do not build/run a capability or write anything to machine stdout.
#[test]
fn usage_errors_exit_two_without_stdout() -> Result<()> {
    // Missing command, missing event and an unknown operator option fail before application entry.
    for args in [
        vec![],
        vec!["emit"],
        vec!["--unknown", "emit", "profile-search"],
    ] {
        let completion = run_cli(args)?;
        assert_eq!(completion.status.code(), Some(2));
        assert!(completion.stdout.is_empty());
        assert!(!completion.stderr.is_empty());
    }
    Ok(())
}

/// A pre-emit bootstrap file changes service policy through the named application builder.
#[test]
fn bootstrap_before_emit_changes_the_result_limit() -> Result<()> {
    let fixture = watchdog::Fixture::new()?;
    let path = fixture.path("profile limits.json");
    fs::write(&path, br#"{"profiles":{"limit":1}}"#)?;

    // Spaces remain inside one native path argument; default policy returns all three seed rows.
    let default = run_cli(["emit", "profile-search", "--id", "abc"])?;
    assert_eq!(envelope(&default, 0)?["total"], 3);
    let configured = run_cli([
        OsStr::new("-b"),
        path.as_os_str(),
        OsStr::new("emit"),
        OsStr::new("profile-search"),
        OsStr::new("--id"),
        OsStr::new("abc"),
    ])?;
    let value = envelope(&configured, 0)?;
    assert_eq!(value["total"], 1);
    assert_eq!(
        value["results"],
        json!([{"id": "abc123", "name": "Ada Lovelace"}])
    );
    assert!(configured.stderr.is_empty());
    Ok(())
}

/// File syntax and application validation failures are startup errors, not public operation output.
#[test]
fn invalid_bootstrap_exits_two_without_stdout() -> Result<()> {
    let fixture = watchdog::Fixture::new()?;
    let path = fixture.path("invalid config.json");
    // The first failure precedes build; the second reaches the real profiles config validator.
    for text in ["{malformed", r#"{"profiles":{"limit":0}}"#] {
        fs::write(&path, text)?;
        let completion = run_cli([
            OsStr::new("-b"),
            path.as_os_str(),
            OsStr::new("emit"),
            OsStr::new("profile-search"),
            OsStr::new("--id"),
            OsStr::new("abc"),
        ])?;
        assert_eq!(completion.status.code(), Some(2));
        assert!(completion.stdout.is_empty());
        assert!(!completion.stderr.is_empty());
    }
    Ok(())
}

/// Post-event bootstrap tokens remain capability data and cannot load an operator config file.
#[test]
fn bootstrap_after_event_remains_capability_data() -> Result<()> {
    let fixture = watchdog::Fixture::new()?;
    let path = fixture.path("target bootstrap.json");
    fs::write(&path, br#"{"profiles":{"limit":1}}"#)?;

    // Stealing -b would load limit one or fail on the absent file; neither may affect execution.
    for target in [path, fixture.path("absent config.json")] {
        let completion = run_cli([
            OsStr::new("emit"),
            OsStr::new("profile-search"),
            OsStr::new("--id"),
            OsStr::new("abc"),
            OsStr::new("-b"),
            target.as_os_str(),
        ])?;
        let value = envelope(&completion, 0)?;
        assert_eq!(value["total"], 3);
        assert_eq!(value["results"].as_array().unwrap().len(), 3);
        assert!(completion.stderr.is_empty());
    }
    Ok(())
}

/// The real executable selects the optional preview plugin when its source is compiled.
#[cfg(feature = "experimental")]
#[test]
fn binary_selects_the_enabled_experimental_plugin() -> Result<()> {
    let preview = run_cli(["emit", "profile-preview", "--id", "abc123"])?;
    let search = run_cli(["emit", "profile-search", "--id", "abc123"])?;
    assert_eq!(envelope(&preview, 0)?, envelope(&search, 0)?);
    assert!(preview.stderr.is_empty());
    Ok(())
}

/// The real executable cannot invoke preview when the experimental implementation is excluded.
#[cfg(not(feature = "experimental"))]
#[test]
fn binary_excludes_the_disabled_experimental_plugin() -> Result<()> {
    let completion = run_cli(["emit", "profile-preview", "--id", "abc123"])?;
    assert_eq!(envelope(&completion, 1)?["code"], 404);
    Ok(())
}

/// Deliberately block only when selected by the watchdog's exact ignored-child marker.
#[test]
#[ignore = "launched only by the watchdog self-test"]
fn watchdog_blocked_child() {
    // A synchronously stuck child proves the parent deadline does not depend on Tokio progress.
    loop {
        std::thread::park();
    }
}

/// A known blocked child must be killed and reaped promptly by this exact capture/watchdog path.
#[test]
fn watchdog_kills_and_reaps_a_blocked_child() -> Result<()> {
    let mut command = Command::new(std::env::current_exe()?);
    command.args(["--ignored", "--exact", "watchdog_blocked_child"]);
    let completion = watchdog::run(&mut command, Duration::from_millis(250))?;
    assert!(completion.timed_out);
    assert!(!completion.status.success());
    Ok(())
}

/// Overrun the capture ceiling before blocking, only when explicitly selected by its parent.
#[test]
#[ignore = "launched only by the capture-limit self-test"]
fn watchdog_noisy_child() {
    // Flush more than the 64 KiB policy before blocking so the parent must take its error path.
    let mut stdout = std::io::stdout();
    stdout.write_all(&vec![b'x'; 128 * 1024]).unwrap();
    stdout.flush().unwrap();
    loop {
        std::thread::park();
    }
}

/// A capture-limit early return must still kill/reap its blocked child through the owner guard.
#[test]
fn watchdog_rejects_excess_output_and_cleans_up() -> Result<()> {
    let mut command = Command::new(std::env::current_exe()?);
    command.args(["--ignored", "--exact", "watchdog_noisy_child"]);
    let error = watchdog::run(&mut command, Duration::from_secs(10))
        .err()
        .expect("the child must exceed its output allowance");
    assert!(error.to_string().contains("capture limit"));
    Ok(())
}
