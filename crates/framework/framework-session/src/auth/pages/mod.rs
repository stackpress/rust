//! Native auth process handlers and source-selected public presentation props.
//!
//! Every process is independently reusable with the ordinary handler signature. Normal
//! returns are Continue; a source conditional return never reserves a route or aborts
//! another registered handler. Dependency reads occur only at the corresponding source
//! branch. No global CSRF, Origin, default-deny, redirect allowlist or expiry is added.

mod challenge;
mod email;
mod password;
use super::{AuthConfig, bridge as b};
use crate::{Service, columns::Value};
pub use challenge::{magic_link, otp, two_factor};
pub use email::{email, magic_link_signin, otp_signin};
pub use password::{password_signin, phone, username};
use serde_json::json;
use stackpress_framework_csrf::Service as Csrf;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

/// Project only the source auth fields; full page projection belongs to presentation.
pub(crate) fn props(res: &mut Response, config: &AuthConfig) -> Result<()> {
    // Preserve response admission before converting the selected public configuration fields.
    res.data_mut()?.set(
        "auth",
        stackpress_lib::Value::try_from(json!({
            "base": config.base(),
            "roles": config.roles,
            "menu": config.menu,
            "password": config.password,
        }))?,
    )?;
    Ok(())
}
/// Ask the exact cookie-backed Session whether this request is a guest.
pub(crate) async fn guest(req: &Request, inv: &Invocation<'_>) -> Result<bool> {
    Ok(inv.service::<Service>()?.load_request(req).guest().await)
}
/// Source chooser has no form/CSRF generation or POST process.
/// # Errors
/// Preserves late config/service and response mutation failures.
pub async fn signin(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let csrf = inv.service::<Csrf>()?;
    props(res, &b::config(&inv)?)?;
    if !guest(req, &inv).await? {
        csrf.clear(req, res)?;
        res.redirect(&b::text(req, "redirect_uri", "/")?)?;
    }
    Ok(Flow::Continue)
}
/// Source signup generates CSRF before validating POST, then reuses auth-signup.
/// # Errors
/// Preserves dependency/dispatch/data errors; ordinary validation stays in the response.
pub async fn signup(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let csrf = inv.service::<Csrf>()?;
    let config = b::config(&inv)?;
    props(res, &config)?;
    csrf.generate(res)?;
    if req.method.as_str() == "POST" {
        if !csrf.valid(req, res)? {
            return Ok(Flow::Continue);
        }
        inv.emit("auth-signup", req, res).await?;
        if !b::ok(res) {
            return Ok(Flow::Continue);
        }
        csrf.clear(req, res)?;
        // Source config.path defaults on missing, not on an explicitly empty base.
        let base = config.base.as_deref().unwrap_or("/auth");
        res.redirect(&b::text(req, "redirect_uri", &format!("{base}/signin"))?)?;
    } else if !guest(req, &inv).await? {
        csrf.clear(req, res)?;
        res.redirect(&b::text(req, "redirect_uri", "/")?)?;
    }
    Ok(Flow::Continue)
}
/// Source GET signout resolves into the existing pair, then applies redirect precedence.
/// # Errors
/// Propagates nested handler and response failures; no CSRF or current-session check occurs.
pub async fn signout(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.emit("auth-signout", req, res).await?;
    if b::ok(res) {
        let config = b::config(&inv)?;
        res.redirect(&b::text(
            req,
            "redirect_uri",
            config.redirect.as_deref().unwrap_or("/"),
        )?)?;
    }
    Ok(Flow::Continue)
}
/// Construct the shared final URL without adding a fallback where the source has none.
pub(crate) fn handoff(
    config: &AuthConfig,
    profile: &Value,
    auth: &Value,
    challenge: &str,
    redirect: &str,
) -> String {
    format!(
        "{}/signin/2fa/{}/{}/{}?redirect_uri={}",
        config.literal_base(),
        b::encode(&crate::columns::string(profile)),
        b::encode(&crate::columns::string(auth)),
        b::encode(challenge),
        b::encode(redirect)
    )
}
