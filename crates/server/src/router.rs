//! Host-only route descriptors and capture injection over unchanged lib dispatch.

use stackpress_lib::{EventPattern, Handler, ListenerSpec, MethodPattern, Nest, handler};
use std::sync::Arc;

/// How a host route exposes its listener-local named captures.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CapturePolicy {
    /// Atomically overlay literal aggregate-data keys; incoming route captures win.
    #[default]
    NamedIntoData,
    /// Keep captures solely in Invocation, as ordinary lib registrations do.
    InvocationOnly,
}

/// Explicit method/path contribution; duplicate matchers remain independent listeners.
///
/// The default capture policy atomically overlays named captures onto literal aggregate-data
/// keys before this route action. Incoming captures win; query/post and positional captures
/// remain separate. Replacement by listener ID retains this policy and original provenance.
///
/// # Examples
///
/// ```rust
/// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
/// use stackpress_server::ServerBuilder;
/// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
///     res.results(&"hello")?;
///     Ok(Flow::Continue)
/// }
/// use stackpress_lib::MethodPattern;
/// use stackpress_server::RouteSpec;
/// let mut app = ServerBuilder::new();
/// let spec = RouteSpec::new(MethodPattern::GET, "/hello").priority(10).label("hello");
/// app.route_with(spec, greet)?;
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
#[derive(Debug)]
#[must_use = "the configured value must be used"]
pub struct RouteSpec {
    /// Checked method matcher, including Any and extension verbs.
    pub(crate) method: MethodPattern,
    /// Pattern text, compiled before insertion.
    pub(crate) path: String,
    /// Signed ordering priority, with stable registration ties.
    pub(crate) priority: i64,
    /// Optional diagnostic label.
    pub(crate) label: Option<String>,
    /// Host capture policy retained across handle replacement.
    pub(crate) captures: CapturePolicy,
}

impl RouteSpec {
    /// Describe a route at priority zero with incoming named captures winning.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// use stackpress_lib::MethodPattern;
    /// use stackpress_server::RouteSpec;
    /// let mut app = ServerBuilder::new();
    /// let spec = RouteSpec::new(MethodPattern::GET, "/hello").priority(10).label("hello");
    /// app.route_with(spec, greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(method: MethodPattern, path: &str) -> Self {
        // Defer fallible pattern compilation to the registration boundary.
        Self {
            method,
            path: path.into(),
            priority: 0,
            label: None,
            captures: CapturePolicy::NamedIntoData,
        }
    }
    /// Set execution priority without changing specificity or equal-priority order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// use stackpress_lib::MethodPattern;
    /// use stackpress_server::RouteSpec;
    /// let mut app = ServerBuilder::new();
    /// let spec = RouteSpec::new(MethodPattern::GET, "/hello").priority(10).label("hello");
    /// app.route_with(spec, greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn priority(mut self, priority: i64) -> Self {
        // Lib remains the sole sorting authority.
        self.priority = priority;
        self
    }
    /// Set a diagnostic label, validated before the listener is installed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::ServerBuilder;
    /// async fn greet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.results(&"hello")?;
    ///     Ok(Flow::Continue)
    /// }
    /// use stackpress_lib::MethodPattern;
    /// use stackpress_server::RouteSpec;
    /// let mut app = ServerBuilder::new();
    /// let spec = RouteSpec::new(MethodPattern::GET, "/hello").priority(10).label("hello");
    /// app.route_with(spec, greet)?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn label(mut self, label: impl Into<String>) -> Self {
        // Labels are metadata rather than replacement authority.
        self.label = Some(label.into());
        self
    }
    /// Select whether this host route injects named captures into aggregate data.
    pub fn captures(mut self, policy: CapturePolicy) -> Self {
        // Policy follows the stable listener ID through later replacement.
        self.captures = policy;
        self
    }
    /// Compile the route into the foundation's common all-matching specification.
    pub(crate) fn listener(self) -> stackpress_lib::Result<ListenerSpec> {
        let mut spec = ListenerSpec::pattern(EventPattern::route(self.method, &self.path)?)
            .priority(self.priority);
        // No label means the foundation generates a stable listener identity label.
        if let Some(label) = self.label {
            spec = spec.label(label);
        }
        Ok(spec)
    }
}

/// Wrap only host routes; raw lib captures and nesting semantics remain unchanged.
pub(crate) fn wrap(policy: CapturePolicy, action: impl Handler + 'static) -> impl Handler {
    // Share the compiled action explicitly; only this wrapper applies host capture policy.
    let action = Arc::new(action);
    handler(move |req, res, inv| {
        let action = Arc::clone(&action);
        Box::pin(async move {
            // Only opted-in host routes overlay aggregate data; raw lib captures remain untouched.
            if policy == CapturePolicy::NamedIntoData {
                // Literal keys prevent punctuation in parameter names becoming a data path.
                let mut captures = Nest::with_limits(req.data.limits());
                // Copy named parameters only; positional captures stay private to this Invocation.
                for (name, value) in &inv.captures().params {
                    captures.set(name, value.clone())?;
                }
                // Nest stages its shallow overlay atomically under aggregate-data limits.
                req.data.merge(captures)?;
            }
            // The sealed Handler supertrait already preserves the borrowing Send future.
            action.invoke(req, res, inv).await
        })
    })
}
