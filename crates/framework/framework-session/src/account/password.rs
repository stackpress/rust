//! Source password assertions and unguarded sequential credential updates.
use super::*;
use crate::{columns, storage::PasswordConfig};
use serde_json::json;
use stackpress_lib::Flow;

/// Source ordered password errors followed by the independent current/difference check.
/// Reuses the native password assertion implementation; signup-only fields are supplied
/// internally and discarded so the same special-character/UTF-16 rules stay consistent.
pub fn assert_password(secret: &str, current: &str, rules: &PasswordConfig) -> Option<Record> {
    // No engine IO is needed for assertions; the reusable pure rules are evaluated below.
    let mut errors = Record::new();
    if let Some(message) = crate::storage::password_error(secret, rules) {
        errors.insert("secret".into(), message.into());
    }
    if current.is_empty() {
        errors.insert("current".into(), "Current password is required".into());
    } else if current == secret {
        errors.insert(
            "current".into(),
            "Current password must be different from new password".into(),
        );
    }
    (!errors.is_empty()).then_some(errors)
}
/// Change all auth secrets after one matching current hash, ignoring individual updates.
/// # Errors
/// Native dispatch failures propagate; returned update errors do not stop the source loop.
pub async fn password(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let session = inv.service::<Service>()?.load_request(req);
    let data = session.data().await;
    props(res, &inv)?;
    let Some(data) = data else {
        res.status_code(401, Some("Unauthorized".into()))?;
        return Ok(Flow::Continue);
    };
    let data: Record = data
        .clone()
        .into_iter()
        .map(|(k, v)| (k, Value::from(v)))
        .collect();
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
    if req.method.as_str() != "POST" {
        return Ok(Flow::Continue);
    }
    let secret = b::text(req, "secret", "")?;
    let current = b::text(req, "current", "")?;
    if let Some(errors) = assert_password(&secret, &current, &b::config(&inv)?.password) {
        // Source inserts secret then current; retain that field-error message order.
        let message = ["secret", "current"]
            .iter()
            .filter_map(|key| errors.get(*key))
            .map(columns::string)
            .collect::<Vec<_>>()
            .join(", ");
        b::error(
            res,
            &format!("Invalid Parameters: {message}"),
            serde_json::to_value(errors)?,
        )?;
        return Ok(Flow::Continue);
    }
    let auth = b::resolve(
        &inv,
        "auth-search",
        Record::from_iter([("eq".into(), Value::from(json!({"profileId":id(&data)})))]),
    )
    .await?;
    let rows = b::result(&auth)?;
    let Some(rows) = rows.as_array().filter(|rows| !rows.is_empty()) else {
        b::error(res, "No authentication found.", json!({}))?;
        return Ok(Flow::Continue);
    };
    if !super::profile::matches_password(rows, &current.into(), false) {
        b::error(
            res,
            "Invalid Parameters",
            json!({"current":"Current password is incorrect"}),
        )?;
        return Ok(Flow::Continue);
    }
    for row in rows {
        b::resolve(
            &inv,
            "auth-update",
            Record::from_iter([
                ("id".into(), row.get("id").cloned().unwrap_or(Value::Null)),
                ("secret".into(), secret.clone().into()),
            ]),
        )
        .await?;
    }
    flash(res, "Your password has been updated.", None)?;
    res.redirect(&format!("{}/account", base(&inv)?))?;
    Ok(Flow::Continue)
}
