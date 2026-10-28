//! Location-aware locale selection and fixed same-origin actions.
use crate::adapters::web::{CookieScope, unique_cookie};
pub use crate::adapters::web::{LOCALE_COOKIE, MAX_COOKIE_BYTES, MAX_PREFERENCE_BYTES};
use crate::{BasePath, PublicLocation};
use serde::{Deserialize, Serialize};
use stackpress_framework_language::{LocaleRegistry, PreferenceError};

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

/// Checked route/cookie integration borrowing a pure locale registry.
pub struct LocaleRouting<'a> {
    /// Immutable authority; no duplicate locale validation lives in view.
    registry: &'a LocaleRegistry,
    /// Existing checked view mount, with unchanged public path/query codecs.
    base: &'a BasePath,
}
impl<'a> LocaleRouting<'a> {
    /// Bind immutable routing inputs without registering handlers or effects.
    pub fn new(registry: &'a LocaleRegistry, base: &'a BasePath) -> Self {
        // Borrow immutable authority only; selection installs no effects or handlers.
        Self { registry, base }
    }
    /// Apply prefix > valid saved cookie > default under one checked mount.
    ///
    /// # Errors
    /// Rejects an unsafe/out-of-mount location or oversized cookie input.
    pub fn select(&self, location: &str, cookie: &str) -> Result<Selection, PreferenceError> {
        // Validate and strip the trusted mount before considering any prefix or saved value.
        let base = self.base;
        PublicLocation::new(location).map_err(|_| PreferenceError::Request)?;
        let local = base.strip(location).ok_or(PreferenceError::Request)?;
        // Preserve the whole-header limit even when a route prefix already determines locale.
        if cookie.len() > MAX_COOKIE_BYTES {
            return Err(PreferenceError::Limit);
        }
        // Mount stripping guarantees the first slash; ASCII separators preserve
        // UTF-8 boundaries and query bytes without a second decode.
        let end = local[1..]
            .find(['/', '?'])
            .map_or(local.len(), |index| index + 1);
        // A compiled canonical/alias prefix wins; leave the remaining query bytes untouched.
        if let Some(locale) = self.registry.canonical(&local[1..end]) {
            let rest = &local[end..];
            // A bare locale root becomes slash; query-only remainders need the same slash.
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
        // The shared codec rejects duplicates, including bare matching keys. Saved
        // aliases still do not count: only canonical persisted tokens may select a locale.
        let saved = unique_cookie(cookie, LOCALE_COOKIE)
            .ok()
            .flatten()
            .filter(|saved| self.registry.locales().any(|locale| locale == *saved));
        // A single canonical cookie is used only when no compiled prefix matched.
        if let Some(saved) = saved {
            return Ok(Selection {
                locale: saved.to_owned(),
                source: LocaleSource::Cookie,
                route: local.to_owned(),
            });
        }
        // Missing, malformed, unknown or ambiguous saved values use one fallback.
        Ok(Selection {
            locale: self.registry.default_locale().to_owned(),
            source: LocaleSource::Default,
            route: local.to_owned(),
        })
    }

    /// Enumerate compiled canonical and alias prefixes for host route registration.
    ///
    /// Legacy unprefixed routes are included first. The host registers these routes
    /// before handling requests; locale selection never re-enters a top-level lifecycle.
    pub fn prefixes(&self) -> Vec<String> {
        // Preserve canonical-then-alias order; the legacy prefix stays first.
        std::iter::once(String::new())
            .chain(
                self.registry
                    .locales()
                    .chain(self.registry.aliases().map(|(alias, _)| alias))
                    .map(|key| format!("/{key}")),
            )
            .collect()
    }

    /// Replace a recognized prefix, preserving mount and exact encoded query bytes.
    ///
    /// # Errors
    /// Rejects unknown locale tokens, unsafe paths and paths outside the mount.
    pub fn location(&self, location: &str, locale: &str) -> Result<String, PreferenceError> {
        // Borrow the same trusted mount for the canonical locale replacement.
        let base = self.base;
        // Resolve an explicit registered target; unknown tokens never become route segments.
        let locale = self
            .registry
            .canonical(locale)
            .ok_or(PreferenceError::Locale)?;
        let current = self.select(location, "")?;
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
    /// One validated owner of origin admission, cookie Path and HTTPS flags.
    scope: CookieScope,
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
        // Reuse the same origin/scope checks as combined settings and flash.
        let scope = CookieScope::new(origin, base)?;
        Ok(Self { registry, scope })
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
        // An exact origin check precedes input projection and all cookie construction.
        if origin != Some(self.scope.origin.as_str()) {
            return Err(PreferenceError::Origin);
        }
        // Bound the already-decoded fixed fields before canonicalization/path work.
        if update.locale.len() + update.location.len() > MAX_PREFERENCE_BYTES {
            return Err(PreferenceError::Limit);
        }
        // Resolve an explicit registered target; unknown tokens never become route segments.
        let locale = self
            .registry
            .canonical(&update.locale)
            .ok_or(PreferenceError::Locale)?
            .to_owned();
        // Re-prefix only the checked app path; encoded query text is preserved.
        let location = LocaleRouting::new(&self.registry, &self.scope.base)
            .location(&update.location, &locale)?;
        // The shared scope emits the unchanged locale identity, lifetime and flags.
        let cookie = self.scope.header(LOCALE_COOKIE, &locale, 31536000);
        Ok(PreferenceReply {
            locale,
            location,
            cookie,
        })
    }
}

#[cfg(test)]
mod tests;
