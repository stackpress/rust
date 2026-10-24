//! Real codegen host, output/check/removal and source-error integration.

#![cfg(feature = "profiles")]

use std::error::Error as StdError;
use std::path::Path;

#[path = "support/cli.rs"]
mod cli;

use cli::command;
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
        "model Profile { id String }",
        "model Profile { id String name String extra String }",
        "model Profile { id String name String } model Other { id String }",
        "model Profile { name String id String }",
        "model Profile { id String? name String }",
        "model Profile { id String[] name String }",
        "model Profile @unsupported { id String name String }",
        "model Profile { id String @unsupported name String }",
        "plugin \"profile-demo\" { secret \"not-a-setting\" }",
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
    assert_eq!(report.response.try_to_json().unwrap()["code"], 400);
    let error = generate(&input, &output, false).unwrap_err();
    assert!(
        error
            .source()
            .unwrap()
            .to_string()
            .contains("schema.idea:1:")
    );
}

/// The app-owned binary routes through main_with rather than a private generator command parser.
#[test]
fn native_codegen_command_has_one_public_envelope() {
    let temp = temporary();
    let input = temp.path().join("schema.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, source()).unwrap();
    let result = command(&input, &output, &[]);
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

    // Both advertised freshness spellings use capability data and preserve CLI grammar.
    for arguments in [&["-c"][..], &["--check", "true"][..]] {
        assert!(command(&input, &output, arguments).status.success());
    }
    std::fs::write(&input, "").unwrap();
    let stale = command(&input, &output, &["-c"]);
    assert_eq!(stale.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<Value>(&stale.stdout).unwrap()["code"],
        400
    );
    assert!(output.join("contracts/profile.rs").exists());

    // A present false takes precedence over -c; this request therefore installs model removal.
    assert!(
        command(&input, &output, &["--check", "false", "-c"])
            .status
            .success()
    );
    assert!(!output.join("contracts/profile.rs").exists());
}

/// Ordinary machine output exposes typed source coordinates but never the bad source body.
#[test]
fn malformed_input_has_coordinates_exit_one_and_no_source_excerpt() {
    let temp = temporary();
    let input = temp.path().join("broken.idea");
    let output = temp.path().join("generated");
    // EOF after a multibyte comment proves line/column mapping across the full CLI boundary.
    std::fs::write(&input, "// private-sentinel 😀\nmodel Profile {").unwrap();
    let result = command(&input, &output, &[]);
    assert_eq!(result.status.code(), Some(1));
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["code"], 400);
    assert!(
        envelope["error"]
            .as_str()
            .unwrap()
            .contains("broken.idea:2:16:")
    );
    for bytes in [&result.stdout, &result.stderr] {
        assert!(!String::from_utf8_lossy(bytes).contains("private-sentinel"));
    }
    assert!(!output.exists());

    // OS failures keep the existing sanitized internal-error envelope and a failing exit.
    let missing = command(&temp.path().join("missing.idea"), &output, &[]);
    assert_eq!(missing.status.code(), Some(1));
    let envelope: Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(envelope["code"], 500);
    assert_eq!(envelope["error"], "Internal Server Error");
}

/// Read exactly the installed files so a refactor from inline schema to imports proves byte parity.
fn snapshot(root: &Path) -> std::collections::BTreeMap<std::path::PathBuf, Vec<u8>> {
    let mut files = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            // Retain relative nested names, not host-specific absolute temporary paths.
            for (path, bytes) in snapshot(&entry.path()) {
                files.insert(
                    std::path::PathBuf::from(entry.file_name()).join(path),
                    bytes,
                );
            }
        } else {
            files.insert(
                entry.file_name().into(),
                std::fs::read(entry.path()).unwrap(),
            );
        }
    }
    files
}

/// The actual codegen CLI composes a sibling file and keeps generated Rust/manifest byte-identical.
#[test]
fn sibling_schema_import_is_stable_and_reports_child_coordinates() {
    let temp = temporary();
    let input = temp.path().join("schema.idea");
    let child = temp.path().join("types.idea");
    let output = temp.path().join("generated");
    std::fs::write(&input, source()).unwrap();
    assert!(command(&input, &output, &[]).status.success());
    let original = snapshot(&output);

    // Keep the checked-in first example small; this disposable fixture proves the import path.
    std::fs::write(&input, "plugin \"profile-demo\" { } use \"./types.idea\"").unwrap();
    std::fs::write(&child, "model Profile { id String name String }").unwrap();
    assert!(command(&input, &output, &[]).status.success());
    assert_eq!(original, snapshot(&output));
    assert!(command(&input, &output, &["-c"]).status.success());

    // A child error keeps the child's coordinates, never a fabricated root-file location.
    std::fs::write(&child, "// child-private-sentinel 😀\nmodel Profile {").unwrap();
    let result = command(&input, &output, &[]);
    assert_eq!(result.status.code(), Some(1));
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(envelope["code"], 400);
    assert!(
        envelope["error"]
            .as_str()
            .unwrap()
            .contains("types.idea:2:16:")
    );
    assert!(!String::from_utf8_lossy(&result.stdout).contains("child-private-sentinel"));
    assert_eq!(original, snapshot(&output));
}

/// A malformed leaf at the default import-depth boundary must retain public coordinates.
#[test]
fn deepest_default_import_reports_the_leaf_diagnostic() {
    let temp = temporary();
    let output = temp.path().join("generated");
    let depth = stackpress_idea::transformer::CompositionLimits::default().max_depth;
    for index in 0..depth {
        let source = if index + 1 == depth {
            "model Profile {".to_owned()
        } else {
            format!("use \"{}.idea\"", index + 1)
        };
        std::fs::write(temp.path().join(format!("{index}.idea")), source).unwrap();
    }
    // Every import adds cause context; diagnosis must survive the entire allowed native chain.
    let result = command(&temp.path().join("0.idea"), &output, &[]);
    let envelope: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert_eq!(envelope["code"], 400);
    assert!(
        envelope["error"]
            .as_str()
            .unwrap()
            .contains(&format!("{}.idea:1:16:", depth - 1))
    );
    assert!(!output.exists());
}
