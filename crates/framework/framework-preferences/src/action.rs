//! Pure settings selection and fixed same-origin acknowledgement.
use crate::cookie::{CookieScope, unique_cookie};
use serde::{Deserialize, Serialize};
use stackpress_framework_language::{
    LocaleRegistry, MAX_COOKIE_BYTES, MAX_PREFERENCE_BYTES, PreferenceAction, PreferenceError,
    PreferenceUpdate,
};

/// Nonsecret preference cookie; locale retains its existing M0 cookie identity.
const THEME_COOKIE: &str = "stackpress-theme";

/// Stable host-seeded theme mode; system colors are resolved by CSS before paint.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    /// Explicit neutral light surfaces.
    Light,
    /// Explicit neutral dark surfaces.
    Dark,
    /// Deterministic `system` token; CSS media queries choose the effective colors.
    #[default]
    System,
}
impl ThemePreference {
    /// Stable attribute and cookie token, without browser preference acquisition.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
    /// Parse only the three accepted exact tokens.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "system" => Some(Self::System),
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
    pub theme: ThemePreference,
}

/// Closed action fields, deliberately excluding arbitrary cookie names or URLs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsUpdate {
    /// Canonical or explicitly registered alias.
    pub locale: String,
    /// Typed theme rejects unknown wire values.
    pub theme: ThemePreference,
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
    /// Explicit fallback when theme cookie is missing, corrupt or ambiguous.
    default_theme: ThemePreference,
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
        let locale = unique_cookie(cookie, stackpress_framework_language::LOCALE_COOKIE)?;
        let theme = unique_cookie(cookie, THEME_COOKIE)?;
        if locale != Some(reply.preferences.locale.as_str())
            || theme != Some(reply.preferences.theme.as_str())
        {
            return Err(PreferenceError::Request);
        }
        // Only the original write endpoint emits Set-Cookie. Confirmation is read-only.
        reply.cookies.clear();
        Ok(reply)
    }
    /// Configure one action; hosts additionally enforce session CSRF when applicable.
    pub fn new(
        registry: LocaleRegistry,
        scope: CookieScope,
        default_theme: ThemePreference,
    ) -> Self {
        Self {
            registry,
            scope,
            default_theme,
        }
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
        let locale = self
            .registry
            .select(&self.scope.base, location, cookie)?
            .locale;
        let theme = unique_cookie(cookie, THEME_COOKIE)
            .ok()
            .flatten()
            .and_then(ThemePreference::parse)
            .unwrap_or(self.default_theme);
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
        Ok(SettingsReply {
            preferences: Preferences {
                locale: reply.locale,
                theme: update.theme,
            },
            location: reply.location,
            cookies: vec![
                reply.cookie,
                self.scope
                    .header(THEME_COOKIE, update.theme.as_str(), 31536000),
            ],
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
        let expected =
            self.registry
                .location(&self.scope.base, &submitted.location, &submitted.locale)?;
        if Some(reply.preferences.locale.as_str()) != self.registry.canonical(&submitted.locale)
            || reply.preferences.theme != submitted.theme
            || reply.location != expected
        {
            return Err(PreferenceError::Request);
        }
        Ok(reply)
    }
}
