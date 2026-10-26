//! Cooperative cancellation and consumed unwind at the Resource loader's Owner.
//!
//! Wrap the loader inside its Resource fetcher. Cleanup wakes the detached task
//! and drops its loader on its next poll. CPU-bound/non-yielding work cannot be
//! interrupted; side effects are not rolled back. Loaders must be async, bounded
//! and subject to host deadlines. Native unwind is converted to a safe outcome;
//! panic-abort targets and arbitrary panicking destructors cannot be recovered.

use futures::{
    FutureExt,
    future::{AbortHandle, Abortable},
};
use leptos::prelude::Owner;
use serde::{Deserialize, Serialize};
use std::{fmt, future::Future, sync::Arc};

/// Safe serializable failure categories; developer panic text is separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceFailureKind {
    /// The owning Resource was disposed, or no Owner admitted the loader.
    Cancelled,
    /// A native yielding loader unwound; its consumed future is never retried.
    Panicked,
}

/// Public failure with an optional native-only cause omitted from replay.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceFailure {
    /// Stable public category, safe to serialize through Resource's codec.
    kind: ResourceFailureKind,
    /// Only explicit native cause traversal receives the developer panic payload.
    #[serde(skip)]
    cause: Option<Arc<dyn std::error::Error + Send + Sync>>,
}

impl ResourceFailure {
    /// Read the safe outcome without inspecting private diagnostic data.
    pub fn kind(&self) -> ResourceFailureKind {
        self.kind
    }
    /// Cancellation has no synthetic developer cause to serialize or expose.
    fn cancelled() -> Self {
        Self {
            kind: ResourceFailureKind::Cancelled,
            cause: None,
        }
    }
}

impl fmt::Display for ResourceFailure {
    /// Ordinary UI Display never includes developer messages.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.kind {
            ResourceFailureKind::Cancelled => "Resource cancelled",
            ResourceFailureKind::Panicked => "Resource failed",
        })
    }
}
impl std::error::Error for ResourceFailure {
    /// Preserve the concrete native failure without copying its Display into an envelope.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause
            .as_deref()
            .map(|cause| cause as &(dyn std::error::Error + 'static))
    }
}

/// Developer-only textual panic payload; no public serializer is implemented.
#[derive(Debug)]
struct LoaderPanic(String);
impl LoaderPanic {
    /// Preserve supported textual panic payloads after consuming the failed future.
    fn new(payload: &(dyn std::any::Any + Send)) -> Self {
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("non-text panic payload");
        Self(message.to_owned())
    }
}
impl fmt::Display for LoaderPanic {
    /// This message is available only to native diagnostic traversal.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for LoaderPanic {}

/// Cooperatively drop a loader on Owner cleanup and consume native loader unwind.
///
/// Call **inside** the Resource fetcher. A completed result is unchanged by later
/// cleanup. Missing Owner cancels before polling. Source changes do not forcibly
/// interrupt an already pending upstream Resource. The future remains Send only
/// when the loader is Send; no executor or Send assertion is replaced.
///
/// # Examples
///
/// ```no_run
/// use leptos::prelude::*;
/// use stackpress_framework_ui::resources::cancel_on_cleanup;
/// let resource = Resource::new(|| (), |_| cancel_on_cleanup(async { "public data".to_owned() }));
/// ```
///
/// A non-Send loader remains rejected even through this wrapper:
///
/// ```compile_fail
/// use leptos::prelude::*;
/// use stackpress_framework_ui::resources::cancel_on_cleanup;
/// let resource = Resource::new(|| (), |_| cancel_on_cleanup(async {
///     let local = std::rc::Rc::new(1);
///     std::future::pending::<()>().await;
///     *local
/// }));
/// ```
pub fn cancel_on_cleanup<F: Future>(
    loader: F,
) -> impl Future<Output = Result<F::Output, ResourceFailure>> {
    let (abort, registration) = AbortHandle::new_pair();
    if Owner::current().is_some() {
        // Abort wakes even a loader that never wakes itself.
        Owner::on_cleanup(move || abort.abort());
    } else {
        // Never start unowned work which cannot participate in cleanup.
        abort.abort();
    }
    async move {
        // Catch inside ScopedFuture's poll: its Owner/observer restoration then
        // follows the normal return path. The failed future is consumed once.
        let loader = std::panic::AssertUnwindSafe(loader).catch_unwind();
        match Abortable::new(loader, registration).await {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(payload)) => Err(ResourceFailure {
                kind: ResourceFailureKind::Panicked,
                cause: Some(Arc::new(LoaderPanic::new(payload.as_ref()))),
            }),
            Err(_) => Err(ResourceFailure::cancelled()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Missing/disposed owners never poll work; completed data survives later cleanup.
    #[test]
    fn cancellation_before_poll_and_after_completion() {
        let polled = Arc::new(AtomicBool::new(false));
        let capture = polled.clone();
        let future = cancel_on_cleanup(async move {
            capture.store(true, Ordering::SeqCst);
            7
        });
        assert_eq!(
            futures::executor::block_on(future).unwrap_err().kind(),
            ResourceFailureKind::Cancelled
        );
        assert!(!polled.load(Ordering::SeqCst));
        let owner = Owner::new();
        owner.set();
        let future = cancel_on_cleanup(async { 7 });
        owner.cleanup();
        assert_eq!(
            futures::executor::block_on(future).unwrap_err().kind(),
            ResourceFailureKind::Cancelled
        );
        let result = futures::executor::block_on(cancel_on_cleanup(async { 9 }));
        owner.unset_with_forced_cleanup();
        assert_eq!(result.unwrap(), 9);
        assert_eq!(
            ResourceFailure::cancelled().to_string(),
            "Resource cancelled"
        );
    }
}
