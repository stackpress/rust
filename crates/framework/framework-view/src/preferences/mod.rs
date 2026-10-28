//! Optional public settings, single-writer save state and location-aware locale policy.
//!
//! Core contracts are browser-safe and perform no registration or storage access.
//! Host entry points require `web-preferences` and explicit selection; file persistence is opt-in.
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

#[cfg(feature = "web-preferences")]
mod host;
#[cfg(feature = "web-preferences")]
pub use host::{Admission, Config, Service, register};
