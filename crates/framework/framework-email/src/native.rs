//! Compiled source transport adapters with explicit native-provider fallback.

use crate::{
    EmailConfig, EmailResult, MailFuture, MessageOutput, NativeSendMailOptions, SendMailOptions,
    SentMessageInfo, Transport, TransportFactory,
};
use crate::{
    backend::{
        LettreBackend, NativeProcessBackend, ProcessBackend, SesClient, SmtpBackend, SmtpRequest,
    },
    compose::{metadata, mime, normalize_json},
    content::{ContentResolver, NativeContentResolver},
    options::{TransportKind, TransportOptions, flag},
};
use serde_json::json;
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, ReadBuf};

mod capabilities;
mod sendmail;
mod ses;
use capabilities::{unsupported, unsupported_config, unsupported_message};

/// Native transport selection. Providers replace unsupported SDK capabilities explicitly.
#[derive(Clone)]
pub struct NativeFactory {
    /// File/URL IO seam; native readers are owned directly by each message.
    resolver: Arc<dyn ContentResolver>,
    /// Approved lettre SDK seam, replaceable for no-delivery testing.
    smtp: Arc<dyn SmtpBackend>,
    /// Source sendmail process seam, replaceable without spawning a real process.
    process: Arc<dyn ProcessBackend>,
    /// Native SESv2 client/command binding, corresponding to source SES handles.
    ses: Option<Arc<dyn SesClient>>,
    /// Complete original config/options go here when built-in support is insufficient.
    provider: Option<Arc<dyn TransportFactory>>,
}

impl Default for NativeFactory {
    /// Select compiled native providers without opening sockets or spawning processes.
    fn default() -> Self {
        Self {
            resolver: Arc::new(NativeContentResolver),
            smtp: Arc::new(LettreBackend),
            process: Arc::new(NativeProcessBackend),
            ses: None,
            provider: None,
        }
    }
}

impl NativeFactory {
    /// Bind file/URL resolution; this is also the deterministic URL/file test seam.
    pub fn with_content(mut self, resolver: Arc<dyn ContentResolver>) -> Self {
        self.resolver = resolver;
        self
    }
    /// Replace the SMTP SDK call without changing source option mapping.
    pub fn with_smtp(mut self, smtp: Arc<dyn SmtpBackend>) -> Self {
        self.smtp = smtp;
        self
    }
    /// Replace process execution while retaining exact executable/argument mapping.
    pub fn with_process(mut self, process: Arc<dyn ProcessBackend>) -> Self {
        self.process = process;
        self
    }
    /// Supply the source-required native SES client/command callbacks.
    pub fn with_ses(mut self, ses: Arc<dyn SesClient>) -> Self {
        self.ses = Some(ses);
        self
    }
    /// Bind full native support for source capabilities lettre cannot represent.
    pub fn with_provider(mut self, provider: Arc<dyn TransportFactory>) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Preserve one complete-provider construction or the caller's named capability error.
    fn provider_transport(
        &self,
        config: &EmailConfig,
        reason: &str,
    ) -> EmailResult<Box<dyn Transport>> {
        // The provider receives the original section, never the partially resolved options.
        match &self.provider {
            Some(provider) => provider.create(config),
            None => Err(unsupported(reason)),
        }
    }
}

impl TransportFactory for NativeFactory {
    /// Resolve source constructor defaults once per call; unsupported settings never disappear.
    fn create(&self, config: &EmailConfig) -> EmailResult<Box<dyn Transport>> {
        let options = match TransportOptions::resolve(config) {
            Ok(options) => options,
            Err(error)
                if matches!(config, EmailConfig::ConnectionString(_))
                    || matches!(config,EmailConfig::Options(fields) if flag(fields,"url")) =>
            {
                return self
                    .provider_transport(config, &format!("legacy connection URL syntax: {error}"));
            }
            Err(error) => return Err(error),
        };
        if let Some(reason) = unsupported_config(&options) {
            return self.provider_transport(config, &reason);
        }
        if options.kind == TransportKind::Ses && self.ses.is_none() {
            return self
                .provider_transport(config, "SES.sesClient/SendEmailCommand native binding");
        }
        Ok(Box::new(NativeTransport {
            factory: self.clone(),
            config: config.clone(),
            options,
        }))
    }
}

/// One source invocation; it owns its selected adapter and never shares mutable transport state.
struct NativeTransport {
    /// Shared immutable IO/provider bindings; the mutable transport remains per-call.
    factory: NativeFactory,
    /// Original section forwarded intact if a message needs the complete provider.
    config: EmailConfig,
    /// Resolved constructor defaults used only by the selected built-in adapter.
    options: TransportOptions,
}

