//! Source SMTP envelope/message error fields with the native SDK cause retained.

use lettre::transport::smtp::Error;
use serde_json::{Value, json};
use std::fmt;

/// A source SMTP failure whose enumerable details survive native error downcasting.
#[derive(Debug)]
pub struct SmtpError {
    /// Source code, command, responseCode/response and any recipient rejection details.
    pub fields: Value,
    /// Source-shaped public explanation, including the last SMTP response when available.
    message: String,
    /// Original native protocol error; no SDK cause is suppressed.
    cause: Option<Error>,
}
impl SmtpError {
    /// Map a protocol response at its source phase while retaining the original cause.
    pub(crate) fn response(error: Error, message: &str, code: &str, command: &str) -> Self {
        use std::error::Error as _;
        let status = error.status().map(|value| value.to_string());
        let response = status.as_ref().map(|status| {
            format!(
                "{status} {}",
                error.source().map(ToString::to_string).unwrap_or_default()
            )
            .trim_end()
            .to_owned()
        });
        let mut fields = json!({"code":code,"command":command});
        if let Some(response) = &response {
            fields["response"] = json!(response);
        }
        if let Some(status) = status.and_then(|value| value.parse::<u16>().ok()) {
            fields["responseCode"] = json!(status);
        }
        Self {
            fields,
            message: response
                .map(|response| format!("{message}: {response}"))
                .unwrap_or_else(|| message.into()),
            cause: Some(error),
        }
    }
    /// Transfer the SDK cause when assembling the source aggregate rejection error.
    pub(crate) fn take_cause(&mut self) -> Option<Error> {
        self.cause.take()
    }
    /// Source's no-recipient API error occurs after MAIL FROM succeeds.
    pub(crate) fn no_recipients() -> Self {
        Self {
            fields: json!({"code":"EENVELOPE","command":"API"}),
            message: "Can't send mail - no recipients defined".into(),
            cause: None,
        }
    }
}
impl fmt::Display for SmtpError {
    /// Preserve the source explanation rather than replacing it with a generic transport status.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for SmtpError {
    /// Expose the SDK cause separately from the source-shaped fields.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_ref()
            .map(|error| error as &dyn std::error::Error)
    }
}
