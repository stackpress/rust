//! Native source values enter a checked public snapshot before synchronous Leptos execution.
use blog_shared::{Kind, Page};
use leptos::prelude::*;
use serde_json::{Value, json};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageRenderer, PageSnapshot, PublicLocation,
    PublicStatus, ViewError,
};
use stackpress_lib::{Body, Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

/// Build the deliberately public page data; neither config services nor model descriptors serialize.
async fn project(kind: Kind, req: &Request, res: &Response, inv: &Invocation<'_>) -> Result<Page> {
    let mut identity = Response::new();
    let mut request = Request::builder().session(req.view().session).build()?;
    inv.emit("me", &mut request, &mut identity).await?;
    let language = inv.service::<stackpress_framework::language::Service>()?;
    let selected = language.load(req, &language.config.locale);
    let translations = selected
        .translations()
        .cloned()
        .map(Value::Object)
        .unwrap_or(json!({}));
    let config = inv.service::<stackpress_framework::view::Settings>()?;
    let code = res.status().map_or(200, |status| status.code);
    let data = stackpress_framework_session::auth::events::json_result(res)?;
    let title = match kind {
        Kind::Home => "The Blog".into(),
        Kind::Article => data["title"].as_str().unwrap_or_default().into(),
        Kind::Error => {
            if code == 404 {
                "Not Found".into()
            } else {
                "Oops...".into()
            }
        }
    };
    let brand = inv.config::<crate::config::Brand>()?.public();
    Ok(Page {
        kind,
        title,
        data,
        code,
        error: res.error().unwrap_or_default().into(),
        path: req.url.path().into(),
        mode: config.mode.clone(),
        stack: res
            .data()
            .get("stack")
            .map(|value| value.to_json())
            .transpose()?
            .unwrap_or(json!([])),
        session: stackpress_framework_session::auth::events::json_result(&identity)?,
        locale: selected.locale.clone(),
        translations,
        brand,
        styles: config
            .config
            .get("styles")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        theme: None,
    })
}

/// Frame one owned snapshot; the renderer disposes its Owner before returning a Send future.
/// # Errors
/// Retains checked snapshot/asset/SSR and shared source formatting failures.
pub async fn document(page: Page) -> Result<String> {
    let mut contract = PageContract::new(1, "blog-v1").map_err(Error::from_source)?;
    // Source cookie/dictionary names retain underscores; the document contract uses BCP47.
    let document_locale = page.locale.replace('_', "-");
    contract.locale = document_locale.clone();
    contract.theme = page.theme.clone();
    let assets = PageAssets {
        build: "blog-v1".into(),
        base: BasePath::default(),
        module: PublicLocation::new("/assets/blog.js").map_err(Error::from_source)?,
        stylesheet: Some(PublicLocation::new("/styles/global.css").map_err(Error::from_source)?),
        theme_bootstrap: Some(
            stackpress_framework_view::theme::bootstrap_location().map_err(Error::from_source)?,
        ),
    };
    let snapshot = PageSnapshot {
        revision: 1,
        build: "blog-v1".into(),
        page: PageId::try_from(
            match page.kind {
                Kind::Home => "blog-home",
                Kind::Article => "blog-article",
                Kind::Error => "blog-error",
            }
            .to_owned(),
        )
        .map_err(Error::from_source)?,
        location: page.path.clone(),
        locale: document_locale,
        theme: page.theme.clone(),
        status: PublicStatus::new(
            page.code,
            (!page.error.is_empty()).then(|| page.error.clone()),
        )
        .map_err(Error::from_source)?,
        data: page,
    };
    let renderer = stackpress_framework_view::leptos::render::LeptosRenderer::new(
        contract,
        assets,
        |snapshot: &PageSnapshot<Page>| {
            let body =
                blog_shared::views::page(snapshot.data.clone()).map_err(ViewError::from_source)?;
            Ok((blog_shared::views::head(&snapshot.data), body).into_any())
        },
    )
    .map_err(Error::from_source)?;
    Ok(renderer
        .render(snapshot)
        .await
        .map_err(Error::from_source)?
        .into_html())
}

/// Apply the original view's redirect/json/string-body skips before native projection/rendering.
pub async fn page(
    kind: Kind,
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let settings = inv.service::<stackpress_framework::view::Settings>()?;
    let flag = settings
        .config
        .get("noview")
        .and_then(Value::as_str)
        .unwrap_or("json");
    if res.redirected()
        || req.data.has(flag)
        || (kind != Kind::Error && matches!(res.body(), Body::Text(_)))
    {
        return Ok(Flow::Continue);
    }
    if kind == Kind::Article && res.result().is_none() {
        return Ok(Flow::Continue);
    }
    let value = project(kind, req, res, &inv).await?;
    let code = value.code;
    res.html(document(value).await?)?;
    res.status_code(code, None)?;
    Ok(Flow::Continue)
}
