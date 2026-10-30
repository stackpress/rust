//! Original reusable session-as-code exchange; app secret presence is not equality validation.
use super::{completed, environment, property, truthy};
use crate::helpers;
use stackpress_lib::DataMap;
use stackpress_lib::{Error, Flow, Invocation, Nest, Request, Response, Result, Value};

/// Exchange a persisted Session.id for its unchanged ID/secret and selected joined user fields.
///
/// Every method requires the parsed app secret to be nonempty, including GET. The secret is
/// not compared here. Only a strictly past expiry rejects; no consumption or active check is
/// introduced. SQL detail owns its own default row selection and joined relations.
/// The native feature decodes SQL ISO Dates and serializes rich profile Dates; direct
/// non-native consumers supply JSON-compatible profile fields.
/// # Errors
/// Propagates lookup/response failures; a missing profile or non-Date expiry has the source
/// failure branch, expressed as a Rust error. ISO Dates emitted by API SQL are decoded explicitly.
pub async fn token(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(authorization) = helpers::authorize(req, res)? else {
        return Ok(Flow::Continue);
    };
    let Some(code) = req.data.get("code").filter(|value| truthy(value)) else {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    };
    if authorization.secret.is_empty() {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    }
    let mut input = Nest::new();
    input.set("id", code.clone())?;
    let session = helpers::resolve(&inv, "session-detail", input).await?;
    let Some(data) = session.result().filter(|value| truthy(value)) else {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    };
    if property(data, "applicationId").and_then(Value::as_str) != Some(&authorization.id) {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    }
    let env = environment(&inv);
    let expiry = property(data, "expires")
        .filter(|value| truthy(value))
        .map(millis)
        .transpose()?;
    // Preserve both Date.now reads: time can advance between admission and expires_in.
    if let Some(expiry) = expiry
        && expiry < (env.now)()?
    {
        helpers::unauthorized(res)?;
        return Ok(Flow::Continue);
    }
    let remaining = if let Some(expiry) = expiry {
        // Floating arithmetic matches JS subtraction/floor and avoids native signed overflow.
        ((expiry as f64 - (env.now)()? as f64) / 1000.0).floor()
    } else {
        0.0
    };
    let profile = property(data, "profile")
        .filter(|value| !matches!(value, Value::Null))
        .ok_or_else(|| Error::new("token session profile is missing"))?;
    let mut user = DataMap::new();
    for field in ["id", "name", "image", "created"] {
        if let Some(value) = property(profile, field) {
            user.insert(field.into(), wire(value)?);
        }
    }
    // Missing source members are omitted, not fabricated as null or an authenticated user.
    let mut result = DataMap::from_iter([
        ("token_type".into(), Value::from("Bearer")),
        (
            "expires_in".into(),
            // Integral seconds match the source JSON spelling. Both inputs are i64 millis,
            // so their difference divided by 1000 remains safely inside i64.
            Value::from(remaining as i64),
        ),
        ("user".into(), Value::Object(user)),
    ]);
    for (field, source) in [("access_token", "id"), ("access_secret", "secret")] {
        if let Some(value) = property(data, source) {
            result.insert(field.into(), value.clone());
        }
    }
    res.results_value(Value::Object(result))?;
    Ok(Flow::Continue)
}

/// Import an in-process Date or the canonical ISO Date wire spelling from M1 SQL events.
fn millis(value: &Value) -> Result<i64> {
    if let Value::DateTime(time) = value {
        return time
            .seconds
            .checked_mul(1000)
            .and_then(|seconds| seconds.checked_add(i64::from(time.nanoseconds() / 1_000_000)))
            .ok_or_else(|| Error::invalid("token Date exceeds native milliseconds"));
    }
    #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
    if let Value::String(iso) = value {
        use stackpress_framework_session::columns;
        let millis = columns::date_millis(&columns::Value::from(iso.clone()), 0);
        // Only the canonical wire spelling is a Date adapter; arbitrary strings keep the source
        // getTime failure instead of being coerced to epoch by the generated column codec.
        if serde_json::to_value(columns::date(millis))? == serde_json::Value::String(iso.clone()) {
            return Ok(millis);
        }
    }
    Err(Error::invalid(
        "expires.getTime is not a function (expected native or SQL wire Date)",
    ))
}

/// Source Date.toJSON at the response boundary; ordinary JSON values retain their shape.
fn wire(value: &Value) -> Result<Value> {
    if let Value::DateTime(_) = value {
        #[cfg(all(feature = "native", not(target_arch = "wasm32")))]
        {
            // Reuse the owner codec rather than JSON marker objects or a second Date parser.
            return Value::try_from(serde_json::to_value(
                stackpress_framework_session::columns::date(millis(value)?),
            )?);
        }
        #[cfg(not(all(feature = "native", not(target_arch = "wasm32"))))]
        return Err(Error::invalid(
            "native feature is required to serialize source Date to JSON",
        ));
    }
    match value {
        // JSON serialization visits nested Date values too; custom event results can contain them.
        Value::Array(values) => Ok(Value::Array(
            values.iter().map(wire).collect::<Result<_>>()?,
        )),
        Value::Object(values) => Ok(Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), wire(value)?)))
                .collect::<Result<_>>()?,
        )),
        _ => Ok(value.clone()),
    }
}
