//! Serializable source page values; no Engine, signing seed or native model descriptor.
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The compiled public route view, shared across SSR, hydration and CSR.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Published articles in their native query order.
    #[default]
    Home,
    /// One source article with its separately loaded comments.
    Article,
    /// The original source error view.
    Error,
}

/// Public JSON page envelope, selected explicitly from native response/configuration values.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Page {
    /// Selected compiled view.
    pub kind: Kind,
    /// Document title.
    pub title: String,
    /// Source response results, retaining author/comment joins and article HTML.
    pub data: Value,
    /// Native response status.
    pub code: u16,
    /// Source public error message.
    pub error: String,
    /// Location path, excluding native adapter resources.
    pub path: String,
    /// Source production/development mode for error details.
    pub mode: String,
    /// Captured source stack/snippets, only data returned by the source error operation.
    pub stack: Value,
    /// Existing session authorization result, never the signing seed or Service.
    pub session: Value,
    /// Source locale cookie selection, retaining underscore locale names.
    pub locale: String,
    /// Source translation phrase dictionary.
    pub translations: Value,
    /// Deliberately public brand fields.
    pub brand: Value,
    /// Source additional styles.
    pub styles: Vec<String>,
    /// Acknowledged saved theme; absence uses the framework's existing one-shot bootstrap.
    pub theme: Option<String>,
}
impl Page {
    /// Match the source phrase lookup's truthy fallback for string translations.
    pub fn phrase(&self, value: &str) -> String {
        self.translations
            .get(value)
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .unwrap_or(value)
            .into()
    }
}
