//! Source removal checks and soft-delete sequencing; no outer transaction or CSRF gate.
use super::*;
use crate::columns;
use serde_json::json;
use stackpress_lib::Flow;

/// Verify current password, then remove auth records in order before the profile.
/// # Errors
/// Stops at the first returned removal error, retaining earlier soft removals.
pub async fn remove(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
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
    if req.method.as_str() == "POST" && columns::truthy(&b::get(req, "confirmed", Value::Null)?) {
        let secret = b::get(req, "secret", Value::Null)?;
        let auth = b::resolve(
            &inv,
            "auth-search",
            Record::from_iter([("eq".into(), Value::from(json!({"profileId":id(&data)})))]),
        )
        .await?;
        if !b::ok(&auth) {
            res.apply_envelope(auth.to_envelope())?;
            return Ok(Flow::Continue);
        }
        if !columns::truthy(&secret) {
            b::error(
                res,
                "Invalid Parameters",
                json!({"secret":"Password is required"}),
            )?;
            return Ok(Flow::Continue);
        }
        let rows = b::result(&auth)?;
        let Some(rows) = rows.as_array().filter(|rows| !rows.is_empty()) else {
            b::error(res, "No authentication found.", json!({}))?;
            return Ok(Flow::Continue);
        };
        if !super::profile::matches_password(rows, &secret, false) {
            b::error(
                res,
                "Invalid Parameters",
                json!({"secret":"Password is incorrect"}),
            )?;
            return Ok(Flow::Continue);
        }
        for row in rows {
            let removed = b::resolve(
                &inv,
                "auth-remove",
                Record::from_iter([("id".into(), row.get("id").cloned().unwrap_or(Value::Null))]),
            )
            .await?;
            if !b::ok(&removed) {
                res.apply_envelope(removed.to_envelope())?;
                return Ok(Flow::Continue);
            }
        }
        resolve_into(
            &inv,
            "profile-remove",
            Record::from_iter([("id".into(), id(&data))]),
            res,
        )
        .await?;
        if b::ok(res) {
            flash(res, "Your account has been scheduled for removal.", None)?;
            res.redirect(&format!("{}/signout", base(&inv)?))?;
            return Ok(Flow::Continue);
        }
    }
    props(res, &inv)?;
    Ok(Flow::Continue)
}
