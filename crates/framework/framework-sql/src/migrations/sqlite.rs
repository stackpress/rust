//! One dedicated writer transaction owns schema verification, statements and ledger publication.

use stackpress_sql::{
    Compare, Create, Expr, Field, Insert, Order, Select, SqlType, SqlValue, Update,
};
use stackpress_sql_sqlite::maintenance::{MaintenanceSession, SqliteMaintenance};

use super::*;

/// Explicit SQLite migration execution; no automatic startup, retries or baseline adoption.
pub struct MigrationRunner;

impl MigrationRunner {
    /// Recheck authoritative state after acquiring the writer; apply and commit exactly once.
    ///
    /// # Errors
    /// Refuses stale approval/head/catalog and non-discardable reverse data. Statement failures
    /// await rollback/retirement. Commit/cleanup failure after submission is UnknownOutcome;
    /// call again only to reconcile the ledger, never to blindly replay individual statements.
    pub async fn apply(
        owner: SqliteMaintenance,
        plan: &MigrationPlan,
        approval: &PlanHash,
    ) -> DataResult<ApplyOutcome> {
        if approval != &plan.hash {
            owner.close().await?;
            return Err(
                DataError::new("migration approval hash is stale").with_kind(DataErrorKind::Stale)
            );
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
                Ok(ApplyOutcome::Applied)
            }
            Ok(ApplyOutcome::AlreadyApplied) => {
                session.rollback().await?;
                Ok(ApplyOutcome::AlreadyApplied)
            }
            Err(primary) => {
                // Keep the original failure primary; cleanup has its own inspectable slot.
                match session.rollback().await {
                    Ok(()) => Err(primary),
                    Err(cleanup) => Err(primary.with_cleanup(cleanup)),
                }
            }
        }
    }
}

