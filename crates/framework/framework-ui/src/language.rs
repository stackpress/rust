//! Checked, scoped phrase lookup and explicit text/node interpolation.
//!
//! Missing slots are empty and excess slots are ignored. Whitespace and falsy
//! text values are preserved exactly; no text is interpreted as HTML or code.

use leptos::prelude::*;
use std::collections::BTreeMap;

/// Integer cardinal categories supported by the initial counted-phrase policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluralCategory {
    /// Singular form (French includes zero; English includes only one).
    One,
    /// All remaining supported integer counts.
    Other,
}

/// Bounded public locale, phrase map and literal placeholder token.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "LanguageData")]
pub struct Language {
    /// Checked locale token; not inferred from a cookie or browser global.
    locale: String,
    /// Ordered public phrase translations; absence falls back to the source phrase.
    phrases: BTreeMap<String, String>,
    /// Nonempty literal separator identifying interpolation positions.
    placeholder: String,
}

/// Private wire intermediate; decoding always passes the checked constructor.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct LanguageData {
    /// Explicit public locale key.
    locale: String,
    /// Untrusted phrase data, checked before becoming mounted state.
    phrases: BTreeMap<String, String>,
    /// Untrusted literal token, checked against empty/oversized values.
    placeholder: String,
}

impl TryFrom<LanguageData> for Language {
    type Error = &'static str;

    /// Prevent snapshot deserialization from bypassing the language budgets.
    fn try_from(data: LanguageData) -> Result<Self, Self::Error> {
        Self::new(data.locale, data.phrases, data.placeholder)
    }
}

impl Language {
    /// Check locale/map/placeholder before publishing them to a UI root.
    ///
    /// # Errors
    /// Rejects invalid locale tokens, empty/over-32-byte separators, over 256
    /// phrases, over-4096-byte phrases, or a map larger than 64 KiB.
    pub fn new(
        locale: String,
        phrases: BTreeMap<String, String>,
        placeholder: String,
    ) -> Result<Self, &'static str> {
        // Locale is an explicit data key, never a path or executable selector.
        if locale.is_empty()
            || locale.len() > 64
            || !locale
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            return Err("Invalid locale key");
        }
        if placeholder.is_empty() || placeholder.len() > 32 {
            return Err("Translation placeholder must contain 1–32 bytes");
        }
        if phrases.len() > 256
            || phrases
                .iter()
                .any(|(key, value)| key.len() > 4096 || value.len() > 4096)
            || phrases
                .iter()
                .map(|(key, value)| key.len() + value.len())
                .sum::<usize>()
                > 64 * 1024
        {
            return Err("Translation map is too large");
        }
        Ok(Self {
            locale,
            phrases,
            placeholder,
        })
    }

    /// Access the exact locale key for metadata integration.
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Select an integer cardinal category from the explicit locale.
    ///
    /// English uses One only for 1; French uses One for 0 and 1. Regional
    /// variants share their base-language rule. Other locales currently use
    /// the English one/other fallback; fractional and additional categories
    /// are outside this initial policy.
    pub fn plural_category(&self, count: u64) -> PluralCategory {
        let base = self.locale.split(['-', '_']).next().unwrap_or_default();
        let one = if base.eq_ignore_ascii_case("fr") {
            count <= 1
        } else {
            count == 1
        };
        if one {
            PluralCategory::One
        } else {
            PluralCategory::Other
        }
    }

    /// Select a source phrase before dictionary lookup or text/node interpolation.
    pub fn plural_phrase<'a>(&self, count: u64, one: &'a str, other: &'a str) -> &'a str {
        match self.plural_category(count) {
            PluralCategory::One => one,
            PluralCategory::Other => other,
        }
    }

    /// Select the locale's counted phrase and format its ordinary text slots.
    ///
    /// Count selects grammar; explicit slot values retain the same escaping,
    /// whitespace and missing/excess policies as [`Self::text`].
    pub fn plural_text(&self, count: u64, one: &str, other: &str, values: &[String]) -> String {
        self.text(self.plural_phrase(count, one, other), values)
    }

    /// Look up the phrase, retaining the original phrase when no entry exists.
    pub fn phrase<'a>(&'a self, phrase: &'a str) -> &'a str {
        self.phrases
            .get(phrase)
            .map(String::as_str)
            .unwrap_or(phrase)
    }

    /// Split translated text into literal fragments around deliberate slots.
    pub fn template(&self, phrase: &str) -> Vec<String> {
        self.phrase(phrase)
            .split(&self.placeholder)
            .map(str::to_owned)
            .collect()
    }

    /// Format text-only variables, preserving empty/zero/false and exact whitespace.
    ///
    /// Node values cannot enter this API: its input type is a string slice.
    /// Missing slots are empty; excess variables are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_framework_ui::language::Language;
    /// let language = Language::new("en_US".into(), Default::default(), "%s".into())?;
    /// assert_eq!(language.text("%s / %s", &["0".into(), "false".into()]), "0 / false");
    /// # Ok::<(), &'static str>(())
    /// ```
    pub fn text(&self, phrase: &str, values: &[String]) -> String {
        let mut output = String::new();
        for (index, fragment) in self.template(phrase).into_iter().enumerate() {
            // Insert variable n before fragment n+1; fragment zero has no predecessor.
            if index > 0 {
                output.push_str(
                    values
                        .get(index - 1)
                        .map(String::as_str)
                        .unwrap_or_default(),
                );
            }
            output.push_str(&fragment);
        }
        output
    }
}

