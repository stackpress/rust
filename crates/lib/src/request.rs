//! Independently constructible input, explicit source precedence and one-attempt loading.
//!
//! ```
//! use stackpress_lib::{Body, LoadResult, Request, RequestView, Result};
//! async fn load(_: RequestView) -> Result<LoadResult> {
//!     Ok(LoadResult {
//!         body: Some(Body::Text("loaded".into())),
//!         post: None,
//!     })
//! }
//! # async fn example() -> Result<()> {
//! let mut req = Request::builder()
//!     .url("/profiles?q=Ada")
//!     .data(serde_json::json!({"profile": {"name": "Ada"}}))
//!     .build()?;
//! req.load_with(load).await?;
//! assert_eq!(
//!     req.data.path_as::<String>("profile.name")?,
//!     Some("Ada".into())
//! );
//! #     Ok(())
//! # }
//! ```

use crate::data::{IntoNest, parse_query};
use crate::session::{Session, cookie};
use crate::{Body, Error, ErrorKind, Headers, Method, Nest, ParseLimits, Result};
use serde::Serialize;
use std::fmt;
use std::future::Future;

/// One-attempt state; interruption remains Attempted and cannot implicitly retry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadState {
    /// A reader has not been entered.
    Unloaded,
    /// A reader was entered; it may currently be pending or have been dropped.
    Attempted,
    /// Loading/finalization completed successfully.
    Loaded,
    /// A reader or staged-result validation failed.
    Failed,
}
/// Staged updates; None means absent, not an empty/null body.
#[derive(Default)]
pub struct LoadResult {
    /// Replacement body when supplied.
    pub body: Option<Body>,
    /// New post fields overlaid into both post and aggregate operation data.
    pub post: Option<Nest>,
}
/// An isolated data snapshot supplied to a loader, with no native read attempt state.
///
/// Owning the snapshot allows ordinary named async functions and FnOnce closures
/// to borrow a Send native reader from their caller without a static callback
/// requirement. Snapshot mutation cannot partially mutate the original request.
#[derive(Clone)]
pub struct RequestView {
    /// Aggregate operation data snapshot.
    pub data: Nest,
    /// Separate query input snapshot.
    pub query: Nest,
    /// Separate post input snapshot.
    pub post: Nest,
    /// Validated header snapshot.
    pub headers: Headers,
    /// Cookie-backed value snapshot, not identity.
    pub session: Session,
    /// Parsed URL, without forwarded-host authority.
    pub url: url::Url,
    /// Normalized request method.
    pub method: Method,
    /// Existing buffered body.
    pub body: Body,
    /// Existing MIME selection.
    pub mimetype: String,
}
impl fmt::Debug for RequestView {
    /// Redact snapshot payloads just as the original request does.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Snapshot Debug has the same redaction boundary as the original request.
        f.debug_struct("RequestView")
            .field("method", &self.method)
            .field("data", &self.data)
            .field("body_type", &self.body.type_name())
            .finish_non_exhaustive()
    }
}

