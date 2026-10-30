//! Native source view contribution at priority -100; pure components own no event host.
use super::{ApiOauthHead, ApiOauthPage, Consent};
use leptos::prelude::*;
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};
use stackpress_server::{RouteSpec, ServerBuilder, ServerInvocationExt};

/// App-owned presentation props; absence retains the source default phrase/assets behavior.
#[derive(Clone)]
pub struct Presentation {
    /// Existing literal phrase provider.
    pub language: stackpress_leptos::language::Language,
    /// Ordered source additional styles.
    pub styles: Vec<String>,
    /// Acknowledged explicit theme; None uses the shared one-shot theme sampler.
    pub theme: Option<stackpress_framework_view::preferences::ThemePreference>,
}
impl Default for Presentation {
    /// Source English phrases, no extra styles, no saved theme.
    fn default() -> Self {
        let page = Consent::new(
            serde_json::json!({}),
            serde_json::json!({}),
            serde_json::json!({}),
        );
        Self {
            language: page.language,
            styles: Vec::new(),
            theme: None,
        }
    }
}
/// Direct optional view contribution; the API plugin applies its original endpoint activation gate.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.route_with(
        RouteSpec::new(stackpress_lib::MethodPattern::Any, "/auth/oauth").priority(-100),
        oauth,
    )?;
    Ok(())
}
/// Render only a successful nonredirected source OAuth result; leave earlier failures untouched.
pub async fn oauth(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Only the process's selected data result enters this view. A developer's existing
    // text/byte response has already completed that process and remains its output.
    if res.status().map(|status| status.code) != Some(200)
        || res.redirected()
        || matches!(res.body(), Body::Text(_) | Body::Bytes(_))
        || !res.data().has("api")
    {
        return Ok(Flow::Continue);
    }
    let presentation = presentation(req, &inv)?;
    let mut page = Consent::new(
        res.data().to_json()?,
        req.data.to_json()?,
        res.result()
            .map(|value| value.to_json())
            .transpose()?
            .unwrap_or(serde_json::Value::Null),
    );
    page.language = presentation.language;
    page.styles = presentation.styles;
    page.theme = presentation.theme;
    res.html(document(page).map_err(stackpress_lib::Error::from_source)?)?;
    Ok(Flow::Continue)
}
/// Synchronous owner cleanup precedes returning owned bytes to the Send server future.
pub fn document(page: Consent) -> Result<String> {
    let owner = Owner::new();
    let result=owner.with(|| {
        let head=view!{<ApiOauthHead page=page.clone()/>}.to_html();
        let body=view!{<ApiOauthPage page=page.clone()/>}.to_html();
        let seed=super::seed(&page)?;
        let theme=page.theme.map(|theme|theme.as_str()).unwrap_or("light");
        let marker=if page.theme.is_none(){" data-theme-unset"}else{""};
        let bootstrap=if page.theme.is_none(){stackpress_framework_view::theme::bootstrap_tag(&Default::default(),Some(&stackpress_framework_view::theme::bootstrap_location().map_err(stackpress_lib::Error::from_source)?),None).map_err(stackpress_lib::Error::from_source)?}else{String::new()};
        Ok(format!("<!doctype html><html lang=\"{}\" data-theme=\"{}\"{marker}><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">{bootstrap}{head}</head><body><div id=\"app\">{body}</div><script type=\"application/json\" id=\"api-consent-seed\">{seed}</script><script type=\"module\" src=\"/assets/api-checkpoint.js\"></script></body></html>",page.language.locale(),theme))
    });
    owner.cleanup();
    result
}

/// Consume the existing app-selected settings mechanism; API owns no preference state.
pub(crate) fn presentation(req: &Request, inv: &Invocation<'_>) -> Result<Presentation> {
    let mut presentation = inv.service::<Presentation>().cloned().unwrap_or_default();
    if let Ok(settings) = inv.service::<stackpress_framework_view::preferences::SettingsAction>() {
        let selected = settings
            .select(req.url.path(), req.headers.get("cookie").unwrap_or(""))
            .map_err(stackpress_lib::Error::from_source)?;
        presentation.theme = selected.theme;
    }
    Ok(presentation)
}
