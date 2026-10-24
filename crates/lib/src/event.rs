//! Immutable all-matching dispatch with sequential mutation and Send futures.
//!
//! Registrations are immutable at runtime; explicitly share a table with Arc when needed.
//! Handlers run sequentially against an exclusive request/response pair. Errors keep
//! logical invocation metadata; application panics propagate without rollback or an
//! automatic error event. Dropping a pending dispatch stops its remaining phases.

//!
//! ```
//! use stackpress_lib::{EmitterBuilder, Flow, Invocation, Request, Response, Result};
//! async fn greet(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
//!     let name = req
//!         .data
//!         .path_as::<String>("profile.name")?
//!         .unwrap_or_else(|| "friend".into());
//!     res.results(&format!("Hello, {name}"))?;
//!     Ok(Flow::Continue)
//! }
//! let mut builder = EmitterBuilder::new();
//! builder.on_priority("greet", 10, greet)?;
//! let emitter = builder.build();
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

use crate::{
    DispatchOutcome, Error, ErrorKind, EventPattern, Flow, HandlerFuture, InvocationFrame,
    MatchData, MethodPattern, Nest, Request, Response, ResponseEnvelope, Result, SourceLocation,
};
use std::any::Any;
use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

/// Accepted owned inputs avoid implicit cloning of request/load attempt state.
mod input_contract {
    /// Seal normal capability-input conversion to the documented neutral shapes.
    pub trait Sealed {}
    impl Sealed for crate::Request {}
    impl Sealed for crate::Nest {}
    impl Sealed for serde_json::Value {}
}
/// Plain object data becomes a request; an owned Request preserves its current state.
pub trait IntoRequest: input_contract::Sealed + Send {
    /// Construct or move the invocation input without cloning its attempt state.
    ///
    /// # Errors
    /// Returns request construction errors for plain data inputs.
    ///
    /// ```
    /// use stackpress_lib::IntoRequest;
    /// let request = serde_json::json!({"id": 7}).into_request()?;
    /// assert_eq!(request.data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    fn into_request(self) -> Result<Request>;
}
impl IntoRequest for Request {
    /// Move the original mutable invocation input.
    fn into_request(self) -> Result<Request> {
        // Move the exact invocation, including load-attempt state; no new native reader is implied.
        Ok(self)
    }
}
impl IntoRequest for Nest {
    /// Move plain rich operation data into a fresh request.
    fn into_request(self) -> Result<Request> {
        // Plain rich data initializes fresh request state while preserving explicitly Shared
        // values.
        Request::builder().data(self).build()
    }
}
impl IntoRequest for serde_json::Value {
    /// Validate plain JSON object input.
    fn into_request(self) -> Result<Request> {
        // Plain JSON must still pass the request's object-root validation.
        Request::builder().data(self).build()
    }
}

/// A capability result keeps execution outcome distinct from response fields.
#[derive(Clone, Debug)]
pub struct Resolution {
    /// Completed/aborted/no-listener, never automatically an HTTP status.
    pub outcome: DispatchOutcome,
    /// Partial status/results/errors view, retaining rich in-process values.
    pub response: ResponseEnvelope,
}

/// Allocate construction identities only; invocation state is never global.
static NEXT_REGISTRY: AtomicU64 = AtomicU64::new(1);

/// Stable value identity qualified by its originating construction registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ListenerId {
    /// Origin identity, retained even when the listener is moved into another builder.
    registry: u64,
    /// Never-reused registration order within the original registry.
    ordinal: u64,
}
impl fmt::Display for ListenerId {
    /// Render a useful opaque diagnostic identity.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Qualify the ordinal with its originating registry so imported handles cannot collide.
        write!(f, "r{}:l{}", self.registry, self.ordinal)
    }
}

/// Construction-time matcher, priority and explicit optional owner/label.
pub struct ListenerSpec {
    pattern: EventPattern,
    priority: i64,
    label: Option<String>,
    plugin: Option<String>,
}
impl ListenerSpec {
    /// Start an exact listener specification at priority zero.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// let spec = ListenerSpec::exact("feature-run");
    /// let mut builder = EmitterBuilder::new();
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// builder.listen(spec, action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn exact(event: &str) -> Self {
        // Exact registration uses the same specification path as explicit pattern registration.
        Self::pattern(EventPattern::exact(event))
    }
    /// Start from an explicit compiled pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// let spec = ListenerSpec::pattern(EventPattern::express("/profiles/:id", '/')?);
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// EmitterBuilder::new().listen(spec, action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn pattern(pattern: EventPattern) -> Self {
        // A specification owns its compiled matcher and starts with no implicit plugin/label
        // policy.
        Self {
            pattern,
            priority: 0,
            label: None,
            plugin: None,
        }
    }
    /// Set an explicit signed execution priority.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// EmitterBuilder::new().listen(ListenerSpec::exact("feature-run").priority(10), action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn priority(mut self, priority: i64) -> Self {
        // Priority changes scheduling only; it does not identify a listener for replacement.
        self.priority = priority;
        self
    }
    /// Set a human-readable label, which is not an implicit replacement key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// EmitterBuilder::new().listen(ListenerSpec::exact("feature-run").label("audit"), action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn label(mut self, label: impl Into<String>) -> Self {
        // Labels aid diagnostics but are deliberately not unique customization keys.
        self.label = Some(label.into());
        self
    }
    /// Set the optional contributing plugin identity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// EmitterBuilder::new().listen(
    ///     ListenerSpec::exact("feature-run").plugin("profiles"),
    ///     action,
    /// )?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn plugin(mut self, plugin: impl Into<String>) -> Self {
        // Record the contributing owner as metadata without a runtime plugin-loading mechanism.
        self.plugin = Some(plugin.into());
        self
    }
}

