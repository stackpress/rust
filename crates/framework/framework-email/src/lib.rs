//! Source-shaped email configuration and per-invocation native transport ownership.
//!
//! M0 establishes the factory/Send and supervised-task contracts. Native transport
//! adapters and server event registration are later milestones, not claimed here.
//! Factories receive all configured options and transports return unfiltered provider info.

mod config;
mod service;
pub mod tasks;

pub use config::EmailConfig;
pub use service::{MailFuture, Service, Transport, TransportFactory};

/// Native error equivalent of source throws/callback rejection, retaining its cause.
pub type EmailError = Box<dyn std::error::Error + Send + Sync + 'static>;
/// Email operations do not invent a provider status or suppress failures.
pub type EmailResult<T> = Result<T, EmailError>;
/// Source message fields and extension options, forwarded unchanged to the adapter.
pub type SendMailOptions = serde_json::Map<String, serde_json::Value>;
/// Source provider result, with all transport-specific fields retained.
pub type SentMessageInfo = serde_json::Value;
