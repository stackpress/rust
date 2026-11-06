//! Native SQLite commands over actual generated scripts and the explicit durable History service.
//!
//! Full CompiledClient/UI generation is not needed for these ordinary app command bindings.
//! No empty client is fabricated: install/purge use the real generated eight-model definitions.
//! Framework functions retain all execution/transaction/warning policy. This plugin supplies IO.

use serde_json::{Value as Json, json};
use stackpress_framework::{
    commands::{
        Bindings,
        sql::{self, History, Scripts},
    },
    config::{ClientOptions, DatabaseOptions, Environment},
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, Value};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::{Engine, QueryResult, SqlValue};
use std::sync::Arc;

/// Capture complete compiled source metadata and its actual generated native table definitions.
/// # Errors
/// Retains metadata/DDL/codec errors before any database or history IO.
pub fn current() -> Result<Json> {
    let catalog = crate::generated::source::_catalog::describe();
    let tables = crate::plugins::store::tables()?;
    // Keep source catalog order in revisions, distinct from dependency order for installation.
    let tables = catalog
        .tables
        .iter()
        .map(|identity| {
            tables
                .iter()
                .find(|table| table.name == identity.name)
                .cloned()
                .ok_or_else(|| Error::new("generated source table definition missing"))
        })
        .collect::<Result<Vec<_>>>()?;
    crate::history::snapshot(serde_json::to_value(catalog)?, &tables)
}

/// Build scripts from native generated definitions, without evaluating Idea or UI code.
fn scripts() -> Result<Scripts> {
    let schema = current()?;
    let tables = crate::history::definitions(schema.clone())?;
    let tables = crate::history::install_order(&tables)?;
    Scripts::from_tables(&tables, stackpress_sql::DialectId::Sqlite, schema)
}

/// Select commands and explicit native resources. Registration does not read/write history.
/// # Errors
/// Retains ordinary callback/listener registration failures.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(configure)?;
    app.on_listen(listen)
}

/// Supply native command resources for the full aggregate, which owns the SQL inventory.
/// # Errors
/// Retains registration errors; adds only the explicit app revision event, not duplicate SQL.
pub fn register_application(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(configure)?;
    app.on_listen(listen_revision)
}

/// Native revision is the app-owned explicit generation/checkpoint adaptation.
async fn listen_revision(start: &mut StartupContext) -> Result<Flow> {
    start.on("revision", command)?;
    Ok(Flow::Continue)
}

/// Publish compiled scripts and query output; no SQL statement is executed here.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(scripts()?))?;
    let mut bindings = Bindings::default();
    bindings.bind("output", output);
    start.register(Arc::new(bindings))?;
    Ok(Flow::Continue)
}

/// Preserve the source SQL event inventory; revision is the explicit native snapshot adapter.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    for event in [
        "install",
        "migrate",
        "purge",
        "push",
        "query",
        "populate",
        "uninstall",
        "upgrade",
        "revision",
    ] {
        start.on(event, command)?;
    }
    Ok(Flow::Continue)
}

/// Source truthy optional path handling, retaining explicit invalid native string errors.
fn path(value: Option<&Json>) -> Result<Option<&str>> {
    match value {
        None | Some(Json::Null) | Some(Json::Bool(false)) => Ok(None),
        Some(Json::Number(number)) if number.as_f64() == Some(0.0) => Ok(None),
        Some(Json::String(text)) if text.is_empty() => Ok(None),
        Some(value) => value.as_str().map(Some).ok_or_else(|| {
            Error::invalid("source option requires a string at this native boundary")
        }),
    }
}

/// Dispatch source direct functions; required resources remain lazy and missing providers fail.
async fn command(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Source SQL scripts skip an earlier nonzero/non-200 response; a body alone is not a guard.
    if res
        .status()
        .is_some_and(|status| status.code != 0 && status.code != 200)
    {
        return Ok(Flow::Continue);
    }
    let event = inv.event();
    if matches!(event, "query" | "populate") {
        return sql::event(req, res, inv).await;
    }
    if event == "revision" {
        let root = path(inv.config::<ClientOptions>()?.0.get("revisions"))?;
        if let Some(root) = root {
            (inv.service::<History>()?.insert)(root, &current()?)?;
        }
    } else {
        // Delegate the framework's established no-Engine response rather than inventing one.
        let Ok(engine) = inv.shared_service::<Engine>() else {
            return sql::event(req, res, inv).await;
        };
        // Match the source event's missing-Engine precedence before interpreting paths.
        let root = path(inv.config::<ClientOptions>()?.0.get("revisions"))?;
        let force = inv
            .adapter_context::<stackpress_cli::CliContext>()
            .map_or_else(
                || {
                    inv.service::<Environment>()
                        .is_ok_and(|environment| environment.force)
                },
                |cli| cli.force,
            );
        match event {
            "migrate" => {
                let destination = path(inv.config::<DatabaseOptions>()?.0.get("migrations"))?;
                if root.is_some() && destination.is_some() {
                    sql::migrate(&engine, root, destination, inv.service::<History>()?)?;
                }
            }
            "upgrade" => {
                if root.is_some() {
                    sql::upgrade(&engine, root, inv.service::<History>()?, force).await?;
                }
            }
            "push" => {
                if root.is_some() {
                    sql::push(&engine, root, inv.service::<History>()?, force, || {
                        inv.shared_service::<Scripts>()
                    })
                    .await?;
                }
            }
            "install" => {
                let scripts = inv.shared_service::<Scripts>()?;
                let revisions = root
                    .map(|root| inv.service::<History>().map(|history| (root, history)))
                    .transpose()?;
                sql::install(&engine, &scripts, revisions).await?;
            }
            "purge" => sql::purge(&engine, inv.shared_service::<Scripts>()?.as_ref()).await?,
            "uninstall" => {
                sql::uninstall(&engine, inv.shared_service::<Scripts>()?.as_ref()).await?
            }
            _ => return Err(Error::new("unknown blog SQL command")),
        }
    }
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}

/// Convert owned native query cells only at the CLI output boundary.
fn cell(value: &SqlValue) -> Json {
    match value {
        SqlValue::Null => Json::Null,
        SqlValue::Bool(v) => json!(v),
        SqlValue::Integer(v) => json!(v),
        SqlValue::Real(v) => json!(v),
        SqlValue::Text(v) => json!(v),
        SqlValue::Blob(v) => json!(v),
    }
}

/// Present the real statement-local native QueryResult, never a debug string or canned success.
async fn output(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let result = req
        .data
        .get("result")
        .and_then(Value::as_shared)
        .and_then(|native| native.downcast_ref::<QueryResult>())
        .ok_or_else(|| Error::new("query output requires a native QueryResult"))?;
    let rows = result
        .rows
        .iter()
        .map(|row| {
            row.cells()
                .iter()
                .map(|(name, value)| (name.clone(), cell(value)))
                .collect::<serde_json::Map<_, _>>()
        })
        .collect::<Vec<_>>();
    res.results(&json!({"rows":rows,"affected":result.affected,"insertId":result.insert_id}))?;
    Ok(Flow::Continue)
}