/// Mutable input with no Clone that could duplicate an attempted native read.
pub struct Request {
    /// Aggregate operation input, initialized query → post → explicit data.
    pub data: Nest,
    /// Separate original query values; direct edits do not magically resync data.
    pub query: Nest,
    /// Separate post values.
    pub post: Nest,
    /// Validated incoming headers.
    pub headers: Headers,
    /// Cookie-backed values; not an authorization decision.
    pub session: Session,
    /// Parsed URL, independently usable without a socket.
    pub url: url::Url,
    /// Normalized method token.
    pub method: Method,
    /// Current buffered input, replaced only after a complete staged load succeeds.
    body: Body,
    /// Explicit checked media type; incoming headers do not silently select a parser.
    mimetype: String,
    /// Unique one-attempt state; cancellation leaves Attempted rather than authorizing retry.
    load_state: LoadState,
    /// Final construction policy, rechecked when staged reader output is installed.
    limits: ParseLimits,
}
impl Default for Request {
    /// Construct default GET/unknownhost/text/plain input.
    fn default() -> Self {
        // Use the same no-resource default as explicit independent construction.
        Self::new()
    }
}
impl Request {
    /// Construct input without native resources or automatic parsing.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// let req = Request::new();
    /// assert_eq!(req.url.host_str(), Some("unknownhost"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // A fixed valid URL is a proven constructor invariant, not fallible input.
        Self {
            data: Nest::new(),
            query: Nest::new(),
            post: Nest::new(),
            headers: Headers::new(),
            session: Session::new(),
            url: url::Url::parse("http://unknownhost/").expect("constant valid default URL"),
            method: Method::default(),
            body: Body::Empty,
            mimetype: "text/plain".into(),
            // A newly constructed request has never entered a native reader, even when buffered
            // input exists.
            load_state: LoadState::Unloaded,
            limits: ParseLimits::default(),
        }
    }
    /// Start validated construction with explicit source options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// let req = Request::builder().url("/profiles?id=7").build()?;
    /// assert_eq!(req.data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn builder() -> RequestBuilder {
        // Defer fallible source conversion until the final selected budgets are known.
        RequestBuilder::default()
    }
    /// Borrow the exact buffered body.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// assert_eq!(Request::new().body().type_name(), "empty");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn body(&self) -> &Body {
        // Borrow buffered representation without duplicating or advancing any native read attempt.
        &self.body
    }
    /// Borrow the explicitly selected MIME type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// assert_eq!(Request::new().mimetype(), "text/plain");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn mimetype(&self) -> &str {
        // Borrow the explicitly selected type; it is not inferred authority from a header.
        &self.mimetype
    }
    /// Inspect successful load state.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// assert!(!Request::new().loaded());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn loaded(&self) -> bool {
        // Only completed successful finalization counts as loaded; failed/interrupted attempts
        // remain distinct.
        self.load_state == LoadState::Loaded
    }
    /// Inspect interrupted/failed attempts distinctly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// assert_eq!(
    ///     Request::new().load_state(),
    ///     stackpress_lib::LoadState::Unloaded
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn load_state(&self) -> LoadState {
        // Expose interruption honestly so callers cannot mistake cancellation for an untouched
        // reader.
        self.load_state
    }
    /// Clone only data-bearing values for a reader, not attempt/native ownership.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// let req = Request::new();
    /// let mut view = req.view();
    /// // Editing ordinary snapshot data cannot update the original request.
    /// view.data.set("snapshot", true)?;
    /// assert!(!req.data.has("snapshot"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn view(&self) -> RequestView {
        // Copy data only; no native attempt ownership can be recreated through this snapshot.
        RequestView {
            data: self.data.clone(),
            query: self.query.clone(),
            post: self.post.clone(),
            headers: self.headers.clone(),
            session: self.session.clone(),
            url: self.url.clone(),
            method: self.method.clone(),
            body: self.body.clone(),
            mimetype: self.mimetype.clone(),
        }
    }
    /// Finalize without invoking a reader; successful repeated loads are no-ops.
    ///
    /// # Errors
    /// Rejects an already failed/interrupted attempt or current post/aggregate data
    /// exceeding this request's selected budgets. See [`Self::load_with`] for the
    /// retained-data and one-attempt state contracts, including cancellation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// # async fn example() -> Result<()> {
    /// let mut req = Request::new();
    /// // Finalize buffered input without a native reader, consuming the load attempt.
    /// req.load().await?;
    /// assert!(req.loaded());
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub async fn load(&mut self) -> Result<()> {
        // No-reader finalization uses the same attempt-state transition as an actual reader
        // callback.
        self.load_with(|_| async { Ok(LoadResult::default()) })
            .await
    }
    /// Invoke one Send reader using isolated input and stage all returned mutations.
    ///
    /// # Errors
    /// Returns the reader's original error, staged validation error or invalid state.
    /// Cancellation leaves Attempted; no automatic reader retry is permitted.
    /// Reader or validation failure records Failed, retaining the previous body,
    /// post and aggregate data. State rejection happens before invoking the loader;
    /// successful repeated loads are no-ops. External loader side effects are not
    /// rolled back when its staged result is rejected.
    ///
    /// # Panics
    /// A loader panic propagates and leaves the request Attempted; no automatic retry occurs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{LoadResult, Request, RequestView, Result};
    /// async fn read(_: RequestView) -> Result<LoadResult> {
    ///     Ok(LoadResult::default())
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut req = Request::new();
    /// // The reader sees a snapshot; only its successful staged result updates this request.
    /// req.load_with(read).await?;
    /// assert!(req.loaded());
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub async fn load_with<F, Fut>(&mut self, loader: F) -> Result<()>
    where
        F: FnOnce(RequestView) -> Fut + Send,
        Fut: Future<Output = Result<LoadResult>> + Send,
    {
        // Check state before constructing a snapshot or calling the possibly side-effecting reader.
        match self.load_state {
            // Successful repetition is a no-op and must not call another native reader.
            LoadState::Loaded => return Ok(()),
            LoadState::Unloaded => {}
            _ => {
                return Err(
                    Error::new("request load already attempted").with_kind(ErrorKind::State)
                );
            }
        }
        // Only data enters the callback; mutation of its snapshot cannot partially edit this
        // request.
        let snapshot = self.view();
        // Consume the read attempt before the callback is called or suspended.
        self.load_state = LoadState::Attempted;
        // Await one reader result, then validate all staged fields before any visible commit.
        let result = match loader(snapshot).await {
            Ok(result) => self.install_loaded(result),
            Err(error) => Err(error),
        };
        // An ordinary error records Failed; dropping during await leaves the earlier Attempted
        // state.
        self.load_state = if result.is_ok() {
            LoadState::Loaded
        } else {
            LoadState::Failed
        };
        result
    }
    /// Validate/stage the entire update before committing any visible value.
    fn install_loaded(&mut self, loaded: LoadResult) -> Result<()> {
        // Check the returned body before staging both post/data overlays as one atomic update.
        if let Some(body) = &loaded.body {
            body.validate(&self.limits)?;
        }
        let mut post = self.post.clone();
        let mut data = self.data.clone();
        if let Some(fields) = loaded.post {
            // Post and aggregate data each own ordinary copies; only explicitly Shared native
            // identity is reused.
            post.merge(fields.clone())?;
            data.merge(fields)?;
        }
        // Public stores can be replaced; the request's selected limits still govern loading.
        post.value().validate(&self.limits)?;
        data.value().validate(&self.limits)?;
        // Commit body only after both overlays and final limits have succeeded.
        if let Some(body) = loaded.body {
            self.body = body;
        }
        // Publish the staged source and aggregate together; failed preparation never reaches these
        // assignments.
        self.post = post;
        self.data = data;
        Ok(())
    }
}
impl fmt::Debug for Request {
    /// Redact data/body/header/session values; URL query values are not dumped.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not log query/body/header/session secrets merely because the invocation is inspected.
        f.debug_struct("Request")
            .field("method", &self.method)
            .field("data", &self.data)
            .field("body_type", &self.body.type_name())
            .field("load_state", &self.load_state)
            .finish_non_exhaustive()
    }
}

