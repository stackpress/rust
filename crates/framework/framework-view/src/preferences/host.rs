//! Opt-in native HTTP presentation service and ordinary function registration.
//! Config validates before service creation; Route exposes two fixed POST paths and an
//! explicitly selected immutable asset. Importing/enabling the host feature has no effects.
use super::{CookieScope, LocaleRegistry, MAX_PREFERENCE_BYTES, PreferenceError, SettingsAction};
use crate::{BasePath, PublicLocation, theme};
use serde::{Deserialize, Serialize};
use stackpress_lib::{Body, Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::{collections::BTreeMap, sync::Arc};

/// Typed trusted settings under the `view_preferences` configuration section.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Canonical trusted HTTP(S) origin; incoming Host/forwarded headers cannot replace it.
    pub origin: String,
    /// Checked application mount and cookie scope.
    pub base: BasePath,
    /// Canonical compiled locales.
    pub locales: Vec<String>,
    /// Registered fallback locale.
    pub default_locale: String,
    /// Direct aliases to canonical locales.
    pub aliases: BTreeMap<String, String>,
    /// Exact application-relative write path, without captures/query.
    pub settings_path: PublicLocation,
    /// Distinct exact application-relative confirmation path.
    pub confirmation_path: PublicLocation,
    /// Explicit asset contribution; false leaves asset hosting to the application bundle.
    #[serde(default)]
    pub serve_theme_bootstrap: bool,
}
impl Config {
    /// Rebuild immutable locale authority from decoded trusted configuration.
    fn registry(&self) -> Result<LocaleRegistry> {
        // Reconstruct bounded locale authority from trusted typed configuration only.
        LocaleRegistry::new(
            self.locales.clone(),
            self.default_locale.clone(),
            self.aliases.clone(),
        )
        .map_err(Error::from_source)
    }
    /// Validate all identities before startup may publish a service or route.
    fn validate(&self) -> Result<()> {
        // Complete all validation before Config can publish a service or Route can run.
        self.registry()?;
        CookieScope::new(self.origin.clone(), self.base.clone()).map_err(Error::from_source)?;
        super::super::adapters::web::host::literal_path(&self.base, &self.settings_path)?;
        super::super::adapters::web::host::literal_path(&self.base, &self.confirmation_path)?;
        // Both handlers are POST; a shared path would apply write and confirmation together.
        if self.settings_path == self.confirmation_path {
            return Err(Error::invalid(
                "preference write and confirmation paths must differ",
            ));
        }
        Ok(())
    }
}

/// Fresh application-owned session/CSRF admission, independent of protected-operation policy.
/// A selected preferences integration requires this service even in a sessionless host.
#[derive(Clone, Copy)]
pub struct Admission(fn(&Request) -> bool);
impl Admission {
    /// Use fresh authoritative request/session facts; false denies without cookies.
    pub fn new(check: fn(&Request) -> bool) -> Self {
        // Store host code, not a grant decoded from a request or public page snapshot.
        Self(check)
    }
    /// Deliberately declare a sessionless presentation endpoint; exact Origin still applies.
    /// This never supplies admission for protected domain operations.
    pub fn sessionless() -> Self {
        // Deliberate sessionless admission still leaves the action's exact Origin check intact.
        Self(|_| true)
    }
}

/// Immutable native presentation integration, borrowed through ServerInvocationExt.
pub struct Service {
    /// Validated trusted endpoint/mount settings.
    config: Config,
    /// Pure shared settings mechanism; no filesystem or background work.
    action: SettingsAction,
    /// Explicit host admission evaluated again for each POST and confirmation.
    admission: Admission,
}
impl Service {
    /// Borrow pure selection/acknowledgement APIs for an explicit public page projection.
    pub fn action(&self) -> &SettingsAction {
        // Projection borrows pure selection/acknowledgement logic without mutating a response.
        &self.action
    }
}

