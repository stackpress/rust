//! Real adapter child processes with finite watchdogs; no listeners or mock handler.
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{Read, Write},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Kill and reap on assertion, timeout and ordinary completion.
struct ChildGuard(Child);
impl Drop for ChildGuard {
    /// A failed proof cannot leave a child behind.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
/// Run a finite real request with host-side fixture policy explicitly selected.
fn request(input: Value, policy: bool) -> (Value, String) {
    let directory = std::path::Path::new("/private/tmp/s8-lane-b");
    std::fs::create_dir_all(directory).unwrap();
    // Parallel tests may request the same path; a per-call sequence keeps their logs apart.
    static SEQUENCE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let call = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let file = directory.join(format!(
        "finite-{}-{}-{}.log",
        std::process::id(),
        call,
        input["path"].as_str().unwrap().replace('/', "_")
    ));
    let log = File::create(&file).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_stackpress-example-ui-preferences-web"));
    command
        .env("PREFERENCES_BASE", "/demo")
        .env_remove("PREFERENCES_TEST_POLICY");
    if policy {
        command.env("PREFERENCES_TEST_POLICY", "explicit-fixture");
    }
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let mut child = ChildGuard(child);
    child
        .0
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "{}", file.display());
            break;
        }
        assert!(
            Instant::now() < deadline,
            "child timed out: {}",
            file.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut text = String::new();
    File::open(file)
        .unwrap()
        .take(128 * 1024)
        .read_to_string(&mut text)
        .unwrap();
    let (meta, body) = text.split_once('\n').unwrap();
    let meta: Value = serde_json::from_str(meta).unwrap();
    assert_eq!(meta["headers"]["x-preferences-lifecycle"][0], "1/1");
    assert_eq!(meta["headers"]["cache-control"][0], "no-store");
    (meta, body.into())
}
/// Selection and host-first colors work on legacy, canonical and alias routes.
#[test]
fn host_selected_pages_and_route_denials() {
    for (path, cookie, locale, theme) in [
        ("/demo/preferences", "", "en", "system"),
        (
            "/demo/preferences",
            "stackpress-locale=fr; stackpress-theme=dark",
            "fr",
            "dark",
        ),
        (
            "/demo/en/preferences",
            "stackpress-locale=fr",
            "en",
            "system",
        ),
        ("/demo/fr-FR/preferences?q=%252F", "", "fr", "system"),
        (
            "/demo/en_US/preferences",
            "stackpress-theme=light",
            "en",
            "light",
        ),
    ] {
        let (meta, html) = request(json!({"method":"GET","path":path,"cookie":cookie}), false);
        assert_eq!(meta["status"], 200);
        assert!(html.contains(&format!("<html lang=\"{locale}\" data-theme=\"{theme}\"")));
        assert!(html.contains("id=\"draft\""));
        assert!(html.contains("aria-disabled=\"true\""));
        assert!(!html.contains("fixture-csrf"));
    }
    for path in ["/demo/de/preferences", "/demo/en/fr/preferences"] {
        let (meta, _) = request(json!({"method":"GET","path":path}), false);
        assert_eq!(meta["status"], 404);
    }
    let (_, html) = request(
        json!({"method":"GET","path":"/demo/preferences","csr":true}),
        false,
    );
    assert!(html.contains("<div id=\"app\"></div>"));
}
/// Malformed/cross-origin updates issue no cookies; valid responses keep private fields out.
#[test]
fn persistence_and_cookie_scope() {
    let body = json!({"locale":"fr","theme":"dark","location":"/demo/en/preferences?q=a%2Fb&return=%252F"}).to_string();
    let (meta, text) = request(
        json!({"method":"POST","path":"/demo/ui-preferences","origin":"https://preferences.invalid","body":body}),
        false,
    );
    assert_eq!(meta["status"], 200);
    assert_eq!(meta["headers"]["set-cookie"].as_array().unwrap().len(), 2);
    assert!(!text.contains("cookie"));
    assert!(text.contains("/demo/fr/preferences?q=a%2Fb&return=%252F"));
    for (origin, body, code) in [
        (Value::Null, body.clone(), 403),
        (json!("https://evil.invalid"), body, 403),
        (json!("https://preferences.invalid"), "{}".into(), 400),
        (
            json!("https://preferences.invalid"),
            json!({"extra":"x".repeat(1025)}).to_string(),
            413,
        ),
    ] {
        let (meta, _) = request(
            json!({"method":"POST","path":"/demo/ui-preferences","origin":origin,"body":body}),
            false,
        );
        assert_eq!(meta["status"], code);
        assert!(meta["headers"].get("set-cookie").is_none());
    }
    let (meta, html) = request(
        json!({"method":"GET","path":"/demo/preferences","cookie":"stackpress-flash=bad"}),
        false,
    );
    assert!(
        meta["headers"]["set-cookie"][0]
            .as_str()
            .unwrap()
            .contains("Path=/demo; Max-Age=0; HttpOnly; SameSite=Lax; Secure")
    );
    assert!(!html.contains("id=\"flash\""));
}
/// Absent policy, denied callers and forged mutation metadata never call the handler.
#[test]
fn protected_exposure_fails_closed() {
    for (policy, caller, method, origin, csrf, status, calls) in [
        (false, "allowed", "GET", Value::Null, "", 403, "0"),
        (true, "denied", "GET", Value::Null, "", 403, "0"),
        (true, "allowed", "GET", Value::Null, "", 200, "1"),
        (
            true,
            "allowed",
            "POST",
            Value::Null,
            "fixture-csrf",
            403,
            "0",
        ),
        (
            true,
            "allowed",
            "POST",
            json!("https://evil.invalid"),
            "fixture-csrf",
            403,
            "0",
        ),
        (
            true,
            "allowed",
            "POST",
            json!("https://preferences.invalid"),
            "wrong",
            403,
            "0",
        ),
        (
            true,
            "allowed",
            "POST",
            json!("https://preferences.invalid"),
            "fixture-csrf",
            200,
            "1",
        ),
    ] {
        let (meta, _) = request(
            json!({"method":method,"path":"/private","caller":caller,"origin":origin,"csrf":csrf}),
            policy,
        );
        assert_eq!(meta["status"], status);
        assert_eq!(meta["headers"]["x-protected-calls"][0], calls);
    }
}
