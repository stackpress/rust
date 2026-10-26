//! Bounded locale registry, explicit route/cookie/default precedence and actions.
//!
//! This crate has no listener, identity/session implementation or browser globals.
//! Hosts supply exact Origin/method/body metadata and apply the returned cookie.
//! Presentation preferences are not authentication. Session-bearing hosts must
//! additionally enforce their own CSRF policy before admitting this action.

use serde::{Deserialize, Serialize};
use stackpress_framework_view::{BasePath, PublicLocation};
use std::collections::{BTreeMap, BTreeSet};

/// Fixed cookie key; its value is only a registered canonical locale token.
pub const LOCALE_COOKIE: &str = "stackpress-locale";
/// Bounded action input before parsing or policy evaluation.
pub const MAX_PREFERENCE_BYTES: usize = 1024;
/// Total cookie header budget, including unrelated scoped presentation cookies.
pub const MAX_COOKIE_BYTES: usize = 4096;

/// Stable public preference failure without reflecting private/raw input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreferenceError {
    /// Registry/default/alias/origin configuration is invalid.
    Configuration,
    /// Selected locale is unregistered or malformed.
    Locale,
    /// Wrong method, malformed location, or extra/unapproved action data.
    Request,
    /// Origin does not exactly match the configured application origin.
    Origin,
    /// Input exceeds the admission budget.
    Limit,
}
impl std::fmt::Display for PreferenceError {
    /// Public diagnostics never contain cookies or caller input.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Configuration => "Invalid preference configuration",
            Self::Locale => "Invalid locale",
            Self::Request => "Invalid preference request",
            Self::Origin => "Preference origin denied",
            Self::Limit => "Preference input too large",
        })
    }
}
impl std::error::Error for PreferenceError {}

/// Source of the selected locale, useful for explicit host diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocaleSource {
    /// A configured canonical locale or explicit alias in the route wins.
    Prefix,
    /// A valid saved canonical preference wins on a legacy unprefixed route.
    Cookie,
    /// Invalid/missing cookies fall back to the configured default.
    Default,
}

/// Public selection plus the application-local route without its locale prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    /// Canonical registered locale, never a browser-language guess.
    pub locale: String,
    /// Explicit authority used for this selection.
    pub source: LocaleSource,
    /// Checked app-local path/query; unknown prefixes are preserved for host 404s.
    pub route: String,
}

/// Compiled bounded locale/alias registry; immutable and safe to share between roots.
#[derive(Clone, Debug)]
pub struct LocaleRegistry {
    /// Registered canonical tokens; only these may be persisted.
    locales: BTreeSet<String>,
    /// Direct aliases target canonical locales; alias chains/cycles are impossible.
    aliases: BTreeMap<String, String>,
    /// Explicit configured fallback, always in locales.
    default: String,
}

/// Accept a bounded token without path, header or control-character syntax.
fn valid_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 64
        && token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

impl LocaleRegistry {
    /// Validate canonical/default/alias identities before host publication.
    ///
    /// # Errors
    /// Rejects duplicates, unknown defaults/targets, alias collisions and budgets.
    pub fn new(
        locales: Vec<String>,
        default: String,
        aliases: BTreeMap<String, String>,
    ) -> Result<Self, PreferenceError> {
        // Detect duplicate registrations before set collection could hide them.
        let unique: BTreeSet<_> = locales.iter().cloned().collect();
        if locales.is_empty()
            || locales.len() > 16
            || unique.len() != locales.len()
            || locales.iter().any(|locale| !valid_token(locale))
            || !unique.contains(&default)
            || aliases.len() > 32
            || aliases.iter().any(|(alias, target)| {
                !valid_token(alias) || unique.contains(alias) || !unique.contains(target)
            })
        {
            return Err(PreferenceError::Configuration);
        }
        Ok(Self {
            locales: unique,
            aliases,
            default,
        })
    }

    /// Resolve only configured exact tokens, without regional/case guessing.
    pub fn canonical<'a>(&'a self, token: &'a str) -> Option<&'a str> {
        // Canonical keys retain exact spelling; aliases never recurse or guess.
        if self.locales.contains(token) {
            Some(token)
        } else {
            self.aliases.get(token).map(String::as_str)
        }
    }

    /// Apply prefix > valid saved cookie > default under one checked mount.
    ///
    /// # Errors
    /// Rejects an unsafe/out-of-mount location or oversized cookie input.
    pub fn select(
        &self,
        base: &BasePath,
        location: &str,
        cookie: &str,
    ) -> Result<Selection, PreferenceError> {
        PublicLocation::new(location).map_err(|_| PreferenceError::Request)?;
        let local = base.strip(location).ok_or(PreferenceError::Request)?;
        if cookie.len() > MAX_COOKIE_BYTES {
            return Err(PreferenceError::Limit);
        }
        // Mount stripping guarantees the first slash; ASCII separators preserve
        // UTF-8 boundaries and query bytes without a second decode.
        let end = local[1..]
            .find(['/', '?'])
            .map_or(local.len(), |index| index + 1);
        if let Some(locale) = self.canonical(&local[1..end]) {
            let rest = &local[end..];
            let route = if rest.is_empty() {
                "/".to_owned()
            } else if rest.starts_with('?') {
                format!("/{rest}")
            } else {
                rest.to_owned()
            };
            return Ok(Selection {
                locale: locale.to_owned(),
                source: LocaleSource::Prefix,
                route,
            });
        }
        // Duplicate locale cookies are ambiguous and intentionally ignored.
        let mut saved = None;
        let mut duplicate = false;
        for entry in cookie.split(';').map(str::trim) {
            // A bare matching key still counts, so malformed duplicates cannot
            // disappear during split_once and elevate a second value.
            let (name, value) = entry.split_once('=').unwrap_or((entry, ""));
            if name == LOCALE_COOKIE {
                duplicate |= saved.is_some();
                saved = Some(value);
            }
        }
        if !duplicate && let Some(saved) = saved.filter(|saved| self.locales.contains(*saved)) {
            return Ok(Selection {
                locale: saved.to_owned(),
                source: LocaleSource::Cookie,
                route: local.to_owned(),
            });
        }
        // Missing, malformed, unknown or ambiguous saved values use one fallback.
        Ok(Selection {
            locale: self.default.clone(),
            source: LocaleSource::Default,
            route: local.to_owned(),
        })
    }