/// Register through ServerBuilder's existing phases; importing/enabling features does nothing.
///
/// Supply `view_preferences` via ConfigLayers and an explicit Admission service in Config.
/// Application composites may call this function directly; do not select nested plugins.
/// # Errors
/// Rejects duplicate config contributions; build additionally rejects missing/invalid config,
/// missing admission, duplicate services and conflicting endpoint paths before publication.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    // Claim data separately from executable selection; missing config has no host defaults.
    app.config_section("view_preferences", None::<Config>, validate)?;
    // Services are constructed before route exposure; the existing builder owns ordering.
    app.on_config(configure)?;
    app.on_route(routes)?;
    Ok(())
}
/// Missing configuration cannot silently choose a host origin or persistence scope.
fn validate(config: &Option<Config>) -> Result<()> {
    // A selected integration cannot silently choose an origin, mount or endpoint.
    config
        .as_ref()
        .ok_or_else(|| Error::invalid("view_preferences configuration required"))?
        .validate()
}
/// Construct only after typed validation and explicit host admission exist.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    // Typed validation has completed; retain its checked settings for immutable routing.
    let config = start
        .config::<Option<Config>>()?
        .as_ref()
        .ok_or_else(|| Error::invalid("view_preferences configuration required"))?
        .clone();
    // Absence is a startup error, never an implicit allow for preference writes.
    let admission = *start.service::<Admission>()?;
    let action = SettingsAction::new(
        config.registry()?,
        CookieScope::new(config.origin.clone(), config.base.clone()).map_err(Error::from_source)?,
    );
    // Publish only the fully constructed mechanism; failed startup returns no Server.
    start.register(Arc::new(Service {
        config,
        action,
        admission,
    }))?;
    Ok(Flow::Continue)
}
/// Route contribution follows Config, so no callback observes a partially built service.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    // Own route inputs before mutating startup; no per-request state enters the service.
    let config = start.service::<Service>()?.config.clone();
    start.post(
        config
            .base
            .join(config.settings_path.as_str())
            .map_err(Error::from_source)?
            .as_str(),
        save,
    )?;
    start.post(
        config
            .base
            .join(config.confirmation_path.as_str())
            .map_err(Error::from_source)?
            .as_str(),
        confirm,
    )?;
    // Application bundles may own the asset instead; false contributes no GET handler.
    if config.serve_theme_bootstrap {
        let path = config
            .base
            .join(
                theme::bootstrap_location()
                    .map_err(Error::from_source)?
                    .as_str(),
            )
            .map_err(Error::from_source)?;
        start.get(path.as_str(), bootstrap)?;
    }
    Ok(Flow::Continue)
}
/// Keep write/confirmation choice fixed by registration, never by a caller path suffix.
async fn save(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Registration fixes this handler as the sole cookie-writing variant.
    apply(req, res, inv, false)
}
/// Confirmation is read-only and rechecks the same current host admission.
async fn confirm(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Reuse admission/body limits while selecting the read-only cookie-readback path.
    apply(req, res, inv, true)
}
/// Bound and admit input before any outgoing cookie is staged.
fn apply(req: &Request, res: &mut Response, inv: Invocation<'_>, confirm: bool) -> Result<Flow> {
    // Resolve immutable host authority before inspecting caller-controlled body data.
    let service = inv.service::<Service>()?;
    // Re-evaluate current session/CSRF facts on every call; denial stages no cookies.
    if !(service.admission.0)(req) {
        return crate::adapters::web::host::denied(res, 403);
    }
    // Accept only the existing buffered text/byte forms; other forms become invalid empty JSON.
    let bytes = match req.body() {
        Body::Bytes(value) => value.as_slice(),
        Body::Text(value) => value.as_bytes(),
        _ => &[],
    };
    // Bound before UTF-8/JSON work; the limit status remains the existing public 413.
    if bytes.len() > MAX_PREFERENCE_BYTES {
        return crate::adapters::web::host::denied(res, 413);
    }
    // Malformed UTF-8 receives the same public input error, without reflecting raw bytes.
    let Ok(text) = std::str::from_utf8(bytes) else {
        return crate::adapters::web::host::denied(res, 400);
    };
    // Keep write/readback selection internal; a caller path suffix cannot choose it.
    let result = if confirm {
        service.action.confirm(
            req.method.as_str(),
            req.headers.get("origin"),
            text,
            req.headers.get("cookie").unwrap_or_default(),
        )
    } else {
        service
            .action
            .apply(req.method.as_str(), req.headers.get("origin"), text)
    };
    match result {
        Ok(reply) => {
            // The unchanged public envelope excludes native-only cookie headers.
            res.json(&reply)?;
            // Preserve each Set-Cookie independently; confirmation supplies an empty list.
            for cookie in &reply.cookies {
                res.headers_mut()?.append("set-cookie", cookie)?;
            }
            Ok(Flow::Continue)
        }
        // Retain error precedence/status while hiding internal diagnostics from the envelope.
        Err(error) => crate::adapters::web::host::denied(
            res,
            match error {
                PreferenceError::Origin => 403,
                PreferenceError::Limit => 413,
                _ => 400,
            },
        ),
    }
}
/// Tiny cacheable same-origin bytes; no reads, network acquisition or persistence occur.
async fn bootstrap(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // Serve compiled bytes directly: no file read, dynamic fetch or preference write occurs.
    res.set(
        "text/javascript; charset=utf-8",
        Body::Text(theme::BOOTSTRAP_SOURCE.into()),
    )?;
    // The content-addressed basename permits immutable caching at the exact mounted path.
    res.headers_mut()?
        .set("cache-control", "public, max-age=31536000, immutable")?;
    res.headers_mut()?
        .set("x-content-type-options", "nosniff")?;
    Ok(Flow::Continue)
}
