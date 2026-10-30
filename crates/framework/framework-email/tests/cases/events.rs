//! Original event traces plus public registrar/direct reuse; every delivery seam is fake.

use serde_json::{Value as Json, json};
use stackpress_framework_email::{
    EmailConfig, EmailResult, MailFuture, SendMailOptions, Transport, TransportFactory,
    events::{EMAIL_SEND, EmailSend},
    plugin,
};
use stackpress_lib::{
    Body, DispatchOutcome, Flow, Invocation, Request, Response, Result, handler, queue::task,
};
use stackpress_server::{ConfigLayers, ServerBuilder, StartupContext};
use std::{
    error::Error as _,
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

/// Source-derived boundary traces, never handwritten expected event logic.
pub(super) fn vectors() -> Json {
    serde_json::from_str(include_str!("../fixtures/source-events.json")).unwrap()
}

/// Observe provider construction/send and deterministic destruction, including error paths.
struct Factory {
    trace: Arc<Mutex<Vec<Json>>>,
    info: Json,
    failure: Option<String>,
    dropped: Arc<AtomicUsize>,
}
/// A fake transport owns its source constructor/send state independently.
struct Fake {
    trace: Arc<Mutex<Vec<Json>>>,
    info: Json,
    failure: bool,
    dropped: Arc<AtomicUsize>,
}
/// Concrete source callback error extensions survive the framework cause adapter.
#[derive(Debug)]
struct ProviderError(Json);
impl fmt::Display for ProviderError {
    /// Match the original provider's diagnostic without adding a public response code.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0["message"].as_str().unwrap().fmt(f)
    }
}
impl std::error::Error for ProviderError {}
impl TransportFactory for Factory {
    /// Capture original config then construct one independently owned fake.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        self.trace.lock().unwrap().push(json!(["create", config]));
        if self.failure.as_deref() == Some("create") {
            return Err(Box::new(ProviderError(
                json!({"message":"create failed", "code":"EPROVIDER", "extension":{"raw":true}}),
            )));
        }
        Ok(Box::new(Fake {
            trace: self.trace.clone(),
            info: self.info.clone(),
            failure: self.failure.as_deref() == Some("send"),
            dropped: self.dropped.clone(),
        }))
    }
}
impl Transport for Fake {
    /// Mirror only the provider seam; the production process owns every gate/mutation.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            self.trace.lock().unwrap().push(json!(["send", options]));
            if self.failure {
                return Err(Box::new(ProviderError(
                    json!({"message":"send failed", "code":"EPROVIDER", "extension":{"raw":true}}),
                )) as _);
            }
            Ok(self.info.clone().into())
        })
    }
}
impl Drop for Fake {
    /// Assert release after success and callback failure.
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}
/// Construct a fake with an externally visible source call log.
fn factory(info: Json, failure: Option<&str>) -> Arc<Factory> {
    Arc::new(Factory {
        trace: Arc::default(),
        info,
        failure: failure.map(str::to_owned),
        dropped: Arc::default(),
    })
}
/// Configure only the source email section, without a network endpoint or identity plugin.
fn builder(config: Option<Json>, factory: Arc<dyn TransportFactory>) -> ServerBuilder {
    let mut app = ServerBuilder::new();
    plugin::register_with_factory(&mut app, factory).unwrap();
    if let Some(config) = config {
        let mut layers = ConfigLayers::new();
        layers.value("fixture", json!({"email":config})).unwrap();
        app.config_layers(layers).unwrap();
    }
    app
}

