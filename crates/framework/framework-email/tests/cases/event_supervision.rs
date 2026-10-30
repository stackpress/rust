//! Owned source signup extension calls with fresh child state and deterministic task barriers.

use serde_json::json;
use stackpress_framework_email::{
    EmailConfig, EmailResult, MailFuture, SendMailOptions, Transport, TransportFactory,
    events::EMAIL_SEND, plugin, tasks::TaskSupervisor,
};
use stackpress_lib::{Flow, Invocation, Request, Response, handler};
use stackpress_server::{ConfigLayers, ServerBuilder};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio::{
    runtime::Handle,
    sync::{mpsc, oneshot},
};

/// A factory exposes a release barrier per send, proving simultaneous calls have separate handles.
struct Factory {
    entered: mpsc::UnboundedSender<(SendMailOptions, oneshot::Sender<()>)>,
    dropped: Arc<AtomicUsize>,
}
/// One task-owned transport, retained until callback completion or cancellation.
struct Sender {
    entered: mpsc::UnboundedSender<(SendMailOptions, oneshot::Sender<()>)>,
    dropped: Arc<AtomicUsize>,
}
impl TransportFactory for Factory {
    /// Construct without borrowing any parent request or response.
    fn create(&self, _: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        Ok(Box::new(Sender {
            entered: self.entered.clone(),
            dropped: self.dropped.clone(),
        }))
    }
}
impl Transport for Sender {
    /// Wait on an explicit channel; neither timing sleeps nor live providers are involved.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            let (release, wait) = oneshot::channel();
            self.entered.send((options, release)).unwrap();
            wait.await?;
            Ok(json!({"child":"sent"}).into())
        })
    }
}
impl Drop for Sender {
    /// Observe resource release after join or shutdown's abort/reap barrier.
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

/// Use actual source hook names/payloads/order; parent results complete while mail is pending.
#[tokio::test]
async fn source_signup_hooks_are_owned_nonawaited_and_keep_registration_order() {
    let vectors = super::events::vectors();
    assert_eq!(vectors["composite"], json!(["auth", "email", "session"]));
    let source = &vectors["signup"][0]["trace"];
    let hooks: Vec<_> = source
        .as_array()
        .unwrap()
        .iter()
        .filter(|step| step[0] == "resolve")
        .collect();
    assert_eq!(
        hooks.iter().map(|step| step[1].clone()).collect::<Vec<_>>(),
        vec![json!("email-email-send"), json!("auth-phone-verify")]
    );
    let (entered, mut observed) = mpsc::unbounded_channel();
    let dropped = Arc::new(AtomicUsize::new(0));
    let mut app = ServerBuilder::new();
    plugin::register_with_factory(
        &mut app,
        Arc::new(Factory {
            entered,
            dropped: dropped.clone(),
        }),
    )
    .unwrap();
    let trace = Arc::new(Mutex::new(vec![]));
    for ordinal in [1, 2] {
        let trace = trace.clone();
        app.on(
            "email-email-send",
            handler(
                move |req: &mut Request, res: &mut Response, inv: Invocation<'_>| {
                    let trace = trace.clone();
                    Box::pin(async move {
                        trace.lock().unwrap().push(ordinal);
                        if ordinal == 1 {
                            req.data.set("child_mutation", true)?;
                        } else {
                            inv.emit(EMAIL_SEND, req, res).await?;
                        }
                        Ok(Flow::Continue)
                    })
                },
            ),
        )
        .unwrap();
    }
    let mut layers = ConfigLayers::new();
    layers.value("fixture", json!({"email":{}})).unwrap();
    app.config_layers(layers).unwrap();
    let server = Arc::new(app.build().await.unwrap());
    let mut tasks = TaskSupervisor::new();
    let mut submissions = vec![];
    for hook in hooks {
        submissions.push(hook[1].clone());
        tasks.resolve_on(
            &Handle::current(),
            server.clone(),
            hook[1].as_str().unwrap(),
            hook[2].clone(),
        );
    }
    // Source strips secret fields before returning its parent result; child work owns its snapshot.
    let mut parent = Response::new();
    parent
        .results(&source.as_array().unwrap().last().unwrap()[1])
        .unwrap();
    let snapshot = parent.try_to_json().unwrap();
    let (options, release) = observed.recv().await.unwrap();
    assert_eq!(options["email"], "user@example.test");
    assert_eq!(options["child_mutation"], true);
    assert_eq!(*trace.lock().unwrap(), vec![1, 2]);
    assert_eq!(
        submissions,
        json!(["email-email-send", "auth-phone-verify"])
            .as_array()
            .unwrap()
            .clone()
    );
    assert_eq!(parent.try_to_json().unwrap(), snapshot);
    assert!(!tasks.is_empty());
    release.send(()).unwrap();
    while let Some(outcome) = tasks.join_next().await {
        outcome.unwrap().unwrap();
    }
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert_eq!(parent.try_to_json().unwrap(), snapshot);
}

