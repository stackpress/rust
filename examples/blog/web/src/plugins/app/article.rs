//! Source detail lookup and separately resolved comments; no added publication filter.
use serde_json::json;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
/// Resolve exactly the slug query and, on success, its comment query into fresh responses.
pub async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let slug = req
        .data
        .get("slug")
        .map(|value| value.to_json())
        .transpose()?
        .unwrap_or_default();
    let mut search = Request::builder()
        .data(json!({"eq":{"slug":slug}}))
        .build()?;
    let mut articles = Response::new();
    inv.emit("article-search", &mut search, &mut articles)
        .await?;
    if articles.status().map(|status| status.code) != Some(200) {
        return Ok(Flow::Continue);
    }
    let rows = stackpress_framework_session::auth::events::json_result(&articles)?;
    let Some(article) = rows.as_array().and_then(|rows| rows.first()) else {
        return Ok(Flow::Continue);
    };
    let mut article = article.clone();
    let mut query = Request::builder()
        .data(json!({"eq":{"articleId":article["id"]}}))
        .build()?;
    let mut comments = Response::new();
    inv.emit("comment-search", &mut query, &mut comments)
        .await?;
    article["comments"] = stackpress_framework_session::auth::events::json_result(&comments)?;
    if article["comments"].is_null() {
        article["comments"] = json!([]);
    }
    res.results(&article)?;
    Ok(Flow::Continue)
}
/// Use the same source-selected article view in SSR and both browser renderers.
pub async fn view(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    crate::render::page(blog_shared::Kind::Article, req, res, inv).await
}
