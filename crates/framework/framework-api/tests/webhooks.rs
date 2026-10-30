//! Inline source webhook selection, envelope, ordering, cancellation and reentrancy.

use serde_json::{Value as Json, json};
use stackpress_framework_api::{
    ApiConfig, ApiWebhook, helpers, plugin,
    webhooks::{
        self, Fetch, FetchRequest, Transport, Webhook,
        predicate::{PredicateObject, PredicateValue},
    },
};
use stackpress_lib::{
    Body, Error, Flow, HandlerFuture, Invocation, Request, Response, Result, SharedNative,
    Timestamp, Value,
};
use stackpress_server::{ConfigLayers, ServerBuilder, ServerInvocationExt, StartupContext};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicUsize, Ordering},
    sync::{Arc, Mutex},
};

/// Finite transport; every invocation retains exact bytes and can reject without replay.
#[derive(Default)]
struct Fake {
    calls: Mutex<Vec<(String, String, String, Json)>>,
    reject: bool,
}
impl Fetch for Fake {
    fn send<'a>(&'a self, request: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>> {
        Box::pin(async move {
            self.calls.lock().unwrap().push((
                request.uri.into(),
                request.method.into(),
                request.content_type.into(),
                serde_json::from_slice(request.body).unwrap(),
            ));
            tokio::task::yield_now().await;
            if self.reject {
                Err(Error::new("fetch rejected once"))
            } else {
                Ok(500)
            }
        })
    }
}

/// Construct ordinary separately parsed source configuration.
fn config(validity: Json) -> ApiWebhook {
    serde_json::from_value(
        json!({"event": "written", "uri": "http://127.0.0.1/private",
        "method": "post", "validity": validity, "data": {"secret": "fixed"}}),
    )
    .unwrap()
}

/// Build requests with canary fields that must remain unfiltered.
fn request() -> Result<Request> {
    Request::builder()
        .data(stackpress_lib::Nest::from_object(
            json!({"secret": "caller", "scopes": ["unregistered"], "id": "same"}),
        )?)
        .build()
}

#[tokio::test]
async fn exact_envelope_http_status_ignored_and_repeat_calls_send_again() -> Result<()> {
    let fake = Fake::default();
    let hook = Webhook::new(config(json!({"ok": true})))?;
    let mut res = Response::new();
    res.results(&json!({"ok": true, "secret": "result", "nested": {"a": 1}}))?;
    for _ in 0..2 {
        webhooks::send(&fake, &hook, &request()?, &res).await?;
    }
    let calls = fake.calls.lock().unwrap();
    assert_eq!(
        calls.len(),
        2,
        "each invocation is independent, without deduplication"
    );
    assert_eq!(calls[0].0, "http://127.0.0.1/private");
    assert_eq!(
        calls[0].1, "post",
        "source spelling reaches Fetch unchanged"
    );
    assert_eq!(calls[0].2, "application/json");
    assert_eq!(
        calls[0].3,
        json!({"data": {"secret": "fixed"},
        "params": {"secret": "caller", "scopes": ["unregistered"], "id": "same"},
        "results": {"ok": true, "secret": "result", "nested": {"a": 1}}})
    );
    Ok(())
}

