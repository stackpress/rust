//! Rich event values preserve native Date instants without JSON marker objects.

use super::{AuthConfig, AuthSection, Environment};
use crate::columns::{self, Record, Value};
use stackpress_lib::{Error, Invocation, PublicError, Request, Response, Result, Timestamp};
use stackpress_server::ServerInvocationExt;

/// Convert owned native source values to the lib's existing in-process value contract.
pub(crate) fn native(value: Value) -> Result<stackpress_lib::Value> {
    use stackpress_lib::Value as Lib;
    Ok(match value {
        Value::Date { millis, .. } => Lib::DateTime(Timestamp::new(
            millis.div_euclid(1000),
            (millis.rem_euclid(1000) as u32) * 1_000_000,
        )?),
        Value::Array(values) => Lib::Array(values.into_iter().map(native).collect::<Result<_>>()?),
        Value::Object(values) => Lib::Object(
            values
                .into_iter()
                .map(|(key, value)| Ok((key, native(value)?)))
                .collect::<Result<_>>()?,
        ),
        other => Lib::try_from(serde_json::to_value(other)?)?,
    })
}
/// Import rich values; unsupported IO/native payloads fail instead of pretending to be JSON.
pub(crate) fn source(value: &stackpress_lib::Value) -> Result<Value> {
    use stackpress_lib::Value as Lib;
    Ok(match value {
        Lib::DateTime(time) => columns::date(
            time.seconds
                .checked_mul(1000)
                .and_then(|seconds| seconds.checked_add(i64::from(time.nanoseconds() / 1_000_000)))
                .ok_or_else(|| Error::invalid("Date exceeds native milliseconds"))?,
        ),
        Lib::Array(values) => Value::Array(values.iter().map(source).collect::<Result<_>>()?),
        Lib::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), source(value)?)))
                .collect::<Result<_>>()?,
        ),
        other => Value::from(other.to_json()?),
    })
}
/// Read exact aggregate operation input, including rich Date fields from nested events.
pub(crate) fn input(req: &Request) -> Result<Record> {
    req.data
        .entries()
        .iter()
        .map(|(key, value)| Ok((key.clone(), source(value)?)))
        .collect::<Result<_>>()
}
/// Source path fallback happens on missing input, preserving null/false/empty.
pub(crate) fn get(req: &Request, key: &str, fallback: Value) -> Result<Value> {
    req.data
        .get(key)
        .map(source)
        .transpose()
        .map(|value| value.unwrap_or(fallback))
}
/// Source String(...) for route/input fields, with missing fallback selected separately.
pub(crate) fn text(req: &Request, key: &str, fallback: &str) -> Result<String> {
    let value = get(req, key, fallback.into())?;
    // Process String(object) differs from the generated columns' JSON.stringify coercion.
    Ok(if value.is_object() {
        "[object Object]".into()
    } else {
        columns::string(&value)
    })
}
/// Source input mutation uses literal keys and retains the existing Nest budgets.
pub(crate) fn set(req: &mut Request, key: &str, value: Value) -> Result<()> {
    req.data.set(key, native(value)?)?;
    Ok(())
}
/// Set rich results instead of erasing Date into ISO before another event can consume it.
pub(crate) fn results(res: &mut Response, value: Value) -> Result<()> {
    res.results_value(native(value)?)?;
    Ok(())
}
/// Read a child result without changing missing results into a successful record.
pub(crate) fn result(res: &Response) -> Result<Value> {
    res.result()
        .map(source)
        .transpose()
        .map(|value| value.unwrap_or(Value::Null))
}
/// JavaScript ctx.resolve supplies a fresh request/response; Rust explicitly owns both.
pub(crate) async fn resolve(inv: &Invocation<'_>, event: &str, data: Record) -> Result<Response> {
    let mut req = Request::new();
    for (key, value) in data {
        set(&mut req, &key, value)?;
    }
    let mut res = Response::new();
    inv.emit(event, &mut req, &mut res).await?;
    Ok(res)
}
/// Missing listeners leave code zero/unset, as the source raw Response does.
pub(crate) fn ok(res: &Response) -> bool {
    res.status().is_some_and(|status| status.code == 200)
}
/// Source page short circuit leaves earlier responses untouched and does not abort the chain.
pub(crate) fn completed(res: &Response) -> bool {
    crate::plugin::completed(res)
}
/// Read auth only at the source access point; disabled unused routes need no dependencies.
pub(crate) fn config(inv: &Invocation<'_>) -> Result<AuthConfig> {
    optional_config(inv)?.ok_or_else(|| Error::new("auth config is missing"))
}
/// Direct handlers may use a typed Option without selecting the auth registrar.
pub(crate) fn optional_config(inv: &Invocation<'_>) -> Result<Option<AuthConfig>> {
    if let Ok(section) = inv.config::<AuthSection>() {
        return Ok(section.0.clone());
    }
    Ok(inv.config::<Option<AuthConfig>>().ok().cloned().flatten())
}
/// Independent direct handler use can provide the same typed service without a registrar.
pub(crate) fn environment(inv: &Invocation<'_>) -> Environment {
    inv.service::<Environment>().cloned().unwrap_or_default()
}
/// Upgrade source Actions errors without changing field capitalization/status.
pub(crate) fn action_error(res: &mut Response, error: crate::storage::ActionError) -> Result<()> {
    let failure = Error::new("auth action failed")
        .with_code(error.code)
        .with_public_message(error.error.clone())
        .with_errors(serde_json::to_value(&error.errors)?)
        .with_cause(error);
    // Keep the native source chain as well as the same authored public error fields.
    res.apply_error(failure)?;
    Ok(())
}
/// Source setError(string, errors) always selects its default 400 status.
pub(crate) fn error(res: &mut Response, message: &str, errors: serde_json::Value) -> Result<()> {
    // Source setError defaults to 400 even immediately after statusCode(404).
    res.set_error(PublicError {
        code: 400,
        status: "Bad Request".into(),
        error: message.into(),
        errors,
    })?;
    Ok(())
}
/// JavaScript encodeURIComponent leaves only its original ASCII unescaped set.
pub(crate) fn encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&byte) {
            encoded.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(&mut encoded, "%{byte:02X}").expect("String write");
        }
    }
    encoded
}
/// ECMAScript trim includes FEFF and excludes some Rust Unicode whitespace characters.
pub(crate) fn trim(value: &str) -> &str {
    value.trim_matches(|character| matches!(character,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}'))
}