/// Immutable inspection data, never writable registration/callback storage.
#[derive(Clone, Debug)]
pub struct ListenerDescriptor {
    /// Stable origin-qualified identity.
    pub id: ListenerId,
    /// Original literal/pattern expression.
    pub pattern: String,
    /// Human-readable label, defaulting to the listener identity.
    pub label: String,
    /// Optional contributing plugin.
    pub plugin: Option<String>,
    /// Higher values run first.
    pub priority: i64,
    /// Original external registration site.
    pub registered: SourceLocation,
    /// Last explicit replacement site when customized.
    pub replaced: Option<SourceLocation>,
}

/// A concrete listener match in execution order; duplicates do not collapse.
#[derive(Clone, Debug)]
pub struct EventMatch {
    /// Matching registration identity.
    pub listener: ListenerId,
    /// Actual emitted event.
    pub event: String,
    /// Original matching pattern.
    pub pattern: String,
    /// Listener-local named and positional captures.
    pub data: MatchData,
}

/// Internal type erasure and lifetime-family adaptation are sealed here.
mod contract {
    use super::Invocation;
    use crate::{Flow, HandlerFuture, Request, Response, Result};
    use std::future::Future;
    /// A named function's future family for each common argument borrow.
    pub trait HandlerFn<'a> {
        /// The returned future itself must be Send, not only the function item.
        type Future: Future<Output = Result<Flow>> + Send + 'a;
        /// Borrow all invocation inputs for exactly this future.
        fn invoke(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            invocation: Invocation<'a>,
        ) -> Self::Future;
    }
    impl<'a, F, Fut> HandlerFn<'a> for F
    where
        F: Fn(&'a mut Request, &'a mut Response, Invocation<'a>) -> Fut,
        Fut: Future<Output = Result<Flow>> + Send + 'a,
    {
        type Future = Fut;
        /// Return the original future without extending any lifetime.
        fn invoke(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            invocation: Invocation<'a>,
        ) -> Fut {
            // Call the shared Fn for this exact borrow lifetime; no mutable callable lock is
            // required.
            self(req, res, invocation)
        }
    }
    /// Object-safe storage, inaccessible for external manual implementation.
    pub trait Erased: Send + Sync {
        /// Pin/erase only the already-proved Send future.
        fn invoke<'a>(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            invocation: Invocation<'a>,
        ) -> HandlerFuture<'a, Result<Flow>>;
    }
    impl<H> Erased for H
    where
        H: for<'a> HandlerFn<'a> + Send + Sync + 'static,
    {
        /// No mutex, unsafe cast or unstable AsyncFn associated type is needed.
        fn invoke<'a>(
            &'a self,
            req: &'a mut Request,
            res: &'a mut Response,
            invocation: Invocation<'a>,
        ) -> HandlerFuture<'a, Result<Flow>> {
            // Box only the already-Send returned future; type erasure never lengthens a borrow.
            Box::pin(HandlerFn::invoke(self, req, res, invocation))
        }
    }
}

/// A named async function or helper closure accepted directly by registration.
///
/// This sealed bound hides lifetime-family/erasure machinery from plugin authors.
pub trait Handler: contract::Erased {}
impl<H> Handler for H where H: for<'a> contract::HandlerFn<'a> + Send + Sync + 'static {}

/// Secondary closure style: supply a higher-ranked boxed Send future expectation.
///
/// Prefer a named async function. The box erases the closure's borrowed future
/// type; async move retains its arguments across awaits. Clone an Arc capture
/// per call explicitly when sharing immutable configuration is intended.
///
/// ```
/// use stackpress_lib::{EmitterBuilder, Flow, handler};
/// use std::sync::Arc;
/// let label = Arc::new(String::from("configured"));
/// let mut builder = EmitterBuilder::new();
/// builder.on(
///     "observe",
///     handler(move |req, _res, _invocation| {
///         // Keep the captured Arc for later calls and move a cheap shared clone into this one.
///         let label = Arc::clone(&label);
///         // Box supplies one lifetime-indexed future type; move retains inputs.
///         Box::pin(async move {
///             req.data.set("label", label.as_str())?;
///             Ok(Flow::Continue)
///         })
///     }),
/// )?;
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn handler<F>(callback: F) -> F
where
    F: for<'a> Fn(
            &'a mut Request,
            &'a mut Response,
            Invocation<'a>,
        ) -> HandlerFuture<'a, Result<Flow>>
        + Send
        + Sync
        + 'static,
{
    // The identity helper supplies higher-ranked closure inference without hiding registration
    // flow.
    callback
}

