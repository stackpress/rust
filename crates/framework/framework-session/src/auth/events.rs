//! Reusable source auth events. Direct events impose no page CSRF/password/2FA policy.

use super::{Hooks, bridge as b};
use crate::{
    Service,
    columns::{self, Record, Value},
};
use serde_json::json;
use stackpress_framework_sql::source::SessionModels;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;

/// Normalize optional phone input per G44 and the source's ASCII-digit/separator grammar.
pub fn normalize_phone(phone: Option<&str>) -> Option<String> {
    let phone = b::trim(phone?);
    let mut normalized = String::new();
    for (index, character) in phone.chars().enumerate() {
        if character.is_ascii_digit() || (index == 0 && character == '+') {
            normalized.push(character);
        }
    }
    normalized
        .bytes()
        .any(|byte| byte.is_ascii_digit())
        .then_some(normalized)
}
/// Source signup: config roles then request override, sequential SQL, nonawaited hooks,
/// then remove exactly email/phone/username secrets. No global/page guard is added.
/// # Errors
/// Propagates native host/data failures; source Action errors become the public response.
pub async fn signup_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let roles = b::optional_config(&inv)?
        .as_ref()
        .map(|config| config.roles.clone())
        .unwrap_or_default();
    let actions = super::bindings::actions::<M>(&inv)?;
    // Spread defaults first so every supplied request member has the source precedence.
    let mut input = Record::from_iter([("roles".into(), Value::from(json!(roles)))]);
    input.extend(b::input(req)?);
    // Missing phone is optional G44. A supplied nonstring retains source trim failure.
    let phone = match input.get("phone") {
        None => None,
        Some(Value::String(phone)) => normalize_phone(Some(phone)),
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
    // Signup remains sequential; an Action error does not undo earlier profile/auth inserts.
    let mut result = match actions.signup(&input).await {
        Ok(result) => result,
        Err(error) => {
            b::action_error(res, error)?;
            return Ok(Flow::Continue);
        }
    };
    // Copy before stripping secrets: source hook payload is submitted at this exact point.
    for (field, event) in [
        ("email", "email-email-send"),
        ("phone", "auth-phone-verify"),
    ] {
        if let Some(value) = input.get(field).filter(|value| columns::truthy(value)) {
            let mut data = Record::from_iter([(field.into(), value.clone())]);
            data.extend(result.clone());
            inv.service::<Hooks>()?.submit(event, data);
        }
    }
    // Only the three named source credentials are scrubbed; extension records are retained.
    if let Some(auth) = result.get_mut("auth").and_then(Value::as_object_mut) {
        for kind in ["email", "phone", "username"] {
            if let Some(record) = auth.get_mut(kind).and_then(Value::as_object_mut) {
                record.shift_remove("secret");
            }
        }
    }
    b::results(res, Value::Object(result))?;
    Ok(Flow::Continue)
}
/// Source signin preserves truthy password and strict `2fa !== false` flags.
/// # Errors
/// Action failures become source public errors; late config/session/host failures propagate.
pub async fn signin_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    // Boolean(password) is source truthiness; the page may force it, but this reusable event does not.
    let kind = b::text(req, "type", "username")?;
    let password = columns::truthy(&b::get(req, "password", true.into())?);
    let mut result = match super::bindings::actions::<M>(&inv)?
        .signin(&kind, &b::input(req)?, password)
        .await
    {
        Ok(Some(result)) => result,
        Ok(None) => return Err(stackpress_lib::Error::new("signin auth result is missing")),
        Err(error) => {
            b::action_error(res, error)?;
            return Ok(Flow::Continue);
        }
    };
    // The action has already updated/reloaded consumed before auth config is accessed.
    let config = b::config(&inv)?;
    // Only literal false suppresses the search; null/zero/empty still check for a factor.
    if b::get(req, "2fa", true.into())? != Value::Bool(false) {
        let required = ["profileId", "id", "consumed"].map(|name| {
            result
                .get(name)
                .filter(|value| columns::truthy(value))
                .cloned()
        });
        let [Some(profile), Some(auth), Some(consumed)] = required else {
            res.redirect(&format!("{}/signin", config.base()))?;
            return Ok(Flow::Continue);
        };
        // Reuse the existing event, retaining its handlers/order and public failure response.
        let search = b::resolve(
            &inv,
            "auth-search",
            Record::from_iter([(
                "eq".into(),
                Value::from(json!({"type":"2fa","profileId":profile})),
            )]),
        )
        .await?;
        if !b::ok(&search) {
            res.apply_envelope(search.to_envelope())?;
            return Ok(Flow::Continue);
        }
        if b::result(&search)?
            .as_array()
            .is_some_and(|rows| !rows.is_empty())
        {
            let challenge = challenge(&inv, &consumed)?;
            res.redirect(&format!(
                "{}/signin/2fa/{}/{}/{}?redirect_uri={}",
                config.base(),
                b::encode(&columns::string(&profile)),
                b::encode(&columns::string(&auth)),
                b::encode(&challenge),
                b::encode(&b::text(req, "redirect_uri", "/")?)
            ))?;
            return Ok(Flow::Continue);
        }
    }
    // No second factor was required (or the request skipped it); source now removes the secret.
    result.shift_remove("secret");
    b::results(res, Value::Object(result.clone()))?;
    // Results are already set if the later session service or JWT creation fails upstream.
    let profile = result
        .get("profile")
        .and_then(Value::as_object)
        .ok_or_else(|| stackpress_lib::Error::new("signin profile is missing"))?;
    // Project only source claims; auth IDs, tokens and password hashes are not JWT claims.
    let mut claims = serde_json::Map::new();
    for field in ["id", "name"] {
        if let Some(value) = profile.get(field) {
            claims.insert(field.into(), serde_json::to_value(value)?);
        }
    }
    if let Some(image) = profile.get("image").filter(|value| columns::truthy(value)) {
        claims.insert("image".into(), serde_json::to_value(image)?);
    }
    let roles = profile
        .get("roles")
        .filter(|value| value.as_array().is_some_and(|roles| !roles.is_empty()))
        .cloned()
        .unwrap_or_else(|| Value::from(json!(["GUEST"])));
    claims.insert("roles".into(), serde_json::to_value(roles)?);
    // Signing uses the current configured key and existing cookie policy/lifetime without additions.
    let session = inv.service::<Service>()?;
    let token = session
        .create(claims)
        .await
        .map_err(|cause| stackpress_lib::Error::new("session creation failed").with_cause(cause))?;
    res.session_mut()?.set(&session.config().key, token, None)?;
    Ok(Flow::Continue)
}
/// Source signout deletes literal `session` even when the configured identity key differs.
/// # Errors
/// Propagates existing cookie/response mutation failures only.
///
/// ```
/// use stackpress_framework_session::auth::events;
/// use stackpress_server::ServerBuilder;
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> stackpress_lib::Result<()> {
/// let mut app = ServerBuilder::new();
/// app.on("quit", events::signout)?; // no auth registrar/config/Engine is needed
/// let server = app.build().await.map_err(|failure| failure.error)?;
/// assert_eq!(server.resolve("quit", serde_json::json!({})).await?.response.code, Some(200));
/// # Ok(())
/// # }
/// ```
pub async fn signout(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.session_mut()?.remove("session")?;
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}
/// Optional direct SQL adapter for the source auth-search extension event.
/// It has no global operation policy; applications choose which events to expose.
/// # Errors
/// Native input/host failures propagate; source SQL/action errors become public failures.
pub async fn search(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // One generated-SQL adapter owns source seed/skip/filter mechanics; retain this public path.
    crate::account::events::auth_search(req, res, inv).await
}
/// Direct auth-update path reuses the same generated source SQL adapter as account pages.
/// # Errors
/// Preserves source action and native response errors, with missing-engine pass-through.
pub async fn update(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    crate::account::events::auth_update(req, res, inv).await
}
/// Explicit JSON projection for wire/SSR adapters; rich Dates become canonical ISO values.
/// # Errors
/// Rejects unsupported IO/native payloads and out-of-range Date values.
pub fn json_result(res: &Response) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(b::result(res)?)?)
}
/// Hash source consumed.toString with the host's explicit local timezone.
pub(crate) fn challenge(inv: &Invocation<'_>, consumed: &Value) -> Result<String> {
    Ok(crate::codecs::hash(&consumed_text(inv, consumed)?))
}
/// Keep native Date and ordinary string consumed values distinct, as JavaScript does.
pub(crate) fn consumed_text(inv: &Invocation<'_>, consumed: &Value) -> Result<String> {
    if let Value::Date { millis, display } = consumed {
        if let Some(display) = display {
            return Ok(display.clone());
        }
        let environment = b::environment(inv);
        return crate::codecs::date_string(
            *millis,
            environment.offset_minutes,
            &environment.zone_name,
        )
        .map_err(|cause| stackpress_lib::Error::new("Date display failed").with_cause(cause));
    }
    Ok(columns::string(consumed))
}

/// Existing source signup path, selecting the checked-in generated default family.
pub async fn signup(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    signup_with::<stackpress_framework_models::Models>(req, res, inv).await
}
/// Existing source signin path, selecting the same generated default family as signup.
pub async fn signin(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    signin_with::<stackpress_framework_models::Models>(req, res, inv).await
}
