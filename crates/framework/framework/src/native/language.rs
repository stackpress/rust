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

/// Configure source language only when its optional section exists.
pub(crate) fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("language", None::<Config>, |_| Ok(()))?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    Ok(())
}

/// Freeze the source section in Config before any request hook is published.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    if let Some(config) = start.config::<Option<Config>>()?.clone() {
        start.register(Arc::new(Service { config }))?;
    }
    Ok(Flow::Continue)
}

/// A missing source section contributes no request listener.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    if start.config::<Option<Config>>()?.is_some() {
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
