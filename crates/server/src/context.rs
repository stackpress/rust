//! Immutable typed host services and the Invocation access/delegation boundary.

use std::any::Any;
use std::fmt;
use std::sync::Arc;

use stackpress_lib::{
    DispatchOutcome, Error, HandlerFuture, Invocation, Request, Response, Result,
};

use crate::report::{LifecycleStage, StageFailure};
use crate::services::Registry;

/// Read-only registry view shared by an explicitly retained Arc, without an emitter back-reference.
pub struct HostContext {
    /// Decoded and validated package-owned sections, frozen before Config tasks.
    pub(crate) config: Registry,
    /// Services freeze after Config startup completes.
    pub(crate) services: Registry,
}

impl HostContext {
    /// Borrow a package-owned decoded configuration section.
    ///
    /// # Errors
    /// Returns a missing-type error when that section was not registered.
    pub fn config<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Config lookup is independent of caller-controlled Request.data.
        self.config.get()
    }

    /// Borrow a typed shared service; interior synchronization remains T's responsibility.
    ///
    /// # Errors
    /// Returns a missing-type error instead of inferring a service from JSON.
    pub fn service<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Read-only table access holds no lock over the caller's subsequent await.
        self.services.get()
    }

    /// Borrow a named service only when its registered concrete type agrees.
    ///
    /// # Errors
    /// Returns an error for a missing name or mismatched concrete type, and on invocation
    /// shortcuts also for absent host context. No registry state is changed.
    pub fn service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T> {
        // Named slots are immutable after Config, just like singleton slots.
        self.services.named(name)
    }

    /// Inspect original and replacement singleton sites without exposing service contents.
    ///
    /// # Errors
    /// Returns an error when the concrete singleton service was not registered.
    pub fn service_sites<T: Any + Send + Sync>(
        &self,
    ) -> Result<(
        &stackpress_lib::SourceLocation,
        Option<&stackpress_lib::SourceLocation>,
    )> {
        // Provenance remains attached to the frozen slot through replacement.
        self.services.sites::<T>()
    }

    /// Explicitly clone the service's Arc handle, retaining the same allocation.
    ///
    /// # Errors
    /// Returns a missing-type error when no such service exists.
    pub fn shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        // Sharing is visible at the call site and in the returned Arc type.
        self.services.shared()
    }
}

/// Explicit per-root metadata borrowed by lib dispatch and inherited by children.
pub(crate) struct CallContext {
    /// No emitter/callback is stored here, avoiding a framework ownership cycle.
    pub(crate) host: Arc<HostContext>,
    /// None for raw roots; top-level allocation arrives with the M2 lifecycle.
    pub(crate) correlation: Option<String>,
    /// Children inherit the root's active host stage unchanged.
    pub(crate) stage: LifecycleStage,
    /// Original failing stage/error during an Error offer; inherited by nested dispatch.
    pub(crate) failure: Option<Arc<StageFailure>>,
    /// Trusted request-local adapter metadata, never copied from Request.data.
    pub(crate) adapter_context: Option<Arc<dyn Any + Send + Sync>>,
}

/// A required delegated capability was absent; the event name remains diagnostic data.
#[derive(Debug)]
pub enum DelegationError {
    /// The target event matched no registered listener.
    NoListener {
        /// Bounded event name that the caller explicitly tried to delegate.
        event: String,
    },
}

impl fmt::Display for DelegationError {
    /// Identify the missing operation without converting no-listener into success.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The structured variant remains available through the source chain for downcasting.
        match self {
            Self::NoListener { event } => {
                write!(formatter, "delegated capability `{event}` has no listener")
            }
        }
    }
}

impl std::error::Error for DelegationError {}