/// All source skip/config/error/result traces agree with the port's observable boundary.
#[tokio::test]
async fn original_email_invocation_traces() {
    for case in vectors()["cases"].as_array().unwrap() {
        let fake = factory(case["info"].clone(), case["failure"].as_str());
        let event = EmailSend::new(
            (case["missing"] != true).then(|| case["config"].clone()),
            fake.clone(),
        );
        let mut request = Request::builder()
            .data(case["options"].clone())
            .build()
            .unwrap();
        let mut response = Response::new();
        response
            .set(
                "application/json",
                Body::Value(stackpress_lib::Value::try_from(case["body"].clone()).unwrap()),
            )
            .unwrap();
        if let Some(code) = case["code"].as_u64() {
            response.status_code(code as u16, None).unwrap();
        }
        let before = response.to_envelope().try_to_json().unwrap();
        let result = event.handle(&mut request, &mut response).await;
        let expected: Vec<_> = case["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|step| step[0] == "create" || step[0] == "send")
            .cloned()
            .collect();
        assert_eq!(*fake.trace.lock().unwrap(), expected, "{}", case["name"]);
        if case["error"].is_null() {
            assert_eq!(result.unwrap(), Flow::Continue, "{}", case["name"]);
            if case["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|step| step[0] == "results")
            {
                assert_eq!(
                    response.result().unwrap().to_json().unwrap(),
                    case["result"]
                );
                assert_eq!(response.status().unwrap().code, 200);
                assert_eq!(response.total(), 1);
            } else {
                assert_eq!(response.to_envelope().try_to_json().unwrap(), before);
            }
        } else {
            let error = result.unwrap_err();
            let provider = error
                .source()
                .unwrap()
                .source()
                .unwrap()
                .downcast_ref::<ProviderError>()
                .unwrap();
            assert_eq!(provider.0, case["error"]);
            assert_eq!(response.to_envelope().try_to_json().unwrap(), before);
        }
        let sends = expected.iter().filter(|step| step[0] == "send").count();
        assert_eq!(fake.dropped.load(Ordering::SeqCst), sends);
    }
}

/// Registration has/truthy gates yield precisely the source runtime inventory.
#[tokio::test]
async fn original_registration_missing_config_modes_and_no_global_guard() {
    for case in vectors()["registrations"].as_array().unwrap() {
        let fake = factory(json!({"provider":true}), None);
        let app = builder(
            (case["missing"] != true).then(|| case["config"].clone()),
            fake.clone(),
        );
        let server = app.build().await.unwrap();
        let expected = case["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|step| step[0] == "use");
        assert_eq!(server.listeners().count(), usize::from(expected));
        let reply = server
            .resolve(EMAIL_SEND, json!({"extension":true}))
            .await
            .unwrap();
        assert_eq!(
            reply.outcome,
            if expected {
                DispatchOutcome::Completed
            } else {
                DispatchOutcome::NoListener
            }
        );
        assert_eq!(
            fake.trace.lock().unwrap().len(),
            if expected { 2 } else { 0 }
        );
        // No selected session, user, CSRF, method, sender or recipient policy is required.
        let reply = server.resolve("unrelated", json!({})).await.unwrap();
        assert_eq!(reply.outcome, DispatchOutcome::NoListener);
    }
}

/// Reuse exactly the same process through two names and a nested emit/resolve.
#[tokio::test]
async fn direct_and_nested_event_reuse_create_a_transport_per_call() {
    let fake = factory(json!({"provider":"unchanged", "extension":[]}), None);
    let event = EmailSend::new(Some(json!({"pool":true,"custom":false})), fake.clone());
    let mut app = builder(None, fake.clone());
    for name in [EMAIL_SEND, "custom-mail"] {
        let event = event.clone();
        app.on(
            name,
            handler(
                move |req: &mut Request, res: &mut Response, _: Invocation<'_>| {
                    let event = event.clone();
                    Box::pin(async move { event.handle(req, res).await })
                },
            ),
        )
        .unwrap();
    }
    app.on(
        "parent",
        handler(
            |req: &mut Request, res: &mut Response, inv: Invocation<'_>| {
                Box::pin(async move {
                    let child = inv.resolve(EMAIL_SEND, json!({"text":"resolve"})).await?;
                    assert_eq!(
                        child.response.results.unwrap().to_json()?,
                        json!({"provider":"unchanged","extension":[]})
                    );
                    inv.emit("custom-mail", req, res).await?;
                    Ok(Flow::Continue)
                })
            },
        ),
    )
    .unwrap();
    let server = app.build().await.unwrap();
    let reply = server
        .resolve("parent", json!({"text":"emit"}))
        .await
        .unwrap();
    assert_eq!(
        reply.response.results.unwrap().to_json().unwrap(),
        json!({"provider":"unchanged","extension":[]})
    );
    assert_eq!(
        fake.trace
            .lock()
            .unwrap()
            .iter()
            .filter(|step| step[0] == "create")
            .count(),
        2
    );
    assert_eq!(fake.dropped.load(Ordering::SeqCst), 2);
}

/// Existing priorities/FIFO and explicit abort remain the sole event ordering authority.
#[tokio::test]
async fn same_event_priority_fifo_and_abort() {
    let fake = factory(json!({"sent":true}), None);
    let trace = fake.trace.clone();
    let mut app = ServerBuilder::new();
    let before = trace.clone();
    app.on_listen(task(move |start: &mut StartupContext| {
        let trace = before.clone();
        Box::pin(async move {
            start.on(
                EMAIL_SEND,
                handler(
                    move |_: &mut Request, _: &mut Response, _: Invocation<'_>| {
                        let trace = trace.clone();
                        Box::pin(async move {
                            trace.lock().unwrap().push(json!(["before"]));
                            Ok(Flow::Continue)
                        })
                    },
                ),
            )?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    plugin::register_with_factory(&mut app, fake.clone()).unwrap();
    let after = trace.clone();
    app.on_listen(task(move |start: &mut StartupContext| {
        let trace = after.clone();
        Box::pin(async move {
            start.on(
                EMAIL_SEND,
                handler(
                    move |_: &mut Request, res: &mut Response, _: Invocation<'_>| {
                        let trace = trace.clone();
                        Box::pin(async move {
                            assert!(res.result().is_some());
                            trace.lock().unwrap().push(json!(["after"]));
                            Ok(Flow::Continue)
                        })
                    },
                ),
            )?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let higher = trace.clone();
    app.on_priority(
        EMAIL_SEND,
        10,
        handler(
            move |req: &mut Request, _: &mut Response, _: Invocation<'_>| {
                let trace = higher.clone();
                Box::pin(async move {
                    trace.lock().unwrap().push(json!(["high"]));
                    Ok(if req.data.has("abort") {
                        Flow::Abort
                    } else {
                        Flow::Continue
                    })
                })
            },
        ),
    )
    .unwrap();
    let mut layers = ConfigLayers::new();
    layers.value("fixture", json!({"email":{}})).unwrap();
    app.config_layers(layers).unwrap();
    let server = app.build().await.unwrap();
    assert_eq!(
        server.resolve(EMAIL_SEND, json!({})).await.unwrap().outcome,
        DispatchOutcome::Completed
    );
    assert_eq!(
        trace
            .lock()
            .unwrap()
            .iter()
            .map(|step| step[0].clone())
            .collect::<Vec<_>>(),
        vec![
            json!("high"),
            json!("before"),
            json!("create"),
            json!("send"),
            json!("after")
        ]
    );
    trace.lock().unwrap().clear();
    assert_eq!(
        server
            .resolve(EMAIL_SEND, json!({"abort":true}))
            .await
            .unwrap()
            .outcome,
        DispatchOutcome::Aborted
    );
    assert_eq!(*trace.lock().unwrap(), vec![json!(["high"])]);
}

/// The built-in ordinary registrar compiles without an application-supplied factory.
#[tokio::test]
async fn default_registrar_and_deferred_invalid_config() -> Result<()> {
    let mut app = ServerBuilder::new();
    plugin::register(&mut app)?;
    let mut layers = ConfigLayers::new();
    layers.value(
        "fixture",
        json!({"email":{"jsonTransport":true,"skipEncoding":true}}),
    )?;
    app.config_layers(layers)?;
    let server = app.build().await.unwrap();
    let reply = server.resolve(EMAIL_SEND, json!({"text":"hello"})).await?;
    assert_eq!(
        reply.response.results.unwrap().to_json()?["message"]["text"],
        "hello"
    );
    let fake = factory(json!(null), None);
    let server = builder(Some(json!(true)), fake.clone())
        .build()
        .await
        .unwrap();
    let mut req = Request::new();
    let mut res = Response::new();
    res.html("prior")?;
    assert_eq!(
        server.emit(EMAIL_SEND, &mut req, &mut res).await?,
        DispatchOutcome::Completed
    );
    assert!(server.resolve(EMAIL_SEND, json!({})).await.is_err());
    assert!(fake.trace.lock().unwrap().is_empty());
    Ok(())
}
