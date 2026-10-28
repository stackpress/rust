//! Single-writer root save state; transport and draft ownership stay outside it.
use super::{Preferences, SettingsAction, SettingsReply, SettingsUpdate};
use stackpress_framework_language::PreferenceError;

/// Public save outcome; a failed write never changes acknowledged useful content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveState {
    /// No action is running.
    Ready,
    /// One request owns the write slot; duplicate clicks cannot race it.
    Pending,
    /// Saving failed or could not be confirmed. Explicit user action may retry.
    Failed,
}
/// Native-tested root controller. It never reads browser storage or touches drafts.
#[derive(Clone, Debug)]
pub struct PreferenceState {
    /// Last acknowledged settings; failure preserves these values.
    current: Preferences,
    /// Current history path, replaced only after an admitted acknowledgement.
    location: String,
    /// Exact submitted request; the adapter admits one acquisition at a time.
    pending: Option<SettingsUpdate>,
    /// Visible progress/failure outcome.
    status: SaveState,
}
impl PreferenceState {
    /// Seed before first render from a host-validated public projection.
    pub fn new(current: Preferences, location: String) -> Self {
        Self {
            current,
            location,
            pending: None,
            status: SaveState::Ready,
        }
    }
    /// Borrow acknowledged settings without copying an entire dictionary.
    pub fn current(&self) -> &Preferences {
        &self.current
    }
    /// Borrow the acknowledged path/query.
    pub fn location(&self) -> &str {
        &self.location
    }
    /// Read save progress for disabled controls and accessible failure notices.
    pub fn status(&self) -> SaveState {
        self.status
    }
    /// Admit one root write; there is no automatic retry or cancellation-as-rollback.
    pub fn begin(&mut self, desired: Preferences) -> Option<SettingsUpdate> {
        // A pending write keeps exclusive root ownership until completion/failure.
        if self.pending.is_some() {
            return None;
        }
        let update = SettingsUpdate {
            locale: desired.locale,
            theme: desired.theme,
            location: self.location.clone(),
        };
        self.pending = Some(update.clone());
        self.status = SaveState::Pending;
        Some(update)
    }
    /// Validate without publishing, so a browser can stage history first.
    ///
    /// # Errors
    /// Rejects missing/pending drift or any invalid acknowledgement.
    pub fn validate(
        &self,
        action: &SettingsAction,
        text: &str,
    ) -> Result<SettingsReply, PreferenceError> {
        let submitted = self.pending.as_ref().ok_or(PreferenceError::Request)?;
        action.acknowledge(text, submitted)
    }
    /// Validate and publish an acknowledgement against the currently pending request.
    ///
    /// # Errors
    /// Rejects unsolicited, conflicting or malformed replies without changing content.
    /// Browser adapters may call validate first to stage history before this commit.
    pub fn complete(&mut self, action: &SettingsAction, text: &str) -> Result<(), PreferenceError> {
        // Validate everything before replacing either acknowledged setting or path.
        let reply = self.validate(action, text)?;
        self.pending = None;
        self.current = reply.preferences;
        self.location = reply.location;
        self.status = SaveState::Ready;
        Ok(())
    }
    /// Announce failure while retaining content/location and leaving drafts alone.
    pub fn fail(&mut self) {
        // Clear admission only; no rollback or successful-persistence claim is made.
        self.pending = None;
        self.status = SaveState::Failed;
    }
}
