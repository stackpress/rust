//! One exclusive SQLite transaction owns drift checks, reconstruction and ledger publication.

use super::*;
use stackpress_sql::{
    Compare, Create, Engine, Expr, Field, Insert, Order, Select, Sql, SqlType, SqlValue, Update,
};
use stackpress_sql_sqlite::{
    SqliteDatabase, SqliteOptions,
    maintenance::{CatalogObject, MaintenanceSession, SqliteMaintenance},
};

/// Explicit approval binds destructive authority to one reviewed plan and target label.
#[derive(Clone, Debug)]
pub struct LossApproval {
    hash: PlanHash,
    target: String,
}
impl LossApproval {
    /// Select the exact deployment configuration independently of private connection options.
    pub fn new(hash: PlanHash, target: &str) -> DataResult<Self> {
        identity(target)?;
        Ok(Self {
            hash,
            target: target.into(),
        })
    }
}
/// Read-only observational report; application always rechecks under the native writer lock.
#[derive(Clone, Debug, Serialize)]
pub struct DatabaseStatus {
    /// Missing ledger means no owned applied state; existing tables are never adopted.
    pub applied: Option<AppliedRevision>,
    /// Artifact/catalog agreement when a checked snapshot was supplied.
    pub schema_matches: bool,
}
/// Explicit SQLite maintenance runtime; no startup migration or automatic retry.
pub struct MigrationRunner;
impl MigrationRunner {
    /// Validate capabilities, exact approval and target-bound loss authority without acquiring a connection.
    ///
    /// App-owned tools call this before opening maintenance; `apply_to` repeats the same validation.
    /// # Errors
    /// Returns the same pure authority/capability refusals as application, with no filesystem or DB effects.
    pub fn validate(
        plan: &MigrationPlan,
        approval: &PlanHash,
        target: &str,
        loss: Option<&LossApproval>,
    ) -> DataResult<()> {
        preflight(plan, approval, target, loss)
    }
    /// Apply a nondestructive plan with exact hash approval on the default logical target.
    pub async fn apply(
        owner: SqliteMaintenance,
        plan: &MigrationPlan,
        approval: &PlanHash,
    ) -> DataResult<ApplyOutcome> {
        Self::apply_to(owner, plan, approval, "default", None).await
    }
    /// Apply to one explicit target; destructive findings need matching target/hash approval.
    ///
    /// # Errors
    /// Refuses network dialects, stale authority, unsupported conversions and drift before
    /// effects. Failures await rollback. A submitted commit with no acknowledgement is
    /// UnknownOutcome; reopen and reconcile the ledger, never automatically replay.
    pub async fn apply_to(
        owner: SqliteMaintenance,
        plan: &MigrationPlan,
        approval: &PlanHash,
        target: &str,
        loss: Option<&LossApproval>,
    ) -> DataResult<ApplyOutcome> {
        let preflight = preflight(plan, approval, target, loss);
        if let Err(error) = preflight {
            // Preserve authority refusal as primary even if retiring the unopened session fails.
            return match owner.close().await {
                Ok(()) => Err(error),
                Err(cleanup) => Err(error.with_cleanup(cleanup)),
            };
        }
        let mut session = owner.begin_rebuild().await?;
        let result = apply_locked(&mut session, plan).await;
        match result {
            Ok(ApplyOutcome::Applied) => {
                session.commit().await.map_err(|cause| {
                    DataError::new("migration commit outcome requires reconciliation")
                        .with_kind(DataErrorKind::UnknownOutcome)
                        .with_cause(cause)
                })?;
                #[cfg(test)]
                faults::commit_ack()?;
                Ok(ApplyOutcome::Applied)
            }
            Ok(ApplyOutcome::AlreadyApplied) => {
                session.rollback().await?;
                Ok(ApplyOutcome::AlreadyApplied)
            }
            Err(primary) => match session.rollback().await {
                Ok(()) => Err(primary),
                Err(cleanup) => Err(primary.with_cleanup(cleanup)),
            },
        }
    }
    /// Inspect without maintenance policy transitions, adoption or repair.
    ///
    /// The target must already exist; a read-only ordinary connection is closed/awaited.
    /// This report is observational and never authorizes a subsequent application.
    pub async fn status(
        mut options: SqliteOptions,
        snapshot: Option<&RevisionSnapshot>,
    ) -> DataResult<DatabaseStatus> {
        options.read_only = true;
        options.create_if_missing = false;
        let engine = Engine::new(SqliteDatabase::open(options).await?);
        let result = read_status(&engine, snapshot).await;
        let cleanup = engine.close().await;
        match (result, cleanup) {
            (Ok(result), Ok(())) => Ok(result),
            (Err(primary), Ok(())) => Err(primary),
            (Err(primary), Err(cleanup)) => Err(primary.with_cleanup(cleanup)),
            (Ok(_), Err(cause)) => Err(cause.into()),
        }
    }
}
/// Check all pure authority/capability conditions before acquiring a writer.
fn preflight(
    plan: &MigrationPlan,
    approval: &PlanHash,
    target: &str,
    loss: Option<&LossApproval>,
) -> DataResult<()> {
    identity(target)?;
    if plan.dialect() != DialectId::Sqlite {
        return Err(unsupported(
            "network migration application is unavailable; MySQL DDL requires a journal/repair protocol",
        ));
    }
    if approval != &plan.hash {
        return Err(stale("migration approval hash is stale"));
    }
    if !plan.blocked.is_empty() {
        return Err(unsupported(
            "migration has unresolved conversion or capability preconditions",
        ));
    }
    if plan.findings.iter().any(|r| r.destructive)
        && !loss.is_some_and(|a| a.hash == plan.hash && a.target == target)
    {
        return Err(DataError::new(
            "destructive migration needs exact plan and target data-loss approval",
        ));
    }
    Ok(())
}
/// Detect both reserved tables together, rejecting partial/colliding namespaces.
fn reserved_present(catalog: &[CatalogObject]) -> DataResult<bool> {
    let names = ["_stackpress_migrations", "_stackpress_schema_state"];
    let found = names
        .iter()
        .filter(|name| catalog.iter().any(|o| o.name == **name))
        .count();
    if found == 1
        || catalog
            .iter()
            .any(|o| o.name.starts_with("_stackpress_") && !names.contains(&o.name.as_str()))
    {
        return Err(stale("migration namespace collision"));
    }
    Ok(found == 2)
}
/// Recheck ledger/catalog under the native lock, then execute and publish exactly once.
async fn apply_locked(
    session: &mut MaintenanceSession,
    plan: &MigrationPlan,
) -> DataResult<ApplyOutcome> {
    let catalog = session.catalog().await?;
    #[cfg(test)]
    faults::checkpoint(session).await?;
    let present = reserved_present(&catalog)?;
    let mut prior_sequence = 0;
    if present {
        verify_tables(&catalog, &ledger_definitions()?, false)?;
        let heads = execute(
            session,
            &Select::default().from("_stackpress_schema_state").limit(2),
        )
        .await?
        .rows;
        let rows = execute(
            session,
            &Select::default()
                .from("_stackpress_migrations")
                .order_by("sequence", Order::Asc)
                .limit(1025),
        )
        .await?
        .rows;
        let applied = verify_applied(&heads, &rows, Some(&plan.to.app))?;
        prior_sequence = applied.sequence;
        let installed = applied.state == "applied";
        if applied.revision_hash == plan.to.hash
            && applied.revision == plan.to.revision
            && applied.plan_hash == plan.hash.0
        {
            if matches!(plan.recipe.kind, PlanKind::Uninstall) {
                verify_absent(&catalog, &plan.to.tables)?;
            } else {
                verify_tables(&catalog, &plan.to.tables, true)?;
            }
            return Ok(ApplyOutcome::AlreadyApplied);
        }
        if !installed {
            return Err(stale(
                "application was uninstalled; explicit reviewed reinstall is required",
            ));
        }
        if applied.revision_hash != plan.from.hash || applied.revision != plan.from.revision {
            return Err(stale("migration source head is stale"));
        }
        verify_tables(&catalog, &plan.from.tables, true)?;
        if matches!(plan.recipe.kind, PlanKind::Populate { .. }) {
            let rows = execute(
                session,
                &Select::new(["signature"])
                    .from("_stackpress_migrations")
                    .where_eq("plan_hash", plan.hash.0.as_str())
                    .limit(2),
            )
            .await?
            .rows;
            if !rows.is_empty() {
                return Ok(ApplyOutcome::AlreadyApplied);
            }
        }
    } else {
        if !plan.from.tables.is_empty() {
            return Err(stale("UnmanagedExistingSchema: no owned migration ledger"));
        }
        if catalog.iter().any(|o| {
            plan.to
                .tables
                .iter()
                .any(|t| o.name.eq_ignore_ascii_case(&t.name))
        }) {
            return Err(stale(
                "UnmanagedExistingSchema: managed name already exists",
            ));
        }
        reject_dependents(&catalog, &plan.to.tables)?;
        for table in ledger_definitions()? {
            for query in table.create().render(DialectId::Sqlite)?.queries() {
                execute(session, query).await?;
            }
        }
    }
    for (table, column) in &plan.discardable {
        let rows = execute(
            session,
            &Select::new([column.as_str()])
                .from(table.as_str())
                .and_where(
                    Expr::column(column.as_str()).compare(Compare::Ne, Expr::value(SqlValue::Null)),
                )
                .limit(1),
        )
        .await?
        .rows;
        if !rows.is_empty() {
            return Err(DataError::new(
                "reverse would discard populated added-column data",
            ));
        }
    }
    for step in &plan.steps {
        match step {
            MigrationStep::Native(queries) => {
                for query in queries {
                    execute(session, query).await?;
                }
            }
            MigrationStep::Rebuild {
                source,
                target,
                queries,
                preserve_sequence,
            } => {
                let count = row_count(session, source).await?;
                let sequence = if *preserve_sequence {
                    session.sequence_high_water(source).await?
                } else {
                    None
                };
                for query in queries {
                    execute(session, query).await?;
                }
                if row_count(session, target).await? != count {
                    return Err(DataError::new("reconstruction row count changed"));
                }
                if *preserve_sequence {
                    session
                        .preserve_sequence_high_water(target, sequence.unwrap_or(0))
                        .await?;
                }
            }
        }
    }
    let catalog = session.catalog().await?;
    #[cfg(test)]
    faults::checkpoint(session).await?;
    let uninstalled = matches!(plan.recipe.kind, PlanKind::Uninstall);
    if uninstalled {
        verify_absent(&catalog, &plan.from.tables)?;
    } else {
        verify_tables(&catalog, &plan.to.tables, true)?;
    }
    if !session.foreign_key_check().await?.is_empty() {
        return Err(DataError::new("migration left foreign-key violations"));
    }
    #[cfg(test)]
    faults::checkpoint(session).await?;
    let sequence = prior_sequence
        .checked_add(1)
        .ok_or_else(|| DataError::new("migration ledger sequence overflow"))?;
    let state = if uninstalled {
        "uninstalled"
    } else {
        "applied"
    };
    // Execution time comes from the database, never from immutable artifact bytes.
    let now = execute(
        session,
        &Sql::literal(
            DialectId::Sqlite,
            "SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now') AS executed_at",
        ),
    )
    .await?
    .rows;
    let executed_at: String = now[0].try_get("executed_at")?;
    execute(
        session,
        &Insert::new("_stackpress_migrations")
            .columns([
                "sequence",
                "app",
                "revision",
                "parent",
                "signature",
                "plan_hash",
                "dialect",
                "state",
                "operation",
                "executed_at",
            ])
            .values([
                SqlValue::Integer(sequence),
                plan.to.app.clone().into(),
                plan.to.revision.clone().into(),
                plan.from.revision.clone().into(),
                plan.to.hash.clone().into(),
                plan.hash.0.clone().into(),
                "sqlite".into(),
                state.into(),
                operation_name(&plan.recipe.kind).into(),
                executed_at.into(),
            ]),
    )
    .await?;
    if present {
        execute(
            session,
            &Update::new("_stackpress_schema_state")
                .set("sequence", sequence)
                .set("revision", plan.to.revision.as_str())
                .set("signature", plan.to.hash.as_str())
                .set("plan_hash", plan.hash.0.as_str())
                .set("state", state)
                .where_eq("id", 1),
        )
        .await?;
    } else {
        execute(
            session,
            &Insert::new("_stackpress_schema_state")
                .columns([
                    "id",
                    "format",
                    "app",
                    "sequence",
                    "revision",
                    "signature",
                    "plan_hash",
                    "state",
                ])
                .values([
                    SqlValue::Integer(1),
                    SqlValue::Integer(2),
                    plan.to.app.clone().into(),
                    sequence.into(),
                    plan.to.revision.clone().into(),
                    plan.to.hash.clone().into(),
                    plan.hash.0.clone().into(),
                    state.into(),
                ]),
        )
        .await?;
    }
    Ok(ApplyOutcome::Applied)
}
/// Count without fetching unbounded application rows during reconstruction.
async fn row_count(session: &mut MaintenanceSession, table: &str) -> DataResult<i64> {
    identity(table)?;
    let rows = execute(
        session,
        &Sql::trusted(
            DialectId::Sqlite,
            stackpress_sql::TrustedSql::audited(format!(
                "SELECT COUNT(*) AS count FROM \"{table}\""
            )),
        ),
    )
    .await?
    .rows;
    Ok(rows[0].try_get("count")?)
}
/// Decode the complete acknowledged ledger shape, keeping private causes typed.
fn applied_row(row: &stackpress_sql::Row) -> DataResult<AppliedRevision> {
    Ok(AppliedRevision {
        sequence: row.try_get("sequence")?,
        app: row.try_get("app")?,
        revision: row.try_get("revision")?,
        parent: row.try_get("parent")?,
        revision_hash: row.try_get("signature")?,
        plan_hash: row.try_get("plan_hash")?,
        dialect: row.try_get("dialect")?,
        state: row.try_get("state")?,
        executed_at: row.try_get("executed_at")?,
    })
}
/// Compare exact storage tokens, allowing SQLite's identifier quoting after native rename.
fn verify_tables(
    catalog: &[CatalogObject],
    tables: &[TableDefinition],
    dependents: bool,
) -> DataResult<()> {
    let mut expected = BTreeSet::new();
    let names = tables
        .iter()
        .map(|t| t.name.as_str())
        .collect::<BTreeSet<_>>();
    for table in tables {
        for query in table.create().render(DialectId::Sqlite)?.queries() {
            expected.insert(tokens(query.sql()));
        }
    }
    let actual = catalog
        .iter()
        .filter(|o| names.contains(o.table.as_str()))
        .filter_map(|o| o.declaration.as_deref())
        .map(tokens)
        .collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(stale("managed schema declaration drifted"));
    }
    if dependents {
        reject_dependents(catalog, tables)?;
    }
    Ok(())
}
/// Unknown custom objects on managed tables or references from unowned objects block maintenance.
fn reject_dependents(catalog: &[CatalogObject], tables: &[TableDefinition]) -> DataResult<()> {
    let names = tables
        .iter()
        .map(|t| t.name.to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    for object in catalog {
        if object.name.starts_with("sqlite_")
            || object.name.starts_with("_stackpress_")
            || names.contains(&object.table.to_ascii_lowercase())
        {
            continue;
        }
        if matches!(object.kind.as_str(), "view" | "trigger") {
            return Err(unsupported(
                "unowned view or trigger needs explicit dependency review",
            ));
        }
        if object.declaration.as_deref().is_some_and(|sql| {
            tokens(sql).iter().any(|word| {
                // SQLite also accepts single-quoted table references. Treat any matching
                // literal conservatively as a dependency rather than guessing SQL grammar.
                names.contains(word)
                    || word
                        .strip_prefix('\'')
                        .and_then(|v| v.strip_suffix('\''))
                        .is_some_and(|value| names.contains(&value.to_ascii_lowercase()))
            })
        }) {
            return Err(unsupported(
                "unknown dependent object blocks managed maintenance",
            ));
        }
    }
    Ok(())
}
/// Tokenize native declarations for equality/dependency refusal, never for execution or rewriting.
fn tokens(sql: &str) -> Vec<String> {
    let bytes = sql.as_bytes();
    let mut index = 0;
    let mut result = Vec::new();
    while index < bytes.len() {
        if bytes[index].is_ascii_whitespace() {
            index += 1;
            continue;
        }
        let start = index;
        if matches!(bytes[index], b'"' | b'`' | b'[' | b'\'') {
            let quote = bytes[index];
            let end = if quote == b'[' { b']' } else { quote };
            index += 1;
            let content = index;
            while index < bytes.len() {
                if bytes[index] == end {
                    if index + 1 < bytes.len() && bytes[index + 1] == end {
                        index += 2;
                        continue;
                    }
                    break;
                }
                index += 1;
            }
            let value = &sql[content..index];
            result.push(if quote == b'\'' {
                format!("'{value}'")
            } else {
                value.to_ascii_lowercase()
            });
            index = (index + 1).min(bytes.len());
        } else if bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_' {
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            result.push(sql[start..index].to_ascii_lowercase());
        } else {
            // Native unowned declarations may contain non-ASCII identifiers or comments.
            // Advance one complete code point so diagnostics cannot panic on a byte slice.
            index += sql[start..].chars().next().map_or(1, char::len_utf8);
            result.push(sql[start..index].into());
        }
    }
    result
}
/// Define the complete reserved namespace; unrelated pre-existing tables are never adopted.
fn ledger_definitions() -> DataResult<Vec<TableDefinition>> {
    let mut head = Create::new("_stackpress_schema_state");
    for name in ["id", "format", "sequence"] {
        head = head.add_field(Field::new(name, SqlType::Integer).not_null());
    }
    for name in ["app", "revision", "signature", "plan_hash", "state"] {
        head = head.add_field(Field::new(name, SqlType::Text).not_null());
    }
    let mut ledger = Create::new("_stackpress_migrations")
        .add_field(Field::new("sequence", SqlType::Integer).not_null());
    for name in [
        "app",
        "revision",
        "parent",
        "signature",
        "plan_hash",
        "dialect",
        "state",
        "operation",
        "executed_at",
    ] {
        ledger = ledger.add_field(Field::new(name, SqlType::Text).not_null());
    }
    Ok(vec![
        head.primary_key(["id"]).definition()?,
        ledger.primary_key(["sequence"]).definition()?,
    ])
}
/// Stable operator spelling stored in the ledger, independent of the CLI event name.
fn operation_name(kind: &PlanKind) -> &'static str {
    match kind {
        PlanKind::Schema => "schema",
        PlanKind::Purge => "purge",
        PlanKind::Uninstall => "uninstall",
        PlanKind::Populate { .. } => "populate",
    }
}

