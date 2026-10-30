//! Profile-only source CSV export; the warning page precedes explicit download.
use super::*;
use crate::columns;
use stackpress_lib::Flow;

/// Format the original two-column CSV, including flattening and leading formula guard.
/// # Errors
/// Propagates JSON serialization failures for unsupported native Date ranges.
pub fn csv(profile: &Record) -> Result<String> {
    let mut rows = Vec::new();
    for (key, value) in profile {
        let mut cell = match value {
            Value::Null => String::new(),
            Value::Array(_) | Value::Object(_) | Value::Date { .. } => {
                serde_json::to_string(value)?
                    .replace(['"', '{', '}', '[', ']'], "")
                    .replace(',', ", ")
                    .replace(",  ", ", ")
                    .replace(':', ": ")
                    .replace(":  ", ": ")
            }
            Value::String(value) => value.clone(),
            _ => columns::string(value),
        };
        if cell.starts_with(['=', '+', '-', '@']) {
            cell.insert(0, '\'');
        }
        rows.push(format!("{key},{}", serde_json::to_string(&cell)?));
    }
    Ok(rows.join("\n"))
}
/// Require a profile, then render the warning or attach the original CSV filename.
/// # Errors
/// Preserves nested profile errors and native response framing failures.
pub async fn export(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(data) = identity(req, res, &inv).await? else {
        return Ok(Flow::Continue);
    };
    let profile = b::resolve(
        &inv,
        "profile-detail",
        Record::from_iter([("id".into(), id(&data))]),
    )
    .await?;
    let result = b::result(&profile)?;
    if !b::ok(&profile) || !columns::truthy(&result) {
        if profile.status().is_some_and(|status| status.code == 404) {
            res.status_code(401, Some("Unauthorized".into()))?;
        } else {
            res.apply_envelope(profile.to_envelope())?;
        }
        return Ok(Flow::Continue);
    }
    if !columns::truthy(&b::get(req, "download", Value::Null)?) {
        props(res, &inv)?;
        return Ok(Flow::Continue);
    }
    let milliseconds = (b::environment(&inv).clock.now() * 1000.0) as i64;
    res.headers_mut()?.set(
        "Content-Disposition",
        &format!("attachment; filename=account-{milliseconds}.csv"),
    )?;
    // Keep native Dates distinct until the source typeof-object CSV branch runs.
    // The approved ordered Record retains both outer and nested field order.
    let ordered = b::source(
        profile
            .result()
            .ok_or_else(|| stackpress_lib::Error::invalid("profile result is missing"))?,
    )?;
    res.set(
        "text/csv",
        stackpress_lib::Body::Text(csv(ordered
            .as_object()
            .ok_or_else(|| stackpress_lib::Error::invalid("profile is not an object"))?)?),
    )?;
    res.headers_mut()?.set("Content-Type", "text/csv")?;
    Ok(Flow::Continue)
}
