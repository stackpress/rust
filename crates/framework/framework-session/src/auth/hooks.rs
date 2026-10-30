//! Owned nonawaited signup hooks, using Lane B's frozen supervisor contract.

use crate::columns::Record;
use stackpress_framework_email::{
    EmailResult,
    tasks::{TaskOutcome, TaskSupervisor},
};
use stackpress_server::Server;
use std::sync::{Arc, Mutex, Weak};
use tokio::{runtime::Handle, sync::Mutex as AsyncMutex};

/// Application-lifetime group for email-email-send/auth-phone-verify extension events.
///
/// After build, explicitly bind the application's `Arc<Server>` and Tokio Handle before
/// serving calls. A Weak host avoids a host/service ownership cycle. Submissions before
/// binding remain owned and queued; binding starts them in original submission order.
/// The parent never awaits hook completion; [`Self::drain`] retains later failures.
#[derive(Default)]
pub struct Hooks {
    /// Brief synchronous state protects binding and submission ordering only.
    state: Mutex<State>,
    /// Joining never holds the submission mutex across await.
    completed: AsyncMutex<()>,
}
/// One host binding and all jobs owned by this application's supervisor.
#[derive(Default)]
struct State {
    /// Weak runtime ownership, never a registry-to-server strong cycle.
    host: Option<(Weak<Server>, Handle)>,
    /// Source hooks not yet scheduled because the host is still being constructed.
    pending: Vec<(&'static str, Record)>,
    /// Frozen email task ownership, not a separate detached spawn mechanism.
    tasks: TaskSupervisor,
}
impl Hooks {
    /// Bind once the application's immutable server exists, then schedule queued hooks.
    /// Existing in-flight jobs retain their own Arc and are not canceled by rebinding.
    pub fn bind(&self, server: &Arc<Server>, runtime: Handle) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        state.host = Some((Arc::downgrade(server), runtime));
        let pending = std::mem::take(&mut state.pending);
        for (event, data) in pending {
            state.submit(event, data);
        }
    }
    /// Queue or spawn one independent resolve; no parent Request/Response is borrowed.
    pub(crate) fn submit(&self, event: &'static str, data: Record) {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .submit(event, data);
    }
    /// Count hooks still waiting for explicit runtime binding.
    pub fn pending(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .pending
            .len()
    }
    /// Join all currently submitted tasks, preserving each transport/hook failure.
    /// Submissions during this drain belong to the next drain; no submission mutex crosses await.
    /// Canceling this lifecycle wait drops/aborts its taken task group; shutdown is the
    /// explicit reap barrier. Call from the lifecycle owner, never from a job joining itself.
    pub async fn drain(&self) -> Vec<TaskOutcome> {
        let _joining = self.completed.lock().await;
        let mut tasks = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            std::mem::take(&mut state.tasks)
        };
        let mut outcomes = Vec::new();
        while let Some(outcome) = tasks.join_next().await {
            outcomes.push(outcome);
        }
        outcomes
    }
    /// Abort and join the current group and discard hooks that have never been scheduled.
    /// Hosts stop accepting calls before shutdown; concurrent submissions form a new group.
    pub async fn shutdown(&self) {
        let _joining = self.completed.lock().await;
        let mut tasks = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            state.pending.clear();
            std::mem::take(&mut state.tasks)
        };
        tasks.shutdown().await;
    }
}
impl State {
    /// Schedule on the bound runtime, or retain owned work until the application binds.
    fn submit(&mut self, event: &'static str, data: Record) {
        if let Some((host, runtime)) = &self.host
            && let Some(server) = host.upgrade()
        {
            self.tasks.spawn_on(runtime, async move {
                let mut req = stackpress_lib::Request::new();
                for (key, value) in data {
                    super::bridge::set(&mut req, &key, value)?;
                }
                let mut res = stackpress_lib::Response::new();
                server.emit(event, &mut req, &mut res).await?;
                Ok(()) as EmailResult<()>
            });
        } else {
            self.pending.push((event, data));
        }
    }
}