/// Scoped locale handle; replacing language also replaces/clears its phrase map.
#[derive(Clone, Copy)]
pub struct LanguageContext(RwSignal<Language>);

impl LanguageContext {
    /// Initialize deterministically from checked public snapshot values.
    pub fn new(language: Language) -> Self {
        Self(RwSignal::new(language))
    }

    /// Read the locale/map reactively.
    pub fn get(self) -> Language {
        self.0.get()
    }

    /// Replace the locale and entire phrase map atomically; no fetching or persistence.
    pub fn change(self, language: Language) {
        self.0.set(language);
    }

    /// Format text using this root's current language and subscribe the consumer.
    pub fn text(self, phrase: &str, values: &[String]) -> String {
        self.0.with(|language| language.text(phrase, values))
    }

    /// Reactively select grammar and format text from the same scoped language.
    pub fn plural_text(self, count: u64, one: &str, other: &str, values: &[String]) -> String {
        self.0
            .with(|language| language.plural_text(count, one, other, values))
    }
}

/// Read the current root's language, diagnosing a missing provider explicitly.
pub fn use_language() -> Result<LanguageContext, &'static str> {
    use_context().ok_or("Translations require LanguageProvider")
}

/// Provide one checked locale/map to descendants, without a global singleton.
#[component]
pub fn LanguageProvider(
    /// Owned checked initial public language.
    language: LanguageContext,
    /// Consumers that share this scoped language.
    children: Children,
) -> impl IntoView {
    provide_context(language);
    children()
}

/// Compose escaped text fragments with deliberate, reusable Leptos view slots.
#[component]
pub fn Translate(
    /// Explicit context also makes provider-free rendering/test fixtures possible.
    language: LanguageContext,
    /// Stable phrase key, including its literal placeholder positions.
    #[prop(into)]
    phrase: String,
    /// Optional One phrase; when supplied, phrase is the Other form.
    #[prop(optional, into)]
    one: Option<String>,
    /// Integer count selecting grammar, independent of the deliberate node slots.
    #[prop(default = Signal::derive(|| 0), into)]
    count: Signal<u64>,
    /// Slots occupy placeholders in order; missing are empty, excess ignored.
    #[prop(default = Vec::new())]
    slots: Vec<ViewFn>,
) -> impl IntoView {
    // Rebuild only this translation's fragments when its scoped language changes.
    move || {
        let language = language.get();
        // Both text and node callers select from the identical locale category policy.
        let selected = one.as_deref().map_or(phrase.as_str(), |one| {
            language.plural_phrase(count.get(), one, &phrase)
        });
        language
            .template(selected)
            .into_iter()
            .enumerate()
            .map(|(index, fragment)| {
                let slot = index
                    .checked_sub(1)
                    .and_then(|index| slots.get(index))
                    .map(ViewFn::run);
                view! { {slot}{fragment} }
            })
            .collect_view()
    }
}

#[cfg(test)]
#[path = "language/tests.rs"]
mod tests;