#[tokio::test]
async fn only_exact_200_gates_and_predicate_precedes_serialization() -> Result<()> {
    for status in [
        None,
        Some(0),
        Some(199),
        Some(200),
        Some(201),
        Some(204),
        Some(500),
    ] {
        let fake = Fake::default();
        let hook = Webhook::new(config(json!({"ok": true})))?;
        let mut res = Response::new();
        if let Some(status) = status {
            // Response's existing status representation refuses zero, so test absent as source 0.
            if status == 0 {
                continue;
            }
            res.results(&json!({"ok": true}))?;
            res.status_code(status, None)?;
        }
        webhooks::send(&fake, &hook, &request()?, &res).await?;
        assert_eq!(
            fake.calls.lock().unwrap().len(),
            usize::from(status == Some(200))
        );
    }
    let fake = Fake::default();
    let mut req = request()?;
    req.data.set("native", SharedNative::new(7u8))?;
    let mut res = Response::new();
    res.results(&json!({"ok": false}))?;
    webhooks::send(
        &fake,
        &Webhook::new(config(json!({"ok": true})))?,
        &req,
        &res,
    )
    .await?;
    assert!(
        fake.calls.lock().unwrap().is_empty(),
        "predicate skips before unsupported params"
    );
    assert!(
        webhooks::send(&fake, &Webhook::new(config(json!({})))?, &req, &res)
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn predicate_subset_null_array_scalars_and_reference_identity() -> Result<()> {
    for (assertion, actual, passes) in [
        (json!({"a": null}), json!({"a": null}), true),
        (json!({"a": null}), json!({}), false),
        (json!({"n": 1}), json!({"n": "1"}), false),
        (
            json!({"a": {"b": true}}),
            json!({"a": {"b": true, "extra": 3}}),
            true,
        ),
        (json!({"a": []}), json!({"a": {}}), false),
        (json!({"a": [2, 1, 2]}), json!({"a": [1, 2]}), true),
        (json!({"a": [{"x": 1}]}), json!({"a": [{"x": 1}]}), false),
        (json!({"0": 1, "length": 1}), json!([1]), true),
        (json!({"0": "a", "length": 2}), json!("ab"), true),
        (json!({"a": 1}), json!(false), false),
    ] {
        let fake = Fake::default();
        let hook = Webhook::new(config(assertion))?;
        let mut res = Response::new();
        res.results(&actual)?;
        webhooks::send(&fake, &hook, &request()?, &res).await?;
        assert_eq!(!fake.calls.lock().unwrap().is_empty(), passes);
    }
    let shared = PredicateValue::Object(Arc::new(BTreeMap::from([(
        "x".into(),
        PredicateValue::Number(1.0),
    )])));
    let list = PredicateValue::Array(Arc::new(vec![shared.clone()]));
    let object: PredicateObject = Arc::new(BTreeMap::from([("a".into(), list)]));
    let mut hook = Webhook::new(config(json!({})))?;
    hook.predicate = object.clone();
    let mut res = Response::new();
    res.results_value(Value::Shared(SharedNative::new(PredicateValue::Object(
        object,
    ))))?;
    let fake = Fake::default();
    webhooks::send(&fake, &hook, &request()?, &res).await?;
    assert_eq!(
        fake.calls.lock().unwrap()[0].3["results"],
        json!({"a": [{"x": 1.0}]})
    );
    Ok(())
}

#[tokio::test]
async fn empty_missing_null_text_error_state_and_date_wire() -> Result<()> {
    let fake = Fake::default();
    let empty = Webhook::new(config(json!({})))?;
    let nonempty = Webhook::new(config(json!({"a": 1})))?;
    let mut res = Response::new();
    res.status_code(200, None)?;
    webhooks::send(&fake, &empty, &request()?, &res).await?;
    assert_eq!(fake.calls.lock().unwrap()[0].3["results"], Json::Null);
    assert!(
        webhooks::send(&fake, &nonempty, &request()?, &res)
            .await
            .is_err()
    );
    helpers::unauthorized(&mut res)?;
    res.status_code(200, None)?;
    res.set("text/plain", Body::Text("plain result".into()))?;
    webhooks::send(&fake, &empty, &request()?, &res).await?;
    assert_eq!(fake.calls.lock().unwrap()[1].3["results"], "plain result");
    // Wire format is explicitly selected by API; rich dates remain dates for predicate matching.
    for (seconds, nanos, expected) in [
        (0, 0, json!("1970-01-01T00:00:00.000Z")),
        (-1, 999_999_999, json!("1970-01-01T00:00:00.000Z")),
        (-1, 0, json!("1969-12-31T23:59:59.000Z")),
        (951_827_696, 789_900_000, json!("2000-02-29T12:34:56.789Z")),
        (-62_167_219_200, 0, json!("0000-01-01T00:00:00.000Z")),
        (253_402_300_800, 0, json!("+010000-01-01T00:00:00.000Z")),
        (8_640_000_000_000, 0, json!("+275760-09-13T00:00:00.000Z")),
        (-8_640_000_000_000, 0, json!("-271821-04-20T00:00:00.000Z")),
        (8_640_000_000_001, 0, Json::Null),
    ] {
        assert_eq!(
            webhooks::predicate::wire(&Value::DateTime(Timestamp::new(seconds, nanos)?))?,
            expected
        );
    }
    res.results_value(Value::Object(stackpress_lib::DataMap::from([(
        "date".into(),
        Value::DateTime(Timestamp::new(0, 0)?),
    )])))?;
    let hook = Webhook::new(config(json!({"date": {}})))?;
    assert!(
        !webhooks::predicate::matches(&hook.predicate, res.result())?,
        "source isObject excludes Date even for an empty expected object"
    );
    Ok(())
}

#[tokio::test]
async fn explicit_undefined_and_nonfinite_carrier_serialize_like_source() -> Result<()> {
    let object = PredicateValue::Object(Arc::new(BTreeMap::from([
        ("missing".into(), PredicateValue::Undefined),
        (
            "array".into(),
            PredicateValue::Array(Arc::new(vec![
                PredicateValue::Undefined,
                PredicateValue::Number(f64::NAN),
                PredicateValue::Number(f64::INFINITY),
            ])),
        ),
    ])));
    assert_eq!(
        webhooks::predicate::wire(&Value::Shared(SharedNative::new(object)))?,
        json!({"array": [null, null, null]})
    );
    Ok(())
}

/// Operation runs before the late observer and keeps all canary result fields.
async fn operation(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&json!({"ok": true}))?;
    Ok(Flow::Continue)
}
/// Explicit application Config installs one fake transport for selected API Listen.
async fn transport(start: &mut StartupContext) -> Result<Flow> {
    let fake = Arc::new(Fake::default());
    start.register(fake.clone())?;
    start.register(Arc::new(Transport(fake)))?;
    Ok(Flow::Continue)
}
/// A same-event tail proves the awaited webhook returns Continue.
async fn tail(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(inv.service::<Fake>()?.calls.lock().unwrap().len(), 2);
    Ok(Flow::Continue)
}

#[tokio::test]
async fn selected_webhook_only_listen_phase_duplicates_priority_and_no_startup_send() -> Result<()>
{
    let mut app = ServerBuilder::new();
    app.on_config(transport)?;
    app.plugin("api", plugin::register)?;
    app.on("written", operation)?;
    app.on_priority("written", -201, tail)?;
    let mut layers = ConfigLayers::new();
    layers.value(
        "test",
        json!({"api": ApiConfig {
            webhooks: vec![config(json!({"ok": true})), config(json!({}))],
            ..ApiConfig::default()
        }}),
    )?;
    app.config_layers(layers)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    assert!(
        server
            .host()
            .service::<Fake>()?
            .calls
            .lock()
            .unwrap()
            .is_empty()
    );
    let listeners: Vec<_> = server
        .listeners()
        .filter(|listener| listener.priority == -200)
        .collect();
    assert_eq!(listeners.len(), 2);
    assert!(
        listeners
            .iter()
            .all(|listener| listener.plugin.as_deref() == Some("api"))
    );
    server
        .emit("written", &mut request()?, &mut Response::new())
        .await?;
    Ok(())
}

#[tokio::test]
async fn direct_registration_abort_and_transport_error_stop_later_listeners() -> Result<()> {
    for abort in [false, true] {
        let fake = Arc::new(Fake {
            reject: true,
            ..Fake::default()
        });
        let mut app = ServerBuilder::new();
        app.on("written", operation)?;
        if abort {
            app.on_priority(
                "written",
                -199,
                stackpress_lib::handler(|_, _, _| Box::pin(async { Ok(Flow::Abort) })),
            )?;
        }
        webhooks::register(&mut app, Webhook::new(config(json!({})))?, fake.clone())?;
        let tail_calls = Arc::new(AtomicUsize::new(0));
        let observed = tail_calls.clone();
        app.on_priority(
            "written",
            -201,
            stackpress_lib::handler(move |_, _, _| {
                observed.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Ok(Flow::Continue) })
            }),
        )?;
        let server = app.build().await.map_err(|failure| failure.error)?;
        let outcome = server
            .emit("written", &mut request()?, &mut Response::new())
            .await;
        assert_eq!(outcome.is_err(), !abort);
        assert_eq!(fake.calls.lock().unwrap().len(), usize::from(!abort));
        assert_eq!(tail_calls.load(Ordering::SeqCst), 0);
    }
    Ok(())
}

