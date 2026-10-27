//! Thread-local Owner restoration shared by the three native render lifetimes.
//!
//! This guard restores only the caller context. Each renderer retains its own
//! root/queue cleanup order, and creates the guard within one synchronous poll
//! or Drop; an entered thread-local context never crosses an await boundary.
use leptos::prelude::Owner;

/// Restore a retained caller, or clear a render left current during unwind.
pub(super) struct RestoreOwner(pub(super) Option<Owner>);

impl Drop for RestoreOwner {
    /// Cleanup/unwind cannot strand the renderer's Owner in the caller's thread.
    fn drop(&mut self) {
        if let Some(previous) = &self.0 {
            // Retaining the original Owner keeps its context available through cleanup.
            previous.set();
        } else if let Some(current) = Owner::current() {
            // A caller without an Owner must return to that same empty context.
            current.unset();
        }
    }
}
