//! Finite actual HttpAdapter processes, with kill/reap deadlines and no listener.
use serde_json::{Value, json};
use std::{
    fs::File,
    io::{Read, Write},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Own a finite fixture process until it is reaped on every exit path.
struct ChildGuard(Child);
impl Drop for ChildGuard {
    /// A stalled fixture cannot outlive the parent assertion/watchdog.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run one real HTTP request and return its owned output envelope/body.
fn request(
    label: &str,
    method: &str,
    path: &str,
    cookie: &str,
    origin: Option<&str>,
    body: &str,
    csr: bool,
) -> (Value, String) {
    let directory = std::env::var_os("GALLERY_OUTPUT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/private/tmp/s8-m0".into());
    std::fs::create_dir_all(&directory).unwrap();
    let file = directory.join(format!("finite-{label}-{}.log", std::process::id()));
    let log = File::create(&file).unwrap();
    let child = Command::new(env!("CARGO_BIN_EXE_stackpress-example-ui-gallery-web"))
        .env("GALLERY_BASE", "/demo")
        .stdin(Stdio::piped())
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let mut child = ChildGuard(child);
    let input =
        json!({"method":method,"path":path,"cookie":cookie,"origin":origin,"body":body,"csr":csr})
            .to_string();
    child
        .0
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "{}", file.display());
            break;
        }
        assert!(
            Instant::now() < deadline,
            "fixture timed out: {}",
            file.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut output = String::new();
    File::open(file)
        .unwrap()
        .take(4 * 1024 * 1024)
        .read_to_string(&mut output)
        .unwrap();
    let (meta, body) = output.split_once('\n').unwrap();
    let meta: Value = serde_json::from_str(meta).unwrap();
    assert_eq!(meta["headers"]["x-gallery-lifecycle"], "1");
    (meta, body.to_owned())
}

/// Prefix/cookie/default and the real guarded Resource renderer agree before JS starts.
#[test]
fn locale_pages_and_csr_seed() {
    for (label, path, cookie, locale) in [
        ("prefix", "/demo/en/gallery", "stackpress-locale=fr", "en"),
        ("cookie", "/demo/gallery", "stackpress-locale=fr", "fr"),
        ("default", "/demo/gallery", "", "en"),
        ("alias", "/demo/fr-FR/gallery?theme=dark", "", "fr"),
    ] {
        let (meta, html) = request(label, "GET", path, cookie, None, "", false);
        assert_eq!(meta["status"], 200);
        assert!(html.contains(&format!("<html lang=\"{locale}\"")));
        assert!(html.contains("id=\"resource\""));
        assert!(html.contains("stackpress-resources"));
        assert_eq!(meta["headers"]["cache-control"], "no-store");
    }
    let (_, html) = request("csr", "GET", "/demo/gallery", "", None, "", true);
    assert!(html.contains("<div id=\"app\"></div>"));
    let (meta, body) = request("resource", "GET", "/demo/resource", "", None, "", false);
    assert_eq!(meta["status"], 200);
    assert!(body.contains("globalThis.pwned"));
}

/// Only valid same-origin actions can create a scoped HttpOnly cookie; errors disclose no cause.
#[test]
fn preference_action_edges() {
    let input = r#"{"locale":"fr","location":"/demo/en/gallery?q=kept"}"#;
    let (meta, body) = request(
        "saved",
        "POST",
        "/demo/ui-preferences",
        "",
        Some("https://gallery.invalid"),
        input,
        false,
    );
    assert_eq!(meta["status"], 200);
    assert!(
        meta["headers"]["set-cookie"]
            .as_str()
            .unwrap()
            .contains("HttpOnly; SameSite=Lax; Secure")
    );
    assert!(!body.contains("cookie"));
    assert!(body.contains("/demo/fr/gallery?q=kept"));
    let large =
        json!({"locale":"fr","location":"/demo/gallery","extra":"x".repeat(1025)}).to_string();
    for (label, origin, body, status) in [
        ("missing", None, input, 403),
        ("foreign", Some("https://evil.invalid"), input, 403),
        ("malformed", Some("https://gallery.invalid"), "{}", 400),
        ("large", Some("https://gallery.invalid"), &large, 413),
    ] {
        let (meta, _) = request(
            label,
            "POST",
            "/demo/ui-preferences",
            "",
            origin,
            body,
            false,
        );
        assert_eq!(meta["status"], status);
        assert!(meta["headers"].get("set-cookie").is_none());
    }
}
