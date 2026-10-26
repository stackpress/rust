//! One bounded public decoder shared by HTTP and IPC consumers.
//!
//! A valid rejected operation is data, distinct from failed acquisition or an
//! invalid wire response. Only the host's explicit public error fields survive.

use super::{REVISION, SearchReply};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The exact non-success host envelope; private/unknown fields are rejected.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostError {
    /// Public HTTP/application status, from 400 through 599.
    pub code: u16,
    /// Safe public status label, including a deliberately empty label.
    pub status: String,
    /// Safe plain-text error, escaped by the UI rather than interpreted as HTML.
    pub error: String,
    /// Deliberately public field details, preserved as bounded JSON data.
    pub errors: BTreeMap<String, serde_json::Value>,
    /// Errors carry zero results; a nonzero total is invalid.
    pub total: u64,
}

impl HostError {
    /// Validate decoded metadata without treating an error as an empty success.
    ///
    /// # Errors
    /// Rejects success codes, nonzero totals and over-8192-byte status/error text.
    pub fn validate(&self) -> Result<(), ReplyDecodeError> {
        if !(400..=599).contains(&self.code)
            || self.total != 0
            || self.status.len() > 8192
            || self.error.len() > 8192
        {
            return Err(ReplyDecodeError::Invalid);
        }
        Ok(())
    }

    /// Project only pinned public fields from a native response envelope.
    ///
    /// Native diagnostics/results/stack are not copied into IPC. Required public
    /// fields must exist and retain their types, including an explicit zero total.
    ///
    /// # Errors
    /// Rejects missing/mistyped public fields or invalid host-error metadata.
    pub fn from_envelope(value: &serde_json::Value) -> Result<Self, ReplyDecodeError> {
        let mut public = serde_json::Map::new();
        for key in ["code", "status", "error", "errors", "total"] {
            public.insert(
                key.into(),
                value.get(key).cloned().ok_or(ReplyDecodeError::Invalid)?,
            );
        }
        let error: Self =
            serde_json::from_value(public.into()).map_err(|_| ReplyDecodeError::Invalid)?;
        error.validate()?;
        Ok(error)
    }

    /// Keep empty public error text useful without introducing native diagnostics.
    pub fn message(&self) -> &str {
        if !self.error.is_empty() {
            &self.error
        } else if !self.status.is_empty() {
            &self.status
        } else {
            "Search rejected"
        }
    }
}

/// A decoded operation response is either useful rows or a valid public rejection.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SearchOutcome {
    /// Successful validated public search result.
    Success(SearchReply),
    /// Valid host error, distinct from acquisition/protocol failure.
    Rejected(HostError),
}

/// Safe response-decoding failures; input bodies and native causes are never echoed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplyDecodeError {
    /// Malformed, missing, duplicate, unknown or inconsistent fields.
    Invalid,
    /// A successful reply's public contract revision drifted.
    ContractMismatch,
    /// Encoded reply exceeds the public one-MiB budget.
    Limit,
    /// HTTP status disagrees with the decoded success/error body.
    StatusMismatch,
}

impl std::fmt::Display for ReplyDecodeError {
    /// Expose actionable fixed messages without disclosing response bytes.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Invalid => "Invalid search response",
            Self::ContractMismatch => "Page version changed. Refresh to continue.",
            Self::Limit => "Search reply is too large",
            Self::StatusMismatch => "Search response status is inconsistent",
        })
    }
}
impl std::error::Error for ReplyDecodeError {}