impl Transport for NativeTransport {
    /// Ordinary source options are lifted into the same native adapter path.
    fn send(&mut self, options: SendMailOptions) -> MailFuture<'_> {
        self.send_native(options.into())
    }

    /// Preserve all native options; fallback happens before consuming any reader or performing IO.
    fn send_native(&mut self, mut mail: NativeSendMailOptions) -> MailFuture<'_> {
        Box::pin(async move {
            // These three source transport settings override per-message values, including false.
            for key in [
                "disableFileAccess",
                "disableUrlAccess",
                "normalizeHeaderKey",
            ] {
                if let Some(value) = self.options.fields.get(key) {
                    mail.fields.insert(key.into(), value.clone());
                }
            }
            if let Some(reason) = unsupported_message(&mail, self.options.kind) {
                if let Some(provider) = &self.factory.provider {
                    return provider.create(&self.config)?.send_native(mail).await;
                }
                return Err(unsupported(&reason));
            }
            let meta = match metadata(&mail.fields) {
                Ok(meta) => meta,
                Err(error) => {
                    if let Some(provider) = &self.factory.provider {
                        return provider.create(&self.config)?.send_native(mail).await;
                    }
                    return Err(io::Error::other(format!(
                        "native provider required for message address/metadata syntax: {error}"
                    ))
                    .into());
                }
            };
            match self.options.kind {
                TransportKind::Json => {
                    let data = normalize_json(mail, &meta, &self.factory.resolver).await?;
                    let message = if flag(&self.options.fields, "skipEncoding") {
                        data
                    } else {
                        json!(serde_json::to_string(&data)?)
                    };
                    Ok(json!({"envelope":meta.envelope,"messageId":meta.message_id,"message":message}).into())
                }
                TransportKind::Stream => {
                    let mut result = SentMessageInfo::from(
                        json!({"envelope":meta.envelope,"messageId":meta.message_id}),
                    );
                    if flag(&self.options.fields, "buffer") {
                        result.message = Some(MessageOutput::Buffer(
                            mime(mail, &meta, &self.factory.resolver, true).await?,
                        ));
                    } else {
                        // Source non-buffer results return before content stream errors occur.
                        let resolver = self.factory.resolver.clone();
                        result.message =
                            Some(MessageOutput::Stream(Box::pin(DeferredMessage::new(
                                Box::pin(async move { mime(mail, &meta, &resolver, true).await }),
                            ))));
                    }
                    Ok(result)
                }
                TransportKind::Smtp | TransportKind::Pool => {
                    let bytes = mime(mail, &meta, &self.factory.resolver, false).await?;
                    let mut result = self
                        .factory
                        .smtp
                        .send(SmtpRequest {
                            options: self.options.smtp(),
                            envelope: meta.smtp_envelope.clone(),
                            message: bytes,
                        })
                        .await?;
                    // Source adds envelope/Message-ID to the provider's existing info object.
                    if let Some(fields) = result.fields.as_object_mut() {
                        fields.insert("envelope".into(), json!({"from":meta.smtp_envelope["from"],"to":meta.smtp_envelope["to"]}));
                        fields.insert("messageId".into(), json!(meta.message_id));
                    }
                    Ok(result)
                }
                TransportKind::Sendmail => sendmail::send(self, mail, meta).await,
                TransportKind::Ses => ses::send(self, mail, meta).await,
            }
        })
    }
}

/// Deferred MIME preparation retains source stream error timing without detaching a task.
struct DeferredMessage {
    /// Pending compilation is retained; completion clears it before exposing bytes or an error.
    future: Option<crate::content::ContentFuture<'static>>,
    /// Compiled bytes are immutable once available and remain owned until stream drop.
    bytes: Vec<u8>,
    /// Next unread byte, always within 0..=bytes.len().
    offset: usize,
}
impl DeferredMessage {
    /// Own the unpolled compiler until a caller first reads the stream result.
    fn new(future: crate::content::ContentFuture<'static>) -> Self {
        Self {
            future: Some(future),
            bytes: Vec::new(),
            offset: 0,
        }
    }
}
impl AsyncRead for DeferredMessage {
    /// Poll once into owned bytes, then satisfy ordinary incremental AsyncRead calls.
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if let Some(future) = &mut self.future {
            match future.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => {
                    // Never poll a completed future again, including after a read error.
                    self.future = None;
                    match result {
                        Ok(bytes) => self.bytes = bytes,
                        Err(error) => return Poll::Ready(Err(io::Error::other(error))),
                    }
                }
            }
        }
        // Advance only by bytes copied; partial reads and repeated EOF retain the cursor bound.
        let count = (self.bytes.len() - self.offset).min(buf.remaining());
        buf.put_slice(&self.bytes[self.offset..self.offset + count]);
        self.offset += count;
        Poll::Ready(Ok(()))
    }
}