    /// Enumerate compiled canonical and alias prefixes for host route registration.
    ///
    /// Legacy unprefixed routes are included first. The host registers these routes
    /// before handling requests; locale selection never re-enters a top-level lifecycle.
    pub fn prefixes(&self) -> Vec<String> {
        std::iter::once(String::new())
            .chain(
                self.locales
                    .iter()
                    .chain(self.aliases.keys())
                    .map(|key| format!("/{key}")),
            )
            .collect()
    }

    /// Replace a recognized prefix, preserving mount and exact encoded query bytes.
    ///
    /// # Errors
    /// Rejects unknown locale tokens, unsafe paths and paths outside the mount.
    pub fn location(
        &self,
        base: &BasePath,
        location: &str,
        locale: &str,
    ) -> Result<String, PreferenceError> {
        let locale = self.canonical(locale).ok_or(PreferenceError::Locale)?;
        let current = self.select(base, location, "")?;
        // Only the first registered prefix is removed; a duplicate remains a route
        // segment, so a host catalog can reject it rather than silently repairing it.
        base.join(&format!("/{locale}{}", current.route))
            .map(|value| value.as_str().to_owned())
            .map_err(|_| PreferenceError::Request)
    }
}

/// Closed action input: no arbitrary cookie field, URL or redirect destination.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreferenceUpdate {
    /// Requested configured canonical locale or explicit alias.
    pub locale: String,
    /// Current same-app local location; mount/query are preserved, never redirected off-site.
    pub location: String,
}

/// Acknowledged public values plus a host-only Set-Cookie header.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreferenceReply {
    /// Canonical locale for the root dictionary/Head/lang update.
    pub locale: String,
    /// Checked locale-prefixed location for history replacement.
    pub location: String,
    /// Host applies this header; browser code never reads the HttpOnly cookie.
    #[serde(skip)]
    pub cookie: String,
}

/// Fixed same-origin presentation action, independent of a concrete server adapter.
#[derive(Clone, Debug)]
pub struct PreferenceAction {
    /// Registry is immutable; there is no process-global current locale.
    registry: LocaleRegistry,
    /// Exact canonical application origin.
    origin: String,
    /// Cookie Path and return-location authority use the same checked mount.
    base: BasePath,
    /// HTTP development origin explicitly omits Secure; HTTPS always includes it.
    secure: bool,
}

impl PreferenceAction {
    /// Configure an exact canonical HTTP(S) origin and scoped cookie mount.
    ///
    /// # Errors
    /// Rejects credentials, paths/query/fragment or noncanonical/non-HTTP origins.
    pub fn new(
        registry: LocaleRegistry,
        origin: String,
        base: BasePath,
    ) -> Result<Self, PreferenceError> {
        // Compare canonical origin serialization, excluding credentials/path tricks.
        let parsed = url::Url::parse(&origin).map_err(|_| PreferenceError::Configuration)?;
        if !matches!(parsed.scheme(), "http" | "https")
            || parsed.origin().ascii_serialization() != origin
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.query().is_some()
            || parsed.fragment().is_some()
            || parsed.path() != "/"
        {
            return Err(PreferenceError::Configuration);
        }
        Ok(Self {
            registry,
            secure: parsed.scheme() == "https",
            origin,
            base,
        })
    }

    /// Validate one POST and return a canonical acknowledged preference/cookie.
    ///
    /// # Errors
    /// Denies wrong/missing Origin, non-POST, unregistered locale, unsafe/mount-escaping
    /// locations and bounded-input excess. Failure performs no mutation or retry.
    pub fn apply(
        &self,
        method: &str,
        origin: Option<&str>,
        update: PreferenceUpdate,
    ) -> Result<PreferenceReply, PreferenceError> {
        // Reject authority/method failures before constructing any cookie output.
        if method != "POST" {
            return Err(PreferenceError::Request);
        }
        if origin != Some(self.origin.as_str()) {
            return Err(PreferenceError::Origin);
        }
        if update.locale.len() + update.location.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::Limit);
        }
        let locale = self
            .registry
            .canonical(&update.locale)
            .ok_or(PreferenceError::Locale)?
            .to_owned();
        // Re-prefix only the checked app path; encoded query text is preserved.
        let location = self
            .registry
            .location(&self.base, &update.location, &locale)?;
        let path = if self.base.as_str().is_empty() {
            "/"
        } else {
            self.base.as_str()
        };
        let secure = if self.secure { "; Secure" } else { "" };
        // Locale/base came from checked constructors, so no header syntax is accepted.
        let cookie = format!(
            "{LOCALE_COOKIE}={locale}; Path={path}; Max-Age=31536000; HttpOnly; SameSite=Lax{secure}"
        );
        Ok(PreferenceReply {
            locale,
            location,
            cookie,
        })
    }
}

#[cfg(test)]
mod tests;
