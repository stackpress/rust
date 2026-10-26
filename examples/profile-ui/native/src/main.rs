//! Minimal macOS packaged host; no local server or remote frontend is started.

use stackpress_example_profile_ui_native::NativeHost;
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Proof-only listener, selected before building the host; ordinary app startup omits it.
async fn proof_rejection(
    request: &mut stackpress_lib::Request,
    _: &mut stackpress_lib::Response,
    _: stackpress_lib::Invocation<'_>,
) -> stackpress_lib::Result<stackpress_lib::Flow> {
    // Only the finite proof's explicit prefix selects this controlled public rejection.
    if request.data.path_as::<String>("id")?.as_deref() == Some("host-error") {
        return Err(stackpress_lib::Error::new("PRIVATE_PROOF_CAUSE")
            .with_code(422)
            .with_public_message("Choose another prefix."));
    }
    Ok(stackpress_lib::Flow::Continue)
}

/// Build the unchanged generated operation, optionally adding one explicit proof listener.
async fn application(proof: bool) -> stackpress_server::BuildResult<NativeHost> {
    if !proof {
        return NativeHost::new().await;
    }
    let mut builder = stackpress_server::ServerBuilder::new();
    stackpress_example_generated_profile::generated::register(&mut builder)?;
    // Registration is proof-only; no command, privilege or domain implementation is added.
    builder.on("profile-search", proof_rejection)?;
    Ok(NativeHost::from_server(builder.build().await?))
}

/// Bind admission to the authoritative calling WebView and packaged origin.
fn trusted(window: &WebviewWindow) -> Result<(), String> {
    let url = window.url().map_err(|_| "Search denied")?;
    if window.label() != "main" || url.scheme() != "tauri" || url.host_str() != Some("localhost") {
        return Err("Search denied".into());
    }
    Ok(())
}

/// Run one admitted fixed read through the existing native lifecycle.
#[tauri::command]
async fn profile_search(
    window: WebviewWindow,
    host: tauri::State<'_, NativeHost>,
    call_id: String,
    json: String,
) -> Result<String, String> {
    trusted(&window)?;
    host.search(window.label(), &call_id, &json)
        .await
        .map_err(|error| error.to_string())
}

/// Cancellation cannot name an event, service or a different originating window.
#[tauri::command]
fn cancel_profile_search(
    window: WebviewWindow,
    host: tauri::State<'_, NativeHost>,
    call_id: String,
) -> Result<(), String> {
    trusted(&window)?;
    host.cancel(window.label(), &call_id)
        .map_err(|error| error.to_string())
}

/// Build real packaged windows and reject navigation away from bundled assets.
fn main() {
    let proof = std::env::var_os("PROFILE_UI_PROOF").map(std::path::PathBuf::from);
    let host = tauri::async_runtime::block_on(application(proof.is_some()))
        .expect("valid generated Profile host");
    tauri::Builder::default()
        .manage(host)
        .invoke_handler(tauri::generate_handler![profile_search, cancel_profile_search])
        .setup(move |app| {
            let receipt = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
            let labels: &[&str] = if proof.is_some() { &["main", "ungranted"] } else { &["main"] };
            for label in labels {
                let target = proof.clone();
                let receipt = receipt.clone();
                let handle = app.handle().clone();
                let mut window = WebviewWindowBuilder::new(app, *label, WebviewUrl::App("index.html".into()))
                    .title("Stackpress Profiles")
                    .visible(*label == "main")
                    .on_navigation(move |url| {
                        if let Some(target) = &target
                            && url.scheme() == "proof"
                        {
                                let event = url.host_str().unwrap_or("failed").to_owned();
                                if let Ok(mut events) = receipt.lock() {
                                    events.push(event.clone());
                                    if event == "failed" || (events.iter().any(|s| s == "main-ok") && events.iter().any(|s| s == "denied-ok")) {
                                        let host = handle.state::<NativeHost>();
                                        let count = host.in_flight().unwrap_or(usize::MAX);
                                        let report = serde_json::json!({"events": *events, "in_flight": count, "packaged_origin": "tauri://localhost", "details": url.query()});
                                        let _ = std::fs::write(target, report.to_string());
                                        let _ = host.close();
                                        handle.exit(if event == "failed" || count != 0 { 1 } else { 0 });
                                    }
                                }
                            return false;
                        }
                        url.scheme() == "tauri" && url.host_str() == Some("localhost")
                    });
                if proof.is_some() {
                    window = window.initialization_script(format!("window.profileProofWindow = '{}';\n{}", label, include_str!("../launch-proof.js")));
                }
                window.build()?;
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the permitted window closes admission before cancelling owned work.
            if window.label() == "main" && matches!(event, tauri::WindowEvent::Destroyed) {
                let _ = window.state::<NativeHost>().close();
            }
        })
        .run(tauri::generate_context!())
        .expect("Tauri event loop");
}
