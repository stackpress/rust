//! G52's real published-only filter is an app operation, not a framework/store behavior change.
use stackpress_lib::{DataPath, Flow, Invocation, Request, Response, Result};
/// Query the actual generated article event, preserving caller-supplied sort configuration.
pub async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // G52 overrides a caller's status filter on home while leaving the framework query intact.
    req.data
        .set_path(&DataPath::dot("eq.status")?, "PUBLISHED")?;
    if !req.data.has("sort") {
        // A supplied order wins exactly as in the original page; only missing sort defaults.
        req.data.set(
            "sort",
            stackpress_lib::Value::try_from(serde_json::json!({"published":"desc"}))?,
        )?;
    }
    inv.emit("article-search", req, res).await?;
    Ok(Flow::Continue)
}
/// Shared home rendering uses the source noview/body/redirect semantics.
pub async fn view(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    crate::render::page(blog_shared::Kind::Home, req, res, inv).await
}
