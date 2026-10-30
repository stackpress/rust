//! Source SQL script orchestration over an explicit Engine, compiled scripts and revision provider.
//!
//! A revision provider owns source-schema decoding/planning, not command refusal policy.
//! In particular migrate writes warned plans, while upgrade alone checks terminal.force.

use super::super::{
    client::Client,
    config::{ClientOptions, DatabaseOptions, Environment, path, string, truthy},
};
use serde_json::{Value, json};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerInvocationExt;
use stackpress_sql::{Engine, Query, Render, Sql, TableDefinition};
use std::{path::Path, sync::Arc};

/// Already compiled generated-client scripts and the exact schema used for revision insertion.
pub struct Scripts {
    /// Statements in generated install order.
    pub install: Vec<Query>,
    /// Statements in generated purge order.
    pub purge: Vec<Query>,
    /// Statements in generated uninstall order.
    pub uninstall: Vec<Query>,
    /// Original source schema representation, retained outside browser snapshots.
    pub schema: Value,
}

impl Scripts {
    /// Build real SQL from an app-owned catalog in its generated create order.
    /// # Errors
    /// Preserves backend rendering errors before any Engine acquisition.
    pub fn from_tables(
        tables: &[TableDefinition],
        dialect: stackpress_sql::DialectId,
        schema: Value,
    ) -> Result<Self> {
        let mut install = Vec::new();
        let mut purge = Vec::new();
        let mut uninstall = Vec::new();
        // Source install/purge use create order; uninstall uses its reverse.
        for table in tables {
            install.extend(
                table
                    .create()
                    .render(dialect)
                    .map_err(Error::from_source)?
                    .queries()
                    .iter()
                    .cloned(),
            );
            // Coordinator ruling: accepted SQLite DELETE mapping; other dialects retain CASCADE.
            purge.push(
                stackpress_sql::TableCommand::Truncate {
                    table: table.name.clone(),
                    cascade: dialect != stackpress_sql::DialectId::Sqlite,
                }
                .render(dialect)
                .map_err(Error::from_source)?,
            );
        }
        for table in tables.iter().rev() {
            uninstall.push(
                stackpress_sql::TableCommand::Drop {
                    table: table.name.clone(),
                    if_exists: true,
                    cascade: false,
                }
                .render(dialect)
                .map_err(Error::from_source)?,
            );
        }
        Ok(Self {
            install,
            purge,
            uninstall,
            schema,
        })
    }
}

/// One source baseline/adjacent plan, whose warning is enforced only by upgrade.
#[derive(Clone)]
pub struct Migration {
    /// Source target revision date in milliseconds, also the output SQL filename.
    pub epoch: i64,
    /// Already rendered statements, preserving their source order and binds.
    pub queries: Vec<Query>,
    /// Source warning, including destructive/ambiguous metadata formatting.
    pub warning: Option<String>,
}

/// Explicit source revision IO/planning boundary, invoked only during a command.
pub struct History {
    /// Return baseline then adjacent plans in source revision order, with no refusal policy.
    pub all: fn(&str, &Engine) -> Result<Vec<Migration>>,
    /// Load only the latest adjacent revision plan, returning None without a previous revision.
    pub latest: fn(&str, &Engine) -> Result<Option<Migration>>,
    /// Inspect only the last two source revisions for push's install/upgrade decision.
    pub has_pair: fn(&str) -> Result<bool>,
    /// Source insert compares current schema with the most recent schema before writing.
    pub insert: fn(&str, &Value) -> Result<()>,
}

/// Execute a source script in one real transaction; an error retains rollback context.
/// # Errors
/// Propagates acquisition, SQL, commit and rollback failures without retrying.
pub async fn transaction(engine: &Engine, queries: &[Query]) -> Result<()> {
    let mut transaction = engine.begin().await.map_err(Error::from_source)?;
    for query in queries {
        if let Err(error) = transaction.execute(query).await {
            return Err(Error::from_source(transaction.rollback_after(error).await));
        }
    }
    transaction.commit().await.map_err(Error::from_source)
}