/// Execute one checked phase; test-only checkpoints inject native failures after acknowledgement.
async fn execute(
    session: &mut MaintenanceSession,
    query: &(impl stackpress_sql::Render + Sync),
) -> DataResult<stackpress_sql::QueryResult> {
    let result = session.execute(query).await?;
    #[cfg(test)]
    faults::checkpoint(session).await?;
    Ok(result)
}
#[cfg(test)]
#[path = "faults.rs"]
mod faults;

/// Validate the bounded applied chain and its one authoritative head before acknowledgement.
fn verify_applied(
    heads: &[stackpress_sql::Row],
    rows: &[stackpress_sql::Row],
    app: Option<&str>,
) -> DataResult<AppliedRevision> {
    if heads.len() != 1 || rows.is_empty() || rows.len() > 1024 {
        return Err(stale(
            "migration ledger/head is missing or exceeds its budget",
        ));
    }
    let mut prior: Option<AppliedRevision> = None;
    for (index, row) in rows.iter().enumerate() {
        let record = applied_row(row)?;
        if (index == 0 && record.parent != "empty")
            || record.sequence != index as i64 + 1
            || record.dialect != "sqlite"
            || !matches!(record.state.as_str(), "applied" | "uninstalled")
            || record.executed_at.is_empty()
            || record.executed_at.len() > 64
        {
            return Err(stale("migration ledger entry is invalid"));
        }
        identity(&record.app)?;
        identity(&record.revision)?;
        identity(&record.parent)?;
        PlanHash::parse(&record.revision_hash)?;
        PlanHash::parse(&record.plan_hash)?;
        if app.is_some_and(|app| app != record.app)
            || prior.as_ref().is_some_and(|previous| {
                record.parent != previous.revision || record.app != previous.app
            })
        {
            return Err(stale("migration ledger parent or namespace disagrees"));
        }
        prior = Some(record);
    }
    let record = prior.ok_or_else(|| stale("migration ledger is missing"))?;
    let head = &heads[0];
    if head.try_get::<i64>("id")? != 1
        || head.try_get::<i64>("format")? != 2
        || head.try_get::<String>("app")? != record.app
        || head.try_get::<i64>("sequence")? != record.sequence
        || head.try_get::<String>("revision")? != record.revision
        || head.try_get::<String>("signature")? != record.revision_hash
        || head.try_get::<String>("plan_hash")? != record.plan_hash
        || head.try_get::<String>("state")? != record.state
    {
        return Err(stale(
            "migration ledger format, namespace or head disagrees",
        ));
    }
    Ok(record)
}
/// An uninstalled acknowledgement also refuses out-of-band recreation of managed tables.
fn verify_absent(catalog: &[CatalogObject], tables: &[TableDefinition]) -> DataResult<()> {
    if catalog
        .iter()
        .any(|o| tables.iter().any(|t| o.table.eq_ignore_ascii_case(&t.name)))
    {
        return Err(stale("uninstalled managed objects unexpectedly exist"));
    }
    Ok(())
}

