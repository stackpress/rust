//! Buffered application output, explicit envelopes and one-attempt delivery.
//!
//! Mutable accessors return Result because every response edit is guarded by
//! delivery state. After a writer has been attempted (including cancellation),
//! `data_mut()?`/`headers_mut()?` reject edits to already observed output. Request
//! input fields remain directly mutable because they do not represent sent output.

//!
//! ```
//! use serde::Serialize;
//! use stackpress_lib::{Response, Result};
//! #[derive(Serialize)]
//! struct Profile<'a> {
//!     id: &'a str,
//! }
//! let mut res = Response::new();
//! res.results(&Profile { id: "abc123" })?;
//! assert_eq!(res.to_envelope().code, Some(200));
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

use crate::session::Session;
use crate::{
    Body, Error, ErrorKind, Headers, Nest, ParseLimits, PublicError, Result, SourceLocation,
    Status, Value,
};
use serde::Serialize;
use std::fmt;
use std::future::Future;

/// Delivery state distinguishes attempted, physical success and suppressed output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryState {
    /// Values may still be mutated and delivery has not been entered.
    Open,
    /// A callback was entered; interrupted/partly sent output is possible.
    Attempted,
    /// The callback/no-output finalization completed successfully.
    Sent,
    /// Output was deliberately suppressed.
    Stopped,
    /// A callback returned an error; no implicit retry is allowed.
    Failed,
}

/// An explicit partial result/status view, not a full response/native resource copy.
#[derive(Clone, Debug, Default)]
pub struct ResponseEnvelope {
    /// Absent status differs from an explicitly supplied code, including zero.
    pub code: Option<u16>,
    /// Optional status-label override.
    pub status: Option<String>,
    /// Explicit public error message, including an intentionally empty message.
    pub error: Option<String>,
    /// Public field errors; wire conversion still checks rich variants explicitly.
    pub errors: Option<Value>,
    /// In-process result retaining bytes/dates/files/explicit Shared values.
    pub results: Option<Value>,
    /// Total, retaining an explicit zero.
    pub total: Option<u64>,
    /// Source locations only when deliberately supplied/disclosed.
    pub stack: Option<Vec<SourceLocation>>,
}
impl ResponseEnvelope {
    /// Check metadata before Response commits any optional envelope field.
    fn validate_metadata(&self, limits: &ParseLimits) -> Result<()> {
        // Bound status/public-message/stack metadata before Response publishes any optional field.
        if self.status.as_ref().is_some_and(|s| s.len() > 8192)
            || self
                .error
                .as_ref()
                .is_some_and(|s| s.len() > limits.max_bytes)
            || self
                .stack
                .as_ref()
                .is_some_and(|s| s.len() > limits.max_entries)
        {
            return Err(Error::limit("envelope metadata limit exceeded"));
        }
        Ok(())
    }
    /// Select JSON fields explicitly, rejecting rich values at their qualified path.
    ///
    /// # Errors
    /// Returns unsupported non-JSON result/field data, never stringifying it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ResponseEnvelope, Value};
    /// let envelope = ResponseEnvelope {
    ///     total: Some(0),
    ///     results: Some(Value::Bool(false)),
    ///     ..Default::default()
    /// };
    /// assert_eq!(envelope.try_to_json()?["results"], false);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn try_to_json(&self) -> Result<serde_json::Value> {
        // Select presence explicitly so false, zero, empty objects and null are retained while
        // absent fields are omitted.
        let mut object = serde_json::Map::new();
        if let Some(code) = self.code {
            object.insert("code".into(), code.into());
        }
        if let Some(status) = &self.status {
            object.insert("status".into(), status.clone().into());
        }
        if let Some(error) = &self.error {
            object.insert("error".into(), error.clone().into());
        }
        // Both result and field-error values use the explicit rich-to-wire boundary with their
        // envelope field prefix.
        for (key, value) in [
            ("errors", self.errors.as_ref()),
            ("results", self.results.as_ref()),
        ] {
            if let Some(value) = value {
                let json = value.to_json().map_err(|error| {
                    // Retain the unsupported child suffix so diagnostics identify
                    // results.profile.native rather than only results.
                    let path = match error.path().filter(|v| !v.is_empty()) {
                        Some(path) => format!("{key}.{path}"),
                        None => key.into(),
                    };
                    error.at_path(path)
                })?;
                object.insert(key.into(), json);
            }
        }
        // Presence, not truthiness, decides whether total=0 is serialized.
        if let Some(total) = self.total {
            object.insert("total".into(), total.into());
        }
        // An explicitly disclosed stack is ordinary source metadata; private captured backtraces
        // remain absent.
        if let Some(stack) = &self.stack {
            object.insert("stack".into(), serde_json::to_value(stack)?);
        }
        Ok(serde_json::Value::Object(object))
    }
}

