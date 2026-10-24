//! Real codegen host, output/check/removal and source-error integration.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use stackpress_example_profile_codegen::{build_tool, generate};
use stackpress_server::ConfigLayers;

/// Use a physical temporary directory so output never follows macOS temporary aliases.
fn temporary() -> tempfile::TempDir {
    tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap()
}

/// Source used by the public model author, with no generated-type surrogate in the test.
fn source() -> &'static str {
    "plugin \"profile-demo\" { } model Profile { id String name String }"
}

/// Plan/install/repeat/check/removal execute the actual registered example generator.
#[test]
fn generation_is_stable_and_removal_cleans_owned_files() {
    let temp = temporary();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, source()).unwrap();
    assert!(!generate(&input, &output, true).unwrap().is_current());
    generate(&input, &output, false).unwrap();
    let manifest = std::fs::read(output.join(".stackpress-generated.json")).unwrap();
    assert!(generate(&input, &output, false).unwrap().is_current());
    assert_eq!(
        manifest,
        std::fs::read(output.join(".stackpress-generated.json")).unwrap()
    );
    assert!(generate(&input, &output, true).unwrap().is_current());
    // Unsupported shapes fail before replacing a usable generated tree.
    for invalid in [
        "model Customer { id String }",
        "model Profile { id Number name String }",
    ] {
        std::fs::write(&input, invalid).unwrap();
        assert!(generate(&input, &output, false).is_err());
        assert_eq!(
            manifest,
            std::fs::read(output.join(".stackpress-generated.json")).unwrap()
        );
    }
    std::fs::write(&input, "plugin \"profile-demo\" { }").unwrap();
    generate(&input, &output, false).unwrap();
    assert!(!output.join("contracts/profile.rs").exists());
    assert!(!output.join("server/profile.rs").exists());
    assert!(
        !std::fs::read_to_string(output.join("mod.rs"))
            .unwrap()
            .contains("pub mod server")
    );
}

/// The real CLI/server adapter passes target paths/check mode and returns source diagnostics.
#[tokio::test]
async fn generation_host_handles_success_check_and_source_failure() {
    let temp = temporary();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, source()).unwrap();
    let server = build_tool(ConfigLayers::new()).await.unwrap();
    let data = |check| json!({"input":input,"output":output,"check":check});
    assert_eq!(
        server
            .call("generate", data(true))
            .await
            .unwrap()
            .response
            .try_to_json()
            .unwrap()["code"],
        400
    );
    assert_eq!(
        server
            .call("generate", data(false))
            .await
            .unwrap()
            .response
            .try_to_json()
            .unwrap()["code"],
        200
    );
    assert_eq!(
        server
            .call("generate", data(true))
            .await
            .unwrap()
            .response
            .try_to_json()
            .unwrap()["code"],
        200
    );
    std::fs::write(&input, "model X {").unwrap();
    let report = server.call("generate", data(false)).await.unwrap();
    assert_eq!(report.response.try_to_json().unwrap()["code"], 500);
    let error = generate(&input, &output, false).unwrap_err();
    assert!(error.to_string().contains("schema.idea:1:"));
}

/// Native CLI commands are bounded by a kill-and-reap deadline and preserve OS argument boundaries.
fn command(input: &Path, output: &Path) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_profile-codegen"))
        .args(["emit", "generate", "--input"])
        .arg(input)
        .arg("--output")
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("codegen CLI timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// The app-owned binary routes through main_with rather than a private generator command parser.
#[test]
fn native_codegen_command_has_one_public_envelope() {
    let temp = temporary();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, source()).unwrap();
    let result = command(&input, &output);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["code"],
        200
    );
    assert!(output.join("contracts/profile.rs").exists());
}