/// Listener phase is independent of later top-level host lifecycle conventions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListenerPhase {
    /// Per-listener preprocessing; Abort skips action and after.
    Before,
    /// Selected listener action; Abort skips after.
    Action,
    /// Per-listener postprocessing, not a finally block.
    After,
}
impl ListenerPhase {
    /// A stable diagnostic name.
    fn name(self) -> &'static str {
        // Use explicit phase vocabulary in diagnostics rather than an execution-host lifecycle
        // label.
        match self {
            Self::Before => "before",
            Self::Action => "action",
            Self::After => "after",
        }
    }
}

/// Owned callback registration, immutable after build.
struct Listener {
    description: ListenerDescriptor,
    pattern: EventPattern,
    /// Local execution-order sequence independent of an imported listener's origin ordinal.
    sequence: u64,
    action: Box<dyn contract::Erased>,
}
/// A hook has its own call site while observing a selected listener identity.
struct Hook {
    action: Box<dyn contract::Erased>,
    registered: SourceLocation,
}

/// Mutable construction phase; build moves into an immutable runtime table.
pub struct EmitterBuilder {
    /// Lazily allocated own identity; empty construction consumes no global identity.
    registry: Option<u64>,
    /// Consumed builder origins whose stable handles this builder may still customize.
    imports: Vec<u64>,
    sequence: u64,
    /// Owned contributions; mutable only before build, sorted once when frozen.
    listeners: Vec<Listener>,
    /// Receiver-owned before hook; composition does not import another builder's hook.
    before: Option<Hook>,
    /// Receiver-owned after hook, skipped when an earlier phase aborts or fails.
    after: Option<Hook>,
    /// Bounded maximum immutable ancestry depth for nested dispatch.
    depth_limit: usize,
}
impl Default for EmitterBuilder {
    /// Construct an empty registration accumulator.
    fn default() -> Self {
        // Default construction is the same empty builder with finite nested-dispatch limits.
        Self::new()
    }
}
impl EmitterBuilder {
    /// Construct an empty builder without locks or an executor.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// let emitter = builder.build();
    /// assert_eq!(emitter.listeners().count(), 0);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // Allocate registry identity lazily on first registration; an empty builder consumes no
        // global ID.
        Self {
            registry: None,
            imports: Vec::new(),
            sequence: 0,
            listeners: Vec::new(),
            before: None,
            after: None,
            depth_limit: 64,
        }
    }
    /// Register a named async handler on an exact event at priority zero.
    ///
    /// # Errors
    /// Rejects invalid event input or identity/sequence exhaustion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("feature-run", action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn on(&mut self, event: &str, action: impl Handler + 'static) -> Result<ListenerId> {
        // The tracked wrapper forwards the external call site through the common registration path.
        self.listen(ListenerSpec::exact(event), action)
    }
    /// Register priority in one expression; equal priorities retain insertion order.
    ///
    /// # Errors
    /// Returns invalid event or exhausted registration identity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_priority("feature-run", 10, action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn on_priority(
        &mut self,
        event: &str,
        priority: i64,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // A single registration expression preserves the same validation and call-site behavior.
        self.listen(ListenerSpec::exact(event).priority(priority), action)
    }
    /// Register one explicitly compiled matcher.
    ///
    /// # Errors
    /// Returns invalid input or exhausted registration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn on_pattern(
        &mut self,
        pattern: EventPattern,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // Keep compiled pattern identity/capture mapping intact instead of stringifying and
        // reparsing it.
        self.listen(ListenerSpec::pattern(pattern), action)
    }
    /// Register neutral method/path matching without a transport router.
    ///
    /// # Errors
    /// Returns invalid route/pattern or exhausted registration order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.route(MethodPattern::GET, "/profiles/:id", action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn route(
        &mut self,
        method: MethodPattern,
        path: &str,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // Compile neutral method/path matching before the common builder mutation phase.
        self.on_pattern(EventPattern::route(method, path)?, action)
    }
    /// Register explicit pattern/priority/label/plugin metadata atomically.
    ///
    /// # Errors
    /// Returns invalid pattern, metadata or exhausted IDs before inserting a callback.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.listen(
    ///     ListenerSpec::exact("feature-run")
    ///         .priority(10)
    ///         .label("audit"),
    ///     action,
    /// )?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn listen(
        &mut self,
        spec: ListenerSpec,
        action: impl Handler + 'static,
    ) -> Result<ListenerId> {
        // Validate contribution metadata and checked counters before storing the callback.
        spec.pattern.validate()?;
        if spec.label.as_ref().is_some_and(|v| v.len() > 8192)
            || spec.plugin.as_ref().is_some_and(|v| v.len() > 8192)
        {
            return Err(Error::limit("registration metadata byte limit exceeded"));
        }
        // Diagnostic identities must not inject control sequences into host reports.
        if spec
            .label
            .as_ref()
            .is_some_and(|v| v.chars().any(char::is_control))
            || spec
                .plugin
                .as_ref()
                .is_some_and(|v| v.chars().any(char::is_control))
        {
            return Err(Error::invalid(
                "registration metadata contains control characters",
            ));
        }
        // Reserve the next ordinal before any callback is inserted; exhaustion cannot reuse an
        // identity.
        let next = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| Error::limit("listener sequence exhausted"))?;
        // A builder retains one origin; imported order is separate from this identity namespace.
        let registry = if let Some(id) = self.registry {
            id
        } else {
            let id = NEXT_REGISTRY
                // Relaxed ordering is sufficient for uniqueness: this atomic publishes no
                // invocation data.
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
                .map_err(|_| Error::limit("registry identity exhausted"))?;
            self.registry = Some(id);
            id
        };
        let id = ListenerId {
            registry,
            ordinal: self.sequence,
        };
        // All fallible registration checks are complete before the listener becomes visible.
        self.listeners.push(Listener {
            description: ListenerDescriptor {
                id,
                pattern: spec.pattern.source().into(),
                label: spec.label.unwrap_or_else(|| id.to_string()),
                plugin: spec.plugin,
                priority: spec.priority,
                // Capture the synchronous external registration site, not the later async polling
                // site.
                registered: SourceLocation::caller(),
                replaced: None,
            },
            pattern: spec.pattern,
            sequence: self.sequence,
            action: Box::new(action),
        });
        self.sequence = next;
        Ok(id)
    }
    /// Set the single before hook and capture its external registration site.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.before(action).on("feature-run", action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn before(&mut self, action: impl Handler + 'static) -> &mut Self {
        // Replacing a construction-time hook also records the site that will appear in phase
        // failures.
        self.before = Some(Hook {
            action: Box::new(action),
            registered: SourceLocation::caller(),
        });
        self
    }
    /// Set the single after hook; it is skipped on action abort/error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.after(action).on("feature-run", action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn after(&mut self, action: impl Handler + 'static) -> &mut Self {
        // This is an ordinary success-path phase, not a finally handler or error fallback.
        self.after = Some(Hook {
            action: Box::new(action),
            registered: SourceLocation::caller(),
        });
        self
    }
    /// Set a bounded per-ancestry recursion limit.
    ///
    /// # Errors
    /// Accepts only 1..=256 to retain bounded recursive future/drop behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.depth_limit(8)?.on("feature-run", action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn depth_limit(&mut self, limit: usize) -> Result<&mut Self> {
        // Bound recursive future and Arc-chain depth even when handlers emit the same event again.

        // Reject both zero-depth dispatch and unbounded recursive-drop growth.
        if !(1..=256).contains(&limit) {
            return Err(Error::invalid("dispatch depth must be 1..=256"));
        }
        self.depth_limit = limit;
        Ok(self)
    }
    /// Check whether an ID belongs to this builder or a consumed imported builder.
    fn owns(&self, id: &ListenerId) -> bool {
        // Imported origins remain valid handle owners after their builder has been consumed.
        self.registry == Some(id.registry) || self.imports.contains(&id.registry)
    }
    /// Remove a construction-time listener by stable handle.
    ///
    /// # Errors
    /// Rejects a foreign origin rather than removing an unrelated registration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// let id = builder.on("feature-run", action)?;
    /// assert!(builder.remove(&id)?);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn remove(&mut self, id: &ListenerId) -> Result<bool> {
        // Reject foreign identity before searching; a stale owned handle is an ordinary false
        // result.
        if !self.owns(id) {
            return Err(
                Error::invalid("foreign listener handle").with_kind(ErrorKind::Registration)
            );
        }
        // Removal preserves the registration order of every surviving listener.
        if let Some(index) = self.listeners.iter().position(|v| v.description.id == *id) {
            self.listeners.remove(index);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    /// Replace deliberately, preserving ID/pattern/priority/order and original site.
    ///
    /// # Errors
    /// Rejects stale/foreign handles without mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// let id = builder.on("feature-run", action)?;
    /// builder.replace(&id, action)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn replace(&mut self, id: &ListenerId, action: impl Handler + 'static) -> Result<()> {
        // Find the exact stable handle before swapping its action; duplicate registration order is
        // irrelevant.
        let listener = self
            .listeners
            .iter_mut()
            .find(|v| v.description.id == *id)
            .ok_or_else(|| {
                Error::invalid("stale or foreign listener handle")
                    .with_kind(ErrorKind::Registration)
            })?;
        // Preserve matcher, priority, sequence and original site; change only the action and
        // replacement provenance.
        listener.action = Box::new(action);
        listener.description.replaced = Some(SourceLocation::caller());
        Ok(())
    }
    /// Clear all registrations of the same full typed pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("feature-run", action)?;
    /// assert_eq!(builder.clear(&EventPattern::exact("feature-run")), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn clear(&mut self, pattern: &EventPattern) -> usize {
        // Compare typed identities so equivalent regex display text cannot erase different capture
        // mappings.
        let before = self.listeners.len();
        self.listeners.retain(|v| !v.pattern.same_pattern(pattern));
        before - self.listeners.len()
    }
    /// Consume another builder, preserving compiled patterns/IDs and import order.
    ///
    /// # Errors
    /// Rejects exhausted order arithmetic before moving any registration.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// let mut plugin = EmitterBuilder::new();
    /// plugin.on("feature-run", action)?;
    /// builder.extend(plugin)?;
    /// assert_eq!(builder.build().listeners().count(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn extend(&mut self, other: Self) -> Result<()> {
        // Preflight the complete sequence range before moving any imported contribution.
        let count = u64::try_from(other.listeners.len())
            .map_err(|_| Error::limit("listener count overflow"))?;
        let next = self
            .sequence
            .checked_add(count)
            .ok_or_else(|| Error::limit("listener sequence exhausted"))?;
        if let Some(registry) = other.registry {
            self.imports.push(registry);
        }
        self.imports.extend(other.imports);
        // Assign receiver execution order while retaining each contribution's original stable
        // ID/site.
        for mut listener in other.listeners {
            listener.sequence = self.sequence;
            self.sequence += 1;
            self.listeners.push(listener);
        }
        self.sequence = next;
        // Source hooks intentionally do not become receiver hooks during composition.
        Ok(())
    }
    /// Move into a runtime table sorted once by priority then insertion sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("feature-run", action)?;
    /// let emitter = builder.build();
    /// assert_eq!(emitter.listeners().count(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn build(mut self) -> EventEmitter {
        // Sort once while mutable, then move callbacks and hooks into a runtime table with no
        // registry locks.
        self.listeners.sort_by(|a, b| {
            b.description
                .priority
                .cmp(&a.description.priority)
                .then(a.sequence.cmp(&b.sequence))
        });
        EventEmitter {
            listeners: self.listeners,
            before: self.before,
            after: self.after,
            depth_limit: self.depth_limit,
        }
    }
}

