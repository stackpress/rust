//! Source event wrappers; no OperationPolicy, global guard, route ownership or startup IO.
use super::{
    ActionError, BatchMode, BatchStatus, NativeActions, SourceActions, SourceModel, SourceStore,
    StoreQuery,
};
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
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
/// Register source model events during Listen; every handler remains independently composable.
/// This registrar calls the model family's associated actions. Dynamic batch inputs
/// use the additional [`SourceEventActions`] contract; no action override is discarded.
/// # Errors
/// Preserves builder errors; no Engine or IO is required at registration/startup.
pub fn register<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    app: &mut ServerBuilder,
) -> Result<()> {
    register_with::<M>(app, ResponseValues::Json)
}
/// Choose the in-process response boundary explicitly; ordinary registration stays JSON.
#[derive(Clone, Copy, Debug, Default)]
pub enum ResponseValues {
    /// Source Date.toJSON wire text, matching the existing public event surface.
    #[default]
    Json,
    /// Native Dates for nested source processes; the final host owns JSON serialization.
    Native,
}
/// Queue the same inventory with an explicit response representation, without IO or policy.
pub fn register_with<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    app: &mut ServerBuilder,
    values: ResponseValues,
) -> Result<()> {
    app.on_listen(stackpress_lib::queue::task(
        move |start: &mut StartupContext| Box::pin(async move { listen::<M>(start, values).await }),
    ))
}
/// Queue the same catalog inventory in source Listen order; no await or resource acquisition.
async fn listen<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    start: &mut StartupContext,
    values: ResponseValues,
) -> Result<Flow> {
    let model = M::Schema::describe();
    let prefix = event_prefix(&model.name, &model.module, &M::Schema::catalog());
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
        if matches!(event, Event::Detail | Event::Update | Event::Remove)
            && !model.fields.iter().any(|f| f.key)
        {
            continue;
        }
        if matches!(event, Event::Restore) && !model.fields.iter().any(|f| f.active) {
            continue;
        }
        start.on_priority(
            &format!("{prefix}-{}", event.name()),
            0,
            action_bound::<M>(event, None, values),
        )?;
    }
    Ok(Flow::Continue)
}
/// Source event spelling follows source names, independent of Rust keyword/module escaping.
fn event_prefix(
    name: &str,
    fallback: &str,
    catalog: &stackpress_framework_schema::SourceCatalog,
) -> String {
    catalog
        .tables
        .iter()
        .find(|table| table.model == name)
        .map(|table| table.name.as_str())
        .unwrap_or(fallback)
        .replace('_', "-")
}

