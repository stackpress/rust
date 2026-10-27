//! Opt-in finite simulator receipts and closed phase barriers; absent in normal apps.
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tauri::Manager;

/// Public proof state lives only in this disposable app container.
pub(super) struct MobileProof {
    directory: PathBuf,
    mode: String,
    report: Mutex<serde_json::Value>,
    suspended: AtomicUsize,
    resumed: AtomicUsize,
}
impl MobileProof {
    /// Select only declared fixture modes, using the real app-config path.
    pub(super) fn new(directory: PathBuf) -> Option<Arc<Self>> {
        // iOS launch inherits the fixture environment. Android uses a fixed
        // debug-only app-private selector written by run-as before launching;
        // release applications never consult that selector.
        #[cfg(target_os = "ios")]
        let mode = std::env::var("PROFILE_UI_MOBILE_PROOF").ok()?;
        #[cfg(target_os = "android")]
        let mode = {
            if !cfg!(debug_assertions) {
                return None;
            }
            let path = directory.join("mobile-proof-mode");
            let metadata = std::fs::symlink_metadata(&path).ok()?;
            if !metadata.is_file() || metadata.len() > 16 {
                return None;
            }
            std::fs::read_to_string(path).ok()?
        };
        if !matches!(mode.as_str(), "first" | "restart" | "recovery") {
            return None;
        }
        Some(Arc::new(Self {
            directory,
            mode,
            report: Mutex::new(serde_json::Value::Null),
            suspended: AtomicUsize::new(0),
            resumed: AtomicUsize::new(0),
        }))
    }
    /// Install observation before any bundled page module can mount a root.
    pub(super) fn script(&self) -> String {
        format!(
            "window.mobileProofMode={};window.mobileProofOrigin={};\n{}",
            serde_json::to_string(&self.mode).unwrap_or_else(|_| "null".into()),
            serde_json::to_string(if cfg!(target_os = "android") {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            })
            .unwrap_or_default(),
            include_str!("../mobile-proof.js")
        )
    }
    /// Consume only opt-in diagnostic navigation; no application operation is dispatched.
    pub(super) fn navigation(&self, url: &tauri::Url) -> bool {
        if url.scheme() != "mobile-proof" {
            return false;
        }
        if url.as_str().len() <= 16 * 1024 {
            let value = url
                .query_pairs()
                .find(|(key, _)| key == "data")
                .and_then(|(_, value)| serde_json::from_str::<serde_json::Value>(&value).ok());
            if let Some(value) = value {
                if let Ok(mut report) = self.report.lock() {
                    *report = value;
                }
                self.write();
            }
        }
        true
    }
    /// Record actual platform suspension/foreground events, not a timer guess.
    pub(super) fn lifecycle(&self, resumed: bool) {
        if resumed {
            self.resumed.fetch_add(1, Ordering::SeqCst);
        } else {
            self.suspended.fetch_add(1, Ordering::SeqCst);
        }
        self.write();
    }
    /// Snapshot outside locks before file I/O or WebView callbacks can re-enter.
    fn write(&self) {
        let report = self
            .report
            .lock()
            .map(|report| report.clone())
            .unwrap_or(serde_json::Value::Null);
        let value = serde_json::json!({"mode":self.mode,"suspended":self.suspended.load(Ordering::SeqCst),"resumed":self.resumed.load(Ordering::SeqCst),"javascript":report});
        if std::fs::create_dir_all(&self.directory).is_ok() {
            if let Ok(text) = serde_json::to_vec(&value) {
                let temporary = self.directory.join("mobile-proof.pending");
                if std::fs::write(&temporary, text).is_ok() {
                    let _ = std::fs::rename(temporary, self.directory.join("mobile-proof.json"));
                }
            }
        }
    }
    /// A bounded test-only control file carries closed phase names, never JavaScript.
    pub(super) fn monitor(self: &Arc<Self>, handle: tauri::AppHandle) {
        let proof = self.clone();
        tauri::async_runtime::spawn(async move {
            let deadline = std::time::Instant::now() + Duration::from_secs(90);
            let mut previous = String::new();
            while std::time::Instant::now() < deadline {
                let path = proof.directory.join("mobile-proof-control");
                if let Ok(metadata) = std::fs::symlink_metadata(&path) {
                    if metadata.is_file() && metadata.len() <= 16 {
                        if let Ok(phase) = std::fs::read_to_string(&path) {
                            if phase != previous
                                && matches!(phase.as_str(), "fail" | "continue" | "finish")
                            {
                                previous = phase.clone();
                                if let Some(window) = handle.get_webview_window("main") {
                                    let code = format!(
                                        "window.mobileProofControl={};window.dispatchEvent(new Event('mobile-proof-control'));",
                                        serde_json::to_string(&phase).unwrap_or_default()
                                    );
                                    let _ = window.eval(code);
                                }
                            }
                        }
                    }
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });
    }
}
