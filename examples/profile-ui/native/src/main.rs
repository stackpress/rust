//! Minimal packaged host: fixed commands, authoritative admission and bounded shutdown.

mod proof;
use proof::Proof;
use stackpress_example_profile_ui_native::{
    NativeError, NativeHost, register_application, validate_caller,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

/// Only one bounded shutdown coordinator may run; later exit events await its completion.
#[derive(Default)]
struct Shutdown {
    started: AtomicBool,
    finished: AtomicBool,
    failed: AtomicBool,
}

/// Normal startup reuses the generated application. Proof fixtures are explicitly selected.
async fn application(proof: Option<&Arc<Proof>>) -> stackpress_server::BuildResult<NativeHost> {
    let Some(proof) = proof else {
        return NativeHost::new().await;
    };
    let mut builder = stackpress_server::ServerBuilder::new();
    register_application(&mut builder)?;
    proof.register(&mut builder)?;
    Ok(NativeHost::from_server(builder.build().await?))
}

/// Read native WebView identity; callers cannot supply these values as DTO fields.
fn trusted(window: &WebviewWindow) -> Result<(), String> {
    let url = window
        .url()
        .map_err(|_| NativeError::Denied.code().to_owned())?;
    validate_caller(window.label(), url.scheme(), url.host_str())
        .map_err(|error| error.code().to_owned())
}

/// One fixed read owns its host clone; no registry mutex crosses await.
#[tauri::command]
async fn profile_search(
    window: WebviewWindow,
    host: tauri::State<'_, NativeHost>,
    call_id: String,
    json: String,
) -> Result<String, String> {
    trusted(&window)?;
    let native = host.inner().clone();
    native
        .search(window.label(), &call_id, &json)
        .await
        .map_err(|error| error.code().to_owned())
}

/// Cancellation is qualified by the authoritative window, never by a caller-provided identity.
#[tauri::command]
fn cancel_profile_search(
    window: WebviewWindow,
    host: tauri::State<'_, NativeHost>,
    call_id: String,
) -> Result<(), String> {
    trusted(&window)?;
    host.cancel(window.label(), &call_id)
        .map_err(|error| error.code().to_owned())
}

/// Close admission first, then await disposal before permitting the event loop to exit.
fn shutdown(handle: &tauri::AppHandle, code: i32) {
    let state = handle.state::<Arc<Shutdown>>().inner().clone();
    if state.started.swap(true, Ordering::SeqCst) {
        return;
    }
    let host = handle.state::<NativeHost>().inner().clone();
    let handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let result = host.shutdown().await;
        state.failed.store(result.is_err(), Ordering::SeqCst);
        state.finished.store(true, Ordering::SeqCst);
        handle.exit(if result.is_ok() { code } else { 1 });
    });
}

/// Build real packaged windows; no network listener, remote frontend or plugin grant is used.
fn main() {
    let proof = Proof::from_env();
    let host = tauri::async_runtime::block_on(application(proof.as_ref()))
        .expect("valid generated Profile host");
    let selected = proof.clone();
    let app = tauri::Builder::default()
        .manage(host)
        .manage(Arc::new(Shutdown::default()))
        .invoke_handler(tauri::generate_handler![
            profile_search,
            cancel_profile_search
        ])
        .setup(move |app| {
            let labels: &[&str] = if selected
                .as_ref()
                .is_some_and(|proof| proof.mode == "navigation")
            {
                &["main", "ungranted"]
            } else {
                &["main"]
            };
            for label in labels {
                let proof = selected.clone();
                let handle = app.handle().clone();
                let mut window =
                    WebviewWindowBuilder::new(app, *label, WebviewUrl::App("index.html".into()))
                        .title("Stackpress Profiles")
                        .visible(*label == "main")
                        .on_navigation(move |url| {
                            // Only an explicitly selected fixture may consume its diagnostic URLs.
                            if proof.as_ref().is_some_and(|proof| proof.record_navigation(&handle, url)) {
                                return false;
                            }
                            // History stays on packaged resources; native navigation cannot open a remote origin.
                            url.scheme() == "tauri" && url.host_str() == Some("localhost")
                        });
                if let Some(proof) = &selected {
                    window = window.initialization_script(format!(
                        "window.profileProofWindow='{}';window.profileProofMode='{}';window.profileProofCustomization={};\n{}",
                        label,
                        proof.mode,
                        cfg!(feature = "demo-customization"),
                        include_str!("../launch-proof.js")
                    ));
                }
                window.build()?;
            }
            if let Some(proof) = &selected {
                proof.monitor(app.handle().clone());
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main" {
                match event {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        api.prevent_close();
                        shutdown(window.app_handle(), 0);
                    }
                    tauri::WindowEvent::Destroyed => {
                        let _ = window.state::<NativeHost>().close();
                    }
                    _ => {}
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("valid packaged application");
    app.run(move |handle, event| match event {
        tauri::RunEvent::ExitRequested { api, code, .. } => {
            if !handle
                .state::<Arc<Shutdown>>()
                .finished
                .load(Ordering::SeqCst)
            {
                api.prevent_exit();
                shutdown(handle, code.unwrap_or(0));
            }
        }
        tauri::RunEvent::Exit => {
            if let Some(proof) = &proof {
                let state = handle.state::<Arc<Shutdown>>();
                proof.finish(
                    &handle.state::<NativeHost>(),
                    state.finished.load(Ordering::SeqCst) && !state.failed.load(Ordering::SeqCst),
                );
            }
        }
        _ => {}
    });
}
