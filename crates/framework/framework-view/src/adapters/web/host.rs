//! Shared native response/path boundaries, not a second plugin or route system.
use crate::{BasePath, PublicLocation};
use stackpress_lib::{Error, Result};
#[cfg(feature = "web-preferences")]
use stackpress_lib::{Flow, Response};
/// Restrict contribution paths to exact literal routes, then check their trusted mount.
pub(crate) fn literal_path(base: &BasePath, path: &PublicLocation) -> Result<()> {
    BasePath::new(path.as_str()).map_err(Error::from_source)?;
    base.join(path.as_str()).map_err(Error::from_source)?;
    Ok(())
}
/// Preserve the existing public failure envelope and status without private causes.
#[cfg(feature = "web-preferences")]
pub(crate) fn denied(res: &mut Response, status: u16) -> Result<Flow> {
    res.json(&serde_json::json!({"error":"Action unavailable"}))?;
    res.status_code(status, None)?;
    Ok(Flow::Continue)
}