/// Perform every state check and schema/ledger effect under the same native writer lock.
async fn apply_locked(
    session: &mut MaintenanceSession,
    plan: &MigrationPlan,
) -> DataResult<ApplyOutcome> {
    let catalog = session.catalog().await?;
    let ledger_present = catalog
        .iter()
        .any(|object| object.name == "_stackpress_schema_state");
    let ledger = ledger_definitions()?;
    if ledger_present {
        // Validate the reserved namespace before reading it, so collisions are never adopted.
        verify_tables(&catalog, &ledger, false)?;
        let rows = session
            .execute(
                &Select::default()
                    .from("_stackpress_schema_state")
                    .where_eq("id", 1),
            )
            .await?
            .rows;
        if rows.len() != 1 {
            return Err(stale("migration head is missing"));
        }
        let row = &rows[0];
        let format: i64 = row.try_get("format")?;
        let app: String = row.try_get("app")?;
        let revision: String = row.try_get("revision")?;
        let signature: String = row.try_get("signature")?;
        let applied_plan: String = row.try_get("plan_hash")?;
        if format != 1 || app != plan.to.app {
            return Err(stale("migration ledger format or app mismatch"));
        }
        let latest = session
            .execute(
                &Select::default()
                    .from("_stackpress_migrations")
                    .order_by("sequence", Order::Desc)
                    .limit(1),
            )
            .await?
            .rows;
        if latest.len() != 1
            || latest[0].try_get::<String>("revision")? != revision
            || latest[0].try_get::<String>("signature")? != signature
            || latest[0].try_get::<String>("plan_hash")? != applied_plan
        {
            return Err(stale("migration ledger and head disagree"));
        }
        if signature == plan.to.hash && revision == plan.to.revision && applied_plan == plan.hash.0
        {
            verify_tables(&catalog, &plan.to.tables, true)?;
            return Ok(ApplyOutcome::AlreadyApplied);
        }
        if signature != plan.from.hash || revision != plan.from.revision {
            return Err(stale("migration source head is stale"));
        }
        verify_tables(&catalog, &plan.from.tables, true)?;
    } else {
        if !plan.from.tables.is_empty()
            || catalog
                .iter()
                .any(|object| !object.name.starts_with("sqlite_"))
        {
            return Err(stale("existing schema has no owned migration ledger"));
        }
        // Both reserved tables and target installation are inside this same transaction.
        for table in &ledger {
            for query in table.create().render(DialectId::Sqlite)?.queries() {
                session.execute(query).await?;
            }
        }
    }
    for (table, column) in &plan.discardable {
        let predicate =
            Expr::column(column.as_str()).compare(Compare::Ne, Expr::value(SqlValue::Null));
        let rows = session
            .execute(
                &Select::new([column.as_str()])
                    .from(table.as_str())
                    .and_where(predicate)
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
    // All structured statements were rendered and budgeted before acquiring this owner.
    for query in &plan.queries {
        session.execute(query).await?;
    }
    let catalog = session.catalog().await?;
    verify_tables(&catalog, &plan.to.tables, true)?;
    if !session.foreign_key_check().await?.is_empty() {
        return Err(DataError::new("migration left foreign-key violations"));
    }
    let latest = session
        .execute(
            &Select::new(["sequence"])
                .from("_stackpress_migrations")
                .order_by("sequence", Order::Desc)
                .limit(1),
        )
        .await?
        .rows;
    let sequence = latest
        .first()
        .map(|row| row.try_get::<i64>("sequence"))
        .transpose()?
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(|| DataError::new("migration ledger sequence overflow"))?;
    session
        .execute(
            &Insert::new("_stackpress_migrations")
                .columns(["sequence", "revision", "parent", "signature", "plan_hash"])
                .values([
                    SqlValue::Integer(sequence),
                    plan.to.revision.clone().into(),
                    plan.from.revision.clone().into(),
                    plan.to.hash.clone().into(),
                    plan.hash.0.clone().into(),
                ]),
        )
        .await?;
    if ledger_present {
        session
            .execute(
                &Update::new("_stackpress_schema_state")
                    .set("revision", plan.to.revision.as_str())
                    .set("signature", plan.to.hash.as_str())
                    .set("plan_hash", plan.hash.0.as_str())
                    .where_eq("id", 1),
            )
            .await?;
    } else {
        session
            .execute(
                &Insert::new("_stackpress_schema_state")
                    .columns(["id", "format", "app", "revision", "signature", "plan_hash"])
                    .values([
                        SqlValue::Integer(1),
                        SqlValue::Integer(1),
                        plan.to.app.clone().into(),
                        plan.to.revision.clone().into(),
                        plan.to.hash.clone().into(),
                        plan.hash.0.clone().into(),
                    ]),
            )
            .await?;
    }
    Ok(ApplyOutcome::Applied)
}

/// Reject drift/unknown dependents by comparing the complete checked owned object declarations.
fn verify_tables(
    catalog: &[stackpress_sql_sqlite::maintenance::CatalogObject],
    tables: &[TableDefinition],
    reject_unknown: bool,
) -> DataResult<()> {
    let mut expected = BTreeSet::new();
    let names: BTreeSet<_> = tables.iter().map(|table| table.name.as_str()).collect();
    for table in tables {
        for query in table.create().render(DialectId::Sqlite)?.queries() {
            expected.insert(query.sql().to_owned());
        }
    }
    let actual: BTreeSet<_> = catalog
        .iter()
        .filter(|object| names.contains(object.table.as_str()))
        .filter_map(|object| object.declaration.clone())
        .collect();
    if actual != expected {
        return Err(stale("managed schema declaration drifted"));
    }
    // M0 deliberately refuses unowned objects rather than parsing arbitrary SQLite SQL to
    // infer dependency ownership. Later catalog breadth cannot weaken this refusal silently.
    if reject_unknown
        && catalog.iter().any(|object| {
            !names.contains(object.table.as_str())
                && !matches!(
                    object.table.as_str(),
                    "_stackpress_migrations" | "_stackpress_schema_state"
                )
                && !object.name.starts_with("sqlite_")
        })
    {
        return Err(unsupported("M0 migration target contains unowned objects"));
    }
    Ok(())
}

/// Define the entire reserved namespace with checked core builders, without opening a file.
fn ledger_definitions() -> DataResult<Vec<TableDefinition>> {
    let head = Create::new("_stackpress_schema_state")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("format", SqlType::Integer).not_null())
        .add_field(Field::new("app", SqlType::Text).not_null())
        .add_field(Field::new("revision", SqlType::Text).not_null())
        .add_field(Field::new("signature", SqlType::Text).not_null())
        .add_field(Field::new("plan_hash", SqlType::Text).not_null())
        .primary_key(["id"])
        .definition()?;
    let ledger = Create::new("_stackpress_migrations")
        .add_field(Field::new("sequence", SqlType::Integer).not_null())
        .add_field(Field::new("revision", SqlType::Text).not_null())
        .add_field(Field::new("parent", SqlType::Text).not_null())
        .add_field(Field::new("signature", SqlType::Text).not_null())
        .add_field(Field::new("plan_hash", SqlType::Text).not_null())
        .primary_key(["sequence"])
        .definition()?;
    Ok(vec![head, ledger])
}

/// Refuse a stale approved plan without guessing a repair or retry.
fn stale(context: &'static str) -> DataError {
    DataError::new(context).with_kind(DataErrorKind::Stale)
}
