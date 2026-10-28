//! Optional public settings, single-writer save state and location-aware locale policy.
//!
//! Core contracts are browser-safe and perform no registration or storage access.
//! Host entry points require `web-preferences` and explicit selection; file persistence is opt-in.
//!
//! # Examples
//! Select saved public settings without a server, browser query or persistence write.
//! ```
//! use stackpress_framework_view::{BasePath, preferences::{CookieScope, LocaleRegistry, SettingsAction}};
//! let locales = LocaleRegistry::new(vec!["en".into(), "fr".into()], "en".into(), Default::default())?;
//! let scope = CookieScope::new("https://app.invalid".into(), BasePath::default())?;
//! let action = SettingsAction::new(locales, scope);
//! let selected = action.select("/preferences", "stackpress-locale=fr; stackpress-theme=system")?;
//! assert_eq!(selected.locale, "fr");
//! assert_eq!(selected.theme, None); // Legacy saved theme leaves the locale intact.
//! # Ok::<(), stackpress_framework_view::preferences::PreferenceError>(())
//! ```
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
