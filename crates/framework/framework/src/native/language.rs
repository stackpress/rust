//! Source locale-cookie and prefix dispatch over buffered request projections.
//!
//! A native reader remains owned by its execution adapter and must finish loading before
//! handing off its buffered request. No Request/reader is cloned or serialized; only its
//! explicitly cloneable RequestView values form the child. Raw nested dispatch shares the
//! response and host context, without another Input/request/response lifecycle.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::Arc;

/// Source language settings, retaining locale keys without preference-registry normalization.
#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Source cookie/data key, default locale.
    pub key: String,
    /// Source fallback locale, default en_US.
    pub locale: String,
    /// Source locale data, including labels/translations; membership uses exact keys.
    pub languages: Map<String, Value>,
}

impl Default for Config {
    /// Construct source defaults without a process-global language singleton.
    fn default() -> Self {
        Self {
            key: "locale".into(),
            locale: "en_US".into(),
            languages: Map::new(),
        }
    }
}

/// Immutable per-host language configuration; individual requests keep their own locale.
pub struct Service {
    /// Source section, with no independent access or preference policy added.
    pub config: Config,
}

/// Uninterpreted source section, retaining false/zero/empty-string deactivation.
#[derive(Default, Serialize, Deserialize)]
#[serde(transparent)]
struct Section(Value);

impl Service {
    /// Source locale keys in insertion order, with no normalization.
    pub fn locales(&self) -> Vec<&str> {
        self.config.languages.keys().map(String::as_str).collect()
    }

    /// Copy one source language's label/translations; missing/falsy entries return null.
    pub fn language(&self, name: &str) -> Option<Value> {
        let value = self
            .config
            .languages
            .get(name)
            .filter(|v| super::config::truthy(v))?;
        Some(
            serde_json::json!({"label":value.get("label"),"translations":value.get("translations").and_then(Value::as_object).cloned().unwrap_or_default()}),
        )
    }

    /// Load the request cookie, retaining unknown locale names and the caller's fallback.
    pub fn load<'a>(&'a self, req: &Request, defaults: &str) -> Language<'a> {
        let locale = req
            .session
            .get(&self.config.key)
            .and_then(|v| v.as_str())
            .filter(|v| !v.is_empty())
            .unwrap_or(defaults)
            .to_owned();
        Language {
            service: self,
            locale,
        }
    }
}

/// Per-request language selection borrowing immutable host configuration.
pub struct Language<'a> {
    /// Immutable per-host source map, replacing only the mechanics of a process singleton.
    service: &'a Service,
    /// Unknown locale strings remain valid source instance values.
    pub locale: String,
}

/// Source interpolation scalars, including undefined separately from null.
pub enum Scalar<'a> {
    /// A source string, including replacement-token dollar signs.
    String(&'a str),
    /// A source JavaScript number.
    Number(f64),
    /// A source boolean.
    Boolean(bool),
    /// An explicit source null.
    Null,
    /// A missing source variable.
    Undefined,
}

impl Scalar<'_> {
    /// Match source String(scalar) before its first string-pattern replacement.
    fn text(&self) -> String {
        match self {
            Self::String(value) => (*value).into(),
            Self::Number(value) if value.is_nan() => "NaN".into(),
            Self::Number(value) if *value == f64::INFINITY => "Infinity".into(),
            Self::Number(value) if *value == f64::NEG_INFINITY => "-Infinity".into(),
            Self::Number(value) if *value == 0.0 => "0".into(),
            Self::Number(value) if value.abs() >= 1e21 || value.abs() < 1e-6 => {
                let exponent = format!("{value:e}");
                let (mantissa, power) = exponent.split_once('e').expect("scientific notation");
                format!(
                    "{mantissa}e{}{power}",
                    if power.starts_with('-') { "" } else { "+" }
                )
            }
            Self::Number(value) => value.to_string(),
            Self::Boolean(value) => value.to_string(),
            Self::Null => "null".into(),
            Self::Undefined => "undefined".into(),
        }
    }
}

/// Preserve JS replacement tokens even when the search pattern is the literal %s string.
fn replace_first(input: &str, replacement: &str) -> String {
    let Some(index) = input.find("%s") else {
        return input.into();
    };
    let (before, tail) = input.split_at(index);
    let after = &tail[2..];
    let mut output = before.to_owned();
    let mut chars = replacement.chars().peekable();
    while let Some(character) = chars.next() {
        if character == '$' {
            match chars.peek() {
                Some('$') => {
                    output.push('$');
                    chars.next();
                }
                Some('&') => {
                    output.push_str("%s");
                    chars.next();
                }
                Some('`') => {
                    output.push_str(before);
                    chars.next();
                }
                Some('\'') => {
                    output.push_str(after);
                    chars.next();
                }
                _ => output.push(character),
            }
        } else {
            output.push(character);
        }
    }
    output.push_str(after);
    output
}

