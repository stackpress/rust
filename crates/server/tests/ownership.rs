//! Compiler-rejection proofs against this build's actual server/lib types.
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
#[path = "../../lib/tests/support/watchdog.rs"]
mod watchdog;

/// Locate the freshly built production rlib beside this exact test executable.
fn library(deps: &std::path::Path, prefix: &str) -> PathBuf {
    // Link only the newest production rlib beside this test executable, including coverage builds.
    std::fs::read_dir(deps)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().is_some_and(|n| {
                n.to_string_lossy().starts_with(prefix) && n.to_string_lossy().ends_with(".rlib")
            })
        })
        .max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok())
        .expect("production rlib")
}

/// Every fixture fails for its intended ownership/Send/lifetime constraint within ten seconds.
#[test]
fn host_ownership_and_send_contracts_are_compiler_enforced() {
    // Each fixture must fail for the intended type/lifetime restriction, not an unrelated compiler
    // error.
    let deps = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("../../target/m0");
    std::fs::create_dir_all(&output).unwrap();
    // Every negative fixture has expected semantic diagnostics to reject incidental compile errors.
    for (fixture, diagnostics) in [
        (
            "non_send_service.rs",
            vec!["Rc<()>", "cannot be sent", "cannot be shared"],
        ),
        ("escaping_host.rs", vec!["lifetime", "static"]),
        (
            "stateful_clone.rs",
            vec![
                "no method named `clone`",
                "ServerBuilder",
                "OutputGuard",
                "Server",
            ],
        ),
        (
            "non_send_startup.rs",
            vec!["cannot be sent", "Rc<()>", "on_config"],
        ),
    ] {
        // Link this exact build's production libraries; no alternate checkout supplies the API.
        let mut command = Command::new("rustc");
        command
            .arg("--extern")
            .arg(format!(
                "stackpress_lib={}",
                library(&deps, "libstackpress_lib-").display()
            ))
            .arg("--extern")
            .arg(format!(
                "stackpress_server={}",
                library(&deps, "libstackpress_server-").display()
            ))
            .arg("-L")
            .arg(format!("dependency={}", deps.display()))
            .args(["--edition=2024", "--crate-type=bin"])
            .arg(root.join("tests/ui").join(fixture))
            .arg("--out-dir")
            .arg(&output);
        // Compile under an independent process deadline so rejection proofs cannot hang the suite.
        let completion = watchdog::run(&mut command, fixture, Duration::from_secs(10)).unwrap();
        assert!(!completion.timed_out, "{}", completion.output);
        assert!(
            !completion.status.success(),
            "fixture unexpectedly compiled: {fixture}"
        );
        // Failure is meaningful only when every intended ownership/Send/lifetime restriction
        // appears in the compiler diagnostics.
        for diagnostic in diagnostics {
            assert!(
                completion.output.contains(diagnostic),
                "{fixture}: {}",
                completion.output
            );
        }
    }
}
