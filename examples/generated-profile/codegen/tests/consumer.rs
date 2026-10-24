//! Real Cargo consumers prove bootstrap, contracts isolation and selected/disabled application DX.

#![cfg(feature = "profiles")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

/// Execute a finite child with file-backed logs, avoiding pipe saturation during Cargo builds.
fn run(command: &mut Command, success: bool) -> (String, String) {
    let stdout = tempfile::tempfile().unwrap();
    let stderr = tempfile::tempfile().unwrap();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(stdout.try_clone().unwrap())
        .stderr(stderr.try_clone().unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        // Cargo gets a longer bound than native CLI tests; timeout still kills and reaps it.
        if let Some(status) = child.try_wait().unwrap() {
            let out = read_log(stdout);
            let err = read_log(stderr);
            assert_eq!(status.success(), success, "{command:?}\n{out}\n{err}");
            return (out, err);
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "consumer command timed out: {command:?}\n{}",
                read_log(stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Read a shared log handle from the beginning after its child has finished writing.
fn read_log(mut file: fs::File) -> String {
    use std::io::{Read, Seek, SeekFrom};
    // File clones share their offset, so explicitly rewind before reading retained output.
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut text = String::new();
    file.read_to_string(&mut text).unwrap();
    text
}

/// Copy handwritten Rust only; the independent tool must bootstrap an absent generated root.
fn copy_source(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        // Generated files are intentionally absent, rather than copied then destructively removed.
        if entry.file_name() == "generated" {
            continue;
        }
        let output = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_source(&entry.path(), &output);
        } else {
            fs::copy(entry.path(), output).unwrap();
        }
    }
}

/// Turn an example package into an independent workspace using only current-checkout path edges.
fn copy_package(example: &Path, name: &str, root: &Path, repository: &Path) -> PathBuf {
    let destination = root.join(name);
    copy_source(&example.join(name).join("src"), &destination.join("src"));
    let mut manifest = fs::read_to_string(example.join(name).join("Cargo.toml")).unwrap();
    for (key, value) in [
        ("version", "0.1.0"),
        ("edition", "2024"),
        ("rust-version", "1.96"),
    ] {
        manifest = manifest.replace(
            &format!("\n{key}.workspace = true"),
            &format!("\n{key} = {value:?}"),
        );
    }
    // Runtime/dev dependency paths resolve to the same checkout, never another lane or branch.
    manifest = manifest
        .replace(
            "../../../crates/",
            &format!("{}/crates/", repository.display()),
        )
        .replace(
            "../../../tools/",
            &format!("{}/tools/", repository.display()),
        )
        .replace(
            "../../capability-host",
            &format!("{}/examples/capability-host", repository.display()),
        )
        .replace("[lints]\nworkspace = true", "[workspace]");
    fs::write(destination.join("Cargo.toml"), manifest).unwrap();
    destination
}

/// Create Cargo commands with a private target directory and no inherited coverage instrumentation.
fn cargo(root: &Path, package: &Path, operation: &str) -> Command {
    let mut command = Command::new(env!("CARGO"));
    // Nested builds are verification consumers, not coverage substitutes for app source.
    command
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("LLVM_PROFILE_FILE");
    command.env("CARGO_TARGET_DIR", root.join("target"));
    command
        .arg(operation)
        .arg("--offline")
        .arg("--manifest-path")
        .arg(package.join("Cargo.toml"));
    command
}

/// Native generated app commands retain the exact public exit policy and JSON envelope.
fn search(binary: &Path, code: u64) {
    let (stdout, _) = run(
        Command::new(binary).args(["emit", "profile-search", "--id", "abc123"]),
        code == 200,
    );
    let envelope: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(envelope["code"], code);
    if code == 200 {
        // This is an independently compiled application using the real generated contract.
        assert_eq!(envelope["results"][0]["name"], "Ada Lovelace");
    }
}

/// Bootstrap, freshness, mismatch rejection, model removal and disabled selection use real builds.
#[test]
fn independent_consumers_compile_and_remove_stale_runtime_outputs() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let temp = tempfile::tempdir_in(parent).unwrap();
    let root = temp.path();
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let repository = example.parent().unwrap().parent().unwrap();
    let app = copy_package(example, "app", root, repository);
    let tool = copy_package(example, "codegen", root, repository);
    fs::copy(example.join("selection.rs"), root.join("selection.rs")).unwrap();
    fs::copy(example.join("app/schema.idea"), app.join("schema.idea")).unwrap();
    assert!(!app.join("src/generated").exists());

    // Resolve each independent offline lock once, then require locked Cargo operations.
    for package in [&tool, &app] {
        run(&mut cargo(root, package, "generate-lockfile"), true);
    }
    run(cargo(root, &tool, "build").arg("--locked"), true);
    let binary = root.join("target/debug/profile-codegen");
    run(Command::new(&binary).args(["-v", "emit", "generate"]), true);
    let generated = app.join("src/generated");
    let manifest = fs::read(generated.join(".stackpress-generated.json")).unwrap();
    run(Command::new(&binary).args(["-v", "emit", "generate"]), true);
    assert_eq!(
        manifest,
        fs::read(generated.join(".stackpress-generated.json")).unwrap()
    );
    run(
        Command::new(&binary).args(["-v", "emit", "generate", "-c"]),
        true,
    );
    run(cargo(root, &app, "build").arg("--locked"), true);
    search(&root.join("target/debug/profile-app"), 200);

    // Compiled selection and shared contract revisions reject incompatible pre-generated code.
    let (_, errors) = run(
        cargo(root, &app, "check").args(["--locked", "--no-default-features"]),
        false,
    );
    assert!(errors.contains("assertion failed"));
    let selection = fs::read_to_string(root.join("selection.rs")).unwrap();
    fs::write(
        root.join("selection.rs"),
        selection.replace("REVISION: u32 = 1", "REVISION: u32 = 2"),
    )
    .unwrap();
    let (_, errors) = run(cargo(root, &app, "check").arg("--locked"), false);
    assert!(errors.contains("assertion failed"));
    fs::write(root.join("selection.rs"), selection).unwrap();

    // The public data partition compiles with Serde only, without any host or generation edge.
    let contracts = root.join("contracts");
    fs::create_dir_all(contracts.join("src")).unwrap();
    fs::copy(
        generated.join("contracts/profile.rs"),
        contracts.join("src/lib.rs"),
    )
    .unwrap();
    fs::write(
        contracts.join("Cargo.toml"),
        concat!(
            "[package]\nname = \"profile-contract-consumer\"\n",
            "version = \"0.0.0\"\nedition = \"2024\"\n",
            "[dependencies]\nserde = { version = \"1\", features = [\"derive\"] }\n",
            "[workspace]\n",
        ),
    )
    .unwrap();
    run(&mut cargo(root, &contracts, "generate-lockfile"), true);
    run(cargo(root, &contracts, "check").arg("--locked"), true);
    let (tree, _) = run(
        cargo(root, &contracts, "tree").args(["--locked", "--edges", "normal"]),
        true,
    );
    for forbidden in ["stackpress-", "tempfile", "sha2"] {
        assert!(!tree.contains(forbidden), "{tree}");
    }

    // Model removal first reports stale bytes, then removes both files and registrations.
    fs::write(app.join("schema.idea"), "plugin \"profile-demo\" { }\n").unwrap();
    run(
        Command::new(&binary).args(["-v", "emit", "generate", "-c"]),
        false,
    );
    assert!(generated.join("contracts/profile.rs").exists());
    run(Command::new(&binary).args(["-v", "emit", "generate"]), true);
    assert!(!generated.join("contracts/profile.rs").exists());
    assert!(!generated.join("server/profile.rs").exists());
    run(cargo(root, &app, "build").arg("--locked"), true);
    search(&root.join("target/debug/profile-app"), 404);
    fs::create_dir_all(app.join("tests")).unwrap();
    fs::write(
        app.join("tests/absence.rs"),
        include_str!("support/absence.rs"),
    )
    .unwrap();
    run(
        cargo(root, &app, "test").args(["--locked", "--test", "absence"]),
        true,
    );

    // Feature-disabled codegen can replace selected output with an aggregate-only registry.
    fs::copy(example.join("app/schema.idea"), app.join("schema.idea")).unwrap();
    run(Command::new(&binary).args(["-v", "emit", "generate"]), true);
    fs::copy(
        example.join("app/schema-disabled.idea"),
        app.join("schema.idea"),
    )
    .unwrap();
    run(
        cargo(root, &tool, "build").args(["--locked", "--no-default-features"]),
        true,
    );
    run(Command::new(&binary).args(["-v", "emit", "generate"]), true);
    assert!(!generated.join("contracts/profile.rs").exists());
    assert!(!generated.join("server/profile.rs").exists());
    run(
        cargo(root, &app, "build").args(["--locked", "--no-default-features"]),
        true,
    );
    search(&root.join("target/debug/profile-app"), 404);
    run(
        cargo(root, &app, "test").args(["--locked", "--test", "absence", "--no-default-features"]),
        true,
    );
}
