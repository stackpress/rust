//! Application packaging hooks preserving the source build event's non-awaited timing.
//!
//! Keep an Arc to Packaging as the lifecycle handle. Completion errors never become build
//! response errors. Explicit shutdown closes submissions, aborts work and waits for destruction;
//! dropping the last owner aborts unfinished jobs rather than detaching them.

use super::NativeFuture;
use crate::native::config::string;
use serde_json::Value;
use stackpress_lib::Result;
use std::{
    future::poll_fn,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard},
};
use tokio::{
    runtime::Handle,
    sync::Mutex as AsyncMutex,
    task::{JoinError, JoinSet},
};

/// Owned source paths supplied to an optional application packaging callback.
pub struct PackagePaths {
    /// Source server.cwd, already resolved by the view configuration phase.
    pub cwd: PathBuf,
    /// Source server.build, defaulting to cwd/.build.
    pub build: PathBuf,
}

/// App-owned packaging implementation; Rust supplies no npm manifest-writing implementation.
pub type Package = Arc<dyn Fn(PackagePaths) -> NativeFuture<()> + Send + Sync>;

/// Provider completion or task panic/cancellation, retained for the lifecycle owner.
pub type PackageOutcome = std::result::Result<Result<()>, JoinError>;

/// One application's optional callback and supervised, non-detached packaging jobs.
pub struct Packaging {
    /// Explicit executor: no current-runtime lookup or runtime creation during registration.
    runtime: Handle,
    /// Invoked inside each owned task, never while holding the supervision mutex.
    callback: Package,
    /// Only brief mutation/poll sections lock task ownership; no lock spans an await.
    state: Mutex<State>,
    /// Only one waiter polls JoinSet at a time, retaining its completion waker.
    joining: AsyncMutex<()>,
}

/// Closed submission state and every task not yet joined by its lifecycle owner.
struct State {
    /// Shutdown closes this before awaiting, preventing new jobs after the reap barrier.
    accepting: bool,
    /// JoinSet owns jobs even after completion; no JoinHandle is detached.
    jobs: JoinSet<Result<()>>,
}

impl Packaging {
    /// Bind an explicitly owned runtime and callback without spawning or executing anything.
    pub fn new(runtime: Handle, callback: Package) -> Self {
        Self {
            runtime,
            callback,
            state: Mutex::new(State {
                accepting: true,
                jobs: JoinSet::new(),
            }),
            joining: AsyncMutex::new(()),
        }
    }

    /// Borrow task state briefly; no application code runs under this mutex.
    fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Submit at source build.ts:21; neither callback completion nor failure reaches its parent.
    pub(super) fn submit(&self, cwd: String, destination: Value) {
        let mut state = self.state();
        // The lifecycle owner may keep the host alive after shutdown, but no new job can escape.
        if !state.accepting {
            return;
        }
        let callback = self.callback.clone();
        state.jobs.spawn_on(
            async move {
                // Path validation belongs to the asynchronous packaging call, as upstream does.
                let build = PathBuf::from(string(Some(&destination), "")?);
                callback(PackagePaths {
                    cwd: cwd.into(),
                    build,
                })
                .await
            },
            &self.runtime,
        );
    }

    /// Join every retained completion without changing a previously returned build response.
    ///
    /// Concurrent submissions remain owned; shutdown can abort a suspended callback even while
    /// drain is waiting. Cancelling drain leaves unfinished jobs in this supervision group.
    pub async fn drain(&self) -> Vec<PackageOutcome> {
        let _joining = self.joining.lock().await;
        self.completions().await
    }

    /// Poll only while briefly holding state, so submission and shutdown never wait on a job.
    async fn completions(&self) -> Vec<PackageOutcome> {
        let mut outcomes = Vec::new();
        while let Some(outcome) = poll_fn(|context| self.state().jobs.poll_join_next(context)).await
        {
            outcomes.push(outcome);
        }
        outcomes
    }

    /// Stop accepting, abort pending work and reap it before returning; safe with a live drain.
    pub async fn shutdown(&self) {
        {
            let mut state = self.state();
            state.accepting = false;
            // Abort before acquiring the join gate: a concurrent drain must be woken first.
            state.jobs.abort_all();
        }
        let _joining = self.joining.lock().await;
        let _ = self.completions().await;
    }
}
