//! Source generated event wrappers: independent handlers, priority zero and Continue.
use super::defaults;
use crate::models::{
    Actions, ApplicationModel, BatchMode, ModelKind, Record, SessionModel, StoreQuery, Value,
};
use stackpress_lib::{
    Body, Flow, HandlerFuture, Invocation, PublicError, Request, Response, Result, handler,
};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::Engine;

/// Source generated event vocabulary (physical delete is a direct action, not an event).
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Batch attempt status rows.
    Batch,
    /// Create one record.
    Create,
    /// Find by ID.
    Detail,
    /// Find by key/value.
    Get,
    /// Truncate the model table.
    Purge,
    /// Soft delete.
    Remove,
    /// Restore including inactive rows.
    Restore,
    /// Search and count.
    Search,
    /// Update by ID.
    Update,
    /// Source ID-first upsert.
    Upsert,
}
impl Event {
    /// Exact source event suffix.
    pub fn name(self) -> &'static str {
        match self {
            Self::Batch => "batch",
            Self::Create => "create",
            Self::Detail => "detail",
            Self::Get => "get",
            Self::Purge => "purge",
            Self::Remove => "remove",
            Self::Restore => "restore",
            Self::Search => "search",
            Self::Update => "update",
            Self::Upsert => "upsert",
        }
    }
}
/// Select all generated API data events through the ordinary Listen phase.
/// No resource is required or acquired until a selected event actually runs.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_listen(listen)?;
    Ok(())
}
/// Install every independent source event; no route or event ownership is claimed.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    for event in [
        Event::Batch,
        Event::Create,
        Event::Detail,
        Event::Get,
        Event::Purge,
        Event::Remove,
        Event::Restore,
        Event::Search,
        Event::Update,
        Event::Upsert,
    ] {
        start.on_priority(
            &format!("application-{}", event.name()),
            0,
            action::<ApplicationModel>(event),
        )?;
        start.on_priority(
            &format!("session-{}", event.name()),
            0,
            action::<SessionModel>(event),
        )?;
    }
    Ok(Flow::Continue)
}
/// Direct event callback, equivalent to the selected registrar and composable under G42.
pub fn action<M: ModelKind>(
    event: Event,
) -> impl for<'a> Fn(
    &'a mut Request,
    &'a mut Response,
    Invocation<'a>,
) -> HandlerFuture<'a, Result<Flow>>
+ Send
+ Sync {
    handler(move |req, res, invocation| {
        Box::pin(async move {
            // The source's body/code guard precedes even the optional database lookup.
            if body_present(res.body())
                || res
                    .status()
                    .map(|status| status.code)
                    .is_some_and(|code| code != 0 && code != 200)
            {
                return Ok(Flow::Continue);
            }
            let Ok(engine) = invocation.shared_service::<Engine>() else {
                return Ok(Flow::Continue);
            };
            let actions = Actions::<M>::new(engine, "");
            let mut data = source(req.data.value())?
                .as_object()
                .cloned()
                .unwrap_or_default();
            let outcome = invoke(event, &actions, &mut data, req, res).await;
            if let Err(error) = outcome {
                res.set_error(PublicError {
                    code: error.code,
                    status: error.status().into(),
                    error: error.error,
                    errors: serde_json::to_value(error.errors)?,
                })?;
            }
            Ok(Flow::Continue)
        })
    })
}
/// Source truthiness guard, retaining text/byte/native body mechanics.
fn body_present(body: &Body) -> bool {
    match body {
        Body::Empty => false,
        Body::Text(text) => !text.is_empty(),
        Body::Bytes(_) => true,
        Body::Value(value) => match value {
            stackpress_lib::Value::Null => false,
            stackpress_lib::Value::Bool(value) => *value,
            stackpress_lib::Value::String(value) => !value.is_empty(),
            stackpress_lib::Value::Number(value) => value.as_f64().is_none_or(|n| n != 0.0),
            _ => true,
        },
    }
}
/// Run one generated wrapper, preserving source error capitalization and request mutations.
async fn invoke<M: ModelKind>(
    event: Event,
    actions: &Actions<M>,
    data: &mut Record,
    req: &mut Request,
    res: &mut Response,
) -> crate::models::Result<()> {
    use crate::models::ActionError;

    let invalid = |name: &str, message: &str| {
        ActionError::new(
            400,
            "Invalid Parameters",
            Record::from_iter([(name.into(), message.into())]),
        )
    };
    match event {
        Event::Create => {
            // Preparation, validation and native return fallback belong to Actions.
            results(res, Some(actions.create(data).await?))?;
        }
        Event::Upsert => {
            // A present missing ID returns null; successful writes mutate the input clock.
            let row = actions.upsert(data).await?;
            copy_updated(data, req)?;
            results(res, row)?;
        }
        Event::Detail | Event::Update | Event::Remove | Event::Restore => {
            // Only absent/null/empty IDs are rejected; other source scalar types flow on.
            let id = data
                .get("id")
                .filter(|value| !value.is_null() && value.as_str() != Some(""))
                .cloned()
                .ok_or_else(|| invalid("id", "Missing or invalid value"))?;
            // Detail only reads eq/columns; other request query fields are irrelevant.
            let mut query = StoreQuery {
                eq: if matches!(event, Event::Detail) {
                    data.get("eq")
                        .and_then(Value::as_object)
                        .cloned()
                        .unwrap_or_default()
                } else {
                    Record::new()
                },
                columns: defaults::selected(data.get("columns"))
                    .unwrap_or_else(defaults::columns::<M>),
                ..Default::default()
            };
            if matches!(event, Event::Detail) && data.get("eq").is_some_and(Value::is_object) {
                // req.data.path returns the shared object upstream; only a present object
                // receives this mutation. The default temporary {} stays detached.
                req.data
                    .set("eq.id", native(&id).map_err(wire)?)
                    .map_err(wire)?;
            }
            query.eq.insert("id".into(), id);
            // Restore explicitly removes the base active predicate; ordinary updates do not.
            if matches!(event, Event::Restore) {
                query.eq.insert("active".into(), (-1).into());
            }
            match event {
                Event::Detail => {
                    let row = actions
                        .find(&query)
                        .await?
                        .ok_or_else(|| ActionError::new(404, "Not Found", Record::new()))?;
                    results(res, Some(row))?;
                }
                Event::Update => {
                    let rows = actions
                        .update(
                            &StoreQuery {
                                columns: vec!["*".into()],
                                ..query
                            },
                            data,
                        )
                        .await?;
                    if let Some(value) = data.get("updated") {
                        req.data
                            .set("updated", native(value).map_err(wire)?)
                            .map_err(wire)?;
                    }
                    results(res, rows.into_iter().next())?;
                }
                Event::Remove | Event::Restore => {
                    query.columns = vec!["*".into()];
                    let rows = if matches!(event, Event::Remove) {
                        actions.remove(&query).await?
                    } else {
                        actions.restore(&query).await?
                    };
                    results(res, rows.into_iter().next())?;
                }
                _ => unreachable!(),
            }
        }
        Event::Get => {
            // Get tests presence only, preserving explicit null and unknown property keys.
            let mut errors = Record::new();
            if !data.contains_key("value") {
                errors.insert("value".into(), "Missing or invalid value".into());
            }
            if !data.contains_key("key") {
                errors.insert("key".into(), "Missing or invalid key".into());
            }
            if !errors.is_empty() {
                return Err(ActionError::new(400, "Invalid Parameters", errors));
            }
            // Computed JS property keys are stringified, including explicit null.
            let key = crate::models::source_text(&data["key"], 0);
            let mut query = StoreQuery::eq(key, data["value"].clone());
            // Invalid supplied selectors fall back to base *, while omission uses @query.
            query.columns = if data.contains_key("columns") {
                defaults::selected(data.get("columns")).unwrap_or_else(|| vec!["*".into()])
            } else {
                defaults::columns::<M>()
            };
            let row = actions
                .find(&query)
                .await?
                .ok_or_else(|| ActionError::new(404, "Not Found", Record::new()))?;
            results(res, Some(row))?;
        }
        Event::Search => {
            // Search changes the original columns before both independent read/count calls.
            let columns =
                defaults::selected(data.get("columns")).unwrap_or_else(defaults::columns::<M>);
            req.data
                .set(
                    "columns",
                    columns
                        .iter()
                        .cloned()
                        .map(stackpress_lib::Value::from)
                        .collect::<Vec<_>>(),
                )
                .map_err(wire)?;
            data.insert(
                "columns".into(),
                Value::from(serde_json::to_value(columns).map_err(wire)?),
            );
            let query = defaults::query(data)?;
            // The source does not wrap these reads in an implicit transaction or retry.
            let found = actions.find_all(&query).await?;
            let total = actions.count(&query).await?;
            rows(res, found.into_iter().map(Value::Object).collect(), total)?;
        }
        Event::Batch => {
            // Validate the outer array only; item failures are reported and rolled back inside.
            let Some(inputs) = data.get("rows").and_then(Value::as_array) else {
                return Err(invalid("rows", "Missing or invalid value"));
            };
            let mut inputs = inputs.clone();
            let mode = match data.get("mode").and_then(Value::as_str).unwrap_or("upsert") {
                "create" => BatchMode::Create,
                "update" => BatchMode::Update,
                "upsert" => BatchMode::Upsert,
                // The source branch ladder does nothing for an unknown mode.
                _ => {
                    let _ = actions.batch(&mut [], BatchMode::Upsert).await;
                    res.rows(&Vec::<Value>::new(), 0).map_err(wire)?;
                    return Ok(());
                }
            };
            // The awaited batch retains the exact transaction until commit/rollback/drop.
            let statuses = actions.batch_values(&mut inputs, mode).await;
            req.data
                .set(
                    "rows",
                    inputs
                        .iter()
                        .map(native)
                        .collect::<Result<Vec<_>>>()
                        .map_err(wire)?,
                )
                .map_err(wire)?;
            // Item success is code 200, including null upserts; missing selectors have no code.
            let code = if statuses.iter().all(|status| status.code == Some(200)) {
                200
            } else {
                400
            };
            // Reinsert native results after status serialization; the final row serializer
            // then selects source ISO Date wire text for the existing HTTP/JSON boundary.
            let values = statuses
                .iter()
                .map(|status| {
                    let mut value = Value::from(serde_json::to_value(status).map_err(wire)?);
                    if let Some(result) = &status.results
                        && let Some(object) = value.as_object_mut()
                    {
                        object.insert("results".into(), result.clone());
                    }
                    Ok(value)
                })
                .collect::<crate::models::Result<Vec<_>>>()?;
            rows(res, values, statuses.len() as u64)?;
            res.status_code(
                code,
                Some(if code == 200 { "OK" } else { "Bad Request" }.into()),
            )
            .map_err(wire)?;
        }
        Event::Purge => {
            // Source sets its table text even after reporting a truncate failure.
            if let Err(error) = actions.purge(false).await {
                res.set_error(PublicError {
                    code: error.code,
                    status: error.status().into(),
                    error: error.error,
                    errors: serde_json::to_value(error.errors).map_err(wire)?,
                })
                .map_err(wire)?;
            }
            res.set("text/plain", Body::Text(M::TABLE.into()))
                .map_err(wire)?;
        }
    }
    Ok(())
}

