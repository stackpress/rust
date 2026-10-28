//! Real HTTP progress/lifecycle/cancellation matrix in a kill-and-reap child.
use stackpress_server_http::HttpDelivery;
use std::{
    process::{Child, Command, Stdio},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

/// Keep watchdog child diagnostics under the same configurable proof artifact directory.
fn proof_output(name: &str) -> std::path::PathBuf {
    let directory = std::env::var_os("UI_STREAM_OUTPUT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/private/tmp/s8-stream".into());
    std::fs::create_dir_all(&directory).unwrap();
    directory.join(name)
}

/// All hang-capable rendering and IO run within the externally bounded child.
#[test]
#[ignore = "launched by parent watchdog"]
fn child() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        for mode in [
            "success",
            "precommit",
            "late",
            "disconnect",
            "deadline",
            "shutdown",
        ] {
            for base in ["", "/demo"] {
                let start = Instant::now();
                let mut shell = None;
                let mut settled = None;
                let result = stackpress_example_ui_stream_web::exchange(
                    mode,
                    base,
                    |chunk| {
                        if chunk.contains("id=\"shell\"") {
                            shell = Some(start.elapsed());
                        }
                        if chunk.contains("id=\"details\"") {
                            settled = Some(start.elapsed());
                        }
                    },
                    std::future::pending(),
                )
                .await
                .unwrap();
                assert!(result.probe.released(), "{mode}: capture leaked");
                let text = result.chunks.join("");
                assert!(!text.contains("private demo"));
                if mode == "precommit" {
                    assert!(result.head.starts_with("HTTP/1.1 500"));
                    assert!(text.contains("Page unavailable"));
                    assert!(!text.contains("stackpress-page"));
                    assert_eq!(result.probe.errors.load(Ordering::SeqCst), 1);
                    assert_eq!(result.probe.hooks.load(Ordering::SeqCst), 1);
                } else {
                    assert!(
                        result.head.starts_with("HTTP/1.1 200"),
                        "{mode}: {}",
                        result.head
                    );
                    assert_eq!(result.probe.errors.load(Ordering::SeqCst), 0);
                    assert_eq!(result.probe.cleanups.load(Ordering::SeqCst), 1);
                    assert!(shell.is_some());
                    if mode == "success" {
                        assert!(result.terminal_chunk);
                        assert!(settled.unwrap() > shell.unwrap() + Duration::from_millis(500));
                        assert!(text.contains("stackpress-stream-complete"));
                        assert!(text.contains("data-stackpress-replay"));
                        assert!(text.contains("\\u003c"));
                        assert_eq!(
                            result.report.exchanges.last().unwrap().delivery,
                            HttpDelivery::Completed
                        );
                    } else {
                        assert!(
                            !result.terminal_chunk,
                            "{mode}: unexpected terminal HTTP chunk"
                        );
                        assert!(!text.contains("stackpress-stream-complete"));
                        assert!(text.contains("stackpress-stream-recovery"));
                        assert_ne!(
                            result.report.exchanges.last().unwrap().delivery,
                            HttpDelivery::Completed
                        );
                    }
                    assert_eq!(result.probe.hooks.load(Ordering::SeqCst), 1);
                }
                println!(
                    "PASS {mode} {base}: {:?}",
                    result.report.exchanges.last().map(|report| report.delivery)
                );
            }
        }
    });
    println!("S8_STREAM_HOST_COMPLETE");
}
/// The test executor cannot hide a render, join or native IO deadlock.
#[test]
fn watchdog() {
    struct Reap(Child);
    impl Drop for Reap {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let path = proof_output(&format!("host-child-{}.log", std::process::id()));
    let log = std::fs::File::create(&path).unwrap();
    let mut child = Reap(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "child", "--ignored", "--nocapture"])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            let output = std::fs::read_to_string(&path).unwrap();
            assert!(status.success(), "{output}");
            assert!(
                output.lines().any(|line| line == "S8_STREAM_HOST_COMPLETE"),
                "{output}"
            );
            break;
        }
        assert!(Instant::now() < deadline, "host child timed out: {path:?}");
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Exercise the actual finite CLI protocol used by the browser, with a directly
/// owned child, file-backed output and an independent wall-clock deadline.
#[test]
fn cli_protocol() {
    use std::process::Stdio;
    struct Reap(Child);
    impl Drop for Reap {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let path = proof_output(&format!("cli-protocol-{}.log", std::process::id()));
    let log = std::fs::File::create(&path).unwrap();
    let mut child = Reap(
        Command::new(env!("CARGO_BIN_EXE_stackpress-example-ui-stream-web"))
            .args(["success", "/demo"])
            .stdin(Stdio::piped())
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut report = None;
    loop {
        // ASCII line framing preserves U+2028/U+2029 inside valid JSON strings.
        let output = std::fs::read_to_string(&path).unwrap();
        for line in output.lines() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(line)
                && value["done"] == true
            {
                report = Some(value);
            }
        }
        if report.is_some() {
            drop(child.0.stdin.take());
        }
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "{output}");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "CLI protocol timed out: {path:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let report = report.expect("CLI must emit a final completion record");
    assert_eq!(report["released"], true);
    assert_eq!(report["hooks"], 1);
    assert_eq!(report["delivery"], "Some(Completed)");
    assert!(report["head"].as_str().unwrap().starts_with("HTTP/1.1 200"));
}
