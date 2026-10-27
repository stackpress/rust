//! Pure public seed admission and the compiled locale/phrase catalog.
use serde::{Deserialize, Serialize};
use stackpress_framework_language::{LocaleRegistry, PreferenceError};
use stackpress_framework_preferences::{
    AllowedActions, CookieScope, FlashNotice, Preferences, SettingsAction, ThemePreference,
};
use stackpress_framework_ui::language::Language;
use stackpress_framework_view::BasePath;
use std::collections::BTreeMap;

/// Closed app seed, with no headers, identity, roles or token snapshots.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seed {
    /// Exact compiled asset/data identity.
    pub build: String,
    /// Host-selected preferences before first render.
    pub preferences: Preferences,
    /// Checked mount and public return location.
    pub base: BasePath,
    /// Exact encoded public path/query.
    pub location: String,
    /// One public bounded notice, consumed by the host.
    pub flash: Option<FlashNotice>,
    /// Safe layout hints; changing these cannot authorize a host operation.
    pub allowed: AllowedActions,
}
/// Fixed exact registry shared by route compilation, host selection and decoder.
pub fn registry() -> LocaleRegistry {
    LocaleRegistry::new(
        vec!["en".into(), "fr".into()],
        "en".into(),
        BTreeMap::from([("fr-FR".into(), "fr".into()), ("en_US".into(), "en".into())]),
    )
    .expect("valid compiled locale registry")
}
/// Fixed host origin and checked mount; no caller-supplied destination exists.
pub fn action(base: BasePath) -> SettingsAction {
    action_at_origin(base, "https://preferences.invalid").expect("valid compiled origin")
}
/// Construct presentation persistence for an explicit trusted canonical HTTP(S) origin.
/// # Errors
/// Rejects an invalid cookie origin before rendering or persistence.
pub fn action_at_origin(base: BasePath, origin: &str) -> Result<SettingsAction, PreferenceError> {
    Ok(SettingsAction::new(
        registry(),
        CookieScope::new(origin.into(), base)?,
        ThemePreference::System,
    ))
}

/// Decode and admit the host projection before mounting; no browser storage read.
///
/// # Errors
/// Rejects build drift, malformed/excess data and locale/location disagreement.
pub fn decode(text: &str) -> Result<Seed, PreferenceError> {
    if text.len() > 16 * 1024 {
        return Err(PreferenceError::Limit);
    }
    let seed: Seed = serde_json::from_str(text).map_err(|_| PreferenceError::Request)?;
    if seed.build != "s8-b-m1" {
        return Err(PreferenceError::Request);
    }
    let selected = registry().select(
        &seed.base,
        &seed.location,
        &format!("stackpress-locale={}", seed.preferences.locale),
    )?;
    if selected.route.split('?').next() != Some("/preferences")
        || selected.locale != seed.preferences.locale
        || registry().canonical(&seed.preferences.locale) != Some(seed.preferences.locale.as_str())
    {
        return Err(PreferenceError::Locale);
    }
    Ok(seed)
}
/// Use the existing checked phrase facility, including exact falsy/whitespace rules.
pub fn language(locale: &str) -> Language {
    let phrases = if locale == "fr" {
        BTreeMap::from([
            ("Preferences".into(), "Préférences".into()),
            ("Draft".into(), "Brouillon".into()),
            ("Home".into(), "Accueil".into()),
            ("%s item".into(), "%s article".into()),
            ("%s items".into(), "%s articles".into()),
            ("January 2, 2026".into(), "2 janvier 2026".into()),
            ("1,234.50".into(), "1\u{202f}234,50".into()),
        ])
    } else {
        BTreeMap::new()
    };
    Language::new(locale.to_owned(), phrases, "%s".into()).expect("bounded compiled phrases")
}
/// CSS is in the document head before first paint, including system-mode selection.
pub const STYLES: &str = include_str!("../style.css");