/// Preserve typed native failures under the source exception shape.
fn wire(error: impl std::fmt::Display) -> crate::models::ActionError {
    crate::models::ActionError::new(500, error.to_string(), Record::new())
}

/// Preserve native Date values in source request mutations, including nested batch rows.
fn native(value: &Value) -> Result<stackpress_lib::Value> {
    Ok(match value {
        Value::Date { millis, .. } => stackpress_lib::Timestamp::new(
            millis.div_euclid(1000),
            (millis.rem_euclid(1000) as u32) * 1_000_000,
        )?
        .into(),
        Value::Array(values) => {
            stackpress_lib::Value::Array(values.iter().map(native).collect::<Result<_>>()?)
        }
        Value::Object(values) => stackpress_lib::Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), native(value)?)))
                .collect::<Result<_>>()?,
        ),
        _ => stackpress_lib::Value::try_from(serde_json::to_value(value)?)?,
    })
}
/// Only successful update/upsert writes add the source timestamp to the original request.
fn copy_updated(data: &Record, req: &mut Request) -> crate::models::Result<()> {
    if let Some(value) = data.get("updated") {
        req.data
            .set("updated", native(value).map_err(wire)?)
            .map_err(wire)?;
    }
    Ok(())
}

/// The existing JSON/HTTP Response boundary requires explicit Date wire conversion.
/// Model values select source Date.toJSON ISO text; direct Actions keep native Dates.
fn results(res: &mut Response, row: Option<Record>) -> crate::models::Result<()> {
    res.results(&row.map(Value::Object).unwrap_or(Value::Null))
        .map_err(wire)?;
    Ok(())
}
/// Source row counts include zero; model serialization supplies explicit Date wire text.
fn rows(res: &mut Response, values: Vec<Value>, total: u64) -> crate::models::Result<()> {
    res.rows(&values, total).map_err(wire)?;
    Ok(())
}
/// Convert the host's native Date carrier back into source model values before preparation.
fn source(value: &stackpress_lib::Value) -> Result<Value> {
    Ok(match value {
        stackpress_lib::Value::DateTime(time) => {
            let millis = time
                .seconds
                .checked_mul(1000)
                .and_then(|n| n.checked_add(i64::from(time.nanoseconds() / 1_000_000)))
                .ok_or_else(|| {
                    stackpress_lib::Error::invalid("source Date exceeds millisecond representation")
                })?;
            crate::models::date(millis)
        }
        stackpress_lib::Value::Array(values) => {
            Value::Array(values.iter().map(source).collect::<Result<_>>()?)
        }
        stackpress_lib::Value::Object(values) => Value::Object(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), source(value)?)))
                .collect::<Result<_>>()?,
        ),
        _ => Value::from(value.to_json()?),
    })
}
