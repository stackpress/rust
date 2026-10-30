//! Source enrollment/removal flows; deliberately displayed provisioning data stays public.
use super::*;
use crate::{auth::totp, columns};
use serde_json::json;
use stackpress_framework_csrf::Service as Csrf;
use stackpress_lib::Flow;

/// Match the source's double encoding of the issuer inside the account label.
pub fn provisioning_uri(issuer: &str, name: &str, secret: &str) -> String {
    let issuer = b::encode(issuer);
    let label = b::encode(&format!(
        "{issuer}:{}",
        if name.is_empty() { "User" } else { name }
    ));
    format!("otpauth://totp/{label}?secret={secret}&issuer={issuer}")
}
/// Encode a QR PNG data URL with source M correction, four-module margin and scale four.
/// # Errors
/// Reports QR capacity or PNG encoding failure; no untrusted markup enters the view.
pub fn qr_data_url(uri: &str) -> Result<String> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let code = qrcode::QrCode::with_error_correction_level(uri.as_bytes(), qrcode::EcLevel::M)
        .map_err(stackpress_lib::Error::from_source)?;
    let modules = code.width();
    let size = (modules + 8) * 4;
    let mut pixels = vec![255; size * size];
    for y in 0..modules {
        for x in 0..modules {
            if code[(x, y)] == qrcode::Color::Dark {
                for dy in 0..4 {
                    for dx in 0..4 {
                        pixels[((y + 4) * 4 + dy) * size + (x + 4) * 4 + dx] = 0;
                    }
                }
            }
        }
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, size as u32, size as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .map_err(stackpress_lib::Error::from_source)?;
        writer
            .write_image_data(&pixels)
            .map_err(stackpress_lib::Error::from_source)?;
    }
    Ok(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}