/// Read bounded operator observations without changing connection policy or claiming writer authority.
async fn read_status(
    engine: &Engine,
    snapshot: Option<&RevisionSnapshot>,
) -> DataResult<DatabaseStatus> {
    // Inspect only this configured database; ordinary query budgets bound retained catalog bytes.
    let rows = engine
        .fetch(&Sql::literal(
            DialectId::Sqlite,
            "SELECT type, name, tbl_name, sql FROM sqlite_schema ORDER BY type, name LIMIT 1025",
        ))
        .await?;
    if rows.len() > 1024 {
        return Err(stale("catalog budget exceeded"));
    }
    let catalog = rows
        .iter()
        .map(|row| {
            Ok(CatalogObject {
                kind: row.try_get("type")?,
                name: row.try_get("name")?,
                table: row.try_get("tbl_name")?,
                declaration: row.try_get("sql")?,
            })
        })
        .collect::<DataResult<Vec<_>>>()?;
    // A reserved namespace is adopted only when its complete known declarations and chain agree.
    let applied = if reserved_present(&catalog)? {
        verify_tables(&catalog, &ledger_definitions()?, false)?;
        let heads = engine
            .fetch(&Select::default().from("_stackpress_schema_state").limit(2))
            .await?;
        let ledger = engine
            .fetch(
                &Select::default()
                    .from("_stackpress_migrations")
                    .order_by("sequence", Order::Asc)
                    .limit(1025),
            )
            .await?;
        Some(verify_applied(
            &heads,
            &ledger,
            snapshot.map(|s| s.app.as_str()),
        )?)
    } else {
        None
    };
    // A pending history head can differ from the applied one; report disagreement without repair.
    let schema_matches = snapshot.is_some_and(|schema| {
        verify_tables(&catalog, &schema.tables, false).is_ok()
            && applied
                .as_ref()
                .is_some_and(|head| head.revision_hash == schema.hash && head.state == "applied")
    });
    Ok(DatabaseStatus {
        applied,
        schema_matches,
    })
}