/// Handler shortcuts over the host context; authors import this trait, never implement it.
pub trait ServerInvocationExt {
    /// Access the complete immutable host context, or error for a non-host lib invocation.
    ///
    /// # Errors
    /// Returns an error when the invocation carries no Stackpress host context.
    fn host(&self) -> Result<&HostContext>;
    /// Borrow a typed service directly from the current invocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
    /// async fn greet(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    ///     res.results(inv.service::<String>()?)?;
    ///     Ok(Flow::Continue)
    /// }
    /// async fn configure(start: &mut StartupContext) -> Result<Flow> {
    ///     start.register(Arc::new("ready".to_owned()))?;
    ///     Ok(Flow::Continue)
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on_config(configure)?;
    /// app.on("greet", greet)?;
    /// let reply = app.build().await?.call("greet", serde_json::json!({})).await?;
    /// assert_eq!(reply.response.result().and_then(|v| v.as_str()), Some("ready"));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns an error for absent host context or an unregistered concrete service type.
    fn service<T: Any + Send + Sync>(&self) -> Result<&T>;
    /// Borrow a typed config section directly from the current invocation.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::{ServerBuilder, ServerInvocationExt};
    /// async fn limit(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    ///     res.results(inv.config::<u64>()?)?;
    ///     Ok(Flow::Continue)
    /// }
    /// fn validate(_: &u64) -> Result<()> {
    ///     Ok(())
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.config_section("limit", 4_u64, validate)?;
    /// app.on("limit", limit)?;
    /// let server = app.build().await?;
    /// let reply = server.call("limit", serde_json::json!({})).await?;
    /// assert!(reply.response.error().is_none());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns an error for absent host context or an unregistered concrete config type.
    fn config<T: Any + Send + Sync>(&self) -> Result<&T>;
    /// Explicitly retain a service Arc directly from the current invocation.
    ///
    /// # Errors
    /// Returns an error for absent host context or an unregistered service type;
    /// no new service allocation is created.
    fn shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>>;
    /// Inspect the inherited root stage, or error without host context.
    ///
    /// # Errors
    /// Returns an error when the invocation has no Stackpress host context.
    fn stage(&self) -> Result<LifecycleStage>;
    /// Inspect a top-level correlation, None for raw host dispatch roots.
    ///
    /// # Errors
    /// Returns an error without host context; a raw host root instead returns Ok(None).
    fn correlation_id(&self) -> Result<Option<&str>>;
    /// Inspect the retained original failure during an Error offer and its nested calls.
    ///
    /// # Errors
    /// Returns an error without host context; no active Error offer instead returns Ok(None).
    fn failure(&self) -> Result<Option<&StageFailure>>;
    /// Borrow typed trusted adapter metadata, absent for ordinary raw roots/lib calls.
    fn adapter_context<T: Any + Send + Sync>(&self) -> Option<&T>;
    /// Borrow a deliberately named service from the current host.
    ///
    /// # Errors
    /// Returns an error for a missing name or mismatched concrete type, and on invocation
    /// shortcuts also for absent host context. No registry state is changed.
    fn service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T>;
    /// Delegate explicitly: child Abort maps to Abort, completion to Continue, missing to error.
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
    /// use stackpress_server::ServerInvocationExt;
    /// async fn expose(
    ///     req: &mut Request,
    ///     res: &mut Response,
    ///     inv: Invocation<'_>,
    /// ) -> Result<Flow> {
    ///     inv.delegate("greet", req, res).await
    /// }
    /// # #[tokio::main(flavor = "current_thread")]
    /// # async fn main() -> stackpress_server::BuildResult<()> {
    /// let mut app = ServerBuilder::new();
    /// app.on("greet", greet)?;
    /// app.get("/hello", expose)?;
    /// let reply = app.build().await?.call("GET /hello", serde_json::json!({})).await?;
    /// assert!(reply.report.is_success(&reply.response));
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns nested dispatch/handler failures or a typed [`DelegationError::NoListener`].
    /// Completed request/response mutations are retained; cancellation drops the active
    /// child future without rolling back application side effects.
    fn delegate<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<stackpress_lib::Flow>>;
}

impl ServerInvocationExt for Invocation<'_> {
    /// Look up only the server's trusted context type, not values in caller data.
    fn host(&self) -> Result<&HostContext> {
        // Raw lib calls remain valid, but cannot pretend to carry a host registry.
        Ok(&call_context(self)?.host)
    }

    /// Borrow the requested service through the common missing-type error path.
    fn service<T: Any + Send + Sync>(&self) -> Result<&T> {
        // This shortcut retains exactly the same borrow as explicit host access.
        self.host()?.service()
    }

    /// Borrow a decoded typed section without cloning its data.
    fn config<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Typed config remains independent from JSON request values.
        self.host()?.config()
    }

    /// Retain a deliberately shared service allocation.
    fn shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        // The explicit Arc return makes lifetime extension visible to the caller.
        self.host()?.shared_service()
    }

    /// Return this root stage without confusing it with a listener phase.
    fn stage(&self) -> Result<LifecycleStage> {
        // The concrete call context is immutable throughout nested dispatch.
        Ok(call_context(self)?.stage)
    }

    /// Borrow the current root correlation, if this is a top-level invocation.
    fn correlation_id(&self) -> Result<Option<&str>> {
        // Raw roots intentionally report None rather than allocate an unrelated ID.
        Ok(call_context(self)?.correlation.as_deref())
    }

    /// Preserve the original failing stage while this invocation itself runs in Error.
    fn failure(&self) -> Result<Option<&StageFailure>> {
        // Children inherit the same immutable diagnostic ownership.
        Ok(call_context(self)?.failure.as_deref())
    }

    /// Downcast only the trusted adapter slot, never caller-controlled request JSON.
    fn adapter_context<T: Any + Send + Sync>(&self) -> Option<&T> {
        // Plain lib calls and raw host roots honestly have no adapter metadata.
        call_context(self)
            .ok()?
            .adapter_context
            .as_deref()?
            .downcast_ref()
    }

    /// Borrow a named slot with the same checked type boundary as HostContext.
    fn service_named<T: Any + Send + Sync>(&self, name: &str) -> Result<&T> {
        // The registry does not acquire a lock or infer services from input data.
        self.host()?.service_named(name)
    }

    /// Map the existing raw outcome explicitly, preserving nested errors and ancestry.
    fn delegate<'a>(
        &'a self,
        event: &'a str,
        req: &'a mut Request,
        res: &'a mut Response,
    ) -> HandlerFuture<'a, Result<stackpress_lib::Flow>> {
        Box::pin(async move {
            // No lifecycle, request loading or response merging is hidden in delegation.
            match self.emit(event, req, res).await? {
                DispatchOutcome::Completed => Ok(stackpress_lib::Flow::Continue),
                DispatchOutcome::Aborted => Ok(stackpress_lib::Flow::Abort),
                DispatchOutcome::NoListener => {
                    Err(Error::from_source(DelegationError::NoListener {
                        event: event.to_owned(),
                    })
                    .with_code(404)
                    .with_public_message("Not Found"))
                }
            }
        })
    }
}

/// Borrow host metadata without exposing a mutable runtime or performing a global lookup.
fn call_context<'a>(inv: &'a Invocation<'_>) -> Result<&'a CallContext> {
    // Any payload type does not imply a static payload reference.
    inv.context()
        .ok_or_else(|| Error::new("invocation has no Stackpress host context"))
}
