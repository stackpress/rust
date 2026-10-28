//! Explicit scoped HTML response integration for public flash notices.
use super::{CookieScope, FlashCookie, FlashNotice};
use crate::{BasePath, PublicLocation};
use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::{collections::BTreeSet, sync::Arc};

/// Trusted `view_flash` configuration, independent of preference selection.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Canonical issuing origin; no incoming host/header chooses cookie policy.
    pub origin: String,
    /// Shared issuing/deleting mount scope.
    pub base: BasePath,
    /// Exact application-relative GET pages allowed to project/consume this cookie.
    pub pages: Vec<PublicLocation>,
}
impl Config {
    /// Reject ambiguous scope and duplicate/nonliteral routes before any hooks are installed.
    fn validate(&self) -> Result<()> {
        CookieScope::new(self.origin.clone(), self.base.clone()).map_err(Error::from_source)?;
        if self.pages.is_empty() || self.pages.len() > 128 {
            return Err(Error::invalid(
                "flash pages must contain 1..=128 exact paths",
            ));
        }
        let mut seen = BTreeSet::new();
        for page in &self.pages {
            crate::adapters::web::host::literal_path(&self.base, page)?;
            if !seen.insert(page.as_str()) {
                return Err(Error::invalid("duplicate flash page"));
            }
        }
        Ok(())
    }
}
/// Immutable bounded mechanism; all per-request data stays borrowed and no root is retained.
pub struct Service {
    /// Cookie codec uses the exact same scope for issuance and deletion.
    cookie: FlashCookie,
    /// Fully mounted exact paths; prefix lookalikes and operations do not consume notices.
    pages: BTreeSet<String>,
}
impl Service {
    /// Deliberately project only a checked public notice for an owned page.
    /// No header is changed here; the selected HTML response hook clears before commitment.
    /// # Errors
    /// Rejects an excessive cookie header; malformed/duplicate notice data projects None.
    pub fn notice(&self, req: &Request) -> Result<Option<FlashNotice>> {
        if !self.owns(req) {
            return Ok(None);
        }
        Ok(self
            .cookie
            .consume(req.headers.get("cookie").unwrap_or_default())
            .map_err(Error::from_source)?
            .notice)
    }
    /// Issue a bounded public notice explicitly; callers append this header after domain success.
    /// # Errors
    /// Returns the existing public flash encoding/size error; no response is mutated.
    pub fn issue(&self, notice: &FlashNotice) -> Result<String> {
        self.cookie.issue(notice).map_err(Error::from_source)
    }
    /// Match only admitted GET pages, independently of caller query/aggregate input.
    fn owns(&self, req: &Request) -> bool {
        req.method.as_str() == "GET" && self.pages.contains(req.url.path())
    }
}
/// Select the independent flash contribution through ordinary server startup.
/// # Errors
/// Rejects duplicate configuration; build rejects missing/invalid config and duplicate service
/// contributions. No route, file, background task or settings prerequisite is installed.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("view_flash", None::<Config>, validate)?;
    app.on_config(configure)?;
    app.on_listen(listen)?;
    Ok(())
}
/// Missing selection data cannot silently choose a cookie scope.
fn validate(config: &Option<Config>) -> Result<()> {
    config
        .as_ref()
        .ok_or_else(|| Error::invalid("view_flash configuration required"))?
        .validate()
}
/// Construct a bounded service only after all typed config validation succeeds.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let config = start
        .config::<Option<Config>>()?
        .as_ref()
        .ok_or_else(|| Error::invalid("view_flash configuration required"))?;
    let cookie = FlashCookie::new(
        CookieScope::new(config.origin.clone(), config.base.clone()).map_err(Error::from_source)?,
    );
    let pages = config
        .pages
        .iter()
        .map(|page| {
            config
                .base
                .join(page.as_str())
                .map(|path| path.as_str().to_owned())
                .map_err(Error::from_source)
        })
        .collect::<Result<_>>()?;
    start.register(Arc::new(Service { cookie, pages }))?;
    Ok(Flow::Continue)
}
/// Register one scoped hook; preference integration is neither inferred nor required.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    start.on("response", consume)?;
    Ok(Flow::Continue)
}
/// Clear only rendered owned HTML responses, retaining notices across failed/non-HTML work.
async fn consume(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let service = inv.service::<Service>()?;
    if !service.owns(req)
        || res.mimetype() != Some("text/html")
        || res
            .status()
            .is_none_or(|status| !(200..300).contains(&status.code))
    {
        return Ok(Flow::Continue);
    }
    let consumed = service
        .cookie
        .consume(req.headers.get("cookie").unwrap_or_default())
        .map_err(Error::from_source)?;
    res.headers_mut()?.set("cache-control", "no-store")?;
    if let Some(clear) = consumed.clear_cookie {
        res.headers_mut()?.append("set-cookie", &clear)?;
    }
    Ok(Flow::Continue)
}
