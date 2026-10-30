//! Username/phone/email password submission and signed-in GET shortcuts.

use super::{b, guest, props};
use crate::{Service, auth::events::normalize_phone};
use stackpress_framework_csrf::Service as Csrf;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

/// Username process sets password=true regardless of submitted bypass flags.
/// # Errors
/// Propagates native dependency/dispatch/data failures.
pub async fn username(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    password_page(req, res, inv, "username").await
}
/// Phone process normalizes input after forcing the password flag and method type.
/// # Errors
/// Propagates native dependency/dispatch/data failures.
pub async fn phone(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    password_page(req, res, inv, "phone").await
}
/// Common source page sequence, preserving the separate guest and POST branches.
async fn password_page(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
    kind: &str,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let csrf = inv.service::<Csrf>()?;
    props(res, &b::config(&inv)?)?;
    csrf.generate(res)?;
    if req.method.as_str() == "POST" {
        if !csrf.valid(req, res)? {
            return Ok(Flow::Continue);
        }
        // POST does not consult guest(); only the source incoming CSRF pair gates submission.
        submit(req, res, &inv, kind).await?;
    } else if !guest(req, &inv).await? {
        csrf.clear(req, res)?;
        res.redirect(&b::text(req, "redirect_uri", "/")?)?;
    }
    Ok(Flow::Continue)
}
/// Direct source PasswordSignin helper for email. The caller owns CSRF validation.
/// # Errors
/// Propagates nested event/native errors; it does not perform another page guard/check.
pub async fn password_signin(
    req: &mut Request,
    res: &mut Response,
    inv: &Invocation<'_>,
) -> Result<()> {
    submit(req, res, inv, "email").await
}
/// Force source flags, emit the same event, and clear/redirect only after code 200.
async fn submit(
    req: &mut Request,
    res: &mut Response,
    inv: &Invocation<'_>,
    kind: &str,
) -> Result<()> {
    // The page forces password/type but deliberately leaves the submitted 2fa flag untouched.
    b::set(req, "password", true.into())?;
    b::set(req, "type", kind.into())?;
    if kind == "phone" {
        let phone = b::get(req, "phone", "".into())?;
        let text = phone
            .as_str()
            .ok_or_else(|| stackpress_lib::Error::invalid("phone.trim is not a function"))?;
        if let Some(phone) = normalize_phone(Some(text)) {
            b::set(req, "phone", phone.into())?;
        } else {
            req.data.remove("phone")?;
        }
    }
    inv.emit("auth-signin", req, res).await?;
    // A child 2FA redirect/error is preserved; only code 200 clears CSRF and redirects home.
    if b::ok(res) {
        inv.service::<Csrf>()?.clear(req, res)?;
        res.redirect(&b::text(req, "redirect_uri", "/")?)?;
    }
    Ok(())
}