/// Independent roots cannot leak/cancel one another's transport, request or response.
#[tokio::test]
async fn cancelling_one_root_and_supervisor_shutdown_reap_each_transport() {
    let (entered, mut observed) = mpsc::unbounded_channel();
    let dropped = Arc::new(AtomicUsize::new(0));
    let mut app = ServerBuilder::new();
    plugin::register_with_factory(
        &mut app,
        Arc::new(Factory {
            entered,
            dropped: dropped.clone(),
        }),
    )
    .unwrap();
    let mut layers = ConfigLayers::new();
    layers.value("fixture", json!({"email":{}})).unwrap();
    app.config_layers(layers).unwrap();
    let server = Arc::new(app.build().await.unwrap());
    let root = server.clone();
    let pending = tokio::spawn(async move { root.resolve(EMAIL_SEND, json!({"root":1})).await });
    let (first, first_release) = observed.recv().await.unwrap();
    assert_eq!(first["root"], 1);
    let mut tasks = TaskSupervisor::new();
    tasks.resolve_on(&Handle::current(), server, EMAIL_SEND, json!({"root":2}));
    let (second, second_release) = observed.recv().await.unwrap();
    assert_eq!(second["root"], 2);
    pending.abort();
    assert!(pending.await.unwrap_err().is_cancelled());
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert!(first_release.send(()).is_err());
    tasks.shutdown().await;
    assert!(tasks.is_empty());
    assert_eq!(dropped.load(Ordering::SeqCst), 2);
    assert!(second_release.send(()).is_err());
}

/// Raw child failures are retained, whereas source response errors and absent hooks resolve normally.
#[tokio::test]
async fn supervised_resolve_retains_throw_and_does_not_invent_missing_hook_errors() {
    let mut app = ServerBuilder::new();
    app.on(
        "throw",
        handler(|_: &mut Request, _: &mut Response, _: Invocation<'_>| {
            Box::pin(async { Err(stackpress_lib::Error::new("fixture throw")) })
        }),
    )
    .unwrap();
    app.on(
        "response-error",
        handler(|_: &mut Request, res: &mut Response, _: Invocation<'_>| {
            Box::pin(async move {
                res.status_code(419, None)?;
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    let server = Arc::new(app.build().await.unwrap());
    let mut tasks = TaskSupervisor::new();
    tasks.resolve_on(&Handle::current(), server.clone(), "throw", json!({}));
    assert!(
        tasks
            .join_next()
            .await
            .unwrap()
            .unwrap()
            .unwrap_err()
            .to_string()
            .contains("fixture throw")
    );
    for event in ["missing-extension", "response-error"] {
        tasks.resolve_on(&Handle::current(), server.clone(), event, json!({}));
        tasks.join_next().await.unwrap().unwrap().unwrap();
    }
}
