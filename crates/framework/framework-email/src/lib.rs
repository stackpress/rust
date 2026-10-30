//! Source-shaped email configuration and per-invocation native transport ownership.
//!
//! Direct SMTP, JSON, stream, sendmail and SES adapters use owned native providers.
//! Pool and SDK/compiler capabilities outside lettre are explicitly provider-bound.
//! The optional `server` feature exposes the source `email-send` event and registrar.
//! Factories receive all configured options and transports return unfiltered provider info.
//!
//! ```
//! # async fn example() -> stackpress_framework_email::EmailResult<()> {
//! use std::sync::Arc;
//! use stackpress_framework_email::{EmailConfig, NativeFactory, Service};
//! let config = serde_json::from_value::<EmailConfig>(serde_json::json!({
//!     "jsonTransport": true, "skipEncoding": true
//! }))?;
//! let service = Service::new(config, Arc::new(NativeFactory::default()));
//! let options = serde_json::from_value(serde_json::json!({"text":"Hello"}))?;
//! let info = service.send(options).await?;
//! assert_eq!(info["message"]["text"], "Hello");
//! # Ok(()) }
//! ```

pub mod backend;
pub mod compose;
mod config;
pub mod content;
#[cfg(feature = "server")]
pub mod events;
mod message;
mod native;
pub mod options;
#[cfg(feature = "server")]
pub mod plugin;
mod service;
mod smtp_error;
pub mod tasks;

pub use config::EmailConfig;
pub use message::{Content, MailReader, MessageOutput, NativeSendMailOptions, SentMessageInfo};
pub use native::NativeFactory;
pub use service::{MailFuture, Service, Transport, TransportFactory};
pub use smtp_error::SmtpError;

/// Native error equivalent of source throws/callback rejection, retaining its cause.
pub type EmailError = Box<dyn std::error::Error + Send + Sync + 'static>;
/// Email operations do not invent a provider status or suppress failures.
pub type EmailResult<T> = Result<T, EmailError>;
/// Source message fields and extension options, forwarded unchanged to the adapter.
pub type SendMailOptions = serde_json::Map<String, serde_json::Value>;
