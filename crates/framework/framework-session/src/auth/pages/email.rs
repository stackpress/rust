//! Email password, OTP issuance and magic-link issuance, including source no-op points.

use super::{b, guest, password_signin, props};
use crate::{
    Service,
    auth::{events, templates},
    columns::{self, Record, Value},
};
use serde_json::json;
use stackpress_framework_csrf::Service as Csrf;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

/// Source email process generates CSRF and chooses pass/otp/magic only on POST.
/// # Errors
/// Propagates native host/provider/dispatch failures without replacing source field errors.
pub async fn email(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let csrf = inv.service::<Csrf>()?;
    props(res, &b::config(&inv)?)?;
    // Outgoing generation precedes validation of the independent incoming cookie/form pair.
    csrf.generate(res)?;
    if req.method.as_str() == "POST" {
        if !csrf.valid(req, res)? {
            return Ok(Flow::Continue);
        }
        match b::text(req, "auth", "pass")?.as_str() {
            "pass" => password_signin(req, res, &inv).await?,
            "otp" => otp_signin(req, res, &inv).await?,
            "magic" => magic_link_signin(req, res, &inv).await?,
            _ => b::error(
                res,
                "Invalid Parameters",
                json!({"auth":"Invalid sign-in method"}),
            )?,
        }
    } else if !guest(req, &inv).await? {
        csrf.clear(req, res)?;
        res.redirect(&b::text(req, "redirect_uri", "/")?)?;
    }
    Ok(Flow::Continue)
}
/// Source exported OTPSignin helper. Caller chooses any CSRF/session/page checks.
/// # Errors
/// Propagates native errors; validation/child public failures remain in res.
pub async fn otp_signin(req: &mut Request, res: &mut Response, inv: &Invocation<'_>) -> Result<()> {
    issue(req, res, inv, true).await
}
/// Source exported MagicLinkSignin helper; success sends mail and leaves the send response.
/// # Errors
/// Propagates native errors; adds no success redirect, CSRF clearing or session creation.
pub async fn magic_link_signin(
    req: &mut Request,
    res: &mut Response,
    inv: &Invocation<'_>,
) -> Result<()> {
    issue(req, res, inv, false).await
}
/// Rotate, reload, derive and send in exact source order; missing mail config is checked late.
async fn issue(
    req: &mut Request,
    res: &mut Response,
    inv: &Invocation<'_>,
    otp: bool,
) -> Result<()> {
    let email = b::get(req, "email", "".into())?;
    let email = email
        .as_str()
        .ok_or_else(|| stackpress_lib::Error::invalid("email.trim is not a function"))?
        .to_owned();
    let email = b::trim(&email).to_owned();
    if email.is_empty() {
        b::error(
            res,
            "Invalid Parameters",
            json!({"email":"Email Address is required"}),
        )?;
        return Ok(());
    }
    // Look up the email credential and joined profile before issuing any new challenge.
    let search = b::resolve(
        inv,
        "auth-search",
        Record::from_iter([
            ("columns".into(), Value::from(json!(["*", "profile.*"]))),
            (
                "eq".into(),
                Value::from(json!({"type":"email","token":email})),
            ),
        ]),
    )
    .await?;
    if !b::ok(&search) {
        res.apply_envelope(search.to_envelope())?;
        return Ok(());
    }
    let rows = b::result(&search)?;
    let Some(current) = rows.as_array().and_then(|rows| rows.first()) else {
        b::error(
            res,
            "Invalid Parameters",
            json!({"email":"Account not found"}),
        )?;
        return Ok(());
    };
    let config = b::config(inv)?;
    let redirect = b::text(req, "redirect_uri", "/")?;
    let environment = b::environment(inv);
    let id = current.get("id").cloned().unwrap_or(Value::Null);
    let mut rotate = Request::new();
    b::set(&mut rotate, "id", id.clone())?;
    // Use the original response for source ctx.resolve(..., res), including partial results.
    b::set(
        &mut rotate,
        "consumed",
        columns::date((environment.clock.now() * 1000.0) as i64),
    )?;
    inv.emit("auth-update", &mut rotate, res).await?;
    if !b::ok(res) {
        return Ok(());
    }
    // Build the proof from the reloaded persisted timestamp, not the update return map.
    let refreshed = b::resolve(
        inv,
        "auth-search",
        Record::from_iter([
            ("columns".into(), Value::from(json!(["*", "profile.*"]))),
            (
                "eq".into(),
                Value::Object(Record::from_iter([("id".into(), id)])),
            ),
        ]),
    )
    .await?;
    if !b::ok(&refreshed) {
        res.apply_envelope(refreshed.to_envelope())?;
        return Ok(());
    }
    let rows = b::result(&refreshed)?;
    let Some(latest) = rows.as_array().and_then(|rows| rows.first()) else {
        b::error(
            res,
            "Invalid Parameters",
            json!({"email":"Account not found"}),
        )?;
        return Ok(());
    };
    let consumed = latest
        .get("consumed")
        .ok_or_else(|| stackpress_lib::Error::new("consumed is missing"))?;
    // Generate after rotation/reload and before the source late missing-email-config branch.
    let code = if otp {
        let random = (environment.random)()?;
        if !(0.0..1.0).contains(&random) {
            return Err(stackpress_lib::Error::invalid(
                "random fraction must be in [0,1)",
            ));
        }
        format!("{:06}", (random * 1_000_000.0).floor() as u32)
    } else {
        String::new()
    };
    let challenge =
        crate::codecs::hash(&format!("{}{code}", events::consumed_text(inv, consumed)?));
    let kind = if otp { "otp" } else { "link" };
    let id = latest
        .get("id")
        .map(columns::string)
        .unwrap_or_else(|| "undefined".into());
    let link = format!(
        "{}/signin/{kind}/{}/{}?redirect_uri={}",
        config.base(),
        b::encode(&id),
        b::encode(&challenge),
        b::encode(&redirect)
    );
    // Rotation/reload/random have already happened, even if no mail can be sent.
    if !environment.email_configured {
        return Ok(());
    }
    let host = environment
        .host
        .unwrap_or_else(|| format!("http://localhost:{}", environment.port));
    let name = latest
        .get("profile")
        .and_then(|profile| profile.get("name"))
        .and_then(Value::as_str);
    let absolute_link = format!("{host}{link}");
    let input = templates::SigninEmailTemplateInput {
        brand: Some(&environment.brand),
        email: &email,
        link: &absolute_link,
        name,
        pin: Some(&code),
    };
    let template = if otp {
        templates::make_otp_template(input)
    } else {
        templates::make_magic_link_template(input)
    };
    // Templates are rendered before checking the source sender fields.
    let Some(sender) = config
        .email
        .filter(|sender| !sender.address.is_empty() && !sender.name.is_empty())
    else {
        return Ok(());
    };
    let mail = json!({"from":{"name":sender.name,"address":sender.address},"to":email,"subject":template.subject,"text":template.text,"html":template.html});
    let mut send = Request::builder().data(mail).build()?;
    // Lane B owns this event and its per-call transport. Same response is shared deliberately.
    inv.emit("email-send", &mut send, res).await?;
    if !b::ok(res) {
        return Ok(());
    }
    // Magic issuance deliberately retains the send response without clearing or redirecting.
    if otp {
        inv.service::<Csrf>()?.clear(req, res)?;
        res.redirect(&link)?;
    }
    Ok(())
}
