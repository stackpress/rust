//! Synchronous component evaluation followed by owned metadata assembly.

use futures::{StreamExt, stream};
use leptos::prelude::*;
use leptos_meta::{ServerMetaContext, ServerMetaContextOutput};
use serde::Serialize;
use stackpress_framework_view::{PageSnapshot, RenderedPage, ViewError};
use std::sync::atomic::{AtomicBool, Ordering};

/// Only this documented helper marks the native render boundary ready.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize native SSR's process-global Leptos executor, idempotently.
///
/// Call once during host startup inside an active Tokio runtime. Repeated calls
/// and an already-initialized Leptos executor succeed. This enables buffered Head
/// assembly only; it does not enable general asynchronous component SSR.
///
/// # Errors
/// Returns [`ViewError::ExecutorUnavailable`] outside an active Tokio runtime.
pub fn initialize_ssr() -> Result<(), ViewError> {
    tokio::runtime::Handle::try_current().map_err(|_| ViewError::ExecutorUnavailable)?;
    // AlreadySet is the executor's only error: another initialized caller is valid.
    match any_spawner::Executor::init_tokio() {
        Ok(()) | Err(any_spawner::ExecutorError::AlreadySet) => {
            INITIALIZED.store(true, Ordering::Release);
            Ok(())
        }
    }
}

/// Reject missing startup or runtime context before a dependency can panic.
fn require_executor() -> Result<(), ViewError> {
    if !INITIALIZED.load(Ordering::Acquire) || tokio::runtime::Handle::try_current().is_err() {
        return Err(ViewError::ExecutorUnavailable);
    }
    Ok(())
}

/// An owned staging buffer; no Owner, request or host service crosses its await.
pub struct PreparedPage {
    /// Complete body/seed shell awaiting leptos_meta's buffered Head injection.
    html: String,
    /// Channel output and owned title state, independent of reactive Owner lifetime.
    head: ServerMetaContextOutput,
}

/// Synchronous render entry; each call receives a fresh, unshared Owner.
pub struct LeptosRenderer;

impl LeptosRenderer {
    /// Render hydration-marked body and safely stage the public seed.
    ///
    /// Component evaluation is synchronous. The Owner is cleaned on success or
    /// unwind before returning; no global render lock is required.
    /// Call [`initialize_ssr`] during startup and render inside a Tokio runtime.
    ///
    /// # Errors
    /// Rejects missing SSR startup/runtime and oversized/malformed public seed
    /// before component evaluation, without an uninitialized-executor panic.
    pub fn prepare<T: Serialize, V: IntoView>(
        snapshot: &PageSnapshot<T>,
        component: impl FnOnce() -> V,
    ) -> Result<PreparedPage, ViewError> {
        require_executor()?;
        let seed = snapshot.script_json()?;
        let owner = RenderOwner(Owner::new());
        let (meta, head) = ServerMetaContext::new();
        let body = owner.0.with(|| {
            // Metadata is local to this call, and only the owned HTML leaves this scope.
            provide_context(meta);
            leptos_meta::provide_meta_context();
            component().to_html()
        });
        drop(owner);
        // The fixed trusted shell keeps script data separate from component HTML.
        let html = format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><!--HEAD--></head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"stackpress-page\">{seed}</script><script type=\"module\" src=\"/assets/start.js\"></script></body></html>"
        );
        Ok(PreparedPage { html, head })
    }
}

impl PreparedPage {
    /// Finish leptos_meta's owned buffered Head injection, never component async SSR.
    ///
    /// # Errors
    /// Rejects the final complete HTML ceiling before the host can publish bytes.
    pub async fn finish(self) -> Result<RenderedPage, ViewError> {
        require_executor()?;
        // Only owned Send buffers survive this metadata tick; all rendering is already done.
        let chunks = self
            .head
            .inject_meta_context(stream::iter([self.html]))
            .await;
        let html = chunks.collect::<Vec<_>>().await.concat();
        RenderedPage::new(html)
    }
}

/// Scope guard makes cleanup explicit even when a trusted component panics.
struct RenderOwner(Owner);

impl Drop for RenderOwner {
    /// Clean signals/effects before the render exits or unwinds.
    fn drop(&mut self) {
        self.0.cleanup();
    }
}
