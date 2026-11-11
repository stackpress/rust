//! Fake IO boundaries retain real option resolution, MIME compilation and event execution.

use serde_json::{Value as Json, json};
use stackpress_framework_email::{
    EmailConfig, EmailResult, MailFuture, NativeFactory, NativeSendMailOptions, SendMailOptions,
    SentMessageInfo, Transport, TransportFactory,
    backend::{
        ProcessBackend, ProviderFuture, SendmailRequest, SesClient, SesRequest, SmtpBackend,
        SmtpRequest,
    },
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Capture native SDK/process invocations without opening a connection or executable.
#[derive(Default)]
pub struct Seams {
    /// Captured SDK/process data remains private to the native test inspector.
    pub calls: Mutex<Vec<Json>>,
}
impl SmtpBackend for Seams {
    /// Preserve complete resolved options and encoded bytes at the real SMTP adapter boundary.
    fn send(&self, request: SmtpRequest) -> ProviderFuture<'_, SentMessageInfo> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(json!({"kind":"smtp","options":request.options.fields,"envelope":request.envelope,"message":String::from_utf8_lossy(&request.message)}));
            Ok(json!({"accepted":["recipient@example.test"],"rejected":[],"response":"250 fixture","extension":{"retained":true}}).into())
        })
    }
}
impl ProcessBackend for Seams {
    /// Observe exact sendmail program/argv/message mapping without executing it.
    fn run(&self, request: SendmailRequest) -> ProviderFuture<'_, Option<i32>> {
        Box::pin(async move {
            self.calls.lock().unwrap().push(json!({"kind":"sendmail","path":request.program,"args":request.args,"message":String::from_utf8_lossy(&request.message)}));
            Ok(Some(0))
        })
    }
}
impl SesClient for Seams {
    /// The source result projection receives the explicit fake SDK region.
    fn region(&self) -> ProviderFuture<'_, Option<String>> {
        Box::pin(async { Ok(Some("fixture-region".into())) })
    }
    /// Keep complete source SES command input behind the fake SDK boundary.
    fn send(&self, request: SesRequest) -> ProviderFuture<'_, Json> {
        Box::pin(async move {
            self.calls
                .lock()
                .unwrap()
                .push(json!({"kind":"ses","input":request.input}));
            Ok(json!({"MessageId":"fixture-id"}))
        })
    }
}
/// Complete provider binding proves pool/library gaps receive the full original configuration.
struct FullProvider(Arc<Seams>);
/// One owned pool/provider call; no actual native pool is opened by this fake.
struct FullTransport {
    config: EmailConfig,
    seams: Arc<Seams>,
}
impl TransportFactory for FullProvider {
    /// Record the unfiltered source section at its accepted library-capability binding.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        self.0
            .calls
            .lock()
            .unwrap()
            .push(json!({"kind":"provider","config":config}));
        Ok(Box::new(FullTransport {
            config: config.clone(),
            seams: self.0.clone(),
        }))
    }
}
impl Transport for FullTransport {
    /// Pass original message fields through without imposing a new provider DTO.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        self.send_native(options.into())
    }
    /// Observe native slot ownership as well as JSON extensions.
    fn send_native(&mut self, options: NativeSendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            self.seams.calls.lock().unwrap().push(json!({"kind":"provider-send","options":options.fields,"slots":options.content.keys().collect::<Vec<_>>()}));
            Ok(json!({"sourceConfig":self.config,"extension":{"retained":true}}).into())
        })
    }
}
/// Count original createTransport calls while retaining the production NativeFactory adapters.
pub struct Factory {
    /// Inert replacements for every external provider boundary.
    pub seams: Arc<Seams>,
    /// Number of per-call factory entries, including construction failures.
    pub created: AtomicUsize,
    /// Original mail fields observed before the native adapter consumes them.
    pub sent: Arc<Mutex<Vec<SendMailOptions>>>,
    /// Unfiltered source configuration seen by each factory call.
    pub configs: Mutex<Vec<EmailConfig>>,
    /// Deterministic construction failure for the native lifecycle error proof.
    pub fail: bool,
}
impl Factory {
    /// Construct only in-memory capture state; production default delivery drivers are replaced.
    pub fn new(fail: bool) -> Arc<Self> {
        Arc::new(Self {
            seams: Arc::default(),
            created: AtomicUsize::new(0),
            sent: Arc::default(),
            configs: Mutex::new(vec![]),
            fail,
        })
    }
}
/// Keep the real per-call mutable transport owned until its send future completes.
struct Observed {
    inner: Box<dyn Transport>,
    sent: Arc<Mutex<Vec<SendMailOptions>>>,
}
impl Transport for Observed {
    /// The event may call either native or ordinary send without changing the captured shape.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        self.send_native(options.into())
    }
    /// Capture fields before moving native content to its real adapter.
    fn send_native(&mut self, options: NativeSendMailOptions) -> MailFuture<'_> {
        self.sent.lock().unwrap().push(options.fields.clone());
        self.inner.send_native(options)
    }
}
impl TransportFactory for Factory {
    /// A fresh native transport is selected for every event invocation, including string config.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        self.created.fetch_add(1, Ordering::SeqCst);
        self.configs.lock().unwrap().push(config.clone());
        if self.fail {
            return Err(
                std::io::Error::other("provider fixture failure: SMTP_PRIVATE_SENTINEL").into(),
            );
        }
        let native = NativeFactory::default()
            .with_smtp(self.seams.clone())
            .with_process(self.seams.clone())
            .with_ses(self.seams.clone())
            .with_provider(Arc::new(FullProvider(self.seams.clone())));
        Ok(Box::new(Observed {
            inner: native.create(config)?,
            sent: self.sent.clone(),
        }))
    }
}