/// Explicitly shared immutable ancestry; only snapshots allocate full path vectors.
#[derive(Clone, Default)]
struct SharedAncestry(Option<Arc<AncestryNode>>);
/// One bounded ancestry link retained across nested awaits.
struct AncestryNode {
    /// Owned current metadata; no request/response values are stored in ancestry.
    frame: InvocationFrame,
    /// Explicit shared immutable parent link; siblings cannot edit a current-event slot.
    parent: SharedAncestry,
    /// Cached chain length for constant-time recursion checks.
    depth: usize,
}
impl SharedAncestry {
    /// Inspect depth without walking the chain.
    fn depth(&self) -> usize {
        // Cached depth avoids walking every immutable ancestor on each nested call.
        self.0.as_ref().map_or(0, |node| node.depth)
    }
    /// Append one immutable frame, sharing existing ancestry explicitly.
    fn push(&self, frame: InvocationFrame) -> Self {
        // Only this current frame is new; all parent nodes are visibly shared and immutable.
        Self(Some(Arc::new(AncestryNode {
            frame,
            parent: self.clone(),
            depth: self.depth() + 1,
        })))
    }
    /// Materialize a root-first diagnostic snapshot only on request/error.
    fn snapshot(&self) -> Vec<InvocationFrame> {
        // Walk leaf-to-root without recursion, then reverse for developer-readable call order.
        let mut frames = Vec::with_capacity(self.depth());
        let mut pointer = self.0.as_deref();
        while let Some(node) = pointer {
            frames.push(node.frame.clone());
            pointer = node.parent.0.as_deref();
        }
        // Traversal collected newest-first; public diagnostics promise root-first order.
        frames.reverse();
        frames
    }
}

