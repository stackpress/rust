//! Host-authoritative presentation settings, bounded public flash and action projection.
//!
//! This default graph is browser-safe. Hosts validate actual request authority, apply
//! cookies and mark personalized HTML `Cache-Control: no-store`. No identity, token,
//! role, filesystem access, listener or process-global mutable state is provided.
//! The opt-in Unix native-file adapter persists nonsecret installed-app settings.
//!
//! # Examples
//!
//! ```
//! use stackpress_framework_language::LocaleRegistry;
//! use stackpress_framework_preferences::{CookieScope, SettingsAction, ThemePreference};
//! use stackpress_framework_view::BasePath;
//! let locales = LocaleRegistry::new(vec!["en".into(), "fr".into()], "en".into(), Default::default())?;
//! let scope = CookieScope::new("https://app.invalid".into(), BasePath::new("/demo")?)?;
//! let action = SettingsAction::new(locales, scope, ThemePreference::System);
//! let body = r#"{"locale":"fr","theme":"dark","location":"/demo/en/preferences?q=%252F"}"#;
//! let reply = action.apply("POST", Some("https://app.invalid"), body)?;
//! assert_eq!(reply.location, "/demo/fr/preferences?q=%252F");
//! assert_eq!(reply.cookies.len(), 2); // Native host applies these as separate headers.
//! let confirmed = action.confirm("POST", Some("https://app.invalid"), body,
//!     "stackpress-locale=fr; stackpress-theme=dark")?;
//! assert!(confirmed.cookies.is_empty()); // Readback does not repeat the write.
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod action;
mod cookie;
mod flash;
mod native_settings;
mod policy;
mod state;
pub use native_settings::{LoadedPreferences, NativePreferenceUpdate};

/// Fixed, bounded installed-app file storage; unavailable in browser graphs.
#[cfg(all(feature = "native-file", unix))]
pub mod native_file;

pub use action::{Preferences, SettingsAction, SettingsReply, SettingsUpdate, ThemePreference};
pub use cookie::CookieScope;
pub use flash::{FlashConsumption, FlashCookie, FlashNotice, FlashRoot};
pub use policy::{AccessDenied, ActionId, AllowedActions, HostAccess, HostPolicy};
pub use state::{PreferenceState, SaveState};

#[cfg(test)]
mod tests;
