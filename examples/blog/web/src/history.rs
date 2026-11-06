//! Durable source revisions and app-owned History callbacks, independent of full UI generation.
//!
//! Like the original Revisions, files are named by Date.now() milliseconds and an identical
//! serialized schema is a no-op. There is no applied-revision ledger, checksum, rename approval
//! protocol, automatic repair, or startup IO. Command warning/force policy stays in the framework.

mod codec;
mod planning;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_framework::commands::sql::{History, Migration};
use stackpress_lib::{Error, Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::{Engine, TableDefinition};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Source metadata plus prepared native definitions; neither is browser data.
#[derive(Serialize, Deserialize)]
struct Revision {
    source: Value,
    tables: Vec<codec::Table>,
}

/// Construct a native revision from the actual generated catalog and store definitions.
/// # Errors
/// Preserves native codec failures; this performs no filesystem or database IO.
pub fn snapshot(source: Value, tables: &[TableDefinition]) -> Result<Value> {
    Ok(serde_json::to_value(Revision {
        source,
        tables: tables
            .iter()
            .map(codec::Table::capture)
            .collect::<Result<_>>()?,
    })?)
}

/// Decode storage for compiled scripts without retaining an extra source catalog copy.
pub(crate) fn definitions(value: Value) -> Result<Vec<TableDefinition>> {
    Ok(decode(value)?.1)
}

/// Separate owned metadata from owned storage once, so planning does not clone the full catalog.
fn decode(value: Value) -> Result<(Value, Vec<TableDefinition>)> {
    let revision: Revision = serde_json::from_value(value)?;
    let tables = revision
        .tables
        .into_iter()
        .map(codec::Table::definition)
        .collect::<Result<_>>()?;
    Ok((revision.source, tables))
}

/// Reuse source drop ordering in reverse for compiled installation scripts.
pub(crate) fn install_order(tables: &[TableDefinition]) -> Result<Vec<TableDefinition>> {
    Ok(planning::drop_order(tables)?
        .into_iter()
        .rev()
        .cloned()
        .collect())
}

/// Read timestamp filenames only; latest/push must not deserialize obsolete revisions.
fn epochs(root: &str) -> Result<Vec<(i64, PathBuf)>> {
    let root = Path::new(root);
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(root).map_err(Error::from_source)? {
        let path = entry.map_err(Error::from_source)?.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            let epoch = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|stem| stem.parse::<i64>().ok())
                .ok_or_else(|| Error::new("revision filename requires an epoch"))?;
            entries.push((epoch, path));
        }
    }
    // JavaScript Array.sort() uses string order; real Date.now() filenames have equal width.
    entries.sort_by_cached_key(|(epoch, _)| epoch.to_string());
    Ok(entries)
}

/// Read one selected revision with the original JSON/import failure visible.
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(
        &fs::read(path).map_err(Error::from_source)?,
    )?)
}

/// Insert a changed source snapshot at an explicit epoch, also exposing deterministic host clocks.
/// # Errors
/// Propagates existing-file JSON and filesystem errors. Same-epoch writes retain source overwrite behavior.
pub fn insert_at(root: &str, schema: &Value, epoch: i64) -> Result<()> {
    // Compare the complete retained source metadata too: a documentation-only edit is a revision.
    let serialized = serde_json::to_string_pretty(schema)?;
    if let Some((last, path)) = epochs(root)?.last()
        && *last != 0
        && serde_json::to_string_pretty(&read(path)?)? == serialized
    {
        return Ok(());
    }
    // Source writes directly to the millisecond filename. This is not a crash-atomic ledger.
    fs::create_dir_all(root).map_err(Error::from_source)?;
    fs::write(Path::new(root).join(format!("{epoch}.json")), serialized).map_err(Error::from_source)
}

/// Use the source wall clock only when explicitly recording a revision.
fn insert(root: &str, schema: &Value) -> Result<()> {
    insert_at(root, schema, chrono::Utc::now().timestamp_millis())
}

/// Return baseline followed by each adjacent revision plan in source order.
fn all(root: &str, engine: &Engine) -> Result<Vec<Migration>> {
    let mut previous = None;
    let mut result = Vec::new();
    for (index, (epoch, path)) in epochs(root)?.into_iter().enumerate() {
        // Revisions.index uses the source's truthy epoch guard. A zero epoch is absent,
        // and its successor is not silently promoted into a new baseline.
        if epoch == 0 {
            previous = None;
            continue;
        }
        let (source, current) = decode(read(&path)?)?;
        if index == 0 || previous.is_some() {
            result.push(planning::plan(
                previous.as_deref(),
                &current,
                epoch,
                engine.dialect(),
                &source,
            )?);
        }
        previous = Some(current);
    }
    Ok(result)
}

/// Plan only the last adjacent pair; a corrupt obsolete revision cannot block upgrade/push.
fn latest(root: &str, engine: &Engine) -> Result<Option<Migration>> {
    let entries = epochs(root)?;
    if entries.len() < 2
        || entries[entries.len() - 2..]
            .iter()
            .any(|(epoch, _)| *epoch == 0)
    {
        return Ok(None);
    }
    let (epoch, current) = &entries[entries.len() - 1];
    let before = definitions(read(&entries[entries.len() - 2].1)?)?;
    let (source, after) = decode(read(current)?)?;
    Ok(Some(planning::plan(
        Some(&before),
        &after,
        *epoch,
        engine.dialect(),
        &source,
    )?))
}

/// Match push's source existence decision without planning SQL or reading any revision body.
fn has_pair(root: &str) -> Result<bool> {
    let entries = epochs(root)?;
    Ok(entries.len() >= 2 && entries.iter().rev().take(2).all(|(epoch, _)| *epoch != 0))
}

/// Select the durable provider explicitly; no default provider is manufactured by a command.
pub fn binding() -> History {
    History {
        all,
        latest,
        has_pair,
        insert,
    }
}

/// Register History as an ordinary app-level service; registration performs no IO.
/// # Errors
/// Preserves duplicate/startup registration errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(configure)
}

/// Publish callbacks before commands can be invoked; never load revisions during startup.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    start.register(std::sync::Arc::new(binding()))?;
    Ok(Flow::Continue)
}