/// Inherited immutable ancestry and an optional caller-owned context borrow.
#[derive(Default)]
struct DispatchContext<'a> {
    /// Diagnostic ancestry remains independent for each root and sibling.
    ancestry: SharedAncestry,
    /// The caller retains ownership; dispatch cannot extend this borrow's lifetime.
    state: Option<&'a (dyn Any + Send + Sync)>,
}

/// Immutable all-matching dispatch; share the table by reference or explicit Arc.
pub struct EventEmitter {
    listeners: Vec<Listener>,
    before: Option<Hook>,
    after: Option<Hook>,
    depth_limit: usize,
}
impl EventEmitter {
    /// Invoke a capability using plain data or an owned Request and a fresh Response.
    ///
    /// # Errors
    /// Returns input-conversion or original annotated dispatch failures.
    /// No general lifecycle/error handler or native output is run automatically.
    ///
    /// ```
    /// use stackpress_lib::{EmitterBuilder, Flow, Invocation, Request, Response, Result};
    /// async fn create(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&req.data)?;
    ///     Ok(Flow::Continue)
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("article-create", create)?;
    /// // Resolve creates isolated request/response values from the supplied object.
    /// let created = builder
    ///     .build()
    ///     .resolve("article-create", serde_json::json!({"title": "First Post"}))
    ///     .await?;
    /// assert_eq!(created.response.code, Some(200));
    /// #     Ok(())
    /// # }
    /// ```
    pub fn resolve<'a, I: IntoRequest + 'a>(
        &'a self,
        event: &'a str,
        input: I,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        // A root capability call starts fresh ancestry without entering host lifecycle events.
        self.resolve_nested(event, input, DispatchContext::default())
    }
    /// Resolve with explicitly borrowed immutable host state, inherited by nested calls.
    ///
    /// # Errors
    /// Returns the same input/dispatch failures as [`Self::resolve`]. Context is
    /// never inserted into request data, serialized, or interpreted by the foundation.
    ///
    /// ```
    /// use stackpress_lib::{EmitterBuilder, Flow, Invocation, Nest, Request, Response, Result};
    /// async fn read(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    ///     res.results(inv.context::<String>().expect("explicit context"))?;
    ///     Ok(Flow::Continue)
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("read", read)?;
    /// let emitter = builder.build();
    /// let context = String::from("caller owned");
    /// let reply = emitter.resolve_with_context("read", Nest::new(), &context).await?;
    /// assert_eq!(reply.response.results.unwrap().as_str(), Some("caller owned"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve_with_context<'a, T: Any + Send + Sync, I: IntoRequest + 'a>(
        &'a self,
        event: &'a str,
        input: I,
        context: &'a T,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        // A fresh root has no ancestry; only the explicit state borrow is inherited.
        self.resolve_nested(
            event,
            input,
            DispatchContext {
                ancestry: SharedAncestry::default(),
                state: Some(context),
            },
        )
    }
    /// Construct isolated child values while sharing only immutable diagnostic ancestry.
    fn resolve_nested<'a, I: IntoRequest + 'a>(
        &'a self,
        event: &'a str,
        input: I,
        parent: DispatchContext<'a>,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        // The owned input lives inside this Send future and is converted only when polled.
        Box::pin(async move {
            // Input conversion may fail before any child listener runs; the ordinary Result
            // preserves that failure.
            let mut request = input.into_request()?;
            let mut response = Response::new();
            // Await the complete capability before moving its result values into an envelope.
            let outcome = self
                .emit_nested(event, &mut request, &mut response, parent)
                .await?;
            Ok(Resolution {
                outcome,
                // Return status/results separately from Completed/Aborted/NoListener, with no
                // native output side effect.
                response: response.into_envelope(),
            })
        })
    }
    /// Inspect registration metadata in execution order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("feature-run", action)?;
    /// assert_eq!(builder.build().listeners().count(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn listeners(&self) -> impl Iterator<Item = &ListenerDescriptor> {
        // Expose metadata borrows in execution order, never writable callback storage.
        self.listeners.iter().map(|v| &v.description)
    }
    /// Collect every matching registration, including equivalent patterns/duplicates.
    ///
    /// # Errors
    /// Rejects empty/oversized event input.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("feature-run", action)?;
    /// assert_eq!(builder.build().matches("feature-run")?.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn matches(&self, event: &str) -> Result<Vec<EventMatch>> {
        // Public inspection moves out metadata only; execution retains private stable table
        // positions.
        Ok(self
            .collect_matches(event)?
            .into_iter()
            .map(|(_, matched)| matched)
            .collect())
    }
    /// Retain immutable table positions so dispatch never searches all listeners per match.
    fn collect_matches(&self, event: &str) -> Result<Vec<(usize, EventMatch)>> {
        // Validate the event once and retain every matching contribution in the table's stable
        // order.
        if event.is_empty() || event.len() > 8192 {
            return Err(Error::invalid("event is empty/oversized"));
        }
        let mut matches = Vec::new();
        for (index, listener) in self.listeners.iter().enumerate() {
            if let Some(data) = listener.pattern.captures(event)? {
                matches.push((
                    index,
                    EventMatch {
                        listener: listener.description.id,
                        event: event.into(),
                        pattern: listener.description.pattern.clone(),
                        data,
                    },
                ));
            }
        }
        Ok(matches)
    }
    /// Dispatch directly, without top-level lifecycle/status/output side effects.
    ///
    /// # Errors
    /// Returns matching/depth failures or the original annotated callback error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn action(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     Ok(Flow::Continue)
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("feature-run", action)?;
    /// let emitter = builder.build();
    /// let mut req = Request::new();
    /// let mut res = Response::new();
    /// emitter.emit("feature-run", &mut req, &mut res).await?;
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn emit<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        // A root emit mutates the caller's exact pair and carries no ambient invocation state.
        self.emit_nested(event, req, res, DispatchContext::default())
    }
    /// Dispatch with explicit caller-owned immutable state, without a host lifecycle.
    ///
    /// # Errors
    /// Returns ordinary matching/depth/callback failures. The returned Send future
    /// borrows context alongside the table and request/response; it is not `'static`.
    ///
    /// ```
    /// use stackpress_lib::{EmitterBuilder, Flow, Invocation, Request, Response, Result};
    /// async fn read(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&inv.context::<u64>().copied())?;
    ///     Ok(Flow::Continue)
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut builder = EmitterBuilder::new();
    /// builder.on("read", read)?;
    /// builder.build().emit_with_context(
    ///     "read", &mut Request::new(), &mut Response::new(), &7_u64,
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn emit_with_context<'a, T: Any + Send + Sync>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
        context: &'a T,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        // The foundation treats state as an opaque borrow, with no service/global policy.
        self.emit_nested(
            event,
            req,
            res,
            DispatchContext {
                ancestry: SharedAncestry::default(),
                state: Some(context),
            },
        )
    }
    /// Execute a phase with invocation-local captures and immutable ancestry.
    async fn phase(
        &self,
        listener: &Listener,
        matched: &EventMatch,
        phase: ListenerPhase,
        parent: &DispatchContext<'_>,
        req: &mut Request,
        res: &mut Response,
    ) -> Result<Flow> {
        // Select an optional hook or the listener action before creating the phase-local context.

        // Before/after use one receiver hook; Action always belongs to the selected listener.
        let hook = match phase {
            ListenerPhase::Before => self.before.as_ref(),
            ListenerPhase::After => self.after.as_ref(),
            ListenerPhase::Action => None,
        };
        // An absent hook continues without fabricating a phase callback or mutating req/res.
        let action: &dyn contract::Erased = if let Some(hook) = hook {
            hook.action.as_ref()
        } else if phase == ListenerPhase::Action {
            listener.action.as_ref()
        } else {
            return Ok(Flow::Continue);
        };
        let description = &listener.description;
        // Each phase adds owned current metadata while retaining immutable parent nodes.
        let ancestry = parent.ancestry.push(InvocationFrame {
            event: matched.event.clone(),
            pattern: description.pattern.clone(),
            listener: description.id.to_string(),
            label: description.label.clone(),
            plugin: description.plugin.clone(),
            priority: description.priority,
            phase: phase.name().into(),
            registered: description.registered.clone(),
            replaced: description.replaced.clone(),
            phase_registered: hook.map(|v| v.registered.clone()),
        });
        // Borrow only this match's captures; concurrent siblings receive independent match records.
        let invocation = Invocation {
            emitter: self,
            ancestry: ancestry.clone(),
            captures: &matched.data,
            context: parent.state,
        };
        // Await under exclusive req/res borrows, then annotate only failures; no callback lock
        // crosses the await.
        action
            .invoke(req, res, invocation)
            .await
            .map_err(|error| error.with_frames(ancestry.snapshot()))
    }
    /// Box recursive dispatch; no registry or callback lock survives an await.
    fn emit_nested<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
        parent: DispatchContext<'a>,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        // Box only the recursive boundary; all callbacks still borrow caller-owned state
        // sequentially.
        Box::pin(async move {
            // Stop a recursive cycle before allocating its next frame/future chain.
            if parent.ancestry.depth() >= self.depth_limit {
                return Err(Error::limit("dispatch depth limit exceeded")
                    .at_path(event)
                    .with_frames(parent.ancestry.snapshot()));
            }
            let matches = self.collect_matches(event)?;
            // No listener is an execution outcome; leave response status untouched.
            if matches.is_empty() {
                return Ok(DispatchOutcome::NoListener);
            }
            // Matches already follow globally sorted execution order, without winner selection.
            for (index, matched) in matches {
                // The table cannot mutate across await, so this checked position remains stable.
                let listener = self
                    .listeners
                    .get(index)
                    .ok_or_else(|| Error::new("immutable listener position invariant failed"))?;
                // Run phases in dependency order; ? and Abort both prevent entry into every later
                // phase.
                for phase in [
                    ListenerPhase::Before,
                    ListenerPhase::Action,
                    ListenerPhase::After,
                ] {
                    if self
                        .phase(listener, &matched, phase, &parent, req, res)
                        .await?
                        == Flow::Abort
                    {
                        // Cooperative stop preserves completed mutations and does not convert
                        // response status into an abort code.
                        return Ok(DispatchOutcome::Aborted);
                    }
                }
            }
            Ok(DispatchOutcome::Completed)
        })
    }
}