/// Display request/stored/generated secret; POST validates only source CSRF and TOTP checks.
/// # Errors
/// Keeps source dispatch/QR/decoder errors and nontransactional update/create effects.
pub async fn enroll(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let csrf = inv.service::<Csrf>()?;
    inv.service::<Service>()?;
    props(res, &inv)?;
    let Some(data) = identity(req, res, &inv).await? else {
        return Ok(Flow::Continue);
    };
    let profile = b::resolve(
        &inv,
        "profile-detail",
        Record::from_iter([("id".into(), id(&data))]),
    )
    .await?;
    if profile.status().is_some_and(|status| status.code == 404) {
        res.status_code(401, Some("Unauthorized".into()))?;
        return Ok(Flow::Continue);
    }
    let auth = b::resolve(
        &inv,
        "auth-search",
        Record::from_iter([(
            "eq".into(),
            Value::from(json!({"type":"2fa","profileId":id(&data)})),
        )]),
    )
    .await?;
    if !b::ok(&auth) {
        res.apply_envelope(auth.to_envelope())?;
        csrf.generate(res)?;
        return Ok(Flow::Continue);
    }
    let rows = b::result(&auth)?;
    let factor = rows.as_array().and_then(|rows| rows.first());
    // JS evaluates the fallback argument even when request secret exists. Preserve
    // the random-generation branch, but let an explicitly supplied value win.
    let fallback = if let Some(factor) = factor.filter(|_| !req.data.has("regenerate")) {
        factor.get("token").cloned().unwrap_or(Value::Null)
    } else {
        totp::generate_secret(16)?.into()
    };
    let secret = b::get(req, "secret", fallback)?;
    let secret_text = columns::string(&secret);
    let env = b::environment(&inv);
    let config = b::config(&inv)?;
    let issuer = config
        .two_factor
        .as_ref()
        .and_then(|v| v.get("issuer"))
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            _ => v.to_string(),
        })
        .unwrap_or(env.brand);
    let name = data
        .get("name")
        .filter(|v| columns::truthy(v))
        .map(columns::string)
        .unwrap_or_else(|| "User".into());
    let url = qr_data_url(&provisioning_uri(&issuer, &name, &secret_text))?;
    if req.method.as_str() == "POST" {
        if !csrf.valid(req, res)? {
            csrf.generate(res)?;
            return Ok(Flow::Continue);
        }
        let code = b::text(req, "code", "")?;
        let padded = format!(
            "{}{}",
            "0".repeat(6_usize.saturating_sub(code.encode_utf16().count())),
            code
        );
        if code.is_empty()
            || padded.encode_utf16().count() < 6
            || !totp::verify_at(&padded, &secret_text, env.clock.now().floor() as u64, 1)?
        {
            b::error(res, "Invalid code", json!({}))?;
            b::results(res, Value::from(json!({"url":url,"secret":secret})))?;
            csrf.generate(res)?;
            return Ok(Flow::Continue);
        }
        let mut input =
            Record::from_iter([("token".into(), secret.clone()), ("secret".into(), secret)]);
        let event = if let Some(factor) = factor {
            input.insert(
                "id".into(),
                factor.get("id").cloned().unwrap_or(Value::Null),
            );
            "auth-update"
        } else {
            input.insert("profileId".into(), id(&data));
            input.insert("type".into(), "2fa".into());
            "auth-create"
        };
        resolve_into(&inv, event, input, res).await?;
        // Source flashes/redirects even when the returned write response is an error.
        flash(
            res,
            "Two-Factor Authentication has been updated successfully.",
            Some(5000),
        )?;
        res.redirect(&b::text(
            req,
            "redirect_uri",
            &format!("{}/account/security/2fa", base(&inv)?),
        )?)?;
        return Ok(Flow::Continue);
    }
    let mut result = Record::from_iter([
        ("url".into(), url.into()),
        ("secret".into(), secret),
        ("profile".into(), b::result(&profile)?),
    ]);
    if let Some(id) = factor.and_then(|factor| factor.get("id")) {
        result.insert("authId".into(), id.clone());
    }
    b::results(res, Value::Object(result))?;
    csrf.generate(res)?;
    Ok(Flow::Continue)
}
/// Confirmed removal is method-independent and never invokes csrf.valid in the source.
/// # Errors
/// Retains search/remove failure order and CSRF generation on remaining/error branches.
pub async fn remove_factor(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let csrf = inv.service::<Csrf>()?;
    inv.service::<Service>()?;
    props(res, &inv)?;
    let Some(data) = identity(req, res, &inv).await? else {
        return Ok(Flow::Continue);
    };
    resolve_into(
        &inv,
        "profile-detail",
        Record::from_iter([("id".into(), id(&data))]),
        res,
    )
    .await?;
    if res.status().is_some_and(|status| status.code == 404) {
        res.status_code(401, Some("Unauthorized".into()))?;
        return Ok(Flow::Continue);
    }
    if columns::truthy(&b::get(req, "confirmed", Value::Null)?) {
        let base = base(&inv)?;
        let auth = b::text(req, "authId", "undefined")?;
        if auth.is_empty() {
            b::error(res, "Invalid authentication.", json!({}))?;
            res.redirect(&format!("{base}/account/security/2fa"))?;
            csrf.generate(res)?;
            return Ok(Flow::Continue);
        }
        let query = Value::from(json!({"id":auth,"profileId":id(&data),"type":"2fa"}));
        let exists = b::resolve(
            &inv,
            "auth-search",
            Record::from_iter([("eq".into(), query.clone())]),
        )
        .await?;
        if !b::ok(&exists) {
            res.apply_envelope(exists.to_envelope())?;
            csrf.generate(res)?;
            return Ok(Flow::Continue);
        }
        let removed = b::resolve(&inv, "auth-remove", query.as_object().unwrap().clone()).await?;
        if !b::ok(&removed) {
            res.apply_envelope(removed.to_envelope())?;
            csrf.generate(res)?;
            return Ok(Flow::Continue);
        }
        flash(
            res,
            "Two-Factor Authentication has been removed successfully.",
            Some(3000),
        )?;
        res.redirect(&b::text(
            req,
            "redirect_uri",
            &format!("{base}/account/security/2fa"),
        )?)?;
        return Ok(Flow::Continue);
    }
    csrf.generate(res)?;
    Ok(Flow::Continue)
}
