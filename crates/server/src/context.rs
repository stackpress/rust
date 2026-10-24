//! Immutable typed host services and the Invocation access/delegation boundary.

use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use stackpress_lib::{
    DispatchOutcome, Error, HandlerFuture, Invocation, Request, Response, Result,
};

/// Owned construction-time slots; only immutable borrows escape after host build.
#[derive(Default)]
pub(crate) struct Registry {
    /// Each payload is explicitly shared, never cloned as an ordinary config value.
    values: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Registry {
    /// Insert one typed singleton without replacing an earlier contribution.
    pub(crate) fn insert<T: Any + Send + Sync>(&mut self, value: Arc<T>) -> Result<()> {
        // Entry lookup and insertion are synchronous; no lock or callback is involved.
        if self.values.contains_key(&TypeId::of::<T>()) {
            return Err(Error::invalid(format!(
                "duplicate typed value: {}",
                type_name::<T>()
            )));
        }
        self.values.insert(TypeId::of::<T>(), value);
        Ok(())
    }

    /// Read a typed singleton, reporting its missing type instead of panicking.
    pub(crate) fn get<T: Any + Send + Sync>(&self) -> Result<&T> {
        // Checked downcasting also protects the registry invariant during future maintenance.
        self.values
            .get(&TypeId::of::<T>())
            .and_then(|value| value.downcast_ref())
            .ok_or_else(|| Error::new(format!("missing typed value: {}", type_name::<T>())))
    }

    /// Explicitly retain a service beyond one handler borrow when the caller requests sharing.
    fn shared<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        // Arc downcast preserves allocation identity; it does not invoke T::clone.
        self.values
            .get(&TypeId::of::<T>())
            .map(Arc::clone)
            .and_then(|value| value.downcast().ok())
            .ok_or_else(|| Error::new(format!("missing typed value: {}", type_name::<T>())))
    }
}

/// Read-only registry view shared by an explicitly retained Arc, without an emitter back-reference.
pub struct HostContext {
    /// Decoded typed config sections; M0 tests seed these privately pending M1 decoding.
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

    /// Explicitly clone the service's Arc handle, retaining the same allocation.
    ///
    /// # Errors
    /// Returns a missing-type error when no such service exists.
    pub fn shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        // Sharing is visible at the call site and in the returned Arc type.
        self.services.shared()
    }
}

/// Host stage, independent of the lib's per-listener before/action/after phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LifecycleStage {
    /// Adapter input construction/loading.
    Input,
    /// General request preprocessing.
    Request,
    /// Selected capability or method/path event.
    Operation,
    /// General response processing.
    Response,
    /// An explicit error offer.
    Error,
    /// Boundary serialization/delivery.
    Output,
    /// Raw dispatch without top-level lifecycle or invented correlation.
    Dispatch,
}

/// Explicit per-root metadata borrowed by lib dispatch and inherited by children.
pub(crate) struct CallContext {
    /// No emitter/callback is stored here, avoiding a framework ownership cycle.
    pub(crate) host: Arc<HostContext>,
    /// None for raw roots; top-level allocation arrives with the M2 lifecycle.
    pub(crate) correlation: Option<String>,
    /// Children inherit the root's active host stage unchanged.
    pub(crate) stage: LifecycleStage,
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
    fn host(&self) -> Result<&HostContext>;
    /// Borrow a typed service directly from the current invocation.
    fn service<T: Any + Send + Sync>(&self) -> Result<&T>;
    /// Borrow a typed config section directly from the current invocation.
    fn config<T: Any + Send + Sync>(&self) -> Result<&T>;
    /// Explicitly retain a service Arc directly from the current invocation.
    fn shared_service<T: Any + Send + Sync>(&self) -> Result<Arc<T>>;
    /// Inspect the inherited root stage, or error without host context.
    fn stage(&self) -> Result<LifecycleStage>;
    /// Inspect a top-level correlation, None for raw host dispatch roots.
    fn correlation_id(&self) -> Result<Option<&str>>;
    /// Delegate explicitly: child Abort maps to Abort, completion to Continue, missing to error.
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
