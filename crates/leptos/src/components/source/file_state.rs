//! Root-local upload transitions, retaining Frui's captured queue/array behavior.
use super::files::SourceFileState;
use leptos::prelude::*;
use std::sync::{Arc, Mutex};

/// A React-style uploaded-array reference captured by one application upload callback.
#[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]
#[derive(Clone)]
pub(super) struct UploadTicket {
    /// Queue size at selection time; each completion subtracts from this captured size.
    queued: usize,
    /// Uploads selected before a completion share this exact original array reference.
    urls: Arc<Mutex<Vec<String>>>,
}

/// URLs and busy state have independent reactive dependencies, like Frui's separate useState calls.
#[derive(Clone, Copy)]
pub(super) struct FileController {
    /// Current display/form URLs; only URL operations notify onUpdate.
    pub urls: RwSignal<Vec<String>>,
    /// Pending count; queue-only changes never publish the old URL list again.
    pub queued: RwSignal<usize>,
    /// Current source array reference for subsequent upload captures.
    reference: StoredValue<Arc<Mutex<Vec<String>>>>,
    /// Single and list controls share their transition mechanism.
    multiple: bool,
}
impl FileController {
    /// Seed mount-only source values, preserving list empty URLs and single truthiness.
    pub fn new(initial: Vec<String>, multiple: bool) -> Self {
        let urls = if multiple {
            initial
        } else {
            initial
                .into_iter()
                .filter(|url| !url.is_empty())
                .take(1)
                .collect()
        };
        Self {
            urls: RwSignal::new(urls.clone()),
            queued: RwSignal::new(0),
            reference: StoredValue::new(Arc::new(Mutex::new(urls))),
            multiple,
        }
    }
    /// Capture the selected source array and queue, then show the new pending count.
    #[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]
    pub fn begin(self, count: usize) -> UploadTicket {
        let queued = if self.multiple {
            self.queued.get_untracked() + count
        } else {
            1
        };
        self.queued.set(queued);
        UploadTicket {
            queued,
            urls: self.reference.get_value(),
        }
    }
    /// Complete against the original captured array/queue, including late/out-of-order results.
    #[cfg(any(target_arch = "wasm32", all(test, feature = "ssr")))]
    pub fn complete(self, ticket: &UploadTicket, urls: Vec<String>) {
        let mut captured = ticket
            .urls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut next = SourceFileState {
            urls: captured.clone(),
            queued: ticket.queued,
        };
        next.complete(urls, self.multiple);
        // Preserve the shared captured reference, then publish a new array reference for later selections.
        *captured = next.urls.clone();
        drop(captured);
        self.publish(next);
    }
    /// Remove/reset using the current array; pending callbacks retain their earlier captured references.
    pub fn remove(self, index: usize) {
        let mut next = SourceFileState {
            urls: self.urls.get_untracked(),
            queued: self.queued.get_untracked(),
        };
        if self.multiple {
            next.remove(index);
        } else {
            next.urls.clear();
            next.queued = 0;
        }
        self.publish(next);
    }
    /// Replace source URLs once, with queue updates kept outside URL effect dependencies.
    fn publish(self, next: SourceFileState) {
        let _ = self
            .reference
            .try_update_value(|reference| *reference = Arc::new(Mutex::new(next.urls.clone())));
        let _ = self.queued.try_set(next.queued);
        let _ = self.urls.try_set(next.urls);
    }
}