#[cfg(test)]
mod naming_tests {
    use super::*;
    #[test]
    fn source_event_names_do_not_inherit_rust_keyword_escapes() {
        let catalog = stackpress_framework_schema::SourceCatalog {
            tables: vec![stackpress_framework_schema::SourceTable {
                model: "Type".into(),
                name: "type".into(),
            }],
            ..Default::default()
        };
        assert_eq!(event_prefix("Type", "type_", &catalog), "type");
        assert_eq!(
            event_prefix("Manual", "manual_entry", &catalog),
            "manual-entry"
        );
    }
}
/// Explicit native environment for source event codecs; absent values retain source empty-seed/UTC mechanics.
#[derive(Clone, Debug, Default)]
pub struct SourceEnvironment {
    /// Application's database seed; never placed in request/response data.
    pub seed: String,
    /// Explicit local Date offset shared with direct actions.
    pub offset_minutes: i32,
}
/// Direct event callback, equivalent to the selected registrar and composable under G42.
/// Direct callback with source environment read at invocation, as before.
pub fn action<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    event: Event,
) -> impl for<'a> Fn(
    &'a mut Request,
    &'a mut Response,
    Invocation<'a>,
) -> HandlerFuture<'a, Result<Flow>>
+ Send
+ Sync {
    action_bound::<M>(event, None, ResponseValues::Json)
}
/// Direct callback with explicit immutable codec options; useful for source APIs whose
/// constructor always supplies empty seed/UTC instead of reading an ambient model environment.
pub fn action_with<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    event: Event,
    environment: SourceEnvironment,
) -> impl for<'a> Fn(
    &'a mut Request,
    &'a mut Response,
    Invocation<'a>,
) -> HandlerFuture<'a, Result<Flow>>
+ Send
+ Sync {
    action_bound::<M>(event, Some(environment), ResponseValues::Json)
}
fn action_bound<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    event: Event,
    explicit: Option<SourceEnvironment>,
    values: ResponseValues,
) -> impl for<'a> Fn(
    &'a mut Request,
    &'a mut Response,
    Invocation<'a>,
) -> HandlerFuture<'a, Result<Flow>>
+ Send
+ Sync {
    handler(move |req, res, invocation| {
        let explicit = explicit.clone();
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
            let environment = explicit.clone().unwrap_or_else(|| {
                invocation
                    .service::<SourceEnvironment>()
                    .cloned()
                    .unwrap_or_default()
            });
            let actions = M::actions(engine, environment.seed, environment.offset_minutes);
            let mut data = source(req.data.value())?
                .as_object()
                .cloned()
                .unwrap_or_default();
            let outcome = invoke::<M>(event, &actions, &mut data, req, res, values).await;
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
async fn invoke<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    event: Event,
    actions: &M::Actions,
    data: &mut Record,
    req: &mut Request,
    res: &mut Response,
    representation: ResponseValues,
) -> super::SourceResult<()> {
    let model = M::Schema::describe();

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
            results(res, Some(actions.create(data).await?), representation)?;
        }
        Event::Upsert => {
            // A present missing ID returns null; successful writes mutate the input clock.
            let row = actions.upsert(data).await?;
            copy_updated(data, req, &model)?;
            results(res, row, representation)?;
        }
        Event::Detail | Event::Update | Event::Remove | Event::Restore => {
            // Only absent/null/empty IDs are rejected; other source scalar types flow on.
            let mut keys = Record::new();
            for field in model.fields.iter().filter(|f| f.key) {
                let value = data
                    .get(&field.name)
                    .filter(|v| !v.is_null() && v.as_str() != Some(""))
                    .cloned()
                    .ok_or_else(|| invalid(&field.name, "Missing or invalid value"))?;
                keys.insert(field.name.clone(), value);
            }
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
                columns: super::event_defaults::selected(data.get("columns"))
                    .unwrap_or_else(super::event_defaults::columns::<M>),
                ..Default::default()
            };
            if matches!(event, Event::Detail) && data.get("eq").is_some_and(Value::is_object) {
                for (key, value) in &keys {
                    req.data
                        .set(&format!("eq.{key}"), native(value).map_err(wire)?)
                        .map_err(wire)?;
                }
            }
            query.eq.extend(keys);
            if matches!(event, Event::Restore) {
                for f in model.fields.iter().filter(|f| f.active) {
                    query.eq.insert(f.name.clone(), (-1).into());
                }
            }
            match event {
                Event::Detail => {
                    let row = actions
                        .find(&model_query::<M>(actions, &query)?)
                        .await?
                        .ok_or_else(|| ActionError::new(404, "Not Found", Record::new()))?;
                    results(res, Some(row), representation)?;
                }
                Event::Update => {
                    let rows = actions
                        .update(
                            &model_query::<M>(
                                actions,
                                &StoreQuery {
                                    columns: vec!["*".into()],
                                    ..query
                                },
                            )?,
                            data,
                        )
                        .await?;
                    copy_updated(data, req, &model)?;
                    results(res, rows.into_iter().next(), representation)?;
                }
                Event::Remove | Event::Restore => {
                    query.columns = vec!["*".into()];
                    let rows = if matches!(event, Event::Remove) {
                        actions.remove(&model_query::<M>(actions, &query)?).await?
                    } else {
                        actions.restore(&model_query::<M>(actions, &query)?).await?
                    };
                    results(res, rows.into_iter().next(), representation)?;
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
            let key = super::text::source_text(&data["key"], 0);
            let mut query = StoreQuery::eq(key, data["value"].clone());
            // Invalid supplied selectors fall back to base *, while omission uses @query.
            query.columns = if data.contains_key("columns") {
                super::event_defaults::selected(data.get("columns"))
                    .unwrap_or_else(|| vec!["*".into()])
            } else {
                super::event_defaults::columns::<M>()
            };
            let row = actions
                .find(&model_query::<M>(actions, &query)?)
                .await?
                .ok_or_else(|| ActionError::new(404, "Not Found", Record::new()))?;
            results(res, Some(row), representation)?;
        }
        Event::Search => {
            // Search changes the original columns before both independent read/count calls.
            let columns = super::event_defaults::selected(data.get("columns"))
                .unwrap_or_else(super::event_defaults::columns::<M>);
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
            let query = actions.store().query(data)?;
            // The source does not wrap these reads in an implicit transaction or retry.
            let found = actions.find_all(&query).await?;
            let total = actions.count(&query).await?;
            rows(
                res,
                found.into_iter().map(Value::Object).collect(),
                total,
                representation,
            )?;
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
                .collect::<super::SourceResult<Vec<_>>>()?;
            rows(res, values, statuses.len() as u64, representation)?;
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
            res.set("text/plain", Body::Text(model_table::<M>()))
                .map_err(wire)?;
        }
    }
    Ok(())
}

/// Preserve typed native failures under the source exception shape.
fn wire(error: impl std::fmt::Display) -> super::ActionError {
    super::ActionError::new(500, error.to_string(), Record::new())
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
fn copy_updated(
    data: &Record,
    req: &mut Request,
    model: &stackpress_framework_schema::Model,
) -> super::SourceResult<()> {
    for field in model.fields.iter().filter(|field| field.timestamp) {
        if let Some(value) = data.get(&field.name) {
            req.data
                .set(&field.name, native(value).map_err(wire)?)
                .map_err(wire)?;
        }
    }
    Ok(())
}

/// The JSON boundary stays byte-compatible; nested session processes can retain Dates.
fn results(
    res: &mut Response,
    row: Option<Record>,
    values: ResponseValues,
) -> super::SourceResult<()> {
    let value = row.map(Value::Object).unwrap_or(Value::Null);
    match values {
        ResponseValues::Json => {
            res.results(&value).map_err(wire)?;
        }
        ResponseValues::Native => {
            res.results_value(native(&value).map_err(wire)?)
                .map_err(wire)?;
        }
    }
    Ok(())
}
/// Preserve source counts, including zero, in either explicit representation.
fn rows(
    res: &mut Response,
    values: Vec<Value>,
    total: u64,
    representation: ResponseValues,
) -> super::SourceResult<()> {
    match representation {
        ResponseValues::Json => {
            res.rows(&values, total).map_err(wire)?;
        }
        ResponseValues::Native => {
            res.results_value(native(&Value::Array(values)).map_err(wire)?)
                .map_err(wire)?;
            res.set_total(total).map_err(wire)?;
        }
    }
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
            stackpress_framework_schema::source::date(millis)
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

/// Action contract for source generated events, including dynamic primitive/null batch rows.
/// Ordinary direct collaborators can implement SourceActions without selecting server events.
pub trait SourceEventActions: SourceActions<BatchStatus = BatchStatus> {
    /// Same source transaction/status semantics as direct batch, with original dynamic inputs.
    fn batch_values(
        &self,
        input: &mut [Value],
        mode: BatchMode,
    ) -> impl std::future::Future<Output = Vec<BatchStatus>> + Send;
}
impl<S: SourceSchema<Value = Value, Record = Record>> SourceEventActions for NativeActions<S> {
    async fn batch_values(&self, input: &mut [Value], mode: BatchMode) -> Vec<BatchStatus> {
        self.batch_values(input, mode).await
    }
}
/// Construct the associated store's query directly, without a JSON conversion or type cast.
fn model_query<
    M: SourceModel<
            Schema: SourceSchema<Value = Value, Record = Record>,
            Store: SourceStore<Error = ActionError>,
            Actions: SourceEventActions,
        >,
>(
    actions: &M::Actions,
    query: &StoreQuery,
) -> super::SourceResult<<M::Store as SourceStore>::Query> {
    actions.store().query(&query.to_record())
}
/// Generated table spelling for the one source purge response, without a NativeStore downcast.
fn model_table<M: SourceModel>() -> String {
    let model = M::Schema::describe();
    M::Schema::catalog()
        .tables
        .into_iter()
        .find(|table| table.model == model.name)
        .map(|table| table.name)
        .unwrap_or(model.module)
}
