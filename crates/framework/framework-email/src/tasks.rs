//! Supervision for source non-awaited hooks. Own one group per application runtime.
//!
//! The caller submits an owned future (for example a resolve using a fresh request
//! and response). The parent does not await it. A lifecycle owner must later drain
//! or shut down the group. Dropping the group aborts unfinished jobs instead of
//! detaching them; explicit shutdown additionally waits for destruction.

use crate::EmailResult;
use std::future::Future;
use tokio::{
    runtime::Handle,
    task::{JoinError, JoinSet},
};

/// Completion or task failure retained for the lifecycle owner, not the parent response.
pub type TaskOutcome = Result<EmailResult<()>, JoinError>;

/// Owned jobs preserving submission order without promising scheduler completion order.
#[derive(Default)]
pub struct TaskSupervisor {
    /// JoinSet owns every spawned task; no JoinHandle is silently detached.
    jobs: JoinSet<EmailResult<()>>,
}

impl TaskSupervisor {
    /// Construct an empty group; no executor or network resource is acquired.
    pub fn new() -> Self {
        // An empty set may be constructed outside a runtime.
        Self::default()
    }

    /// Schedule an owned hook without awaiting completion or borrowing parent state.
    ///
    /// Explicit runtime ownership avoids an implicit current-runtime panic. The future
    /// must use independent request/response values, matching source resolve semantics.
    ///
    /// ```compile_fail
    /// use stackpress_framework_email::tasks::TaskSupervisor;
    /// fn borrowed(runtime: &tokio::runtime::Handle) {
    ///     let mut parent = String::new();
    ///     let mut jobs = TaskSupervisor::new();
    ///     jobs.spawn_on(runtime, async { parent.push('x'); Ok(()) });
    /// }
    /// ```
    pub fn spawn_on<F>(&mut self, runtime: &Handle, job: F)
    where
        F: Future<Output = EmailResult<()>> + Send + 'static,
    {
        // The runtime polls the future even when the parent never joins it.
        self.jobs.spawn_on(job, runtime);
    }

    /// Submit a non-awaited raw event resolve using an explicitly owned host/input.
    ///
    /// Use the original hook name, such as `email-email-send` or `auth-phone-verify`.
    /// A fresh child response is discarded as in the source non-awaited call; thrown
    /// errors remain available through [`Self::join_next`]. No top-level lifecycle,
    /// parent response merge, missing-listener error or implicit event rename is added.
    /// Submission order is preserved; independent tasks may complete in either order.
    ///
    /// ```compile_fail
    /// use std::sync::Arc;
    /// use stackpress_framework_email::tasks::TaskSupervisor;
    /// use stackpress_lib::Request;
    /// use stackpress_server::Server;
    /// fn borrowed(runtime: &tokio::runtime::Handle, server: Arc<Server>) {
    ///     let mut parent = Request::new();
    ///     TaskSupervisor::new().resolve_on(runtime, server, "email-email-send", &mut parent);
    /// }
    /// ```
    #[cfg(feature = "server")]
    pub fn resolve_on<I>(
        &mut self,
        runtime: &Handle,
        server: std::sync::Arc<stackpress_server::Server>,
        event: impl Into<String>,
        input: I,
    ) where
        I: stackpress_lib::IntoRequest + 'static,
    {
        // Move only owned host/input/event into the job; no Invocation or parent borrow escapes.
        let event = event.into();
        self.spawn_on(runtime, async move {
            server.resolve(&event, input).await?;
            Ok(())
        });
    }

    /// Inspect whether every retained task has been joined.
    pub fn is_empty(&self) -> bool {
        // Finished-but-unjoined jobs remain supervised and therefore count as nonempty.
        self.jobs.is_empty()
    }

    /// Join one completion, retaining provider errors and panic/cancellation diagnostics.
    pub async fn join_next(&mut self) -> Option<TaskOutcome> {
        // Cancellation of this wait leaves tasks owned by the JoinSet.
        self.jobs.join_next().await
    }

    /// Abort all remaining jobs and wait until their futures/resources are dropped.
    pub async fn shutdown(&mut self) {
        // Explicit shutdown is the reap barrier; dropping alone only requests abort.
        self.jobs.shutdown().await;
    }
}