/// Future drop witness, including the case where the executor itself cannot make progress.
struct Dropped(Arc<AtomicUsize>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
/// A reentrant transport owns a host; the nested event has no webhook, avoiding self-recursion.
struct Reentrant {
    server: stackpress_server::Server,
    entered: tokio::sync::Notify,
    dropped: Arc<AtomicUsize>,
}
impl Fetch for Reentrant {
    fn send<'a>(&'a self, _: FetchRequest<'a>) -> HandlerFuture<'a, Result<u16>> {
        Box::pin(async move {
            let _dropped = Dropped(self.dropped.clone());
            self.server.call("lookup", json!({})).await?;
            self.entered.notify_one();
            std::future::pending().await
        })
    }
}

#[test]
#[cfg(unix)]
fn reentrant_webhook_cancellation_has_process_watchdog() {
    assert!(super::proof::watchdog(false, "webhooks::reentrant_child"));
}

#[test]
#[ignore = "run by process watchdog parent"]
fn reentrant_child() -> Result<()> {
    std::fs::write(std::env::var("SLICE10_READY").unwrap(), b"ready")
        .map_err(Error::from_source)?;
    tokio::runtime::Builder::new_current_thread()
        .build()
        .map_err(Error::from_source)?
        .block_on(async {
            let mut app = ServerBuilder::new();
            app.on("lookup", operation)?;
            let fetch = Reentrant {
                server: app.build().await.map_err(|failure| failure.error)?,
                entered: tokio::sync::Notify::new(),
                dropped: Arc::new(AtomicUsize::new(0)),
            };
            let hook = Webhook::new(config(json!({})))?;
            let req = request()?;
            let mut res = Response::new();
            res.results(&json!({}))?;
            let mut send = Box::pin(webhooks::send(&fetch, &hook, &req, &res));
            tokio::select! {
                _ = &mut send => panic!("transport should remain pending"),
                _ = fetch.entered.notified() => {}
            }
            drop(send);
            assert_eq!(fetch.dropped.load(Ordering::SeqCst), 1);
            Ok(())
        })
}
