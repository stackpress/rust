//! Opt-in finite packaged evidence; no additional IPC command or normal-app service.

use stackpress_example_profile_ui_native::NativeHost;
use stackpress_lib::{Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{ServerBuilder, ServerInvocationExt};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tauri::Manager;
use tokio::sync::watch;

/// Native-only receipts and entry barriers. No field points back to the host or app.
pub(super) struct Proof {
    path: PathBuf,
    pub(super) mode: String,
    pub(super) events: Mutex<Vec<String>>,
    pub(super) details: Mutex<Option<String>>,
    entries: AtomicUsize,
    drops: Arc<AtomicUsize>,
    entered: watch::Sender<usize>,
    before_shutdown: AtomicUsize,
}
impl Proof {
    /// Select proof fixtures only for the explicit finite launcher environment.
    pub(super) fn from_env() -> Option<Arc<Self>> {
        let path = std::env::var_os("PROFILE_UI_PROOF")?;
        let mode = std::env::var("PROFILE_UI_PROOF_MODE").unwrap_or_else(|_| "navigation".into());
        let (entered, _) = watch::channel(0);
        Some(Arc::new(Self {
            path: path.into(),
            mode,
            events: Mutex::new(Vec::new()),
            details: Mutex::new(None),
            entries: AtomicUsize::new(0),
            drops: Arc::new(AtomicUsize::new(0)),
            entered,
            before_shutdown: AtomicUsize::new(0),
        }))
    }
    /// Add one controlled rejecting/suspended read after the unchanged generated registration.
    pub(super) fn register(self: &Arc<Self>, app: &mut ServerBuilder) -> Result<()> {
        let proof = Arc::clone(self);
        app.on_config(task(
            move |start: &mut stackpress_server::StartupContext| {
                let proof = Arc::clone(&proof);
                Box::pin(async move {
                    start.register(proof)?;
                    Ok(Flow::Continue)
                })
            },
        ))?;
        app.on("profile-search", operation)?;
        Ok(())
    }
    /// Report real entry to JS, or trigger a real close/exit request while a call is suspended.
    pub(super) fn monitor(self: &Arc<Self>, handle: tauri::AppHandle) {
        let proof = Arc::clone(self);
        let mut entered = self.entered.subscribe();
        tauri::async_runtime::spawn(async move {
            let observe = async {
                let expected = if proof.mode == "navigation" { 2 } else { 1 };
                for _ in 0..expected {
                    entered.changed().await.map_err(|_| ())?;
                    let count = *entered.borrow_and_update();
                    let window = handle.get_webview_window("main").ok_or(())?;
                    if proof.mode == "navigation" {
                        window
                            .eval(format!("window.profileHangEntered={count};window.dispatchEvent(new Event('profile-proof-progress'));"))
                            .map_err(|_| ())?;
                    } else {
                        let live = handle.state::<NativeHost>().in_flight().map_err(|_| ())?;
                        proof.before_shutdown.store(live, Ordering::SeqCst);
                        if live != 1 {
                            return Err(());
                        }
                        if proof.mode == "window-close" {
                            window.close().map_err(|_| ())?;
                        } else {
                            handle.exit(0);
                        }
                    }
                }
                Ok::<_, ()>(())
            };
            // This fixture observer is finite and owns no lifecycle future.
            if tokio::time::timeout(std::time::Duration::from_secs(20), observe).await != Ok(Ok(()))
            {
                handle.exit(1);
            }
        });
    }
    /// Persist only after the app's shutdown path has drained the actual registry.
    pub(super) fn finish(&self, host: &NativeHost, shutdown_ok: bool) {
        let count = host.in_flight().unwrap_or(usize::MAX);
        let entries = self.entries.load(Ordering::SeqCst);
        let drops = self.drops.load(Ordering::SeqCst);
        let before = self.before_shutdown.load(Ordering::SeqCst);
        let mut events = self
            .events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default();
        if self.mode != "navigation" {
            events = vec![
                if shutdown_ok && before == 1 && entries == 1 && drops == 1 && count == 0 {
                    format!("{}-ok", self.mode)
                } else {
                    "failed".into()
                },
            ];
        }
        let details = self.details.lock().ok().and_then(|details| details.clone());
        let report = serde_json::json!({"events":events,"in_flight":count,"packaged_origin":"tauri://localhost","details":details,"entries":entries,"drops":drops,"in_flight_before_shutdown":before,"shutdown_ok":shutdown_ok,"customization":cfg!(feature = "demo-customization")});
        let _ = std::fs::write(&self.path, report.to_string());
    }
}

/// The actual suspended handler stack owns this receipt; map cleanup alone is insufficient.
struct Suspended(Arc<AtomicUsize>);
impl Drop for Suspended {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Explicit proof prefixes exercise the real host; production startup omits this listener.
async fn operation(req: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let id = req.data.path_as::<String>("id")?.unwrap_or_default();
    if id == "host-error" {
        return Err(stackpress_lib::Error::new("PRIVATE_PROOF_CAUSE")
            .with_code(422)
            .with_public_message("Choose another prefix."));
    }
    if matches!(
        id.as_str(),
        "deadline" | "hang" | "window-close" | "app-quit"
    ) {
        let proof = inv.service::<Proof>()?;
        let count = proof.entries.fetch_add(1, Ordering::SeqCst) + 1;
        let _guard = Suspended(Arc::clone(&proof.drops));
        proof.entered.send_replace(count);
        std::future::pending::<()>().await;
    }
    Ok(Flow::Continue)
}