/// Source install always uninstalls first, then installs in the same transaction.
/// # Errors
/// Preserves SQL and revision-provider failures; revision insertion follows transaction completion.
pub async fn install(
    engine: &Engine,
    scripts: &Scripts,
    revisions: Option<(&str, &History)>,
) -> Result<()> {
    let queries: Vec<_> = scripts
        .uninstall
        .iter()
        .chain(&scripts.install)
        .cloned()
        .collect();
    transaction(engine, &queries).await?;
    // A fresh source Revisions has an empty epoch cache, so size() is zero before insert.
    if let Some((root, history)) = revisions {
        (history.insert)(root, &scripts.schema)?;
    }
    Ok(())
}

/// Source purge delegates all compiled purge statements in one caller-selected database transaction.
/// # Errors
/// Preserves native SQL/commit/rollback errors under the accepted dialect contract.
pub async fn purge(engine: &Engine, scripts: &Scripts) -> Result<()> {
    transaction(engine, &scripts.purge).await
}

/// Source uninstall delegates the compiled drop sequence in one transaction.
/// # Errors
/// Preserves native SQL/commit/rollback errors; it does not delete revision history.
pub async fn uninstall(engine: &Engine, scripts: &Scripts) -> Result<()> {
    transaction(engine, &scripts.uninstall).await
}

/// Source push skips an absent revisions root, installs with fewer than two revisions,
/// and otherwise delegates the same upgrade policy. Required scripts are acquired lazily.
/// # Errors
/// Preserves history, required-client, warning and transaction failures.
pub async fn push(
    engine: &Engine,
    root: Option<&str>,
    history: &History,
    force: bool,
    scripts: impl FnOnce() -> Result<Arc<Scripts>> + Send,
) -> Result<()> {
    let Some(root) = root else {
        return Ok(());
    };
    if !(history.has_pair)(root)? {
        let scripts = scripts()?;
        install(engine, &scripts, Some((root, history))).await
    } else {
        upgrade(engine, Some(root), history, force).await
    }
}

/// Populate by raw, sequential nested dispatch, without inspecting child response codes.
/// # Errors
/// Retains malformed native event/data or thrown child errors; a child Abort is local.
pub async fn populate(events: &[Value], inv: &Invocation<'_>) -> Result<()> {
    for entry in events {
        let event = string(entry.get("event"), "")?;
        let mut req = Request::builder()
            .data(entry.get("data").cloned().unwrap_or(json!({})))
            .build()?;
        let mut child = Response::new();
        let _ = inv.emit(&event, &mut req, &mut child).await?;
    }
    Ok(())
}

/// Source migrate writes every nonempty plan as raw SQL, irrespective of warning/force.
/// # Errors
/// Preserves explicit history/filesystem failures; absent paths are a successful source skip.
pub fn migrate(
    engine: &Engine,
    root: Option<&str>,
    destination: Option<&str>,
    history: &History,
) -> Result<()> {
    let (Some(root), Some(destination)) = (root, destination) else {
        return Ok(());
    };
    for plan in (history.all)(root, engine)? {
        if !plan.queries.is_empty() {
            std::fs::create_dir_all(destination).map_err(Error::from_source)?;
            let sql = plan
                .queries
                .iter()
                .map(|query| query.sql())
                .collect::<Vec<_>>()
                .join(";\n");
            std::fs::write(
                Path::new(destination).join(format!("{}.sql", plan.epoch)),
                sql,
            )
            .map_err(Error::from_source)?;
        }
    }
    Ok(())
}

/// Source upgrade uses the latest adjacent plan, never a baseline; force bypasses its warning.
/// # Errors
/// Returns the exact source warning on refusal, or the original planner/SQL error.
pub async fn upgrade(
    engine: &Engine,
    root: Option<&str>,
    history: &History,
    force: bool,
) -> Result<()> {
    let Some(root) = root else {
        return Ok(());
    };
    let Some(plan) = (history.latest)(root, engine)? else {
        return Ok(());
    };
    if !force && let Some(warning) = plan.warning.as_ref().filter(|value| !value.is_empty()) {
        return Err(Error::new(warning));
    }
    if !plan.queries.is_empty() {
        transaction(engine, &plan.queries).await?;
    }
    Ok(())
}

