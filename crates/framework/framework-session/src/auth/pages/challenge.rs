//! OTP/link proofs and the shared source 2FA final checkpoint.

use super::{b, guest, handoff, props};
use crate::{
    Service,
    auth::{events, totp},
    columns::{self, Record, Value},
};
use serde_json::json;
use stackpress_framework_csrf::Service as Csrf;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

/// Source magic link checks the consumed hash on GET or POST, with no CSRF check.
/// # Errors
/// Propagates native errors and preserves source invalid-link/error/redirect responses.
pub async fn magic_link(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Earlier truthy output or a non-200 status belongs to the preceding handler.
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let config = b::config(&inv)?;
    props(res, &config)?;
    let redirect = b::text(req, "redirect_uri", "/")?;
    if !guest(req, &inv).await? {
        res.redirect(&redirect)?;
        return Ok(Flow::Continue);
    }
    // Route captures are ordinary source data; missing proof components stop before SQL.
    let id = b::get(req, "auth", "".into())?;
    let challenge = b::get(req, "challenge", "".into())?;
    if !columns::truthy(&id) || !columns::truthy(&challenge) {
        invalid_link(res, false)?;
        return Ok(Flow::Continue);
    }
    // Resolve into a fresh response so a child failure can be merged at the source point.
    let current = b::resolve(
        &inv,
        "auth-search",
        Record::from_iter([(
            "eq".into(),
            Value::Object(Record::from_iter([("id".into(), id)])),
        )]),
    )
    .await?;
    if !b::ok(&current) {
        res.apply_envelope(current.to_envelope())?;
        return Ok(Flow::Continue);
    }
    let rows = b::result(&current)?;
    let Some(found) = rows.as_array().and_then(|rows| rows.first()) else {
        invalid_link(res, false)?;
        return Ok(Flow::Continue);
    };
    let consumed = found
        .get("consumed")
        .ok_or_else(|| stackpress_lib::Error::new("consumed is missing"))?;
    // The source proof compares consumed.toString only: no independent TTL/type check.
    let answer = events::challenge(&inv, consumed)?;
    if Value::String(answer.clone()) != challenge {
        invalid_link(res, false)?;
        return Ok(Flow::Continue);
    }
    res.redirect(&handoff(
        &config,
        &found["profileId"],
        &found["id"],
        &answer,
        &redirect,
    ))?;
    Ok(Flow::Continue)
}
/// Source OTP GET loads/strips auth data but performs no code/hash/expiry validation.
/// # Errors
/// Native errors propagate; source POST validation and handoff remain public responses.
pub async fn otp(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Earlier truthy output or a non-200 status belongs to the preceding handler.
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let csrf = inv.service::<Csrf>()?;
    let config = b::config(&inv)?;
    let redirect = b::text(req, "redirect_uri", "/")?;
    if !guest(req, &inv).await? {
        csrf.clear(req, res)?;
        res.redirect(&redirect)?;
        return Ok(Flow::Continue);
    }
    props(res, &config)?;
    csrf.generate(res)?;
    // Route captures are ordinary source data; missing proof components stop before SQL.
    let id = b::get(req, "auth", "".into())?;
    let challenge = b::get(req, "challenge", "".into())?;
    if !columns::truthy(&id) || !columns::truthy(&challenge) {
        invalid_link(res, true)?;
        return Ok(Flow::Continue);
    }
    // Resolve into a fresh response so a child failure can be merged at the source point.
    let current = b::resolve(
        &inv,
        "auth-search",
        Record::from_iter([(
            "eq".into(),
            Value::Object(Record::from_iter([("id".into(), id)])),
        )]),
    )
    .await?;
    if !b::ok(&current) {
        res.apply_envelope(current.to_envelope())?;
        return Ok(Flow::Continue);
    }
    let rows = b::result(&current)?;
    let Some(mut found) = rows
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(Value::as_object)
        .cloned()
    else {
        invalid_link(res, true)?;
        return Ok(Flow::Continue);
    };
    // Strip the stored secret before exposing the GET form payload, before POST validation.
    found.shift_remove("secret");
    b::results(res, Value::Object(found.clone()))?;
    // A GET deliberately stops with the auth payload; only a POST checks the submitted code.
    if req.method.as_str() != "POST" {
        return Ok(Flow::Continue);
    }
    if !csrf.valid(req, res)? {
        return Ok(Flow::Continue);
    }
    let code = b::text(req, "code", "")?;
    let code = b::trim(&code).to_owned();
    // padStart counts UTF-16 units and never enforces digits or truncates long input.
    let padded = pad(&code);
    if code.is_empty() || padded.encode_utf16().count() != 6 {
        b::error(
            res,
            "Invalid Parameters",
            json!({"code":"OTP code is required"}),
        )?;
        return Ok(Flow::Continue);
    }
    let consumed = found
        .get("consumed")
        .ok_or_else(|| stackpress_lib::Error::new("consumed is missing"))?;
    let answer = crate::codecs::hash(&format!(
        "{}{padded}",
        events::consumed_text(&inv, consumed)?
    ));
    if Value::String(answer) != challenge {
        b::error(
            res,
            "Invalid OTP Code",
            json!({"code":"The OTP code is invalid or has expired"}),
        )?;
        return Ok(Flow::Continue);
    }
    csrf.clear(req, res)?;
    res.redirect(&handoff(
        &config,
        &found["profileId"],
        &found["id"],
        &events::challenge(&inv, consumed)?,
        &redirect,
    ))?;
    Ok(Flow::Continue)
}
/// Source selected auth challenge precedes optional TOTP; absent 2FA completes even on GET.
/// # Errors
/// Native failures propagate, including invalid base32; no additional global check occurs.
pub async fn two_factor(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Earlier truthy output or a non-200 status belongs to the preceding handler.
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    inv.service::<Service>()?;
    let csrf = inv.service::<Csrf>()?;
    let config = b::config(&inv)?;
    props(res, &config)?;
    csrf.generate(res)?;
    let redirect = b::text(req, "redirect_uri", "/")?;
    if !guest(req, &inv).await? {
        csrf.clear(req, res)?;
        res.redirect(&redirect)?;
        return Ok(Flow::Continue);
    }
    let profile = b::text(req, "profile", "")?;
    let id = b::text(req, "auth", "")?;
    let challenge = b::text(req, "challenge", "")?;
    if id.is_empty() || profile.is_empty() {
        res.redirect(&format!(
            "{}/signin?redirect_uri={}",
            config.base(),
            b::encode(&redirect)
        ))?;
        return Ok(Flow::Continue);
    }
    // Search by the URL profile, not by a new current-session authorization requirement.
    let search = b::resolve(
        &inv,
        "auth-search",
        Record::from_iter([("eq".into(), Value::from(json!({"profileId":profile})))]),
    )
    .await?;
    if !b::ok(&search) {
        res.apply_envelope(search.to_envelope())?;
        return Ok(Flow::Continue);
    }
    let rows = b::result(&search)?;
    let Some(rows) = rows.as_array().filter(|rows| !rows.is_empty()) else {
        b::error(res, "No authentication methods found", json!({}))?;
        return Ok(Flow::Continue);
    };
    let Some(current) = rows
        .iter()
        .find(|auth| auth.get("id").and_then(Value::as_str) == Some(id.as_str()))
    else {
        b::error(res, "Invalid authentication method", json!({}))?;
        return Ok(Flow::Continue);
    };
    let consumed = current
        .get("consumed")
        .ok_or_else(|| stackpress_lib::Error::new("consumed is missing"))?;
    // A stale/mismatched selected-auth proof uses the source flash and auth-type redirect.
    if events::challenge(&inv, consumed)? != challenge {
        res.session_mut()?.set(
            "flash",
            r#"{"type":"error","message":"Two-factor authentication failed."}"#,
            None,
        )?;
        res.redirect(&format!(
            "{}/signin/{}?redirect_uri={}",
            config.base(),
            columns::string(&current["type"]),
            b::encode(&redirect)
        ))?;
        return Ok(Flow::Continue);
    }
    if let Some(two_factor) = rows
        .iter()
        .find(|auth| auth.get("type") == Some(&Value::String("2fa".into())))
    {
        // Enabled 2FA GET deliberately only renders; no code or CSRF validation yet.
        if req.method.as_str() != "POST" {
            return Ok(Flow::Continue);
        }
        if !csrf.valid(req, res)? {
            return Ok(Flow::Continue);
        }
        let code = b::text(req, "code", "")?;
        // padStart counts UTF-16 units and never enforces digits or truncates long input.
        let padded = pad(&code);
        if code.is_empty() || padded.encode_utf16().count() < 6 {
            b::error(res, "Invalid code", json!({}))?;
            return Ok(Flow::Continue);
        }
        let secret = two_factor
            .get("token")
            .and_then(Value::as_str)
            .ok_or_else(|| stackpress_lib::Error::invalid("2fa token is not a string"))?;
        let now = b::environment(&inv).clock.now();
        if !totp::verify_at(&padded, secret, now as u64, 1)? {
            b::error(res, "Invalid code", json!({}))?;
            return Ok(Flow::Continue);
        }
    }
    // No 2FA record bypasses POST/CSRF/code checks, exactly as the shared source checkpoint.
    b::set(req, "password", false.into())?;
    b::set(req, "type", current["type"].clone())?;
    b::set(
        req,
        &columns::string(&current["type"]),
        current["token"].clone(),
    )?;
    b::set(req, "2fa", false.into())?;
    inv.emit("auth-signin", req, res).await?;
    if b::ok(res) {
        csrf.clear(req, res)?;
        res.redirect(&redirect)?;
    }
    Ok(Flow::Continue)
}
/// Source statusCode(404) is subsequently replaced by setError's default 400.
fn invalid_link(res: &mut Response, otp: bool) -> Result<()> {
    res.status_code(404, Some("Not Found".into()))?;
    b::error(
        res,
        if otp {
            "One-time password link is invalid or expired."
        } else {
            "Magic link is invalid or expired."
        },
        json!({}),
    )
}
/// JavaScript padStart uses UTF-16 length, without truncation or a digits-only restriction.
fn pad(code: &str) -> String {
    format!(
        "{}{code}",
        "0".repeat(6_usize.saturating_sub(code.encode_utf16().count()))
    )
}