/// Isolated buffered output passed to a caller-owned native dispatcher.
///
/// It has no attempt state or owned diagnostic source. A dispatcher can borrow
/// a native writer in its FnOnce capture; changing the snapshot never changes
/// the framework's finalized values behind another alias.
#[derive(Clone)]
pub struct ResponseView {
    /// Application status; an HTTP adapter validates its native legality.
    pub status: Option<Status>,
    /// Exact buffered output.
    pub body: Body,
    /// Validated multi-value headers.
    pub headers: Headers,
    /// Cookie changes, with explicit array serialization policy.
    pub session: Session,
    /// Explicit public metadata.
    pub data: Nest,
    /// Public error/result view without developer diagnostics by default.
    pub envelope: ResponseEnvelope,
    /// Explicit MIME choice, if any.
    pub mimetype: Option<String>,
}
impl fmt::Debug for ResponseView {
    /// Redact buffered payloads/credentials even when a view is logged.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Output snapshots redact body/header/session payloads even when a native adapter logs the
        // view.
        f.debug_struct("ResponseView")
            .field("status", &self.status)
            .field("body_type", &self.body.type_name())
            .finish_non_exhaustive()
    }
}

/// Independently owned mutable output; no Clone can duplicate delivery ownership.
pub struct Response {
    /// Current exact output buffer; all mutable access is guarded by delivery state.
    body: Body,
    /// Optional application code/label, independent of dispatch outcome.
    status: Option<Status>,
    /// Validated repeated output fields, never comma-folded by the foundation.
    headers: Headers,
    /// Cookie revision values/scopes pending adapter-selected serialization.
    session: Session,
    /// Explicit metadata independent from the selected body/result.
    data: Nest,
    /// Intentionally public field validation details with an object-root invariant.
    errors: Nest,
    /// Intentionally disclosed error message, not an inferred private cause Display.
    public_error: Option<String>,
    /// Unique owned developer cause retained for inspection or consuming into_error.
    original_error: Option<Error>,
    /// Public locations only when explicitly supplied; native backtraces are never inferred.
    stack: Option<Vec<SourceLocation>>,
    /// Present result count, including a deliberate zero override.
    total: u64,
    /// Optional explicit output MIME selection, without native transport effects.
    mimetype: Option<String>,
    /// Unique output-attempt state; any attempted output freezes subsequent edits.
    delivery: DeliveryState,
    /// Chosen buffered output policy, revalidated before constructing a writer snapshot.
    limits: ParseLimits,
}
impl Default for Response {
    /// Construct unset application status and no output.
    fn default() -> Self {
        // Default output is independently owned and open, with no selected status or body.
        Self::new()
    }
}
impl Response {
    /// Construct output independent of any host or transport.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(Response::new().status().is_none());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // Use the default finite workload policy without choosing a transport or native writer.
        Self::with_limits(ParseLimits::default())
    }
    /// Construct with deliberate body/data budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// let response = Response::with_limits(stackpress_lib::ParseLimits::default());
    /// assert!(response.result().is_none());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn with_limits(limits: ParseLimits) -> Self {
        // Initialize mutable output and its collections under one selected finite policy.
        Self {
            body: Body::Empty,
            status: None,
            headers: Headers::new(),
            session: Session::new(),
            data: Nest::with_limits(limits),
            errors: Nest::with_limits(limits),
            public_error: None,
            original_error: None,
            stack: None,
            total: 0,
            mimetype: None,
            delivery: DeliveryState::Open,
            limits,
        }
    }
    /// Borrow actual buffered output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(matches!(response.body(), Body::Value(_)));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn body(&self) -> &Body {
        // Borrow the exact buffer without inventing a serialized or delivered representation.
        &self.body
    }
    /// Borrow structured results only; raw text/bytes remain distinct Body variants.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(response.result().unwrap().to_json()?["id"], 7);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn result(&self) -> Option<&Value> {
        // Only structured Body::Value has a rich result view; text and bytes remain explicit body
        // forms.
        if let Body::Value(value) = &self.body {
            Some(value)
        } else {
            None
        }
    }
    /// Inspect unset versus explicitly selected status.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(response.status(), Some(&Status::OK));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn status(&self) -> Option<&Status> {
        // An unset application status stays None instead of implying transport success.
        self.status.as_ref()
    }
    /// Inspect the explicit MIME choice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(response.mimetype(), Some("application/json"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn mimetype(&self) -> Option<&str> {
        // Borrow the explicitly selected type rather than guessing it from arbitrary payloads.
        self.mimetype.as_deref()
    }
    /// Inspect the total, including zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(response.total(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn total(&self) -> u64 {
        // Zero is a real result count and must not be treated as an absent override.
        self.total
    }
    /// Inspect the public error message only.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(response.error().is_none());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn error(&self) -> Option<&str> {
        // Expose only the intentional public message; the owned diagnostic cause has a separate
        // accessor.
        self.public_error.as_deref()
    }
    /// Inspect explicitly supplied stack locations; original diagnostics are separate.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(response.stack().is_none());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn stack(&self) -> Option<&[SourceLocation]> {
        // Source locations enter this public view only when a caller explicitly supplied them.
        self.stack.as_deref()
    }
    /// Inspect the original owned failure without converting it to a wire message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.apply_error(Error::new("private"))?;
    /// assert!(response.diagnostic().is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn diagnostic(&self) -> Option<&Error> {
        // Borrow the original owned error without cloning its source chain into another response.
        self.original_error.as_ref()
    }
    /// Inspect the delivery state, including interrupted attempts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(
    ///     response.delivery_state(),
    ///     stackpress_lib::DeliveryState::Open
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn delivery_state(&self) -> DeliveryState {
        // Expose attempted/interrupted/failed output distinctly from successful finalization.
        self.delivery
    }
    /// Legacy finalized convenience; Stopped is finalized without physical delivery.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(!response.sent());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn sent(&self) -> bool {
        // The compatibility predicate means finalized by send or stop; delivery_state gives the
        // precise state.
        matches!(self.delivery, DeliveryState::Sent | DeliveryState::Stopped)
    }
    /// Inspect a redirect by validated case-insensitive Location presence.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.redirect("/profiles")?;
    /// assert!(response.redirected());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn redirected(&self) -> bool {
        // A Location header indicates redirect intent; native status legality still belongs to an
        // adapter.
        self.headers.has("location")
    }
    /// Borrow immutable headers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.headers_mut()?.set("X-Id", "7")?;
    /// assert_eq!(response.headers().get("x-id"), Some("7"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn headers(&self) -> &Headers {
        // Borrow normalized multi-value headers without allowing output-state guards to be
        // bypassed.
        &self.headers
    }
    /// Mutate headers only while output is open.
    ///
    /// # Errors
    /// Rejects any already attempted/finalized output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.headers_mut()?.append("Set-Cookie", "theme=dark")?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn headers_mut(&mut self) -> Result<&mut Headers> {
        // Reject edits after any output attempt because an adapter may already have observed these
        // values.
        self.ensure_open()?;
        Ok(&mut self.headers)
    }
    /// Borrow cookie state/revisions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(response.session().is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn session(&self) -> &Session {
        // Borrow outgoing revisions without marking them delivered or invoking a cookie codec.
        &self.session
    }
    /// Mutate cookie state only while output is open.
    ///
    /// # Errors
    /// Rejects any already attempted/finalized output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.session_mut()?.set("theme", "dark", None)?;
    /// assert_eq!(response.session().len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn session_mut(&mut self) -> Result<&mut Session> {
        // Cookie changes are response output and must freeze at the same attempt boundary as the
        // body.
        self.ensure_open()?;
        Ok(&mut self.session)
    }
    /// Borrow public metadata.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.data_mut()?.set("trace", "local")?;
    /// assert!(response.data().has("trace"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn data(&self) -> &Nest {
        // Borrow response metadata independently from the selected result/body.
        &self.data
    }
    /// Mutate public metadata only while output is open.
    ///
    /// # Errors
    /// Rejects any already attempted/finalized output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.data_mut()?.set_path("meta.page", 1)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn data_mut(&mut self) -> Result<&mut Nest> {
        // Only open output may change metadata that will be copied to a native dispatcher.
        self.ensure_open()?;
        Ok(&mut self.data)
    }
    /// Borrow field errors without exposing delivery mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert!(response.errors().is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn errors(&self) -> &Nest {
        // Borrow explicit public field errors without exposing mutation of delivery state.
        &self.errors
    }
    /// Mutate explicit field errors while open.
    ///
    /// # Errors
    /// Rejects attempted/finalized output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.errors_mut()?.set("id", "required")?;
    /// assert!(response.errors().has("id"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn errors_mut(&mut self) -> Result<&mut Nest> {
        // Field errors are part of the output snapshot and cannot change after an attempt.
        self.ensure_open()?;
        Ok(&mut self.errors)
    }
    /// Guard every output mutation before serializer/callback side effects occur.
    fn ensure_open(&self) -> Result<()> {
        // One guard defines every public mutation boundary; Attempted includes cancelled writers.

        // Any attempt may have caused external output, so mutation cannot resume after
        // failure/cancellation.
        if self.delivery != DeliveryState::Open {
            return Err(Error::new("response output already attempted/finalized")
                .with_kind(ErrorKind::State));
        }
        Ok(())
    }
    /// Select an application code/custom message, not a dispatch result.
    ///
    /// # Errors
    /// Rejects finalized output or an oversized message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.status_code(201, None)?;
    /// assert_eq!(response.status(), Some(&Status::CREATED));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn status_code(&mut self, code: u16, message: Option<String>) -> Result<&mut Self> {
        // Guard output before staging a custom application status label.
        self.ensure_open()?;
        if message.as_ref().is_some_and(|value| value.len() > 8192) {
            return Err(Error::limit("status message limit exceeded"));
        }
        self.status = Some(message.map_or_else(
            || Status::from_code(code),
            |message| Status::new(code, message),
        ));
        Ok(self)
    }
    /// Set an exact body/MIME pair, selecting OK by default.
    ///
    /// # Errors
    /// Returns finalized state, invalid MIME or body budgets before mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.set("application/octet-stream", Body::Bytes(vec![0, 255]))?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set(&mut self, mime: &str, body: Body) -> Result<&mut Self> {
        // Validate MIME and body before atomically selecting the buffer, type and default OK
        // status.
        self.ensure_open()?;
        crate::data::validate_mime(mime)?;
        body.validate(&self.limits)?;
        // MIME/body checks succeeded; publish these fields together before returning the mutable
        // response.
        self.body = body;
        self.mimetype = Some(mime.into());
        self.status = Some(Status::OK);
        Ok(self)
    }
    /// Select HTML text.
    ///
    /// # Errors
    /// Returns finalized state or body limits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.html("<h1>Hello</h1>")?;
    /// assert_eq!(response.mimetype(), Some("text/html"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn html(&mut self, text: impl Into<String>) -> Result<&mut Self> {
        // Select text/html explicitly; no inferred conversion or native write occurs.
        self.set("text/html", Body::Text(text.into()))
    }
    /// Select XML text.
    ///
    /// # Errors
    /// Returns finalized state or body limits.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.xml("<profile/>")?;
    /// assert_eq!(response.mimetype(), Some("text/xml"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn xml(&mut self, text: impl Into<String>) -> Result<&mut Self> {
        // Select text/xml explicitly while retaining an exact owned text buffer.
        self.set("text/xml", Body::Text(text.into()))
    }
    /// Stage typed Serialize data under the selected budgets.
    fn serialized<T: Serialize + ?Sized>(&self, value: &T) -> Result<Value> {
        // Check output state before invoking user Serialize code, then validate its produced tree.
        self.ensure_open()?;
        Value::from_json_with(serde_json::to_value(value)?, &self.limits)
    }
    /// Serialize typed data as JSON text.
    ///
    /// # Errors
    /// Returns serializer, budget or finalized-state failures without mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.json(&serde_json::json!({"id": 7}))?;
    /// assert!(matches!(response.body(), Body::Text(_)));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn json<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self> {
        // JSON is buffered text here; serialization must succeed before the old response is
        // replaced.

        // Perform user serialization before replacing the existing body/status; failures preserve
        // the previous result.
        let value = self.serialized(value)?;
        self.set(
            "application/json",
            Body::Text(serde_json::to_string_pretty(
                &value.to_json_with(&self.limits)?,
            )?),
        )
    }
    /// Serialize typed structured results; total becomes one.
    ///
    /// # Errors
    /// Returns serializer, budget or finalized-state failures without mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// #[derive(serde::Serialize)]
    /// struct Profile {
    ///     id: u64,
    /// }
    /// response.results(&Profile { id: 7 })?;
    /// assert_eq!(response.total(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn results<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<&mut Self> {
        // Stage a typed Serialize result first; result mutation shares the checked rich-value
        // setter.
        self.results_value(self.serialized(value)?)
    }
    /// Retain rich in-process results without an implicit JSON conversion.
    ///
    /// # Errors
    /// Returns budget/finalized failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.results_value(Value::Bytes(vec![0, 255]))?;
    /// assert!(response.to_envelope().try_to_json().is_err());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn results_value(&mut self, value: Value) -> Result<&mut Self> {
        // Keep rich native/date/byte distinctions in process while selecting the conventional
        // results status/type.
        self.set("application/json", Body::Value(value))?;
        self.total = 1;
        Ok(self)
    }
    /// Serialize typed rows, requiring an array result and preserving total zero.
    ///
    /// # Errors
    /// Returns non-array serialization, resource or finalized-state failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.rows(&[serde_json::json!({"id": 7})], 0)?;
    /// assert_eq!(response.total(), 0);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn rows<T: Serialize + ?Sized>(&mut self, value: &T, total: u64) -> Result<&mut Self> {
        // Rows require an actual serialized array; arbitrary typed objects are not silently
        // wrapped.
        let value = self.serialized(value)?;
        // Reject non-array rows before setting body or total, rather than manufacturing an implicit
        // singleton list.
        if !matches!(value, Value::Array(_)) {
            return Err(Error::invalid("rows requires an array serialization"));
        }
        self.set("application/json", Body::Value(value))?;
        self.total = total;
        Ok(self)
    }
    /// Set total explicitly without changing output representation.
    ///
    /// # Errors
    /// Returns finalized-state failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.set_total(0)?;
    /// assert_eq!(response.total(), 0);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set_total(&mut self, total: u64) -> Result<&mut Self> {
        // Guard the override independently so zero remains a deliberate count after convenience
        // helpers.
        self.ensure_open()?;
        self.total = total;
        Ok(self)
    }
    /// Select a Found redirect and validated Location header.
    ///
    /// # Errors
    /// Rejects header injection or finalized output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.redirect("/profiles")?;
    /// assert_eq!(response.headers().get("location"), Some("/profiles"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn redirect(&mut self, location: &str) -> Result<&mut Self> {
        // Validate Location before selecting redirect status so an injection failure preserves
        // prior output.
        self.ensure_open()?;
        // A checked header mutation happens before Found status is selected; invalid framing
        // changes neither.
        self.headers.set("location", location)?;
        self.status = Some(Status::FOUND);
        Ok(self)
    }
    /// Set an explicitly public error without logging developer details.
    ///
    /// # Errors
    /// Rejects invalid field shape/budgets or finalized output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.set_error(PublicError {
    ///     code: 400,
    ///     status: "Bad Request".into(),
    ///     error: "Invalid id".into(),
    ///     errors: serde_json::json!({}),
    /// })?;
    /// assert_eq!(response.error(), Some("Invalid id"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set_error(&mut self, error: PublicError) -> Result<&mut Self> {
        // Validate public fields before replacing status/errors; no private source is inferred from
        // public text.
        self.ensure_open()?;
        let errors = crate::data::IntoNest::into_nest_with(error.errors, self.limits)?;
        if error.error.len() > self.limits.max_bytes || error.status.len() > 8192 {
            return Err(Error::limit("public error limit exceeded"));
        }
        self.status = Some(Status::new(error.code, error.status));
        self.public_error = Some(error.error);
        self.errors = errors;
        self.original_error = None;
        Ok(self)
    }
    /// Own the original diagnostic while disclosing only its chosen public view.
    ///
    /// # Errors
    /// Rejects finalized output or invalid public field details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.apply_error(Error::new("private").with_public_message("Try again"))?;
    /// assert!(response.diagnostic().is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn apply_error(&mut self, error: Error) -> Result<&mut Self> {
        // Preserve the original error even if its public-view application fails; successful
        // application retains it for into_error.

        // If the public view is invalid, attach the original error as its cause instead of losing
        // it.
        if let Err(failure) = self.set_error(error.public_view()) {
            return Err(failure.with_cause(error));
        }
        self.original_error = Some(error);
        Ok(self)
    }
    /// Set explicitly disclosed source locations rather than fabricate V8 frames.
    ///
    /// # Errors
    /// Rejects finalized output or excessive location counts.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.set_stack(vec![stackpress_lib::SourceLocation::caller()])?;
    /// assert!(response.to_envelope().stack.is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set_stack(&mut self, stack: Vec<SourceLocation>) -> Result<&mut Self> {
        // Only explicitly supplied source locations are candidates for public disclosure, under a
        // count budget.
        self.ensure_open()?;
        if stack.len() > self.limits.max_entries {
            return Err(Error::limit("stack location limit exceeded"));
        }
        self.stack = Some(stack);
        Ok(self)
    }
    /// Apply present envelope fields as replacements, with atomic validation.
    ///
    /// # Errors
    /// Rejects invalid fields/budgets/finalized output without partial mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.apply_envelope(ResponseEnvelope {
    ///     results: Some(Value::Bool(false)),
    ///     ..Default::default()
    /// })?;
    /// assert_eq!(response.result().and_then(Value::as_bool), Some(false));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn apply_envelope(&mut self, mut envelope: ResponseEnvelope) -> Result<&mut Self> {
        // Guard delivery first, then validate every fallible field before applying any optional
        // override.
        self.ensure_open()?;
        if let Some(value) = &envelope.results {
            value.validate(&self.limits)?;
        }
        // Take and validate field errors in temporary storage before publishing any other envelope
        // field.
        let errors = envelope
            .errors
            .take()
            .map(|value| Nest::from_value(value, self.limits))
            .transpose()?;
        envelope.validate_metadata(&self.limits)?;
        // A supplied code or label updates status; otherwise the existing selection is retained.
        if envelope.code.is_some() || envelope.status.is_some() {
            let code = envelope
                .code
                .or_else(|| self.status.as_ref().map(|s| s.code))
                .unwrap_or(0);
            self.status = Some(
                envelope
                    .status
                    .map_or_else(|| Status::from_code(code), |label| Status::new(code, label)),
            );
        }
        // An explicit public replacement clears the old private cause so it cannot describe
        // unrelated output.
        if let Some(error) = envelope.error {
            self.public_error = Some(error);
            self.original_error = None;
        }
        if let Some(errors) = errors {
            self.errors = errors;
        }
        // Present null/false/empty results are real overrides; only None preserves the existing
        // body.
        if let Some(result) = envelope.results {
            if matches!(result, Value::Object(_) | Value::Array(_)) {
                self.mimetype = Some("application/json".into());
            }
            self.body = Body::Value(result);
        }
        if let Some(total) = envelope.total {
            self.total = total;
        }
        if let Some(stack) = envelope.stack {
            self.stack = Some(stack);
        }
        Ok(self)
    }
    /// Clone a result/status view; only explicitly Shared native values share.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(response.to_envelope().code, Some(200));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn to_envelope(&self) -> ResponseEnvelope {
        // Clone ordinary rich values only; no delivery state, native writer or owned private cause
        // enters the envelope.
        ResponseEnvelope {
            code: self.status.as_ref().map(|s| s.code),
            status: self.status.as_ref().map(|s| s.message.to_string()),
            error: self.public_error.clone(),
            errors: Some(self.errors.value().clone()),
            results: self.body.clone().into_value(),
            total: Some(self.total),
            stack: self.stack.clone(),
        }
    }
    /// Move result/status values without duplicating delivery state or owned causes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// assert_eq!(response.into_envelope().code, Some(200));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn into_envelope(self) -> ResponseEnvelope {
        // Move public status/result values out while consuming the original output owner.
        ResponseEnvelope {
            code: self.status.as_ref().map(|s| s.code),
            status: self.status.map(|s| s.message.into_owned()),
            error: self.public_error,
            errors: Some(self.errors.into_value()),
            results: self.body.into_value(),
            total: Some(self.total),
            stack: self.stack,
        }
    }
    /// Consume the original failure when present; otherwise derive one from public fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// response.apply_error(Error::new("original cause"))?;
    /// assert_eq!(response.into_error().message(), "original cause");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn into_error(self) -> Error {
        // Prefer the original owned source chain; otherwise derive a diagnostic only from supplied
        // public fields.

        // Consuming a response with an original failure transfers its exact cause chain unchanged.
        if let Some(error) = self.original_error {
            return error;
        }
        let error = Error::new(self.public_error.unwrap_or_else(|| "Unknown Error".into()))
            .with_code(self.status.map_or(500, |s| s.code));
        match self.errors.to_json() {
            Ok(fields) => error.with_errors(fields),
            Err(cause) => error.with_cause(cause),
        }
    }
    /// Clone data-only output for a caller-owned dispatcher.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// let mut view = response.view();
    /// // Ordinary snapshot data is independent of the response's buffered values.
    /// view.data.set("snapshot", true)?;
    /// assert!(!response.data().has("snapshot"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn view(&self) -> ResponseView {
        // Copy buffered data only; the original remains the unique owner of its native attempt
        // state.
        ResponseView {
            status: self.status.clone(),
            body: self.body.clone(),
            headers: self.headers.clone(),
            session: self.session.clone(),
            data: self.data.clone(),
            envelope: self.to_envelope(),
            mimetype: self.mimetype.clone(),
        }
    }
    /// Finalize without native output.
    ///
    /// # Errors
    /// Rejects failed/interrupted output; successful/stopped repetition is a no-op.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// # async fn example() -> Result<()> {
    /// let mut response = Response::new();
    /// // Finalization freezes mutation even when no native writer is supplied.
    /// response.dispatch().await?;
    /// assert!(response.sent());
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub async fn dispatch(&mut self) -> Result<()> {
        // No-output finalization still consumes the same one-attempt state as a native writer.
        self.dispatch_with(|_| async { Ok(()) }).await
    }
    /// Enter one Send native-output callback using an isolated buffered snapshot.
    ///
    /// # Errors
    /// Returns original writer/state failures. Failure/cancellation does not retry.
    ///
    /// # Panics
    /// A writer panic propagates and leaves output Attempted; bytes may already have been written.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// async fn write(_: ResponseView) -> Result<()> {
    ///     Ok(())
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut response = Response::new();
    /// // Await this writer once; failure or cancellation must not trigger an implicit retry.
    /// response.dispatch_with(write).await?;
    /// assert!(response.sent());
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub async fn dispatch_with<F, Fut>(&mut self, dispatcher: F) -> Result<()>
    where
        F: FnOnce(ResponseView) -> Fut + Send,
        Fut: Future<Output = Result<()>> + Send,
    {
        // Successful/explicitly stopped repetition is a no-op; failed/interrupted output is never
        // retried.
        if matches!(self.delivery, DeliveryState::Sent | DeliveryState::Stopped) {
            return Ok(());
        }
        self.ensure_open()?;
        // A caller can replace a mutable store; recheck the response budget before output.
        self.body.validate(&self.limits)?;
        self.data.value().validate(&self.limits)?;
        self.errors.value().validate(&self.limits)?;
        self.headers.validate(&self.limits)?;
        let snapshot = self.view();
        // Consume the send attempt before callback entry/suspension; cancellation must not make a
        // retry appear safe.
        self.delivery = DeliveryState::Attempted;
        // Await the caller-owned writer under the exclusive response borrow; no framework lock is
        // held.
        let result = dispatcher(snapshot).await;
        // An error may follow partial native output: record Failed without claiming nothing was
        // sent.
        self.delivery = if result.is_ok() {
            DeliveryState::Sent
        } else {
            DeliveryState::Failed
        };
        result
    }
    /// Suppress output and freeze further mutation without invoking a writer.
    ///
    /// # Errors
    /// Rejects already attempted/failed output.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     Body, Error, PublicError, Response, ResponseEnvelope, ResponseView, Result, Status,
    /// #     Value,
    /// # };
    /// let mut response = Response::new();
    /// response.results(&serde_json::json!({"id": 7}))?;
    /// // Suppressing output also freezes the response's guarded mutable accessors.
    /// response.stop()?;
    /// assert!(response.data_mut().is_err());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn stop(&mut self) -> Result<()> {
        // Stopping is explicit no-output finalization and prevents later mutation or an implicit
        // writer retry.
        if self.delivery == DeliveryState::Stopped {
            return Ok(());
        }
        self.ensure_open()?;
        self.delivery = DeliveryState::Stopped;
        Ok(())
    }
}
impl fmt::Debug for Response {
    /// Redact buffered public/private values rather than leak them through Debug.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Inspect state/representation without leaking buffered response or cookie data.
        f.debug_struct("Response")
            .field("status", &self.status)
            .field("body_type", &self.body.type_name())
            .field("delivery", &self.delivery)
            .finish_non_exhaustive()
    }
}
