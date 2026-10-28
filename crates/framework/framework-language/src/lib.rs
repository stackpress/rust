//! Pure locale validation and explicit aliases, independent of locations, cookies and hosts.
use std::collections::{BTreeMap, BTreeSet};
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

    /// Borrow canonical locale identities without permitting mutation.
    pub fn locales(&self) -> impl Iterator<Item = &str> {
        self.locales.iter().map(String::as_str)
    }
    /// Borrow explicit alias-to-canonical mappings without regional guessing.
    pub fn aliases(&self) -> impl Iterator<Item = (&str, &str)> {
        self.aliases
            .iter()
            .map(|(alias, target)| (alias.as_str(), target.as_str()))
    }
    /// Borrow the validated configured fallback.
    pub fn default_locale(&self) -> &str {
        &self.default
    }
}

#[cfg(test)]
mod tests;

// Routing, cookies and action DTOs now live in framework-view::preferences::locale.
