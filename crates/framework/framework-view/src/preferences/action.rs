//! Pure settings selection and fixed same-origin acknowledgement.
use super::locale::{
    LocaleRouting, MAX_COOKIE_BYTES, MAX_PREFERENCE_BYTES, PreferenceAction, PreferenceUpdate,
};
use super::{LocaleRegistry, PreferenceError};
use crate::adapters::web::{CookieScope, unique_cookie};
use serde::{Deserialize, Serialize};

/// Nonsecret preference cookie; locale retains its existing M0 cookie identity.
const THEME_COOKIE: &str = "stackpress-theme";

/// An explicit saved choice; absence is represented separately by Option.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Explicit neutral light surfaces.
    #[default]
    Light,
    /// Explicit neutral dark surfaces.
    Dark,
}
impl ThemePreference {
    /// Stable attribute and cookie token, without browser preference acquisition.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
    /// Parse only the two explicit exact tokens; legacy cookies become unset.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

/// Allowlisted initial public settings, independent of a concrete rendering library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preferences {
    /// Canonical registered language.
    pub locale: String,
    /// Exact theme token applied by the host before first render.
    #[serde(default)]
    pub theme: Option<ThemePreference>,
}

/// Closed action fields, deliberately excluding arbitrary cookie names or URLs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsUpdate {
    /// Canonical or explicitly registered alias.
    pub locale: String,
    /// Typed theme rejects unknown wire values.
    #[serde(default)]
    pub theme: Option<ThemePreference>,
    /// Current checked path/query under the app mount.
    pub location: String,
}

/// Public acknowledgement plus native-only cookie outputs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsReply {
    /// Only these acknowledged values may replace root preferences.
    pub preferences: Preferences,
    /// Same-app canonical location for history replacement.
    pub location: String,
    /// Apply each header separately; these never enter browser JSON.
    #[serde(skip)]
    pub cookies: Vec<String>,
}

/// Immutable host policy for one same-origin presentation endpoint.
#[derive(Clone, Debug)]
pub struct SettingsAction {
    /// Shared locale authority and prefix policy.
    registry: LocaleRegistry,
    /// Same Origin and cookie scope for both settings.
    scope: CookieScope,
}
impl SettingsAction {
    /// Confirm the browser sent back both exact cookies after a successful POST.
    ///
    /// # Errors
    /// Uses the same admission as apply, then rejects missing/ambiguous/blocked or
    /// conflicting cookies. This readback issues no cookies and performs no retry.
    pub fn confirm(
        &self,
        method: &str,
        origin: Option<&str>,
        body: &str,
        cookie: &str,
    ) -> Result<SettingsReply, PreferenceError> {
        let mut reply = self.apply(method, origin, body)?;
        let locale = unique_cookie(cookie, super::LOCALE_COOKIE)?;
        let theme = unique_cookie(cookie, THEME_COOKIE)?;
        // Only saved-data boundaries recognize the removed legacy mode as unset.
        let matches_theme = match (reply.preferences.theme, theme) {
            (Some(expected), Some(actual)) => actual == expected.as_str(),
            (None, None | Some("system")) => true,
            _ => false,
        };
        if locale != Some(reply.preferences.locale.as_str()) || !matches_theme {
            return Err(PreferenceError::Request);
        }
        // Only the original write endpoint emits Set-Cookie. Confirmation is read-only.
        reply.cookies.clear();
        Ok(reply)
    }
    /// Configure one action; hosts additionally enforce session CSRF when applicable.
    pub fn new(registry: LocaleRegistry, scope: CookieScope) -> Self {
        Self { registry, scope }
    }

    /// Select prefix > canonical locale cookie > default, and checked theme cookie.
    ///
    /// # Errors
    /// Rejects an unsafe/out-of-mount location or oversized cookie header. Invalid
    /// theme values/duplicates fall back; no browser language guessing occurs.
    pub fn select(&self, location: &str, cookie: &str) -> Result<Preferences, PreferenceError> {
        if cookie.len() > MAX_COOKIE_BYTES {
            return Err(PreferenceError::Limit);
        }
        let locale = LocaleRouting::new(&self.registry, &self.scope.base)
            .select(location, cookie)?
            .locale;
        let theme = unique_cookie(cookie, THEME_COOKIE)
            .ok()
            .flatten()
            .and_then(ThemePreference::parse);
        Ok(Preferences { locale, theme })
    }

    /// Validate a bounded POST; produce no cookies before all validation succeeds.
    ///
    /// # Errors
    /// Denies missing/wrong Origin, wrong method, unknown locale, extra fields,
    /// malformed JSON, unsafe return paths and excess input. No retry is performed.
    pub fn apply(
        &self,
        method: &str,
        origin: Option<&str>,
        body: &str,
    ) -> Result<SettingsReply, PreferenceError> {
        // Authority checks precede decoding to keep denial behavior deterministic.
        if method != "POST" {
            return Err(PreferenceError::Request);
        }
        if origin != Some(self.scope.origin.as_str()) {
            return Err(PreferenceError::Origin);
        }
        if body.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::Limit);
        }
        let update: SettingsUpdate =
            serde_json::from_str(body).map_err(|_| PreferenceError::Request)?;
        let locale_action = PreferenceAction::new(
            self.registry.clone(),
            self.scope.origin.clone(),
            self.scope.base.clone(),
        )?;
        let reply = locale_action.apply(
            method,
            origin,
            PreferenceUpdate {
                locale: update.locale,
                location: update.location,
            },
        )?;
        // An unset value never persists the browser's initial OS-derived color.
        let mut cookies = vec![reply.cookie];
        if let Some(theme) = update.theme {
            cookies.push(self.scope.header(THEME_COOKIE, theme.as_str(), 31536000));
        }
        Ok(SettingsReply {
            preferences: Preferences {
                locale: reply.locale,
                theme: update.theme,
            },
            location: reply.location,
            cookies,
        })
    }

    /// Validate a public acknowledgement against the exact submitted request.
    ///
    /// # Errors
    /// Rejects malformed/oversized data, unknown fields, aliases in the reply,
    /// conflicting settings or any route/query change beyond the requested prefix.
    pub fn acknowledge(
        &self,
        text: &str,
        submitted: &SettingsUpdate,
    ) -> Result<SettingsReply, PreferenceError> {
        if text.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::Limit);
        }
        let reply: SettingsReply =
            serde_json::from_str(text).map_err(|_| PreferenceError::Request)?;
        let expected = LocaleRouting::new(&self.registry, &self.scope.base)
            .location(&submitted.location, &submitted.locale)?;
        if Some(reply.preferences.locale.as_str()) != self.registry.canonical(&submitted.locale)
            || reply.preferences.theme != submitted.theme
            || reply.location != expected
        {
            return Err(PreferenceError::Request);
        }
        Ok(reply)
    }
}