/// Decode a complete bounded reply before publishing either rows or safe host text.
///
/// Pass Some(status) for HTTP and None for IPC. A non-2xx HTTP body must contain
/// its matching host error code; invalid/empty non-2xx output remains protocol
/// failure. No retry, fallback or reinterpretation of diagnostics occurs.
///
/// # Errors
/// Returns a size, shape, revision or HTTP/body agreement failure with a fixed
/// safe Display message. It never returns response bytes as an error message.
///
/// # Examples
///
/// ```
/// use stackpress_example_profile_ui_contracts::{decode_search_reply, SearchOutcome};
/// let body = r#"{"code":400,"status":"Bad Request","error":"Try another prefix.","errors":{},"total":0}"#;
/// assert!(matches!(decode_search_reply(body, Some(400))?, SearchOutcome::Rejected(_)));
/// # Ok::<(), stackpress_example_profile_ui_contracts::ReplyDecodeError>(())
/// ```
pub fn decode_search_reply(
    text: &str,
    http_status: Option<u16>,
) -> Result<SearchOutcome, ReplyDecodeError> {
    if text.len() > 1024 * 1024 {
        return Err(ReplyDecodeError::Limit);
    }
    let outcome: SearchOutcome =
        serde_json::from_str(text).map_err(|_| ReplyDecodeError::Invalid)?;
    match &outcome {
        SearchOutcome::Success(reply) => {
            if reply.revision != REVISION {
                return Err(ReplyDecodeError::ContractMismatch);
            }
            reply.validate().map_err(|_| ReplyDecodeError::Invalid)?;
            if http_status.is_some_and(|status| !(200..=299).contains(&status)) {
                return Err(ReplyDecodeError::StatusMismatch);
            }
        }
        SearchOutcome::Rejected(error) => {
            error.validate()?;
            if http_status.is_some_and(|status| status != error.code) {
                return Err(ReplyDecodeError::StatusMismatch);
            }
        }
    }
    Ok(outcome)
}

/// Acquisition/admission failures have fixed safe messages, independent of host data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFailure {
    /// The bounded local operation deadline expired.
    Timeout,
    /// Native admission is closed, full or already owns the requested ID.
    Busy,
    /// The selected operation deliberately produced no typed result.
    NoContent,
    /// Browser/IPC acquisition could not start.
    Start,
    /// The connection or native bridge is unavailable.
    Unavailable,
    /// Active work was cancelled or its permitted IPC entry was denied.
    CancelledOrDenied,
}

impl std::fmt::Display for TransportFailure {
    /// Avoid serializing native rejection causes into the product UI.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Timeout => "Search timed out. Try again.",
            Self::Busy => "Search busy. Try again.",
            Self::NoContent => "Search returned no content. Try again.",
            Self::Start => "Search could not start. Try again.",
            Self::Unavailable => "Search unavailable. Check the connection and try again.",
            Self::CancelledOrDenied => "Search cancelled or denied. Try again.",
        })
    }
}

impl TransportFailure {
    /// Admit only fixed native bridge tokens; unknown rejection text is never echoed.
    pub fn from_native_code(code: Option<&str>) -> Self {
        match code {
            Some("deadline") => Self::Timeout,
            Some("busy") => Self::Busy,
            Some("no-content" | "suppressed") => Self::NoContent,
            Some("unavailable") => Self::Unavailable,
            _ => Self::CancelledOrDenied,
        }
    }
}

/// Observable UI failure category, retained separately from its escaped message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchFailureKind {
    /// The operation host deliberately returned a valid rejection envelope.
    Host,
    /// Acquisition or bridge admission failed without a valid host envelope.
    Transport,
    /// Acquired output did not satisfy the pinned wire contract.
    Protocol,
}

impl SearchFailureKind {
    /// Stable semantic token for accessible presentation and integration tests.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Transport => "transport",
            Self::Protocol => "protocol",
        }
    }
}

/// Typed completion failure, shared by controller and both transport adapters.
#[derive(Clone, Debug, PartialEq)]
pub enum SearchFailure {
    /// Valid public error fields from a completed host operation.
    Host(HostError),
    /// No valid operation response was acquired.
    Transport(TransportFailure),
    /// Acquired bytes failed public response validation.
    Protocol(ReplyDecodeError),
}

impl SearchFailure {
    /// Retain failure identity even when safe messages happen to be the same.
    pub fn kind(&self) -> SearchFailureKind {
        match self {
            Self::Host(_) => SearchFailureKind::Host,
            Self::Transport(_) => SearchFailureKind::Transport,
            Self::Protocol(_) => SearchFailureKind::Protocol,
        }
    }
}

impl std::fmt::Display for SearchFailure {
    /// Prefix host rejection distinctly; all interpolated text is already public data.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Host(error) => write!(formatter, "Search rejected: {}", error.message()),
            Self::Transport(error) => error.fmt(formatter),
            Self::Protocol(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for SearchFailure {}
