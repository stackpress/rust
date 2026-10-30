//! Application-owned adapters for the actual generated Note admin routes and SQL operations.
use stackpress_lib::{Flow, Invocation, Request, Response, Result};

/// The application renders its generated admin search from actual stored rows.
pub async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let _ = inv.emit("note.search", req, res).await?;
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}

/// The finite admin endpoint dispatches the route's selected generated operation.
pub async fn api(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let operation = req
        .view()
        .url
        .path()
        .rsplit('/')
        .next()
        .unwrap_or("search")
        .to_owned();
    let _ = inv.emit(&format!("note.{operation}"), req, res).await?;
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}

pub mod transfer {
    //! Finite application transfer adapters; all data access uses generated operation policy.
    use super::*;

    /// Export the existing public Note search result.
    pub async fn export(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        let _ = inv.emit("note.search", req, res).await?;
        res.status_code(200, None)?;
        Ok(Flow::Continue)
    }

    /// The fixture's explicit import action reuses the actual generated create operation.
    pub async fn import(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        let _ = inv.emit("note.create", req, res).await?;
        res.status_code(200, None)?;
        Ok(Flow::Continue)
    }
}
