//! Private example state: an acknowledgement belongs to its submitted snapshot, never later edits.
use crate::Draft;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_admin::pages::{PendingEdits, WriteState};

/// Structural versions deliberately treat a changed-then-reverted draft as the same exact value.
#[derive(Clone, Default)]
pub(crate) struct WriteVersions {
    /// Current root-owned controls; reading a response never writes these signals.
    draft: StoredValue<Option<Draft>>,
    /// Formatted partial/error drafts also count as unsaved edits before typed commit.
    pending: StoredValue<Option<PendingEdits>>,
    /// Generated field names observed when rendering current acknowledgement status.
    names: StoredValue<Vec<String>>,
    /// The exact admitted draft at the most recent explicit submission.
    submitted: RwSignal<Option<Value>>,
}
impl WriteVersions {
    /// Attach one editor; read-only and import pages use their own independent state.
    pub(crate) fn attach(&self, draft: Draft, pending: PendingEdits, names: Vec<String>) {
        self.draft.set_value(Some(draft));
        self.pending.set_value(Some(pending));
        self.names.set_value(names);
    }
    /// Freeze only the submitted version; newer values stay in the live draft.
    pub(crate) fn begin(&self, value: Value) {
        self.submitted.set(Some(value));
    }
    /// Observe controls directly, avoiding a race between a deferred edit effect and an HTTP reply.
    fn changed(&self) -> bool {
        let Some(submitted) = self.submitted.get() else {
            return false;
        };
        let current = self.draft.with_value(|draft| {
            draft.as_ref().and_then(|draft| {
                for name in self.names.get_value() {
                    if let Some(value) = draft.get(&name) {
                        let _ = value.get();
                    }
                }
                draft.snapshot().ok()
            })
        });
        let pending = self
            .pending
            .with_value(|pending| pending.as_ref().is_some_and(PendingEdits::blocked));
        changed(&submitted, current.as_ref(), pending)
    }
    /// Give feedback for the submitted version while preserving every later edit and uncertainty.
    pub(crate) fn message(&self, state: &WriteState) -> String {
        message(state, self.changed())
    }
}
/// Failed snapshot admission or an uncommitted formatted edit cannot acknowledge the current draft.
fn changed(submitted: &Value, current: Option<&Value>, pending: bool) -> bool {
    pending || current != Some(submitted)
}
/// Pure feedback rule shared by all CRUD writes, including copy and explicit remove/restore.
fn message(state: &WriteState, newer: bool) -> String {
    match state {
        WriteState::Saved if newer => {
            "Saved the submitted version. You have newer unsaved changes.".into()
        }
        WriteState::Unknown => format!(
            "Write outcome is unknown for the submitted version. Your draft is preserved. {}Check the record before another submission; no automatic retry was made.",
            if newer {
                "You have newer unsaved changes. "
            } else {
                ""
            }
        ),
        WriteState::Failed(error) if newer => {
            format!("Submitted version: {error}. You have newer unsaved changes.")
        }
        _ => state.message(),
    }
}

#[cfg(test)]
#[path = "write_version_tests.rs"]
mod tests;
