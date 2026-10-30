//! Profile projection and sequential linked-identifier edits.
use super::*;
use crate::columns;
use serde_json::json;
use stackpress_lib::{Flow, PublicError};

/// Resolve profile then auth; expose exactly id/type/token/verified per credential type.
/// # Errors
/// Propagates nested dispatch/data failures; non-200 source responses pass through.
pub async fn load_account_profile(inv: &Invocation<'_>, id: Value) -> Result<Response> {
    let mut profile = b::resolve(
        inv,
        "profile-detail",
        Record::from_iter([("id".into(), id.clone())]),
    )
    .await?;
    if !b::ok(&profile) || !columns::truthy(&b::result(&profile)?) {
        return Ok(profile);
    }
    let auth = b::resolve(
        inv,
        "auth-search",
        Record::from_iter([("eq".into(), Value::from(json!({"profileId":id})))]),
    )
    .await?;
    let rows = b::result(&auth)?;
    let Some(rows) = rows.as_array().filter(|rows| !rows.is_empty()) else {
        return Ok(auth);
    };
    if !b::ok(&auth) {
        return Ok(auth);
    }
    let mut map = Record::new();
    for row in rows {
        let mut projected = Record::new();
        for key in ["id", "type", "token", "verified"] {
            if let Some(value) = row.get(key) {
                projected.insert(key.into(), value.clone());
            }
        }
        let kind = row
            .get("type")
            .map(columns::string)
            .unwrap_or_else(|| "undefined".into());
        map.insert(kind, Value::Object(projected));
    }
    let mut result = b::result(&profile)?;
    if let Some(result) = result.as_object_mut() {
        result.insert("auth".into(), Value::Object(map));
    }
    b::results(&mut profile, result)?;
    Ok(profile)
}
/// Source account/security loader maps a missing profile to an authored 401 error.
/// # Errors
/// Propagates native service and event failures.
pub async fn account(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(data) = identity(req, res, &inv).await? else {
        unauthorized(res)?;
        return Ok(Flow::Continue);
    };
    let profile = load_account_profile(&inv, id(&data)).await?;
    if profile.status().is_some_and(|status| status.code == 404) {
        unauthorized(res)?;
        return Ok(Flow::Continue);
    }
    res.apply_envelope(profile.to_envelope())?;
    props(res, &inv)?;
    Ok(Flow::Continue)
}
/// Account's source Exception differs from the status-only failures on other pages.
fn unauthorized(res: &mut Response) -> Result<()> {
    res.set_error(PublicError {
        code: 401,
        status: "Unauthorized".into(),
        error: "Unauthorized".into(),
        errors: json!({}),
    })?;
    Ok(())
}
/// Update existing tokens without reauth; adding a method requires a matching password.
/// # Errors
/// Stops at the same first nested response failure; prior writes remain committed.
pub async fn update(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(data) = identity(req, res, &inv).await? else {
        return Ok(Flow::Continue);
    };
    let profile = load_account_profile(&inv, id(&data)).await?;
    if profile.status().is_some_and(|status| status.code == 404) {
        res.status_code(401, Some("Unauthorized".into()))?;
        return Ok(Flow::Continue);
    }
    if req.method.as_str() != "POST" {
        res.apply_envelope(profile.to_envelope())?;
        props(res, &inv)?;
        return Ok(Flow::Continue);
    }
    let result = b::result(&profile)?;
    if !b::ok(&profile) || !columns::truthy(&result) {
        res.apply_envelope(profile.to_envelope())?;
        return Ok(Flow::Continue);
    }
    let auth = result
        .get("auth")
        .cloned()
        .unwrap_or(Value::Object(Record::new()));
    let auths = b::resolve(
        &inv,
        "auth-search",
        Record::from_iter([("eq".into(), Value::from(json!({"profileId":id(&data)})))]),
    )
    .await?;
    if !b::ok(&auths) {
        res.apply_envelope(auths.to_envelope())?;
        return Ok(Flow::Continue);
    }
    let mut input = b::input(req)?;
    // G44 changes signup only. Update retains normalizePhone(undefined/nonstring)
    // failure; the source form posts an empty phone field when it is not set.
    let phone = match input.get("phone") {
        None => {
            return Err(stackpress_lib::Error::invalid(
                "Cannot read properties of undefined (reading 'trim')",
            ));
        }
        Some(Value::String(value)) => crate::auth::events::normalize_phone(Some(value)),
        Some(_) => {
            return Err(stackpress_lib::Error::invalid(
                "phone.trim is not a function",
            ));
        }
    };
    input.shift_remove("phone");
    if let Some(phone) = phone {
        input.insert("phone".into(), phone.into());
    }
    let rows = b::result(&auths)?;
    let current = input.get("current").filter(|value| columns::truthy(value));
    let adding = ["username", "email", "phone"].iter().any(|kind| {
        !auth
            .get(kind)
            .and_then(|row| row.get("id"))
            .is_some_and(columns::truthy)
            && has_token(input.get(*kind))
    });
    if adding {
        let rows = rows.as_array().filter(|rows| !rows.is_empty());
        // Match both optional inputs without changing source error precedence.
        let message = match (rows, current) {
            (None, _) => Some("No authentication found."),
            (Some(_), None) => Some("Current password is required to add a sign-in method"),
            (Some(rows), Some(current)) if !matches_password(rows, current, true) => {
                Some("Current password is incorrect")
            }
            _ => None,
        };
        if let Some(message) = message {
            b::error(res, "Invalid Parameters", json!({"current":message}))?;
            return Ok(Flow::Continue);
        }
    }
    let mut patch = Record::from_iter([("id".into(), id(&data))]);
    for key in ["name", "image"] {
        if let Some(value) = input.get(key) {
            patch.insert(key.into(), value.clone());
        }
    }
    resolve_into(&inv, "profile-update", patch, res).await?;
    if res
        .status()
        .is_some_and(|status| status.code != 0 && status.code != 200)
    {
        return Ok(Flow::Continue);
    }
    for kind in ["username", "email", "phone"] {
        let existing = auth.get(kind).and_then(|row| row.get("id")).cloned();
        if let Some(response) = update_auth(
            &inv,
            id(&data),
            kind,
            existing,
            input.get(kind).cloned(),
            input.get("current").cloned(),
        )
        .await?
            && response
                .status()
                .is_some_and(|status| status.code != 0 && status.code != 200)
        {
            res.apply_envelope(response.to_envelope())?;
            return Ok(Flow::Continue);
        }
    }
    if b::ok(res) {
        res.redirect(&format!("{}/account", base(&inv)?))?;
    }
    Ok(Flow::Continue)
}
/// Ignore blank tokens, update existing rows or create a missing sign-in method.
/// # Errors
/// Propagates event failures. No deletion, verification reset or transaction is added.
pub async fn update_auth(
    inv: &Invocation<'_>,
    profile: Value,
    kind: &str,
    id: Option<Value>,
    token: Option<Value>,
    secret: Option<Value>,
) -> Result<Option<Response>> {
    if !has_token(token.as_ref()) {
        return Ok(None);
    }
    let mut input = Record::from_iter([("token".into(), token.unwrap())]);
    let event = if let Some(id) = id.filter(columns::truthy) {
        input.insert("id".into(), id);
        "auth-update"
    } else {
        input.insert("profileId".into(), profile);
        input.insert("type".into(), kind.into());
        if let Some(secret) = secret {
            input.insert("secret".into(), secret);
        }
        "auth-create"
    };
    Ok(Some(b::resolve(inv, event, input).await?))
}
/// Source typeof/string/trim guard; whitespace is not a delete operation.
fn has_token(value: Option<&Value>) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| !b::trim(value).is_empty())
}
/// Update restricts password methods; password/removal pages compare every auth row.
pub(super) fn matches_password(
    rows: &[Value],
    current: &Value,
    password_methods_only: bool,
) -> bool {
    let hash = crate::codecs::hash(&columns::string(current));
    rows.iter().any(|row| {
        (!password_methods_only
            || row
                .get("type")
                .and_then(Value::as_str)
                .is_some_and(|kind| ["username", "email", "phone"].contains(&kind)))
            && row.get("secret").and_then(Value::as_str) == Some(&hash)
    })
}
