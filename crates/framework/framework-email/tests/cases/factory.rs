//! Ownership proof: each concurrent send has a Send, deliberately non-Sync transport.

use serde_json::{Value, json};
use stackpress_framework_email::{
    EmailConfig, EmailResult, MailFuture, SendMailOptions, Service, Transport, TransportFactory,
};
use std::{
    cell::Cell,
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::sync::{mpsc, oneshot};

/// Observable construction, config and destruction, independent of execution timing.
struct Factory {
    entered: mpsc::UnboundedSender<(usize, oneshot::Sender<()>)>,
    configs: Mutex<Vec<EmailConfig>>,
    created: AtomicUsize,
    dropped: Arc<AtomicUsize>,
    fail_create: bool,
    fail_send: bool,
}

/// Cell makes the transport non-Sync without making it non-Send.
struct FakeTransport {
    id: Cell<usize>,
    entered: mpsc::UnboundedSender<(usize, oneshot::Sender<()>)>,
    dropped: Arc<AtomicUsize>,
    fail_send: bool,
}

impl TransportFactory for Factory {
    /// Capture exactly what the service gives the source createTransport boundary.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        if self.fail_create {
            return Err(io::Error::other("create failed").into());
        }
        self.configs.lock().unwrap().push(config.clone());
        let id = self.created.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(FakeTransport {
            id: Cell::new(id),
            entered: self.entered.clone(),
            dropped: self.dropped.clone(),
            fail_send: self.fail_send,
        }))
    }
}

impl Transport for FakeTransport {
    /// Await an explicit release barrier to prove concurrent calls have separate handles.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            let (release, wait) = oneshot::channel();
            self.entered.send((self.id.get(), release)).unwrap();
            wait.await?;
            if self.fail_send {
                return Err(io::Error::other("send failed").into());
            }
            Ok(
                json!({"providerId":self.id.get(), "options":options, "accepted":["fixture"], "extension":{"raw":true}}).into(),
            )
        })
    }
}

impl Drop for FakeTransport {
    /// Destruction is observable after completion or cancellation.
    fn drop(&mut self) {
        self.dropped.fetch_add(1, Ordering::SeqCst);
    }
}

/// Construct a controlled factory without any live transport.
fn factory(
    fail_create: bool,
    fail_send: bool,
) -> (
    Arc<Factory>,
    mpsc::UnboundedReceiver<(usize, oneshot::Sender<()>)>,
) {
    let (entered, receiver) = mpsc::unbounded_channel();
    (
        Arc::new(Factory {
            entered,
            configs: Mutex::new(vec![]),
            created: AtomicUsize::new(0),
            dropped: Arc::new(AtomicUsize::new(0)),
            fail_create,
            fail_send,
        }),
        receiver,
    )
}

/// Join two spawned calls: the compiler also proves the whole Service future is Send.
#[tokio::test]
async fn concurrent_calls_get_distinct_owned_transports_and_unfiltered_results() {
    let (factory, mut receiver) = factory(false, false);
    let config = EmailConfig::Options(
        json!({"pool":true,"custom":false})
            .as_object()
            .unwrap()
            .clone(),
    );
    let service = Service::new(config.clone(), factory.clone());
    let options = json!({"from":"sender@example.test", "to":["recipient@example.test"], "attachments":[{"path":"/fixture", "href":"https://example.test/fixture"}], "headers":{"X-Test":"yes"}, "extension":{"nested":false}}).as_object().unwrap().clone();
    let first_service = service.clone();
    let first_options = options.clone();
    let first = tokio::spawn(async move { first_service.send(first_options).await });
    let second_options = options.clone();
    let second = tokio::spawn(async move { service.send(second_options).await });
    let (id1, release1) = receiver.recv().await.unwrap();
    let (id2, release2) = receiver.recv().await.unwrap();
    assert_ne!(id1, id2);
    assert_eq!(factory.created.load(Ordering::SeqCst), 2);
    assert_eq!(factory.dropped.load(Ordering::SeqCst), 0);
    release1.send(()).unwrap();
    release2.send(()).unwrap();
    for result in [
        first.await.unwrap().unwrap(),
        second.await.unwrap().unwrap(),
    ] {
        assert_eq!(result["options"], Value::Object(options.clone()));
        assert_eq!(result["extension"], json!({"raw":true}));
    }
    assert_eq!(
        *factory.configs.lock().unwrap(),
        vec![config.clone(), config]
    );
    assert_eq!(factory.dropped.load(Ordering::SeqCst), 2);
}

/// Source factory/send errors are propagated without a success DTO or retry.
#[tokio::test]
async fn construction_and_send_failures_retain_errors_and_release_handles() {
    let (failed, _) = factory(true, false);
    let service = Service::new(EmailConfig::Options(Default::default()), failed.clone());
    assert_eq!(
        service
            .send(Default::default())
            .await
            .unwrap_err()
            .to_string(),
        "create failed"
    );
    assert_eq!(failed.created.load(Ordering::SeqCst), 0);

    let (failed, mut receiver) = factory(false, true);
    let service = Service::new(EmailConfig::Options(Default::default()), failed.clone());
    let task = tokio::spawn(async move { service.send(Default::default()).await });
    receiver.recv().await.unwrap().1.send(()).unwrap();
    assert_eq!(task.await.unwrap().unwrap_err().to_string(), "send failed");
    assert_eq!(failed.created.load(Ordering::SeqCst), 1);
    assert_eq!(failed.dropped.load(Ordering::SeqCst), 1);
}

/// Cancelling the native send destroys its owned handle after a join barrier.
#[tokio::test]
async fn cancellation_reaps_transport_without_retry() {
    let (factory, mut receiver) = factory(false, false);
    let service = Service::new(EmailConfig::Options(Default::default()), factory.clone());
    let task = tokio::spawn(async move { service.send(Default::default()).await });
    let (_, _release) = receiver.recv().await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(factory.dropped.load(Ordering::SeqCst), 1);
    assert_eq!(factory.created.load(Ordering::SeqCst), 1);
}

/// The approved native library constructs a Send SMTP handle without network IO.
#[test]
fn approved_lettre_builder_has_native_send_ownership() {
    fn assert_send<T: Send>(_: T) {}
    // Builder construction alone performs no SMTP negotiation or message delivery.
    let smtp = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::builder_dangerous("localhost")
        .port(25)
        .build::<lettre::Tokio1Executor>();
    assert_send(smtp);
}