impl Language<'_> {
    /// Source truthy label, otherwise the current locale string.
    pub fn label(&self) -> Value {
        self.service
            .config
            .languages
            .get(&self.locale)
            .and_then(|v| v.get("label"))
            .filter(|v| super::config::truthy(v))
            .cloned()
            .unwrap_or_else(|| self.locale.clone().into())
    }
    /// Borrow source translations; absent locales retain an empty map.
    pub fn translations(&self) -> Option<&Map<String, Value>> {
        self.service
            .config
            .languages
            .get(&self.locale)
            .and_then(|v| v.get("translations"))
            .and_then(Value::as_object)
    }
    /// Save the current locale with the original unscoped cookie behavior.
    /// # Errors
    /// Retains response-cookie validation and finalized-response errors.
    pub fn save(&self, res: &mut Response) -> Result<&Self> {
        res.session_mut()?
            .set(&self.service.config.key, self.locale.as_str(), None)?;
        Ok(self)
    }
    /// Update the locale, then stage its cookie.
    /// # Errors
    /// Retains the same cookie errors as save; selection changes before staging, as upstream.
    pub fn update(&mut self, locale: &str, res: &mut Response) -> Result<&Self> {
        self.locale = locale.into();
        self.save(res)
    }
    /// Translate a phrase and replace one first %s occurrence per supplied scalar.
    pub fn translate(&self, phrase: &str, variables: &[Scalar<'_>]) -> String {
        let mut value = self
            .translations()
            .and_then(|map| map.get(phrase))
            .filter(|v| super::config::truthy(v))
            .and_then(Value::as_str)
            .unwrap_or(phrase)
            .to_owned();
        for variable in variables {
            value = replace_first(&value, &variable.text());
        }
        value
    }
}

/// Configure source language only when its optional section exists.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("language", Section::default(), |_| Ok(()))?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    Ok(())
}

/// Freeze the source section in Config before any request hook is published.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let value = &start.config::<Section>()?.0;
    if super::config::truthy(value) {
        let config = if value.is_object() {
            serde_json::from_value(value.clone())?
        } else {
            Config::default()
        };
        start.register(Arc::new(Service { config }))?;
    }
    Ok(Flow::Continue)
}

/// A missing source section contributes no request listener.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    if super::config::truthy(&start.config::<Section>()?.0) {
        start.on("request", request)?;
    }
    Ok(Flow::Continue)
}

/// Apply data locale, then prefix locale, retaining raw child Abort as a source-local outcome.
/// # Errors
/// Propagates existing cookie, buffered-request construction and nested handler failures.
pub async fn request(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let service = inv.service::<Service>()?;
    let config = &service.config;
    // Source data selection stages the cookie first; a recognized prefix overrides it.
    if let Some(locale) = req.data.get(&config.key).and_then(|value| value.as_str())
        && config.languages.contains_key(locale)
    {
        res.session_mut()?.set(&config.key, locale, None)?;
    }
    let mut path: Vec<_> = req.url.path().split('/').collect();
    let locale = path.get(1).copied().unwrap_or_default();
    if config.languages.contains_key(locale) {
        res.session_mut()?.set(&config.key, locale, None)?;
        path.remove(1);
        let path = path.join("/");
        // Source reconstructs a child URL from origin + stripped path, omitting its query;
        // the separate query/post/data/header/cookie values still survive the projection.
        let view = req.view();
        let mut url = view.url;
        url.set_path(&path);
        url.set_query(None);
        url.set_fragment(None);
        let mut child = Request::builder()
            .url(url.as_str())
            .method(view.method.as_str())
            .headers(view.headers)
            .session(view.session)
            .body(view.body)
            .mimetype(&view.mimetype)
            .query(view.query)
            .post(view.post)
            .data(view.data)
            .build()?;
        // The adapter's native load attempt cannot be replayed by this buffered child.
        child.load().await?;
        let event = format!("{} {path}", child.method.as_str());
        let _ = inv.emit(&event, &mut child, res).await?;
    }
    // Upstream does not abort the outer request when the nested route aborts.
    Ok(Flow::Continue)
}