/// Deferred owned data construction, applying the final selected invocation budget.
type DataInput = Box<dyn FnOnce(ParseLimits) -> Result<Nest> + Send>;

/// Checked construction; setters stage owned inputs, build validates atomically.
///
/// Accidentally discarding a consuming chain is diagnosed; intentional discard uses `let _ =`.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// use stackpress_lib::Request;
/// Request::builder().method("POST");
/// ```
#[derive(Default)]
#[must_use = "the staged request builder must be built or explicitly discarded"]
pub struct RequestBuilder {
    url: Option<String>,
    method: Option<String>,
    headers: Headers,
    session: Option<Session>,
    query: Option<DataInput>,
    query_string: Option<String>,
    post: Option<DataInput>,
    data: Option<DataInput>,
    body: Body,
    mimetype: Option<String>,
    limits: ParseLimits,
}
impl RequestBuilder {
    /// Select an absolute URL or a path relative to the documented unknown host.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder().url("/profiles").build()?;
    /// assert_eq!(req.url.path(), "/profiles");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn url(mut self, url: impl Into<String>) -> Self {
        // Keep supplied URL text owned until build can apply the final policy and relative-path
        // rule.
        self.url = Some(url.into());
        self
    }
    /// Select a method, validated/normalized at build.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder().method("post").build()?;
    /// assert_eq!(req.method.as_str(), "POST");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn method(mut self, method: impl Into<String>) -> Self {
        // Stage text without silently accepting an invalid method before construction finishes.
        self.method = Some(method.into());
        self
    }
    /// Move validated headers into construction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert_eq!(
    ///     Request::builder()
    ///         .headers(headers)
    ///         .build()?
    ///         .headers
    ///         .get("x-id"),
    ///     Some("7")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn headers(mut self, headers: Headers) -> Self {
        // Move validated header values; final build still checks the invocation's selected
        // byte/count limits.
        self.headers = headers;
        self
    }
    /// Supply explicit cookie-backed initial state, winning over header cookies.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let session = Session::from_pairs([("theme", "dark")])?;
    /// assert!(
    ///     Request::builder()
    ///         .session(session)
    ///         .build()?
    ///         .session
    ///         .get("theme")
    ///         .is_some()
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn session(mut self, session: Session) -> Self {
        // Explicit session input deliberately overrides incoming Cookie headers at construction.
        self.session = Some(session);
        self
    }
    /// Supply parsed query values; no scalar coercion is applied a second time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder()
    ///     .query(serde_json::json!({"id": "007"}))
    ///     .build()?;
    /// assert_eq!(req.data.path_as::<String>("id")?, Some("007".into()));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn query<D: IntoNest + Send + 'static>(mut self, data: D) -> Self {
        // Parsed input is not scalar-coerced again; its conversion uses the eventual chosen limits.
        self.query = Some(Box::new(move |limits| data.into_nest_with(limits)));
        self.query_string = None;
        self
    }
    /// Supply query text, winning over URL query.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder().query_string("id=7").build()?;
    /// assert_eq!(req.data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn query_string(mut self, text: impl Into<String>) -> Self {
        // An explicit raw query replaces a previous parsed override and wins over URL query text.
        self.query_string = Some(text.into());
        self.query = None;
        self
    }
    /// Supply parsed post fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder()
    ///     .post(serde_json::json!({"active": false}))
    ///     .build()?;
    /// assert_eq!(req.data.path_as::<bool>("active")?, Some(false));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn post<D: IntoNest + Send + 'static>(mut self, data: D) -> Self {
        // Defer owned post validation under the final limits while preserving its separate source
        // collection.
        self.post = Some(Box::new(move |limits| data.into_nest_with(limits)));
        self
    }
    /// Supply explicit operation data, winning over query and post at initialization.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder()
    ///     .data(serde_json::json!({"id": 7}))
    ///     .build()?;
    /// assert_eq!(req.data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn data<D: IntoNest + Send + 'static>(mut self, data: D) -> Self {
        // Explicit operation input is the final initialization overlay after query and post.
        self.data = Some(Box::new(move |limits| data.into_nest_with(limits)));
        self
    }
    /// Serialize an owned snapshot of typed data; errors are retained until build.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// #[derive(serde::Serialize)]
    /// struct Input {
    ///     id: u64,
    /// }
    /// let req = Request::builder()
    ///     .data_serialized(&Input { id: 7 })
    ///     .build()?;
    /// assert_eq!(req.data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn data_serialized<T: Serialize + ?Sized>(mut self, data: &T) -> Self {
        // Serialize an owned snapshot now; retain its error so build stays the single fallible
        // constructor.
        let value = serde_json::to_value(data).map_err(Error::from);
        self.data = Some(Box::new(move |limits| value?.into_nest_with(limits)));
        self
    }
    /// Supply exact buffered body representation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder().body(Body::Bytes(vec![0, 255])).build()?;
    /// assert_eq!(req.body().type_name(), "bytes");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn body(mut self, body: Body) -> Self {
        // Move an exact buffer into deferred construction without consuming a native reader.
        self.body = body;
        self
    }
    /// Select a checked media type string without inferring parsing policy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder().mimetype("application/json").build()?;
    /// assert_eq!(req.mimetype(), "application/json");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn mimetype(mut self, mime: impl Into<String>) -> Self {
        // Stage the explicit type; build validates syntax without silently selecting it from
        // headers.
        self.mimetype = Some(mime.into());
        self
    }
    /// Select final budgets before any deferred data conversion runs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder()
    ///     .limits(ParseLimits {
    ///         max_bytes: 64,
    ///         ..Default::default()
    ///     })
    ///     .build()?;
    /// assert_eq!(req.data.limits().max_bytes, 64);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn limits(mut self, limits: ParseLimits) -> Self {
        // All deferred source closures receive this final policy, independent of setter order.
        self.limits = limits;
        self
    }
    /// Validate all sources and commit one independently owned request.
    ///
    /// # Errors
    /// Returns URL, header/body-budget, MIME, query/post/data, cookie/session or
    /// method validation failures, in that order. Deferred serialization errors
    /// from [`Self::data_serialized`] surface when the data stage is reached.
    /// No partial request is returned; this consumes the builder and its staged
    /// inputs on failure, so retry requires constructing a new builder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
    /// let req = Request::builder().url("/health").build()?;
    /// assert_eq!(req.method.as_str(), "GET");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn build(self) -> Result<Request> {
        // Resolve input authority first, then validate buffered sources before publishing a
        // complete request.
        let limits = self.limits;
        let url = request_url(
            self.url.as_deref().unwrap_or("http://unknownhost/"),
            &limits,
        )?;
        // Caller-supplied buffered fields must respect the invocation's final policy, not just
        // constructor defaults.
        self.headers.validate(&limits)?;
        self.body.validate(&limits)?;
        let mime = self.mimetype.unwrap_or_else(|| "text/plain".into());
        crate::data::validate_mime(&mime)?;
        // Explicit parsed query wins, then explicit query text, then URL query; parsed values are
        // not re-coerced.
        let query = if let Some(query) = self.query {
            query(limits)?
        } else {
            parse_query(
                self.query_string
                    .as_deref()
                    .unwrap_or_else(|| url.query().unwrap_or("")),
                &limits,
            )?
        };
        let post = self
            .post
            .map_or_else(|| Ok(Nest::with_limits(limits)), |post| post(limits))?;
        let mut data = Nest::with_limits(limits);
        // Source collections remain independently inspectable while aggregate data follows query →
        // post → explicit precedence.
        data.merge(query.clone())?.merge(post.clone())?;
        if let Some(explicit) = self.data {
            data.merge(explicit(limits)?)?;
        }
        let session = initial_session(self.session, &self.headers)?;
        Ok(Request {
            data,
            query,
            post,
            headers: self.headers,
            session,
            url,
            method: self
                .method
                .as_deref()
                .map(Method::new)
                .transpose()?
                .unwrap_or_default(),
            body: self.body,
            mimetype: mime,
            load_state: LoadState::Unloaded,
            limits,
        })
    }
}

