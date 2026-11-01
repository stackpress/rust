//! Suspended native providers own their cancellation state and reenter immutable configuration.

use super::*;
use stackpress_framework::config::ServerOptions;
use tokio::sync::Notify;

/// Explicit synchronization avoids sleeps and permits an external watchdog to catch executor stalls.
#[derive(Default)]
struct Suspension {
    entered: Notify,
    release: Notify,
    dropped: AtomicUsize,
}
/// Cancellation destroys exactly the active provider guard.
struct Active(Arc<Suspension>);
impl Drop for Active {
    fn drop(&mut self) {
        self.0.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
/// Config is read again through a nested event while a parent retains its own immutable reference.
async fn reentrant(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(inv.config::<ServerOptions>()?.0["host"], "finite");
    res.results(&"nested")?;
    Ok(Flow::Continue)
}
/// Retain explicit owned state through a suspension, with no registry lock or detached child.
async fn suspended(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let state = inv.shared_service::<Suspension>()?;
    let _active = Active(state.clone());
    let config = inv.config::<ServerOptions>()?;
    inv.emit("reentrant", req, res).await?;
    assert_eq!(config.0["host"], "finite");
    state.entered.notify_one();
    state.release.notified().await;
    Ok(Flow::Continue)
}
/// Run this exact test under the parent-process deadline as well as the ordinary test suite.
#[tokio::test]
async fn suspended_provider_cancellation_reentrant_config_and_nested_dispatch() {
    let state = Arc::new(Suspension::default());
    let captured = state.clone();
    let mut app = application(json!({"server":{"host":"finite"}}));
    app.on("reentrant", reentrant).unwrap();
    app.on_config(task(move |start: &mut StartupContext| {
        let state = captured.clone();
        Box::pin(async move {
            start.register(state)?;
            let mut bindings = Bindings::default();
            bindings.bind("serve", suspended);
            start.register(Arc::new(bindings))?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let host = Arc::new(app.build().await.unwrap());
    let retained = host.clone();
    let task = tokio::spawn(async move { retained.resolve("serve", json!({})).await });
    state.entered.notified().await;
    assert_eq!(
        host.resolve("reentrant", json!({}))
            .await
            .unwrap()
            .response
            .results
            .unwrap()
            .as_str(),
        Some("nested")
    );
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(state.dropped.load(Ordering::SeqCst), 1);
    state.release.notify_one();
    assert_eq!(
        host.resolve("serve", json!({}))
            .await
            .unwrap()
            .response
            .code,
        Some(200)
    );
    assert_eq!(state.dropped.load(Ordering::SeqCst), 2);
}
