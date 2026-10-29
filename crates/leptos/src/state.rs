//! Owner-scoped theme and immediate notifications, without cookies or timers.
//!
//! Construct state inside a Leptos Owner and provide it through LayoutProvider.
//! Dropping that Owner disposes all signals; no global registry retains them.
//! Mutations are synchronous actions in that root's UI execution context. Hosts
//! marshal completions back to the root; signal handles are not a concurrent
//! cross-thread notification queue, even when their Rust types are Send/Sync.

use leptos::prelude::*;

/// Explicit initial theme; hydration never guesses from browser preferences.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Light neutral surfaces.
    #[default]
    Light,
    /// Dark neutral surfaces.
    Dark,
}

impl Theme {
    /// Return the stable HTML/CSS theme token.
    pub fn as_str(self) -> &'static str {
        // Effective UI state has exactly two DOM tokens; persistence/absence lives in view.
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// Reactive theme owned by one UI root, with no implicit persistence.
#[derive(Clone, Copy)]
pub struct ThemeState(RwSignal<Theme>);

impl ThemeState {
    /// Initialize from the explicit public seed.
    pub fn new(theme: Theme) -> Self {
        // Allocate inside the current Owner so independent roots never share theme state.
        Self(RwSignal::new(theme))
    }

    /// Read reactively in a component or attribute.
    pub fn get(self) -> Theme {
        // Subscribe this view to its own root; no browser or storage acquisition occurs.
        self.0.get()
    }

    /// Replace only this root's theme.
    pub fn set(self, theme: Theme) {
        // Publication is synchronous and root-local; only an application adapter persists it.
        self.0.set(theme);
    }

    /// Toggle explicit choices without reading a cookie/media query.
    pub fn toggle(self) {
        self.0.update(|theme| {
            // Exhaustive variants keep future theme additions deliberate.
            *theme = match *theme {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
            };
        });
    }
}

/// Obtain an ancestor's theme without silently constructing a global default.
///
/// # Errors
/// Returns a static diagnostic without installing state when no ancestor provides ThemeState.
pub fn use_theme() -> Result<ThemeState, &'static str> {
    use_context().ok_or("Theme state requires LayoutProvider")
}

/// Visible severity; the text/role also conveys status without relying on color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeKind {
    /// Informational completion or guidance.
    Info,
    /// A recoverable condition requiring attention.
    Warning,
    /// An operation failed.
    Error,
}

impl NoticeKind {
    /// Safe semantic label for presentation and assistive technology.
    pub fn label(self) -> &'static str {
        match self {
            Self::Info => "Information",
            Self::Warning => "Warning",
            Self::Error => "Error",
        }
    }

    /// Errors interrupt; other notices use polite announcements.
    pub fn role(self) -> &'static str {
        match self {
            Self::Error => "alert",
            Self::Info | Self::Warning => "status",
        }
    }
}

/// A safe text notification, scoped to its owning UI root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// Stable key, never reused during the root's lifetime.
    pub id: u64,
    /// Visible semantic severity.
    pub kind: NoticeKind,
    /// Escaped text only, not trusted HTML.
    pub message: String,
}

/// Bounded immediate notifications; identical active notices are coalesced.
#[derive(Clone, Copy)]
pub struct Notifications {
    /// Checked root-local budgets; no mutable process-wide defaults exist.
    options: NotificationOptions,
    /// Ordered notices in this root; dismissed entries are removed.
    entries: RwSignal<Vec<Notice>>,
    /// Last allocated key; exhaustion rejects instead of reusing a key.
    serial: RwSignal<u64>,
}

/// Public notification settings, checked before they can create a scoped store.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NotificationOptions {
    /// Maximum active notices, from 1 through 20.
    pub max_notices: usize,
    /// Maximum UTF-8 message bytes, from 1 through 4096.
    pub max_message_bytes: usize,
}

impl Default for NotificationOptions {
    /// Explicit finite defaults; no timer, cookie or global mutation is implied.
    fn default() -> Self {
        Self {
            max_notices: 20,
            max_message_bytes: 4096,
        }
    }
}

impl Default for Notifications {
    /// Create an empty store inside the current Owner.
    fn default() -> Self {
        Self::new()
    }
}

impl Notifications {
    /// Create an empty, root-local store with no timer or external side effect.
    pub fn new() -> Self {
        Self {
            options: NotificationOptions::default(),
            entries: RwSignal::new(Vec::new()),
            serial: RwSignal::new(0),
        }
    }

    /// Create a store with checked smaller public budgets.
    ///
    /// # Errors
    /// Rejects zero or budgets exceeding 20 notices / 4096 message bytes.
    pub fn with_options(options: NotificationOptions) -> Result<Self, &'static str> {
        if !(1..=20).contains(&options.max_notices)
            || !(1..=4096).contains(&options.max_message_bytes)
        {
            return Err("Invalid notification settings");
        }
        Ok(Self {
            options,
            ..Self::new()
        })
    }

    /// Read ordered active notices reactively.
    pub fn entries(self) -> Vec<Notice> {
        self.entries.get()
    }

    /// Add safe text, returning the existing key for a duplicate active notice.
    ///
    /// # Errors
    /// Rejects empty/over-4096-byte text, more than 20 notices, or exhausted keys.
    pub fn notify(self, kind: NoticeKind, message: String) -> Result<u64, &'static str> {
        // Validate before changing either list or sequence state.
        if message.is_empty() || message.len() > self.options.max_message_bytes {
            return Err("Notification text exceeds its configured byte limit");
        }
        let entries = self.entries.get_untracked();
        if let Some(existing) = entries
            .iter()
            .find(|n| n.kind == kind && n.message == message)
        {
            // Repeated completion does not create repeated announcements.
            return Ok(existing.id);
        }
        if entries.len() >= self.options.max_notices {
            return Err("Dismiss a notification before adding another");
        }
        let id = self
            .serial
            .get_untracked()
            .checked_add(1)
            .ok_or("Notification keys exhausted")?;
        self.serial.set(id);
        self.entries
            .update(|entries| entries.push(Notice { id, kind, message }));
        Ok(id)
    }

    /// Dismiss exactly one key; repeated or unknown dismissal is harmless.
    pub fn dismiss(self, id: u64) -> bool {
        let mut removed = false;
        self.entries.update(|entries| {
            // Retain order among all other notices.
            entries.retain(|entry| {
                let keep = entry.id != id;
                removed |= !keep;
                keep
            });
        });
        removed
    }

    /// Remove all immediate notices without resetting their stable key sequence.
    pub fn clear(self) {
        self.entries.set(Vec::new());
    }
}

/// Obtain scoped notifications, with a useful missing-provider diagnostic.
///
/// # Errors
/// Returns a static diagnostic without installing state when no ancestor provides Notifications.
pub fn use_notifications() -> Result<Notifications, &'static str> {
    use_context().ok_or("Notification state requires LayoutProvider")
}

#[cfg(test)]
#[path = "state/tests.rs"]
mod tests;
