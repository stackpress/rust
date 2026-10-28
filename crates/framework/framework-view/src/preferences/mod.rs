//! Optional public settings, single-writer save state and location-aware locale policy.
//!
//! This module is browser-safe and performs no registration or storage access.
//! Host integrations are separately selected; native persistence is opt-in.
mod action;
pub mod locale;
mod native_settings;
mod state;
pub use crate::adapters::web::CookieScope;
pub use action::{Preferences, SettingsAction, SettingsReply, SettingsUpdate, ThemePreference};
pub use locale::{
    LOCALE_COOKIE, LocaleRouting, LocaleSource, MAX_COOKIE_BYTES, MAX_PREFERENCE_BYTES,
    PreferenceAction, PreferenceReply, PreferenceUpdate, Selection,
};
pub use native_settings::{LoadedPreferences, NativePreferenceUpdate};
pub use stackpress_framework_language::{LocaleRegistry, PreferenceError};
pub use state::{PreferenceState, SaveState};
