//! Native-tested navigation decisions and retained history, independent of DOM APIs.

use super::{SearchCancellation, SearchTransport, controller::SearchController};
use leptos::prelude::*;
use stackpress_example_profile_ui_contracts::{
    DetailContext,
    routes::{self, Destination},
};
use stackpress_framework_view::leptos::routing::BasePath;
use std::collections::BTreeMap;

/// Thin host adapter: requested location/push intent and canonical URL publication.
#[derive(Clone, Copy)]
pub struct Navigation {
    /// Browser pop/link events; an identical target can still be explicitly requested.
    pub request: RwSignal<Option<(String, bool)>>,
    /// Write a checked public URL; bool selects push rather than replace.
    pub write: UnsyncCallback<(String, bool)>,
}

/// Owned history projection; never a live transport, Owner or host capability.
#[derive(Clone)]
struct Entry {
    /// Exact detail identity and its independent return prefix, or a search page.
    detail: Option<DetailContext>,
    /// Editable input at departure, which can differ from the last successful prefix.
    input: String,
    /// Prefix that produced these useful rows, independent of an unfinished edit.
    prefix: String,
    /// Owned public projection only; no transport, native host or reactive owner.
    rows: Vec<(String, String)>,
}

/// Root-local navigation state. History contains public results only and is bounded.
#[derive(Clone, Copy)]
pub(super) struct PageNavigation {
    /// Selected page context shared with the visible detail view.
    pub(super) detail: RwSignal<Option<DetailContext>>,
    /// Last checked canonical location published through the host adapter.
    location: RwSignal<String>,
    /// Bounded root-local snapshots for history traversal without a refetch.
    history: StoredValue<BTreeMap<String, Entry>>,
    /// Explicit checked mount, never inferred from an untrusted location.
    base: StoredValue<BasePath>,
}

impl PageNavigation {
    /// Seed navigation without fetching, publishing a URL or inspecting a browser.
    pub(super) fn new(
        base: BasePath,
        detail: Option<DetailContext>,
        prefix: &str,
    ) -> Result<Self, &'static str> {
        let location = match &detail {
            Some(detail) => routes::detail_location(&base, &detail.id, &detail.search_prefix),
            None => routes::search_location(&base, prefix),
        }
        .map_err(|_| "Page location invalid. Refresh to continue.")?;
        Ok(Self {
            detail: RwSignal::new(detail),
            location: RwSignal::new(location.into()),
            history: StoredValue::new(BTreeMap::new()),
            base: StoredValue::new(base),
        })
    }

    /// Publish only a successful current search/detail; stale completions cannot reach here.
    pub(super) fn completed(self, controller: SearchController, adapter: Navigation) {
        let location = match self.detail.get_untracked() {
            Some(detail) => {
                routes::detail_location(&self.base.get_value(), &detail.id, &detail.search_prefix)
            }
            None => {
                routes::search_location(&self.base.get_value(), &controller.prefix.get_untracked())
            }
        };
        if let Ok(location) = location {
            let push = location.as_str() != self.location.get_untracked();
            self.location.set(location.as_str().into());
            adapter.write.run((location.into(), push));
        }
    }

    /// Resolve a catalog target, revoke active work, and restore public history or load it.
    pub(super) fn go(
        self,
        location: String,
        push: bool,
        controller: SearchController,
        transport: SearchTransport,
        cancel: Option<SearchCancellation>,
        adapter: Navigation,
    ) {
        let Ok(destination) = routes::resolve(&self.base.get_value(), &location) else {
            return;
        };
        // Save only settled useful state. A pending request has no successful history entry.
        if !controller.pending.get_untracked() {
            let entry = Entry {
                detail: self.detail.get_untracked(),
                input: controller.id.get_untracked(),
                prefix: controller.prefix.get_untracked(),
                rows: controller.rows.get_untracked(),
            };
            self.history.update_value(|history| {
                if history.len() >= 64 {
                    history.clear();
                }
                history.insert(self.location.get_untracked(), entry);
            });
        }
        controller.invalidate(cancel);
        controller.error.set(String::new());
        controller.failure_kind.set(None);
        controller.field_error.set(String::new());
        controller.notices.clear();
        self.location.set(location.clone());
        // Pop traversal replaces the current URL metadata without adding an entry.
        adapter.write.run((location.clone(), push));
        if let Some(entry) = self
            .history
            .with_value(|history| history.get(&location).cloned())
        {
            // Restore rows and input together; no initial/history refetch is necessary.
            batch(|| {
                self.detail.set(entry.detail);
                controller.id.set(entry.input);
                controller.prefix.set(entry.prefix);
                controller.total.set(entry.rows.len() as u64);
                controller.rows.set(entry.rows);
            });
            return;
        }
        let (detail, input) = match destination {
            Destination::Search(prefix) => (None, prefix),
            Destination::Detail { id, search_prefix } => (
                Some(DetailContext {
                    id: id.clone(),
                    search_prefix,
                }),
                id,
            ),
        };
        self.detail.set(detail);
        controller.id.set(input);
        controller.submit(transport, cancel);
    }
}
