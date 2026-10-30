//! Source OAuth presentation and API-specific generated-role adapters over shared Leptos UI.
//! Browser imports contain no SQL/JWT/HTTP host. Native rendering/registration is explicit.
#[cfg(all(feature = "ui-ssr", feature = "sql", not(target_arch = "wasm32")))]
pub mod admin;
mod consent;
#[cfg(all(feature = "ui-ssr", not(target_arch = "wasm32")))]
mod document;
mod icons;
pub mod management;
pub mod roles;
pub use consent::{
    ApiOauthForm, ApiOauthFormProps, ApiOauthHead, ApiOauthHeadProps, ApiOauthPage,
    ApiOauthPageProps, OAuthBody, OAuthBodyProps,
};
pub use consent::{ApiOauthHead as Head, ApiOauthHeadProps as HeadProps};
#[cfg(all(feature = "ui-ssr", not(target_arch = "wasm32")))]
pub mod host;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_leptos::language::Language;

/// Public source page props. Selected application credentials remain present intentionally.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Consent {
    /// The OAuth process's selected api/view/brand response data.
    pub config: Value,
    /// Original aggregate source request data; no caller service is serialized.
    pub request: Value,
    /// Source selected application result, including its plaintext secret.
    pub application: Value,
    /// Source additional stylesheet hrefs, following global.css.
    pub styles: Vec<String>,
    /// Existing literal phrase provider, shared by SSR and browser roots.
    pub language: Language,
    /// Existing acknowledged Light/Dark preference; None admits the shared one-shot bootstrap.
    pub theme: Option<stackpress_framework_view::preferences::ThemePreference>,
}
impl Consent {
    /// Source defaults for a directly constructed page; no resource acquisition occurs here.
    pub fn new(config: Value, request: Value, application: Value) -> Self {
        Self {
            config,
            request,
            application,
            styles: Vec::new(),
            language: Language::new("en".into(), Default::default(), "%s".into())
                .expect("static language"),
            theme: None,
        }
    }
    /// Known requested scope entries in original space-split order, including repeats.
    pub fn items(&self) -> Vec<ScopeItem> {
        let scopes = self.config.pointer("/api/scopes");
        self.request
            .get("scope")
            .and_then(Value::as_str)
            .unwrap_or("")
            .split(' ')
            .filter_map(|id| {
                scopes
                    .and_then(|scopes| scopes.get(id))
                    .map(|scope| ScopeItem {
                        id: id.into(),
                        icon: scope.get("icon").and_then(Value::as_str).map(str::to_owned),
                        name: scope
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .into(),
                        description: scope
                            .get("description")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .into(),
                    })
            })
            .collect()
    }
    /// Source denial split/query algorithm, without normalizing the destination itself.
    pub fn revert(&self) -> String {
        let redirect = self
            .request
            .get("redirect_uri")
            .and_then(Value::as_str)
            .unwrap_or("/");
        let mut parts = redirect.split('?');
        let uri = parts.next().unwrap_or("");
        let pairs = query_pairs(parts.next().unwrap_or(""));
        format!("{uri}?{}", set_parameter(pairs, "error", "denied"))
    }
}
/// One source scope description. Labels are text, never imports or executable markup.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScopeItem {
    /// Exact source scope string submitted in scopes[].
    pub id: String,
    /// Optional source concept mapped to a compiled Lucide glyph.
    pub icon: Option<String>,
    /// Human label, escaped by Leptos.
    pub name: String,
    /// Optional explanatory text.
    pub description: String,
}
/// Scoped API skin; existing component/token styles remain authoritative.
pub const STYLES: &str = include_str!("styles.css");
/// Parse source query pairs through the already selected URL library.
fn query_pairs(query: &str) -> Vec<(String, String)> {
    // UI uses the foundation Request URL type's detached carrier; no network request occurs.
    let mut carrier = stackpress_lib::Request::new().url;
    carrier.set_query(Some(
        &query
            .replace('\t', "%09")
            .replace('\r', "%0D")
            .replace('\n', "%0A"),
    ));
    carrier.query_pairs().into_owned().collect()
}
/// URLSearchParams.set replaces the first duplicate in place and removes remaining duplicates.
fn set_parameter(mut pairs: Vec<(String, String)>, key: &str, value: &str) -> String {
    let mut found = false;
    pairs.retain_mut(|(name, old)| {
        if name != key {
            return true;
        }
        if found {
            return false;
        }
        *old = value.into();
        found = true;
        true
    });
    if !found {
        pairs.push((key.into(), value.into()));
    }
    let mut carrier = stackpress_lib::Request::new().url;
    carrier.set_query(None);
    carrier.query_pairs_mut().extend_pairs(pairs);
    carrier.query().unwrap_or("").into()
}
/// JSON member display preserves ordinary text and avoids private render-error detail.
pub(crate) fn string(value: Option<&Value>, fallback: &str) -> String {
    value.and_then(Value::as_str).unwrap_or(fallback).into()
}
/// Inert seed serialization uses the same JSON escaping convention as existing page hosts.
pub fn seed(value: &impl Serialize) -> Result<String, serde_json::Error> {
    Ok(serde_json::to_string(value)?
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026"))
}
