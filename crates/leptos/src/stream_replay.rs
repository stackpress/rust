//! Request-local serialization queues for inert replay beside streamed fragments.
//!
//! Delegate Leptos identities, errors and blocking-resource semantics to the
//! pinned SSR context. Keep serialization futures here so one pending Resource
//! cannot hold back another section's ready replay. No generated JS is decoded.
use crate::replay::ResourceReplay;
use crate::replay_error::ReplayError;
use hydration_context::{
    PinnedFuture, PinnedStream, SerializedDataId, SharedContext, SsrSharedContext,
};
use leptos::error::{Error, ErrorId};
use std::{
    fmt,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

/// Local render ownership includes every pending serialization future.
#[derive(Default)]
pub(super) struct StreamContext {
    /// Pinned upstream identity/error/deferred behavior stays unchanged.
    inner: SsrSharedContext,
    /// Taken before polling, so a serializer can register another Resource.
    pending: Mutex<Vec<(SerializedDataId, PinnedFuture<String>)>>,
    /// Admission failure is remembered without retaining unbounded futures.
    overflow: AtomicBool,
}
impl fmt::Debug for StreamContext {
    /// Avoid formatting application data or executable serializer state.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamContext").finish_non_exhaustive()
    }
}
impl StreamContext {
    /// Enable hydration identities, matching SsrSharedContext::new().
    pub(super) fn new() -> Self {
        Self {
            inner: SsrSharedContext::new(),
            pending: Mutex::new(Vec::new()),
            overflow: AtomicBool::new(false),
        }
    }
    /// Poll each admitted serializer once, returning only newly ready values.
    pub(super) fn ready(&self, cx: &mut Context<'_>) -> Result<Vec<(usize, String)>, ReplayError> {
        if self.overflow.load(Ordering::Acquire) {
            return Err(ReplayError::SnapshotLimit);
        }
        let pending = std::mem::take(&mut *self.pending.lock().unwrap_or_else(|e| e.into_inner()));
        let mut ready = Vec::new();
        for (id, mut future) in pending {
            // Never hold the queue lock while invoking application serialization.
            match future.as_mut().poll(cx) {
                Poll::Ready(value) => ready.push((id.into_inner(), value)),
                Poll::Pending => self
                    .pending
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push((id, future)),
            }
        }
        Ok(ready)
    }
    /// Report whether EOF can safely publish a complete replay set.
    pub(super) fn is_empty(&self) -> bool {
        self.pending
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }
    /// Break all context -> serializer -> Resource -> Owner cycles at teardown.
    pub(super) fn clear(&self) {
        let queued = std::mem::take(&mut *self.pending.lock().unwrap_or_else(|e| e.into_inner()));
        drop(queued);
        drop(self.inner.await_deferred());
    }
}
impl SharedContext for StreamContext {
    /// This context is native-only.
    fn is_browser(&self) -> bool {
        false
    }
    /// Preserve upstream's exact hydration ID allocation.
    fn next_id(&self) -> SerializedDataId {
        self.inner.next_id()
    }
    /// Bound registrations before retaining their serialization futures.
    fn write_async(&self, id: SerializedDataId, future: PinnedFuture<String>) {
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        if pending.len() >= crate::replay::MAX_RESOURCES {
            self.overflow.store(true, Ordering::Release);
        } else {
            pending.push((id, future));
        }
    }
    /// Native renderers have no browser replay to read.
    fn read_data(&self, _: &SerializedDataId) -> Option<String> {
        None
    }
    /// Native renderers do not await browser replay.
    fn await_data(&self, _: &SerializedDataId) -> Option<String> {
        None
    }
    /// Replay is emitted as inert JSON by this renderer, never upstream JS.
    fn pending_data(&self) -> Option<PinnedStream<String>> {
        None
    }
    /// Native rendering is not browser hydration.
    fn during_hydration(&self) -> bool {
        false
    }
    /// There is no browser hydration phase to finish here.
    fn hydration_complete(&self) {}
    /// Preserve islands/Resource hydration admission.
    fn get_is_hydrating(&self) -> bool {
        self.inner.get_is_hydrating()
    }
    /// Preserve explicit upstream hydration admission changes.
    fn set_is_hydrating(&self, value: bool) {
        self.inner.set_is_hydrating(value);
    }
    /// Keep ErrorBoundary ownership with upstream; never expose native error text.
    fn take_errors(&self) -> Vec<(SerializedDataId, ErrorId, Error)> {
        self.inner.take_errors()
    }
    /// Let upstream ErrorBoundary inspect its own registered errors.
    fn errors(&self, id: &SerializedDataId) -> Vec<(ErrorId, Error)> {
        self.inner.errors(id)
    }
    /// Delegate the sealed boundary invariant.
    fn seal_errors(&self, id: &SerializedDataId) {
        self.inner.seal_errors(id);
    }
    /// Retain ErrorBoundary diagnostics locally, without serializing their Display.
    fn register_error(&self, boundary: SerializedDataId, id: ErrorId, error: Error) {
        self.inner.register_error(boundary, id, error);
    }
    /// Blocking Resources still delay precommit staging.
    fn defer_stream(&self, future: PinnedFuture<()>) {
        self.inner.defer_stream(future);
    }
    /// Transfer owned blocking work to the precommit future.
    fn await_deferred(&self) -> Option<PinnedFuture<()>> {
        self.inner.await_deferred()
    }
    /// Preserve Leptos's incomplete boundary bookkeeping.
    fn set_incomplete_chunk(&self, id: SerializedDataId) {
        self.inner.set_incomplete_chunk(id);
    }
    /// Query upstream's boundary bookkeeping without inventing identities.
    fn get_incomplete_chunk(&self, id: &SerializedDataId) -> bool {
        self.inner.get_incomplete_chunk(id)
    }
}

/// Aggregate policy applies across fragments, not separately to each script.
pub(super) struct ReplayBudget {
    /// Every emitted identity/value stays checked against duplicates and bytes.
    values: Vec<(usize, String)>,
    /// Same logical snapshot policy as buffered SSR.
    limit: usize,
}
impl ReplayBudget {
    /// Start one page's aggregate replay budget.
    pub(super) fn new(limit: usize) -> Self {
        Self {
            values: Vec::new(),
            limit,
        }
    }
    /// Escape newly ready data only after the entire accumulated set validates.
    pub(super) fn append(&mut self, values: Vec<(usize, String)>) -> Result<String, ReplayError> {
        self.values.extend(values.iter().cloned());
        ResourceReplay::new(self.values.clone(), self.limit)?;
        if values.is_empty() {
            return Ok(String::new());
        }
        let json = ResourceReplay::new(values, self.limit)?.script_json()?;
        // Bootstrap validates these inert templates and removes them before hydration.
        Ok(json)
    }
}

impl Drop for StreamContext {
    /// Teardown also runs on precommit errors and trusted component unwind.
    fn drop(&mut self) {
        self.clear();
    }
}
