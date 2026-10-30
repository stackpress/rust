//! Native account loaders and processes, retaining source checks and sequential effects.
//!
//! Select identity plus auth to install routes. SQL events remain explicit application
//! contributions: direct handlers and downstream plugins use the same nested event chain.

pub mod events;
mod export;
mod password;
mod profile;
mod removal;
mod routes;
mod totp;
use crate::{
    Service,
    auth::bridge as b,
    columns::{Record, Value},
};
pub use export::{csv, export};
pub use password::{assert_password, password};
pub use profile::{account, load_account_profile, update, update_auth};
pub use removal::remove;
pub(crate) use routes::routes;
use stackpress_lib::{Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;
pub use totp::{enroll, provisioning_uri, qr_data_url, remove_factor};

/// Load the same cookie JWT; no database-active or global permission check is added.
async fn identity(
    req: &Request,
    res: &mut Response,
    inv: &Invocation<'_>,
) -> Result<Option<Record>> {
    let session = inv.service::<Service>()?.load_request(req);
    let Some(data) = session.data().await else {
        res.status_code(401, Some("Unauthorized".into()))?;
        return Ok(None);
    };
    Ok(Some(
        data.clone()
            .into_iter()
            .map(|(k, v)| (k, Value::from(v)))
            .collect(),
    ))
}
/// Source ctx.resolve borrows the caller response only where explicitly requested.
async fn resolve_into(
    inv: &Invocation<'_>,
    event: &str,
    data: Record,
    res: &mut Response,
) -> Result<()> {
    let mut req = Request::new();
    for (key, value) in data {
        b::set(&mut req, &key, value)?;
    }
    inv.emit(event, &mut req, res).await?;
    Ok(())
}
/// Source data.id is forwarded without adding a claim assertion.
fn id(data: &Record) -> Value {
    data.get("id").cloned().unwrap_or("undefined".into())
}
/// Share the safe auth section projection, without leaking native environment settings.
fn props(res: &mut Response, inv: &Invocation<'_>) -> Result<()> {
    crate::auth::pages::props(res, &b::config(inv)?)
}
/// Source config.path default differs from the views' truthy fallback.
fn base(inv: &Invocation<'_>) -> Result<String> {
    Ok(b::config(inv)?.base.unwrap_or_else(|| "/auth".into()))
}
/// Preserve the original flash JSON wire shape and optional close duration.
fn flash(res: &mut Response, message: &str, close: Option<u64>) -> Result<()> {
    let mut value = serde_json::json!({"type":"success","message":message});
    if let Some(close) = close {
        value["close"] = close.into();
    }
    res.session_mut()?.set("flash", value.to_string(), None)?;
    Ok(())
}
