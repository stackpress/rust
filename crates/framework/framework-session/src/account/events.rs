//! Explicit generated-SQL-style events for source account processes.
//!
//! These are ordinary application-selected events. No Engine means pass-through;
//! they neither install schemas nor add authorization/transactions/global guards.
use super::*;
use crate::storage::{AuthActions, StoreQuery};
use serde_json::json;
use stackpress_lib::Flow;
use stackpress_sql::Engine;

/// Source generated events use database.seed default empty, unlike AuthActions.make's abc123.
fn actions(inv: &Invocation<'_>) -> Option<AuthActions> {
    let engine = inv.shared_service::<Engine>().ok()?;
    let env = b::environment(inv);
    let mut actions = AuthActions::new(engine, env.database_seed.unwrap_or_default());
    actions.store.schema.offset = env.offset_minutes;
    Some(actions)
}
/// ID validation precedes IO; false/zero are retained rather than truthy-rejected.
fn checked_id(req: &Request, res: &mut Response) -> Result<Option<Value>> {
    let id = req.data.get("id").map(b::source).transpose()?;
    if id
        .as_ref()
        .is_none_or(|id| id.is_null() || id.as_str() == Some(""))
    {
        b::error(
            res,
            "Invalid Parameters",
            json!({"id":"Missing or invalid value"}),
        )?;
        Ok(None)
    } else {
        Ok(id)
    }
}
/// Source generated create returns its original create-row codec branch.
/// # Errors
/// Action failures become public responses; native input/host errors propagate.
pub async fn auth_create(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    match actions.create(&b::input(req)?).await {
        Ok(row) => b::results(res, Value::Object(row))?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Source generated search adapter with original filters, columns and missing-engine behavior.
/// # Errors
/// Preserves decoding/action errors; no page or global policy is applied.
pub async fn auth_search(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    let query: StoreQuery = serde_json::from_value(serde_json::to_value(b::input(req)?)?)?;
    match actions.find_all(&query).await {
        Ok(rows) => b::results(
            res,
            Value::Array(rows.into_iter().map(Value::Object).collect()),
        )?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Source generated update validates ID, writes aggregate input and returns its first pre-read row.
/// # Errors
/// Preserves source action/response failures, including partial writes.
pub async fn auth_update(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    match actions
        .update(&StoreQuery::eq("id", id), &mut b::input(req)?)
        .await
    {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Source generated removal selects only ID; extra request fields do not add predicates.
/// # Errors
/// Preserves source action errors and native response failures.
pub async fn auth_remove(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    match actions.remove(&StoreQuery::eq("id", id)).await {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Profile detail preserves eq extensions, selected columns, ID validation and Not Found error.
/// # Errors
/// Missing/invalid ID is source 400; missing row is authored 404; native failures propagate.
pub async fn profile_detail(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    let mut query = StoreQuery::eq("id", id.clone());
    if let Some(Value::Object(eq)) = b::input(req)?.get("eq") {
        query.eq = eq.clone();
        query.eq.insert("id".into(), id);
    }
    if let Some(columns) = req
        .data
        .get("columns")
        .and_then(stackpress_lib::Value::as_array)
        .filter(|columns| {
            columns
                .iter()
                .all(|column| column.as_str().is_some_and(|value| !value.is_empty()))
        })
    {
        query.columns = columns
            .iter()
            .map(|column| column.as_str().unwrap().into())
            .collect();
    }
    match actions.profile().find(&query).await {
        Ok(Some(row)) => {
            // SQL selector order defines the generated source record insertion order.
            let selectors = actions.profile().store.selectors(&query.columns);
            let mut ordered = stackpress_lib::DataMap::new();
            for selector in selectors {
                if let Some(value) = row.get(&selector.field) {
                    ordered.insert(selector.field, b::native(value.clone())?);
                }
            }
            for (key, value) in row {
                if !ordered.contains_key(&key) {
                    ordered.insert(key, b::native(value)?);
                }
            }
            res.results_value(stackpress_lib::Value::Object(ordered))?;
        }
        Ok(None) => {
            b::error(res, "Not Found", json!({}))?;
            res.status_code(404, None)?;
        }
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Update source profile fields after ID admission; the original full input remains the patch.
/// # Errors
/// Preserves source action/native failures without a new policy or transaction.
pub async fn profile_update(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    match actions
        .profile()
        .update(&StoreQuery::eq("id", id), &mut b::input(req)?)
        .await
    {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Soft-remove a profile using the source ID-only generated event filter.
/// # Errors
/// Preserves source action and response errors.
pub async fn profile_remove(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    match actions.profile().remove(&StoreQuery::eq("id", id)).await {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Single-record events retain null on an empty Actions row array.
fn first(res: &mut Response, rows: Vec<Record>) -> Result<()> {
    b::results(
        res,
        rows.into_iter()
            .next()
            .map(Value::Object)
            .unwrap_or(Value::Null),
    )
}

/// Select SQL events only; schemas, resources, routes and permission policy remain explicit.
/// # Errors
/// Preserves normal registration errors and multiple-handler cooperation.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> Result<()> {
    app.on("auth-search", auth_search)?;
    app.on("auth-update", auth_update)?;
    app.on("auth-create", auth_create)?;
    app.on("auth-remove", auth_remove)?;
    app.on("profile-detail", profile_detail)?;
    app.on("profile-update", profile_update)?;
    app.on("profile-remove", profile_remove)?;
    Ok(())
}
