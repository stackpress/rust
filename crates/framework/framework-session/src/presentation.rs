//! Native source view contributions, safe public projection and explicit SSR assets.
//!
//! Runtime route handlers run at priority zero. These ordinary -100 view handlers
//! cooperate with all other route listeners; G42 grants no route ownership.
use crate::{
    auth::bridge as b,
    views::{self, Page, PageProps},
};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus, leptos::render::LeptosRenderer,
};
use stackpress_lib::{
    Body, Error, Flow, Invocation, MethodPattern, Request, Response, Result, handler,
};
use stackpress_server::{RouteSpec, ServerInvocationExt, StartupContext};

/// Native host's explicit public view defaults and compiled asset identity.
/// This service is never serialized, and never opens a listener or acquires preferences.
#[derive(Clone)]
pub struct Presentation {
    /// Deliberately public brand/styles/form defaults. Auth/CSRF are projected per request.
    pub config: views::Config,
    /// Acknowledged saved theme. Unset remains unset and no-JS Light.
    pub theme: Option<String>,
    /// Explicit public locale token.
    pub locale: String,
    /// Optional checked public phrase dictionary; no configuration service is serialized.
    pub language: Option<stackpress_leptos::language::Language>,
    /// Checked renderer asset build/mount/module/stylesheet/bootstrap contract.
    pub assets: PageAssets,
}
impl Default for Presentation {
    /// Source generic brand fallbacks plus conventional app-owned assets; no files are served.
    fn default() -> Self {
        Self {
            config: views::Config {
                brand: Some("Stackpress".into()),
                logo: Some("/logo.png".into()),
                ..Default::default()
            },
            theme: None,
            locale: "en-US".into(),
            language: None,
            assets: PageAssets {
                build: "session-v1".into(),
                base: BasePath::default(),
                module: PublicLocation::new("/assets/session.js").expect("literal asset path"),
                stylesheet: None,
                theme_bootstrap: None,
            },
        }
    }
}
/// Project source-selected view data without serializing any native service/config object.
/// # Errors
/// Native Date/JSON failures propagate; IO/shared handles are never guessed into JSON.
pub fn load(req: &Request, res: &Response, inv: &Invocation<'_>) -> Result<PageProps> {
    let host = inv.service::<Presentation>().cloned().unwrap_or_default();
    let mut config = host.config;
    if let Some(auth) = b::optional_config(inv)? {
        config.base = auth.base.unwrap_or_else(|| "/auth".into());
        config.menu = serde_json::from_value(serde_json::to_value(auth.menu)?)?;
        config.password = Some(serde_json::from_value(serde_json::to_value(
            auth.password,
        )?)?);
    }
    if let Ok(csrf) = inv.service::<stackpress_framework_csrf::Service>() {
        config.csrf_name = csrf.name().into();
        config.csrf_token = res
            .data()
            .get(csrf.name())
            .and_then(|value| value.as_object())
            .and_then(|value| value.get("token"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .into();
    }
    Ok(PageProps {
        config,
        path: req.url.path().into(),
        method: req.method.as_str().into(),
        input: serde_json::to_value(b::input(req)?)?,
        results: crate::auth::events::json_result(res)?,
        code: res.status().map_or(0, |s| s.code),
        error: res.error().unwrap_or("").into(),
        errors: res.errors().to_json()?,
        theme: host.theme,
        language: host.language,
    })
}
/// Render a finite shared page through the existing checked view adapter, without a server.
/// # Errors
/// Requires initialized SSR and a Tokio runtime; preserves view asset/escaping errors.
pub async fn render(kind: Page, props: PageProps, host: &Presentation) -> Result<String> {
    let mut contract = PageContract::new(1, &host.assets.build).map_err(Error::from_source)?;
    contract.locale = host.locale.clone();
    contract.theme = props.theme.clone();
    let snapshot = PageSnapshot {
        revision: 1,
        build: host.assets.build.clone(),
        page: PageId::try_from(format!("session-{kind:?}").to_lowercase())
            .map_err(Error::from_source)?,
        location: props.path.clone(),
        locale: host.locale.clone(),
        theme: props.theme.clone(),
        status: PublicStatus::new(
            if props.code == 0 { 200 } else { props.code },
            (!props.error.is_empty()).then(|| props.error.clone()),
        )
        .map_err(Error::from_source)?,
        data: (kind, props),
    };
    let renderer = LeptosRenderer::new(
        contract,
        host.assets.clone(),
        |snapshot: &PageSnapshot<(Page, PageProps)>| {
            Ok(views::page(snapshot.data.0, snapshot.data.1.clone()))
        },
    )
    .map_err(Error::from_source)?;
    Ok(renderer
        .render(snapshot)
        .await
        .map_err(Error::from_source)?
        .into_html())
}
/// Ordinary view stage preserves previous bodies/downloads/redirects and source json opt-out.
async fn view(
    kind: Page,
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if !matches!(res.body(), Body::Empty | Body::Value(_))
        || res.headers().get("Location").is_some()
        || req.data.has("json")
    {
        return Ok(Flow::Continue);
    }
    let host = inv.service::<Presentation>().cloned().unwrap_or_default();
    let props = load(req, res, &inv)?;
    let code = res.status().cloned();
    res.html(render(kind, props, &host).await?)?;
    if let Some(code) = code {
        res.status_code(code.code, Some(code.message.into_owned()))?;
    }
    Ok(Flow::Continue)
}
/// Contribute a source view at -100 using existing priority/registration-order semantics.
fn route(start: &mut StartupContext, method: MethodPattern, path: &str, kind: Page) -> Result<()> {
    start.route_with(
        RouteSpec::new(method, path).priority(-100),
        handler(move |req, res, inv| Box::pin(view(kind, req, res, inv))),
    )?;
    Ok(())
}
/// Auth registrar's source GET view inventory then matching POST views.
pub(crate) fn auth_routes(start: &mut StartupContext, base: &str) -> Result<()> {
    let pages = [
        ("signup", Page::Signup),
        ("signin", Page::Signin),
        ("signin/2fa/:profile/:auth/:challenge", Page::TwoFactor),
        ("signin/link/:auth/:challenge", Page::Link),
        ("signin/email", Page::Email),
        ("signin/otp/:auth/:challenge", Page::Otp),
        ("signin/phone", Page::Phone),
        ("signin/username", Page::Username),
    ];
    for (path, page) in pages {
        route(start, MethodPattern::GET, &format!("{base}/{path}"), page)?;
    }
    for (path, page) in pages.into_iter().filter(|(_, page)| *page != Page::Signin) {
        route(start, MethodPattern::POST, &format!("{base}/{path}"), page)?;
    }
    // Source auth/plugin.ts:163–167 registers this view-only fallback last.
    // A concrete view's raw HTML passes through; an extension method can use the chooser.
    route(
        start,
        MethodPattern::POST,
        &format!("{base}/signin/:type"),
        Page::Signin,
    )?;
    Ok(())
}
/// Identity registrar's source GET views followed by the five matching POST views.
pub(crate) fn account_routes(start: &mut StartupContext, base: &str) -> Result<()> {
    let pages = [
        ("", Page::Account),
        ("/update", Page::Update),
        ("/security", Page::Security),
        ("/security/export", Page::Export),
        ("/security/remove", Page::Remove),
        ("/security/password", Page::Password),
        ("/security/2fa", Page::Enroll),
        ("/security/2fa/remove", Page::RemoveFactor),
    ];
    for (path, page) in pages {
        route(
            start,
            MethodPattern::GET,
            &format!("{base}/account{path}"),
            page,
        )?;
    }
    for (path, page) in pages.into_iter().filter(|(_, page)| {
        matches!(
            page,
            Page::Update | Page::Remove | Page::Password | Page::Enroll | Page::RemoveFactor
        )
    }) {
        route(
            start,
            MethodPattern::POST,
            &format!("{base}/account{path}"),
            page,
        )?;
    }
    Ok(())
}
