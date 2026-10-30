//! Per-call factory and mutable transport borrow keep native handles owned across await.

use crate::{EmailConfig, EmailResult, SendMailOptions, SentMessageInfo};
use std::{future::Future, pin::Pin, sync::Arc};

/// A native send may borrow its per-call transport, but must move between executor threads.
pub type MailFuture<'a> = Pin<Box<dyn Future<Output = EmailResult<SentMessageInfo>> + Send + 'a>>;

/// One transport instance for one source createTransport/sendMail invocation.
///
/// Only Send is required: an individual transport need not be Sync or Clone.
///
/// ```compile_fail
/// use std::rc::Rc;
/// use stackpress_framework_email::{MailFuture, SendMailOptions, Transport};
/// struct LocalTransport(Rc<()>);
/// impl Transport for LocalTransport {
///     fn send(&mut self, _: SendMailOptions) -> MailFuture<'_> {
///         Box::pin(async { Ok(serde_json::Value::Null) })
///     }
/// }
/// ```
///
/// ```compile_fail
/// use std::rc::Rc;
/// use stackpress_framework_email::{MailFuture, SendMailOptions, Transport};
/// struct Sender;
/// impl Transport for Sender {
///     fn send(&mut self, _: SendMailOptions) -> MailFuture<'_> {
///         let local = Rc::new(());
///         Box::pin(async move {
///             std::future::pending::<()>().await;
///             drop(local);
///             Ok(serde_json::Value::Null)
///         })
///     }
/// }
/// ```
pub trait Transport: Send {
    /// Send the owned message and preserve the provider's info or error.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_>;
}

/// Host-shared construction boundary replacing NodeMailer's createTransport(config).
pub trait TransportFactory: Send + Sync {
    /// Create an independent transport for each invocation without borrowing parent state.
    ///
    /// # Errors
    /// Propagates provider/config construction errors without substituting a fallback.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>>;
}

/// Direct email mechanism; native factory state is not serializable configuration.
#[derive(Clone)]
pub struct Service {
    /// Exact source section, read by the factory on each call.
    config: EmailConfig,
    /// Thread-safe construction handle, not a globally shared per-call transport.
    factory: Arc<dyn TransportFactory>,
}

impl Service {
    /// Bind source options and a native provider/fake factory without sending anything.
    pub fn new(config: EmailConfig, factory: Arc<dyn TransportFactory>) -> Self {
        // Selection has no delivery side effect and adds no startup config policy.
        Self { config, factory }
    }

    /// Create a transport per invocation, await sending, then release that transport.
    ///
    /// # Errors
    /// Propagates factory and send errors. Dropping this future drops its transport;
    /// remote effects already performed are not rolled back or retried.
    pub async fn send(&self, options: SendMailOptions) -> EmailResult<SentMessageInfo> {
        // Keep the mutable transport local; concurrent calls never share this handle.
        let mut transport = self.factory.create(&self.config)?;
        transport.send(options).await
    }
}
