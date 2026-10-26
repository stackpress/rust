//! Natively tested action sequencing, retained results and stale/disposed guards.

use super::{SearchCancellation, SearchCompletion, SearchTransport};
use leptos::prelude::*;
use stackpress_example_profile_ui_contracts::{
    REVISION, ReplyDecodeError, SearchFailure, SearchFailureKind, SearchInput, SearchPage,
    SearchReply,
};
use stackpress_framework_ui::state::{NoticeKind, Notifications};

/// Signal handles for one component; cleanup remains with the owning Leptos root.
#[derive(Clone, Copy)]
pub(super) struct SearchController {
    /// Current editable prefix.
    pub(super) id: RwSignal<String>,
    /// Last successful input, independent of edits and retained rows.
    pub(super) prefix: RwSignal<String>,
    /// Successful publication counter; browser effects only observe completed work.
    pub(super) published: RwSignal<u64>,
    /// Last useful rows, retained until a successful replacement arrives.
    pub(super) rows: RwSignal<Vec<(String, String)>>,
    /// Last successful returned-row count, preserving zero.
    pub(super) total: RwSignal<u64>,
    /// Safe transport/host failure, separate from associated field validation.
    pub(super) error: RwSignal<String>,
    /// Host rejections, transport failures and invalid wire output stay distinct.
    pub(super) failure_kind: RwSignal<Option<SearchFailureKind>>,
    /// Validation failure associated with the prefix input.
    pub(super) field_error: RwSignal<String>,
    /// Whether the newest admitted submission is in flight.
    pub(super) pending: RwSignal<bool>,
    /// Only this generation may publish; invalidation also advances it.
    pub(super) generation: RwSignal<Option<u64>>,
    /// Scoped immediate notifications, disposed with this component's root.
    pub(super) notices: Notifications,
}

impl SearchController {
    /// Consume the public seed once without starting any transport work.
    pub(super) fn new(seed: SearchPage) -> Self {
        Self {
            prefix: RwSignal::new(seed.input.id.clone()),
            published: RwSignal::new(0),
            id: RwSignal::new(seed.input.id),
            rows: RwSignal::new(
                seed.reply
                    .rows
                    .into_iter()
                    .map(|row| (row.id, row.name))
                    .collect(),
            ),
            total: RwSignal::new(seed.reply.total),
            error: RwSignal::new(String::new()),
            failure_kind: RwSignal::new(None),
            field_error: RwSignal::new(String::new()),
            pending: RwSignal::new(false),
            generation: RwSignal::new(Some(0)),
            notices: Notifications::new(),
        }
    }

    /// Invalidate old publication rights before cancellation can settle a promise.
    pub(super) fn invalidate(self, cancel: Option<SearchCancellation>) {
        self.generation.update(|generation| {
            // Exhaustion permanently revokes publication rather than reusing a generation.
            *generation = generation.and_then(|value| value.checked_add(1));
        });
        self.pending.set(false);
        if let Some(cancel) = cancel {
            // Adapter cancellation is best effort; generation checks remain authoritative.
            cancel.run(());
        }
    }

    /// Start validated work, retaining useful rows and refusing stale/disposed writes.
    pub(super) fn submit(self, transport: SearchTransport, cancel: Option<SearchCancellation>) {
        // Invalid submission must also revoke the previous request's publication rights.
        self.invalidate(cancel);
        self.error.set(String::new());
        self.failure_kind.set(None);
        self.field_error.set(String::new());
        self.notices.clear();
        let input = match SearchInput::new(self.id.get_untracked()) {
            Ok(input) => input,
            Err(message) => {
                self.field_error.set(message.into());
                return;
            }
        };
        let Some(current) = self.generation.get_untracked() else {
            self.error
                .set("Search limit reached. Refresh to continue.".into());
            return;
        };
        self.pending.set(true);
        let prefix = input.id.clone();
        let done: SearchCompletion =
            std::sync::Arc::new(move |reply: Result<SearchReply, SearchFailure>| {
                // Cleanup removes the generation signal; supersession changes its value.
                if self.generation.try_get_untracked() != Some(Some(current))
                    || self.pending.try_get_untracked() != Some(true)
                {
                    return;
                }
                self.pending.set(false);
                match reply.and_then(|reply| {
                    reply.validate().map_err(|_| {
                        SearchFailure::Protocol(if reply.revision != REVISION {
                            ReplyDecodeError::ContractMismatch
                        } else {
                            ReplyDecodeError::Invalid
                        })
                    })?;
                    Ok(reply)
                }) {
                    Ok(reply) => {
                        // Replace rows/count together in one reactive batch after wire validation.
                        batch(|| {
                            self.prefix.set(prefix.clone());
                            self.total.set(reply.total);
                            self.rows.set(
                                reply
                                    .rows
                                    .into_iter()
                                    .map(|row| (row.id, row.name))
                                    .collect(),
                            );
                            self.published
                                .update(|value| *value = value.saturating_add(1));
                        });
                    }
                    Err(failure) => {
                        let message = failure.to_string();
                        self.failure_kind.set(Some(failure.kind()));
                        self.error.set(message.clone());
                        // The bounded notification duplicates no active identical notice.
                        let _ = self.notices.notify(NoticeKind::Error, message);
                    }
                }
            });
        transport.run((input, done));
    }

    /// Cancel explicitly, preserve useful results and present a safe retryable state.
    pub(super) fn cancel(self, cancel: Option<SearchCancellation>) {
        self.invalidate(cancel);
        self.failure_kind.set(Some(SearchFailureKind::Transport));
        self.error
            .set("Search cancelled. You can search again.".into());
    }
}