/// Dispatch the source SQL event family; populate deliberately needs neither Engine nor client.
/// # Errors
/// Propagates required client/script/provider failures and the original nested/SQL errors.
pub async fn event(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    if super::server::skipped(res) {
        return Ok(Flow::Continue);
    }
    let event = inv.event().to_owned();
    if event == "populate" {
        let config = &inv.config::<DatabaseOptions>()?.0;
        let events = path(config, "populate").cloned().unwrap_or(json!([]));
        let events = events
            .as_array()
            .ok_or_else(|| Error::invalid("source populate requires an event array"))?;
        // Source resolve uses independent responses, so non-OK codes do not gate later seeds.
        populate(events, &inv).await?;
    } else {
        let Ok(engine) = inv.shared_service::<Engine>() else {
            return super::server::error(res, "No database found");
        };
        execute(&event, &engine, res, inv).await?;
    }
    res.status_code(200, None)?;
    Ok(Flow::Continue)
}

/// Resolve only the resources used by the selected script, preserving source early skips.
async fn execute(
    event: &str,
    engine: &Engine,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<()> {
    if event == "query" {
        let env = inv.service::<Environment>().cloned().unwrap_or_default();
        if let Some(query) = env
            .argv
            .get(2..)
            .unwrap_or_default()
            .last()
            .filter(|v| !v.is_empty())
        {
            let query = Sql::trusted(
                engine.dialect(),
                stackpress_sql::TrustedSql::audited(query.replace('\\', "'")),
            );
            let result = engine.execute(&query).await.map_err(Error::from_source)?;
            // Console ownership receives the real owned result, without choosing a Debug/JSON
            // representation or making native result resources serializable into browser data.
            let mut output = Request::new();
            output.data.set(
                "result",
                stackpress_lib::Value::Shared(stackpress_lib::SharedNative::new(result)),
            )?;
            inv.shared_service::<super::Bindings>()?
                .invoke("output", &mut output, res, inv)
                .await?;
        }
        return Ok(());
    }
    let client = &inv.config::<ClientOptions>()?.0;
    let root = path(client, "revisions")
        .filter(|v| truthy(v))
        .map(|v| string(Some(v), ""))
        .transpose()?;
    if matches!(event, "push" | "upgrade") && root.is_none() {
        return Ok(());
    }
    let database = &inv.config::<DatabaseOptions>()?.0;
    let destination = path(database, "migrations")
        .filter(|v| truthy(v))
        .map(|v| string(Some(v), ""))
        .transpose()?;
    if event == "migrate" && (root.is_none() || destination.is_none()) {
        return Ok(());
    }
    if event == "migrate" {
        return migrate(
            engine,
            root.as_deref(),
            destination.as_deref(),
            inv.service::<History>()?,
        );
    }
    if event == "upgrade" {
        return upgrade(
            engine,
            root.as_deref(),
            inv.service::<History>()?,
            inv.service::<Environment>().is_ok_and(|e| e.force),
        )
        .await;
    }
    if event == "push" {
        let history = inv.service::<History>()?;
        return push(
            engine,
            root.as_deref(),
            history,
            inv.service::<Environment>().is_ok_and(|e| e.force),
            || {
                inv.service::<Client>()?.get(false)?;
                inv.shared_service::<Scripts>()
            },
        )
        .await;
    }
    // Source explicitly requires the generated package for install/uninstall/purge and fallback push.
    inv.service::<Client>()?.get(false)?;
    let scripts: Arc<Scripts> = inv.shared_service()?;
    match event {
        "install" => {
            install(
                engine,
                &scripts,
                root.as_deref()
                    .map(|root| inv.service::<History>().map(|history| (root, history)))
                    .transpose()?,
            )
            .await
        }
        "purge" => purge(engine, &scripts).await,
        "uninstall" => uninstall(engine, &scripts).await,
        _ => Err(Error::new("unknown source SQL command")),
    }
}