/// Borrowed dispatch/capture view for one phase, stable across nested awaits.
pub struct Invocation<'a> {
    /// Borrowed immutable runtime table; the context cannot escape its lifetime.
    emitter: &'a EventEmitter,
    /// Current phase link, retained while a nested child temporarily borrows this invocation.
    ancestry: SharedAncestry,
    /// Borrowed captures for this listener only; raw dispatch never injects them into request data.
    captures: &'a MatchData,
    /// Optional explicit host borrow; child dispatch never replaces it with request data.
    context: Option<&'a (dyn Any + Send + Sync)>,
}
impl Invocation<'_> {
    /// Borrow explicitly supplied typed context, or None for raw/context-type mismatch.
    ///
    /// The returned reference cannot escape this invocation borrow. Nested emit and
    /// resolve preserve the exact context pointer without cloning its payload.
    ///
    /// ```
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// async fn inspect(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    ///     if let Some(host) = inv.context::<String>() {
    ///         assert!(!host.is_empty());
    ///     }
    ///     Ok(Flow::Continue)
    /// }
    /// ```
    pub fn context<T: Any + Send + Sync>(&self) -> Option<&T> {
        // Checked downcasting exposes only immutable access to the caller's chosen type.
        self.context.and_then(|state| state.downcast_ref())
    }
    /// Borrow a named capture for this listener without allocating a String.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     assert_eq!(inv.param("id"), Some("7"));
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn param(&self, name: &str) -> Option<&str> {
        // Borrow this listener's named capture; no clone or automatic request-data injection is
        // needed.
        self.captures.params.get(name).map(String::as_str)
    }
    /// Resolve plain child data into isolated req/res values with inherited breadcrumbs.
    ///
    /// # Errors
    /// Returns input conversion or the original child dispatch failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     // The child receives fresh data; copying its envelope into the parent is explicit.
    ///     let result = inv.resolve("child", serde_json::json!({"id": 7})).await?;
    ///     res.apply_envelope(result.response)?;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn resolve<'a, I: IntoRequest + 'a>(
        &'a self,
        event: &'a str,
        input: I,
    ) -> HandlerFuture<'a, Result<Resolution>> {
        // Nested capability calls retain the parent path but create isolated request/response
        // values.
        self.emitter.resolve_nested(
            event,
            input,
            DispatchContext {
                ancestry: self.ancestry.clone(),
                state: self.context,
            },
        )
    }
    /// Inspect the current immutable frame.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     assert_eq!(inv.listener().pattern, "/profiles/:id");
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn listener(&self) -> &InvocationFrame {
        // Invocation is constructed only with a nonempty ancestry link owned for its borrow
        // lifetime.
        match self.ancestry.0.as_deref() {
            Some(node) => &node.frame,
            None => unreachable!("Invocation always has a frame"),
        }
    }
    /// Inspect this invocation's event rather than a global current-event slot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     assert_eq!(inv.event(), "/profiles/7");
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn event(&self) -> &str {
        // The frame remains stable after a nested await; there is no global current event to
        // restore.
        &self.listener().event
    }
    /// Inspect captures without injecting them into shared request data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     assert_eq!(inv.captures().params["id"], "7");
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn captures(&self) -> &MatchData {
        // Expose immutable capture metadata without permitting a sibling to overwrite it.
        self.captures
    }
    /// Snapshot a root-first path only when needed for inspection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     assert!(!inv.path().is_empty());
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn path(&self) -> Vec<InvocationFrame> {
        // Materialize a diagnostic vector only for explicit inspection, not on every listener
        // entry.
        self.ancestry.snapshot()
    }
    /// Dispatch a nested operation, reborrowing caller values and inheriting ancestry.
    ///
    /// # Errors
    /// Returns the child's original annotated error or depth/matching failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{
    /// #     EmitterBuilder, EventEmitter, EventPattern, Flow, Invocation, ListenerSpec,
    /// #     MethodPattern, Request, Response, Result,
    /// # };
    /// async fn inspect(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     // Reborrow this same pair until the child finishes, then resume the parent.
    ///     inv.emit("child", req, res).await?;
    ///     Ok(Flow::Continue)
    /// }
    /// let mut builder = EmitterBuilder::new();
    /// builder.on_pattern(EventPattern::express("/profiles/:id", '/')?, inspect)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn emit<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<DispatchOutcome>> {
        // Nested emit reborrows the caller pair and shares only immutable parent ancestry.
        self.emitter.emit_nested(
            event,
            req,
            res,
            DispatchContext {
                ancestry: self.ancestry.clone(),
                state: self.context,
            },
        )
    }
}