/// Parse a request URL without accepting malformed absolute input as a relative path.
fn request_url(url: &str, limits: &ParseLimits) -> Result<url::Url> {
    // Only the specific relative-without-base parse case may receive the unknown-host prefix.
    crate::data::check_input(url.len(), limits)?;
    let url = match url::Url::parse(url) {
        Ok(url) => url,
        // Only a real relative path is prefixed; malformed absolute URLs keep their original parse
        // failure.
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            url::Url::parse(&with_unknown_host(url)).map_err(url_error)?
        }
        Err(error) => return Err(url_error(error)),
    };
    Ok(url)
}

/// Retain the URL parser cause while adding only request-boundary context.
fn url_error(cause: url::ParseError) -> Error {
    // Parser details remain private and independently downcastable in the source chain.
    Error::new("request URL could not be parsed")
        .with_kind(crate::ErrorKind::External)
        .with_cause(cause)
}

/// Prefer explicit initial session values; otherwise combine all cookie header lines.
fn initial_session(explicit: Option<Session>, headers: &Headers) -> Result<Session> {
    // Move explicit values without outgoing revisions; otherwise parse all Cookie occurrences
    // together.
    let session = if let Some(session) = explicit {
        session.into_initial()
    } else {
        let cookies = headers.get_all("cookie").unwrap_or_default().join("; ");
        Session::from_pairs(cookie::parse(&cookies)?)?
    };
    Ok(session)
}

/// Add the literal unknown-host prefix, even to protocol-looking supplied text.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::{Body, Headers, ParseLimits, Request, Session};
/// assert_eq!(
///     stackpress_lib::with_unknown_host("/profiles"),
///     "http://unknownhost/profiles"
/// );
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn with_unknown_host(text: &str) -> String {
    // Prefix literally; protocol-looking input is still treated as supplied path text by this
    // helper.
    format!(
        "http://unknownhost{}{}",
        if text.starts_with('/') { "" } else { "/" },
        text
    )
}
