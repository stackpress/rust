//! Explicit generated-SQL-style events for source account processes.
//!
//! These are ordinary application-selected events. No Engine means pass-through;
//! they neither install schemas nor add authorization/transactions/global guards.
use super::*;
use crate::storage::{AccountActions, PasswordConfig, StoreQuery};
use serde_json::json;
use stackpress_framework_sql::source::{SessionModels, SourceActions, SourceStore};
use stackpress_lib::Flow;
use stackpress_sql::Engine;

/// Source generated events use database.seed default empty, unlike AuthActions.make's abc123.
fn actions<M: SessionModels>(inv: &Invocation<'_>) -> Option<AccountActions<M>> {
    let engine = inv.shared_service::<Engine>().ok()?;
    let env = b::environment(inv);
    Some(AccountActions::new(
        engine,
        env.database_seed.unwrap_or_default(),
        env.offset_minutes,
        PasswordConfig::default(),
    ))
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
pub async fn auth_create_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
        return Ok(Flow::Continue);
    };
    match actions.auth().create(&b::input(req)?).await {
        Ok(row) => b::results(res, Value::Object(row))?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Source generated search adapter with original filters, columns and missing-engine behavior.
/// # Errors
/// Preserves decoding/action errors; no page or global policy is applied.
pub async fn auth_search_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
        return Ok(Flow::Continue);
    };
    let query: StoreQuery = serde_json::from_value(serde_json::to_value(b::input(req)?)?)?;
    let auth = actions.auth();
    // Keep the existing String-q decoder before transferring to the app's query type.
    let query = auth.store().query(&query.native().to_record());
    let rows = match query {
        Ok(query) => auth.find_all(&query).await,
        Err(error) => Err(error),
    };
    match rows {
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
pub async fn auth_update_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    let auth = actions.auth();
    let query = auth
        .store()
        .query(&StoreQuery::eq("id", id).native().to_record());
    let rows = match query {
        Ok(query) => auth.update(&query, &mut b::input(req)?).await,
        Err(error) => Err(error),
    };
    match rows {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Source generated removal selects only ID; extra request fields do not add predicates.
/// # Errors
/// Preserves source action errors and native response failures.
pub async fn auth_remove_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    let auth = actions.auth();
    let query = auth
        .store()
        .query(&StoreQuery::eq("id", id).native().to_record());
    let rows = match query {
        Ok(query) => auth.remove(&query).await,
        Err(error) => Err(error),
    };
    match rows {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Profile detail preserves eq extensions, selected columns, ID validation and Not Found error.
/// # Errors
/// Missing/invalid ID is source 400; missing row is authored 404; native failures propagate.
pub async fn profile_detail_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
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
    let profile = actions.profile();
    let query = profile.store().query(&query.native().to_record());
    let row = match query {
        Ok(query) => profile.find(&query).await,
        Err(error) => Err(error),
    };
    match row {
        Ok(Some(row)) => {
            // The generated store retains SQL selector and nested-field insertion order.
            b::results(res, Value::Object(row))?;
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
pub async fn profile_update_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    let profile = actions.profile();
    let query = profile
        .store()
        .query(&StoreQuery::eq("id", id).native().to_record());
    let rows = match query {
        Ok(query) => profile.update(&query, &mut b::input(req)?).await,
        Err(error) => Err(error),
    };
    match rows {
        Ok(rows) => first(res, rows)?,
        Err(error) => b::action_error(res, error)?,
    }
    Ok(Flow::Continue)
}
/// Soft-remove a profile using the source ID-only generated event filter.
/// # Errors
/// Preserves source action and response errors.
pub async fn profile_remove_with<M: SessionModels>(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    if b::completed(res) {
        return Ok(Flow::Continue);
    }
    let Some(actions) = actions::<M>(&inv) else {
        return Ok(Flow::Continue);
    };
    let Some(id) = checked_id(req, res)? else {
        return Ok(Flow::Continue);
    };
    let profile = actions.profile();
    let query = profile
        .store()
        .query(&StoreQuery::eq("id", id).native().to_record());
    let rows = match query {
        Ok(query) => profile.remove(&query).await,
        Err(error) => Err(error),
    };
    match rows {
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

// One inventory expands over both host registration surfaces. A macro avoids allocating
// erased adapter handlers or changing the immediate API's callback/error order.
macro_rules! account_events {
    ($target:expr, $family:ty) => {
        $target.on("auth-search", auth_search_with::<$family>)?;
        $target.on("auth-update", auth_update_with::<$family>)?;
        $target.on("auth-create", auth_create_with::<$family>)?;
        $target.on("auth-remove", auth_remove_with::<$family>)?;
        $target.on("profile-detail", profile_detail_with::<$family>)?;
        $target.on("profile-update", profile_update_with::<$family>)?;
        $target.on("profile-remove", profile_remove_with::<$family>)?;
    };
}

/// Select SQL events immediately for direct composition; all resources and policy stay explicit.
/// Aggregates needing source Listen-phase order use [`register_listen`] instead.
/// # Errors
/// Preserves normal registration errors and multiple-handler cooperation.
pub fn register(app: &mut stackpress_server::ServerBuilder) -> Result<()> {
    register_with::<stackpress_framework_models::Models>(app)
}

/// Register the seven immediate account events over the application's generated family.
/// No schema, engine, policy or global guard is installed.
pub fn register_with<M: SessionModels>(app: &mut stackpress_server::ServerBuilder) -> Result<()> {
    account_events!(app, M);
    Ok(())
}

/// Queue the same seven SQL events for source-order aggregate composition.
///
/// Use [`register`] for immediate direct composition. This function contributes during Listen,
/// after earlier Listen callbacks and before later ones, matching the upstream SQL plugin's
/// model traversal. It selects no schema/Engine, performs no IO and adds no permission gate.
/// # Errors
/// Preserves startup registration errors; handler-registration failures surface during build.
pub fn register_listen(app: &mut stackpress_server::ServerBuilder) -> Result<()> {
    register_listen_with::<stackpress_framework_models::Models>(app)
}

/// Queue the application's account family at the same source Listen phase.
pub fn register_listen_with<M: SessionModels>(
    app: &mut stackpress_server::ServerBuilder,
) -> Result<()> {
    app.on_listen(listen::<M>)
}

/// Expand the shared inventory only when the existing host reaches Listen.
async fn listen<M: SessionModels>(start: &mut stackpress_server::StartupContext) -> Result<Flow> {
    account_events!(start, M);
    Ok(Flow::Continue)
}

// Keep direct callback paths alongside their generic counterparts. The inventory
// above remains the only registrar list; these wrappers select the generated defaults.
macro_rules! default_event {
    ($name:ident, $generic:ident) => {
        /// Direct account callback over the shipped generated default model family.
        pub async fn $name(
            req: &mut Request,
            res: &mut Response,
            inv: Invocation<'_>,
        ) -> Result<Flow> {
            $generic::<stackpress_framework_models::Models>(req, res, inv).await
        }
    };
}
default_event!(auth_create, auth_create_with);
default_event!(auth_search, auth_search_with);
default_event!(auth_update, auth_update_with);
default_event!(auth_remove, auth_remove_with);
default_event!(profile_detail, profile_detail_with);
default_event!(profile_update, profile_update_with);
default_event!(profile_remove, profile_remove_with);
