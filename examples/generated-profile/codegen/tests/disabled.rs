//! Disabled codegen contains no profile transform and owns only its aggregate registry.

#![cfg(not(feature = "profiles"))]

use stackpress_example_profile_codegen::generate;

#[path = "support/cli.rs"]
mod cli;

use serde_json::{Value, json};
use stackpress_example_profile_codegen::build_tool;
use stackpress_server::ConfigLayers;

/// Empty selections generate a compiling registry shape and reject uncompiled plugin blocks.
#[test]
fn disabled_selection_has_no_profile_outputs() {
    // Use a real independent root; the checked-in selected app output remains untouched.
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, "").unwrap();
    generate(&input, &output, false).unwrap();
    assert!(generate(&input, &output, true).unwrap().is_current());
    assert!(!output.join("contracts").exists());
    assert!(!output.join("server").exists());

    // Schema configuration cannot activate a callback removed at compilation.
    std::fs::write(&input, "plugin \"profile-demo\" { }").unwrap();
    assert!(generate(&input, &output, false).is_err());
}

/// The disabled binary still runs the complete generation host and public diagnostic adapter.
#[tokio::test]
async fn disabled_cli_and_host_handle_check_install_and_invalid_sources() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, "").unwrap();
    let host = build_tool(ConfigLayers::new()).await.unwrap();

    // The empty registry is a legitimate install target, while a missing output is still stale.
    let report = host
        .call(
            "generate",
            json!({"input":input,"output":output,"check":true}),
        )
        .await
        .unwrap();
    assert_eq!(report.response.try_to_json().unwrap()["code"], 400);
    assert!(!output.exists());
    let installed = cli::command(&input, &output, &[]);
    assert!(installed.status.success());
    assert_eq!(
        serde_json::from_slice::<Value>(&installed.stdout).unwrap()["code"],
        200
    );
    for arguments in [&["-c"][..], &["--check", "true"][..]] {
        assert!(cli::command(&input, &output, arguments).status.success());
    }
    assert!(!output.join("contracts").exists());
    assert!(!output.join("server").exists());
    let original = std::fs::read(output.join(".stackpress-generated.json")).unwrap();

    // Uncompiled plugin IDs cannot activate code and keep internal causes off machine stdout.
    std::fs::write(&input, "plugin \"profile-demo\" { }").unwrap();
    let uncompiled = cli::command(&input, &output, &[]);
    assert_eq!(uncompiled.status.code(), Some(1));
    let envelope: Value = serde_json::from_slice(&uncompiled.stdout).unwrap();
    assert_eq!(envelope["code"], 500);
    assert_eq!(envelope["error"], "Internal Server Error");

    // Parser diagnostics stay public even when every optional generator is compiled out.
    std::fs::write(&input, "model Profile {").unwrap();
    let malformed = cli::command(&input, &output, &[]);
    assert_eq!(malformed.status.code(), Some(1));
    let envelope: Value = serde_json::from_slice(&malformed.stdout).unwrap();
    assert_eq!(envelope["code"], 400);
    assert!(
        envelope["error"]
            .as_str()
            .unwrap()
            .contains("schema.idea:1:16:")
    );
    assert_eq!(
        original,
        std::fs::read(output.join(".stackpress-generated.json")).unwrap()
    );
}
