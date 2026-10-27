//! Dedicated, consuming SQLite maintenance ownership, selected explicitly at compilation.
//!
//! Applications must quiesce their ordinary pools before maintenance. This API never
//! changes a pooled connection, accepts a caller-authored PRAGMA, or retries a write.
//! Every uncertain operation retires its connection; `close`/`rollback` await retirement.
//! Drop initiates cleanup only: callers must keep the runtime alive to acknowledge it.

use std::cell::Cell;
use std::marker::PhantomData;

use sqlx::{Connection, Executor, SqliteConnection};
use stackpress_sql::{DialectId, ErrorKind, QueryLimits, QueryResult, Render, Result, SqlError};
use tokio::task::JoinHandle;

use crate::{SqliteOptions, execute};

/// An explicit native connection owner, independent of every ordinary runtime pool.
pub struct SqliteMaintenance {
    /// The only connection, removed while an operation can be cancelled.
    slot: ConnectionSlot,
}

/// Exclusive rebuild transaction with verified policy and consuming completion.
///
/// This type is Send, non-Clone and non-Sync. It cannot return its connection to a pool.
pub struct MaintenanceSession {
    /// One dedicated connection or its pending retirement acknowledgement.
    slot: ConnectionSlot,
    /// Observed before BEGIN, rather than inferred from a requested PRAGMA.
    policy: RebuildPolicy,
    /// Disallow concurrent shared session access even if a driver changes its auto traits.
    exclusive: PhantomData<Cell<()>>,
}

/// Policy observations captured before the native writer transaction was started.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RebuildPolicy {
    /// Foreign-key enforcement was on when this connection was acquired.
    pub foreign_keys_initially_enabled: bool,
    /// Foreign-key enforcement was read back as off before BEGIN EXCLUSIVE.
    pub foreign_keys_disabled_before_begin: bool,
}

/// A bounded catalog object. SQL is private operator data, never a public response DTO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CatalogObject {
    /// SQLite object family: table, index, view or trigger.
    pub kind: String,
    /// Exact native object name.
    pub name: String,
    /// Owning table reported by SQLite.
    pub table: String,
    /// Native declaration, absent for an automatic index.
    pub declaration: Option<String>,
}

/// A typed SQLite foreign-key violation, without rejected application values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForeignKeyViolation {
    /// Child table containing the violation.
    pub table: String,
    /// Child rowid, absent for WITHOUT ROWID storage.
    pub rowid: Option<i64>,
    /// Referenced table.
    pub parent: String,
    /// Native constraint ordinal.
    pub constraint: i64,
}

/// A connection can be available, exclusively in flight, or retired; never duplicated.
struct ConnectionSlot {
    /// Present only between acknowledged operations.
    connection: Option<SqliteConnection>,
    /// Drop of an in-flight operation publishes its awaited shutdown here.
    cleanup: Option<JoinHandle<Result<()>>>,
}

impl ConnectionSlot {
    /// Take ownership before the first suspension so cancellation cannot leave reusable state.
    fn take(&mut self) -> Result<InFlight<'_>> {
        let connection = self.connection.take().ok_or_else(closed)?;
        Ok(InFlight {
            connection: Some(connection),
            slot: self,
        })
    }

    /// Close a live connection or await retirement from an earlier cancelled/failed operation.
    async fn close(mut self) -> Result<()> {
        // A normal caller gets native acknowledgement before its successful return.
        if let Some(connection) = self.connection.take() {
            connection.close().await.map_err(execute::driver_error)?;
        }
        // An uncertain operation already transferred ownership into the close worker.
        if let Some(cleanup) = self.cleanup.take() {
            cleanup.await.map_err(|cause| {
                SqlError::with_source(
                    ErrorKind::Closed,
                    "SQLite maintenance cleanup worker failed",
                    cause,
                )
            })??;
        }
        Ok(())
    }
}

/// Local RAII owner while native work is unacknowledged. Failure/cancel retires its slot.
struct InFlight<'a> {
    /// Taken only on acknowledged success or by Drop's cleanup transfer.
    connection: Option<SqliteConnection>,
    /// No mutex or shared native handle crosses an await.
    slot: &'a mut ConnectionSlot,
}

impl InFlight<'_> {
    /// Borrow the one connection owned by this operation.
    fn connection(&mut self) -> Result<&mut SqliteConnection> {
        self.connection.as_mut().ok_or_else(closed)
    }

    /// Restore only fully acknowledged native state for the next exclusive operation.
    fn acknowledge(mut self) {
        self.slot.connection = self.connection.take();
    }
}

impl Drop for InFlight<'_> {
    /// Transfer uncertain native work to retirement, preserving an acknowledgement handle.
    fn drop(&mut self) {
        let Some(connection) = self.connection.take() else {
            return;
        };
        // Maintenance is async-native. Outside its runtime only driver Drop is possible.
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            self.slot.cleanup = Some(
                runtime
                    .spawn(async move { connection.close().await.map_err(execute::driver_error) }),
            );
        }
    }
}

impl Drop for ConnectionSlot {
    /// Initiate retirement on owner/session Drop without claiming synchronous acknowledgement.
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take()
            && let Ok(runtime) = tokio::runtime::Handle::try_current()
        {
            // Dropping this handle detaches cleanup, while the connection itself stays owned.
            runtime.spawn(async move {
                let _ = connection.close().await;
            });
        }
    }
}

impl SqliteMaintenance {
    /// Eagerly open one explicit file using the ordinary bounded WAL/FULL/FK options.
    ///
    /// # Errors
    /// Invalid/read-only options, the finite open deadline and native failures return typed SQL errors.
    pub async fn open(options: SqliteOptions) -> Result<Self> {
        options.validate()?;
        // Reconstruction is a write capability; read-only attachments are rejected before IO.
        if options.read_only {
            return Err(SqlError::new(
                ErrorKind::Configuration,
                "maintenance requires writable options",
            ));
        }
        // The dedicated owner keeps the ordinary finite acquisition budget even though it
        // has no pool. Cancellation drops an unpublished native connection, never a receipt.
        let connect = options.connect_options();
        let connection = tokio::time::timeout(
            options.acquire_timeout(),
            SqliteConnection::connect_with(&connect),
        )
        .await
        .map_err(|cause| {
            SqlError::with_source(
                ErrorKind::PoolTimedOut,
                "SQLite maintenance open timed out",
                cause,
            )
        })?
        .map_err(execute::driver_error)?;
        Ok(Self {
            slot: ConnectionSlot {
                connection: Some(connection),
                cleanup: None,
            },
        })
    }

    /// Consume the owner, disable/read back FK policy, then acquire the sole writer.
    ///
    /// # Errors
    /// Policy mismatch, bounded lock timeout or cancellation retires this dedicated connection.
    /// The caller must reopen and reconcile ledger state after a lost commit acknowledgement.
    pub async fn begin_rebuild(mut self) -> Result<MaintenanceSession> {
        let mut flight = self.slot.take()?;
        let initial: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(flight.connection()?)
            .await
            .map_err(execute::driver_error)?;
        if initial != 1 {
            return Err(SqlError::new(
                ErrorKind::Configuration,
                "SQLite foreign keys were not enabled",
            ));
        }
        // SQLite ignores this transition inside a transaction. Verify the effect before BEGIN.
        flight
            .connection()?
            .execute("PRAGMA foreign_keys = OFF")
            .await
            .map_err(execute::driver_error)?;
        let disabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(flight.connection()?)
            .await
            .map_err(execute::driver_error)?;
        if disabled != 0 {
            return Err(SqlError::new(
                ErrorKind::Configuration,
                "SQLite foreign keys could not be disabled",
            ));
        }
        flight
            .connection()?
            .execute("BEGIN EXCLUSIVE")
            .await
            .map_err(execute::driver_error)?;
        flight.acknowledge();
        Ok(MaintenanceSession {
            slot: self.slot,
            policy: RebuildPolicy {
                foreign_keys_initially_enabled: true,
                foreign_keys_disabled_before_begin: true,
            },
            exclusive: PhantomData,
        })
    }

    /// Close without starting any transaction, awaiting native worker completion.
    ///
    /// # Errors
    /// Returns a typed native shutdown failure; cancellation loses its acknowledgement.
    pub async fn close(self) -> Result<()> {
        self.slot.close().await
    }
}

impl MaintenanceSession {
    /// Read this table's AUTOINCREMENT high-water mark inside the exclusive transaction.
    ///
    /// Only AUTOINCREMENT tables maintain `sqlite_sequence`. Plain INTEGER PRIMARY KEY
    /// tables use max(rowid)+1 and may reuse deleted IDs by SQLite design. An empty
    /// AUTOINCREMENT table, or a plain table, returns None. Callers must independently
    /// verify managed ownership and the checked schema before using this capability.
    ///
    /// # Errors
    /// Invalid/missing tables and malformed, duplicate or negative sequence state are
    /// refused. Native failure/cancellation retires the connection as for `execute`.
    pub async fn sequence_high_water(&mut self, table: &str) -> Result<Option<i64>> {
        let automatic = self.sequence_table(table).await?;
        if !automatic {
            return Ok(None);
        }
        let rows = self
            .execute(
                &stackpress_sql::Select::new(["seq"])
                    .from("sqlite_sequence")
                    .where_eq("name", table)
                    .limit(2),
            )
            .await?
            .rows;
        if rows.len() > 1 {
            return Err(sequence_error());
        }
        let value = rows
            .first()
            .map(|row| row.try_get::<i64>("seq"))
            .transpose()?;
        if value.is_some_and(|value| value < 0) {
            return Err(sequence_error());
        }
        Ok(value)
    }

    /// Preserve a historical minimum without lowering this table's current sequence.
    ///
    /// The target must be an existing AUTOINCREMENT table whose managed definition the
    /// caller has verified. Plain INTEGER PRIMARY KEY tables have no historical sequence;
    /// this operation refuses them rather than promising non-reuse. No other table is
    /// changed. Native copy already accounts for surviving IDs; this operation retains
    /// deleted-ID history, including when the rebuilt table is empty. It never commits.
    ///
    /// # Errors
    /// Refuses negative minima, unsupported tables and malformed sequence state. The
    /// exact result is read back before acknowledgement; cancellation retires the session.
    pub async fn preserve_sequence_high_water(&mut self, table: &str, minimum: i64) -> Result<()> {
        if minimum < 0 {
            return Err(sequence_error());
        }
        if !self.sequence_table(table).await? {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "sequence preservation requires AUTOINCREMENT",
            ));
        }
        let current = self.sequence_high_water(table).await?;
        let expected = current.unwrap_or(0).max(minimum);
        match current {
            Some(_) => {
                self.execute(
                    &stackpress_sql::Update::new("sqlite_sequence")
                        .set("seq", expected)
                        .where_eq("name", table),
                )
                .await?;
            }
            None => {
                self.execute(
                    &stackpress_sql::Insert::new("sqlite_sequence")
                        .columns(["name", "seq"])
                        .values([
                            stackpress_sql::SqlValue::Text(table.into()),
                            stackpress_sql::SqlValue::Integer(expected),
                        ]),
                )
                .await?;
            }
        }
        // The writer lock excludes cooperative writers; readback also catches malformed state.
        if self.sequence_high_water(table).await? != Some(expected) {
            return Err(sequence_error());
        }
        Ok(())
    }

    /// Check one bounded native table declaration without exposing internal-table setters.
    async fn sequence_table(&mut self, table: &str) -> Result<bool> {
        copy_identifier(table)?;
        let rows = self
            .execute(
                &stackpress_sql::Select::new(["sql"])
                    .from("sqlite_schema")
                    .where_eq("name", table)
                    .where_eq("type", "table")
                    .limit(2),
            )
            .await?
            .rows;
        if rows.len() != 1 || table.starts_with("sqlite_") {
            return Err(sequence_error());
        }
        let declaration: String = rows[0].try_get("sql")?;
        if declaration.len() > 1024 * 1024 {
            return Err(sequence_error());
        }
        Ok(has_autoincrement(&declaration))
    }

    /// Inspect the actual pre-BEGIN observations without acquiring another connection.
    pub fn policy(&self) -> RebuildPolicy {
        self.policy
    }

    /// Execute one already validated ordinary query on this transaction's exact connection.
    ///
    /// # Errors
    /// Native execution failures retire the session; no partial result is published.
    /// Pure rendering/input rejection occurs before native work and preserves the current session.
    pub async fn execute(&mut self, query: &(impl Render + Sync)) -> Result<QueryResult> {
        // Render before moving native ownership; pure input rejection has no native effect.
        let query = query.render(DialectId::Sqlite)?;
        query.validate(QueryLimits::default())?;
        let mut flight = self.slot.take()?;
        let result = execute::run(flight.connection()?, query, QueryLimits::default()).await?;
        flight.acknowledge();
        Ok(result)
    }

    /// Inspect the bounded catalog under the same writer lock as the migration.
    ///
    /// # Errors
    /// Refuses more than 1024 objects or 1 MiB of catalog text rather than truncating.
    pub async fn catalog(&mut self) -> Result<Vec<CatalogObject>> {
        let query = stackpress_sql::Sql::literal(
            DialectId::Sqlite,
            "SELECT type, name, tbl_name, sql FROM sqlite_schema ORDER BY type, name LIMIT 1025",
        );
        let rows = self.execute(&query).await?.rows;
        if rows.len() > 1024 {
            return Err(SqlError::new(
                ErrorKind::Limit,
                "SQLite catalog object budget exceeded",
            ));
        }
        let mut objects = Vec::with_capacity(rows.len());
        let mut bytes = 0_usize;
        for row in rows {
            let object = CatalogObject {
                kind: row.try_get("type")?,
                name: row.try_get("name")?,
                table: row.try_get("tbl_name")?,
                declaration: row.try_get("sql")?,
            };
            bytes += object.kind.len()
                + object.name.len()
                + object.table.len()
                + object.declaration.as_ref().map_or(0, String::len);
            if bytes > 1024 * 1024 {
                return Err(SqlError::new(
                    ErrorKind::Limit,
                    "SQLite catalog byte budget exceeded",
                ));
            }
            objects.push(object);
        }
        Ok(objects)
    }

    /// Check all foreign keys with a finite output budget before committing rebuilt tables.
    ///
    /// # Errors
    /// Native/decoding/limit errors retire the connection; violations are owned typed data.
    pub async fn foreign_key_check(&mut self) -> Result<Vec<ForeignKeyViolation>> {
        // The table-valued PRAGMA has a static shape and is not a caller-authored policy command.
        let query = stackpress_sql::Sql::literal(
            DialectId::Sqlite,
            "SELECT \"table\", rowid, parent, fkid FROM pragma_foreign_key_check LIMIT 1001",
        );
        let rows = self.execute(&query).await?.rows;
        if rows.len() > 1000 {
            return Err(SqlError::new(
                ErrorKind::Limit,
                "SQLite FK-check budget exceeded",
            ));
        }
        rows.into_iter()
            .map(|row| {
                Ok(ForeignKeyViolation {
                    table: row.try_get("table")?,
                    rowid: row.try_get("rowid")?,
                    parent: row.try_get("parent")?,
                    constraint: row.try_get("fkid")?,
                })
            })
            .collect()
    }

    /// Commit once and await unconditional retirement, including after acknowledgement loss.
    ///
    /// # Errors
    /// FK violations are rejected with awaited rollback before COMMIT is submitted.
    /// An error after COMMIT was submitted is an unknown outcome. Reopen and read the ledger;
    /// never replay automatically. Successful return also acknowledges worker shutdown.
    pub async fn commit(mut self) -> Result<()> {
        // Enforcement is disabled on this connection, so commit itself must guard integrity.
        // Caller preflight is useful for diagnostics but cannot authorize an invalid commit.
        let integrity = match self.foreign_key_check().await {
            Ok(violations) if violations.is_empty() => Ok(()),
            Ok(_) => Err(SqlError::new(
                ErrorKind::InvalidQuery,
                "maintenance foreign-key integrity check failed",
            )),
            Err(primary) => Err(primary),
        };
        if let Err(primary) = integrity {
            // A rejected commit has not submitted COMMIT; await rollback or retirement and
            // retain a secondary cleanup failure rather than replacing the integrity failure.
            return Err(match self.rollback().await {
                Ok(()) => primary,
                Err(cleanup) => primary.with_cleanup(cleanup),
            });
        }
        let mut flight = self.slot.take()?;
        flight
            .connection()?
            .execute("COMMIT")
            .await
            .map_err(execute::driver_error)?;
        flight.acknowledge();
        self.slot.close().await
    }

    /// Roll back live state or await retirement after a failed/cancelled statement.
    ///
    /// # Errors
    /// Cleanup failures are returned to the caller for retention alongside its primary error.
    pub async fn rollback(mut self) -> Result<()> {
        if self.slot.connection.is_some() {
            let mut flight = self.slot.take()?;
            flight
                .connection()?
                .execute("ROLLBACK")
                .await
                .map_err(execute::driver_error)?;
            flight.acknowledge();
        }
        self.slot.close().await
    }
}

/// Recognize only an unquoted keyword in SQLite's own declaration, skipping names/literals.
/// This is capability classification, not a parser for caller-authored migration SQL.
fn has_autoincrement(declaration: &str) -> bool {
    let bytes = declaration.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if matches!(byte, b'\'' | b'"' | b'`' | b'[') {
            let end = if byte == b'[' { b']' } else { byte };
            index += 1;
            while index < bytes.len() {
                if bytes[index] == end {
                    index += 1;
                    if index < bytes.len() && bytes[index] == end && end != b']' {
                        index += 1;
                        continue;
                    }
                    break;
                }
                index += 1;
            }
        } else if bytes[index..].starts_with(b"--") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if bytes[index..].starts_with(b"/*") {
            index += 2;
            while index < bytes.len() && !bytes[index..].starts_with(b"*/") {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
        } else if byte.is_ascii_alphanumeric() || byte == b'_' {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            if bytes[start..index].eq_ignore_ascii_case(b"AUTOINCREMENT") {
                return true;
            }
        } else {
            index += 1;
        }
    }
    false
}

/// Refuse invalid identity state without printing database data or native diagnostics.
fn sequence_error() -> SqlError {
    SqlError::new(
        ErrorKind::InvalidQuery,
        "invalid SQLite sequence state or target",
    )
}

/// Explain that an uncertain/consumed maintenance connection cannot execute again.
fn closed() -> SqlError {
    SqlError::new(
        ErrorKind::Closed,
        "SQLite maintenance connection is retired",
    )
}

/// One explicitly mapped replacement column; expressions and casts are not accepted.
#[derive(Clone, Debug)]
pub struct CopyColumn {
    /// Checked destination column name.
    pub destination: String,
    /// An existing source column or a checked literal backfill.
    pub source: CopySource,
}

/// Data-preserving column copy sources; no caller SQL expression enters reconstruction.
#[derive(Clone, Debug)]
pub enum CopySource {
    /// Exact existing column, quoted through the typed SELECT renderer.
    Column(String),
    /// Bound literal, never interpolated into SQL.
    Literal(stackpress_sql::SqlValue),
}

/// Fully pre-rendered typed copy, constructed before acquiring maintenance ownership.
#[derive(Clone, Debug)]
pub struct CopyPlan {
    /// One validated ordinary INSERT-SELECT query with detached binds.
    query: stackpress_sql::Query,
}

impl CopyPlan {
    /// Preflight a bounded one-to-one copy with no row filter, cast, omission or expression.
    ///
    /// # Errors
    /// Refuses empty/duplicate/unsafe names and more than 128 mapped columns before IO.
    pub fn new(source: &str, destination: &str, columns: &[CopyColumn]) -> Result<Self> {
        use stackpress_sql::{Expr, Select, Selector, Sql, TrustedSql};
        // SQLite names are quoted locally after a conservative portable-name check. SELECT
        // expressions and literal parameters are still rendered through the SQL core AST.
        let destination = copy_identifier(destination)?;
        copy_identifier(source)?;
        if columns.is_empty() || columns.len() > 128 {
            return Err(SqlError::new(
                ErrorKind::InvalidQuery,
                "invalid replacement column count",
            ));
        }
        let mut targets = std::collections::BTreeSet::new();
        let mut names = Vec::with_capacity(columns.len());
        let mut selectors = Vec::with_capacity(columns.len());
        for column in columns {
            let name = copy_identifier(&column.destination)?;
            if !targets.insert(name.clone()) {
                return Err(SqlError::new(
                    ErrorKind::InvalidQuery,
                    "duplicate replacement column",
                ));
            }
            names.push(name);
            let expression = match &column.source {
                CopySource::Column(name) => {
                    copy_identifier(name)?;
                    Expr::column(stackpress_sql::Identifier::qualified([source, name]))
                }
                CopySource::Literal(value) => Expr::value(value.clone()),
            };
            selectors.push(Selector::expression(expression));
        }
        let select = Select::new(selectors)
            .from(source)
            .render(DialectId::Sqlite)?;
        // This is the adapter's narrowly audited INSERT-SELECT composition. Every name has
        // already been checked, SELECT is core-rendered, and all values stay in its binds.
        let text = format!(
            "INSERT INTO {} ({}) {}",
            destination,
            names.join(", "),
            select.sql()
        );
        let mut statement = Sql::trusted(DialectId::Sqlite, TrustedSql::audited(text));
        for value in select.binds() {
            statement = statement.bind(value.clone());
        }
        Ok(Self {
            query: statement.render(DialectId::Sqlite)?,
        })
    }

    /// Inspect the checked statement and binds independently of execution.
    pub fn query(&self) -> &stackpress_sql::Query {
        &self.query
    }
}

/// Validate and quote one reconstruction identity, without silently normalizing it.
fn copy_identifier(name: &str) -> Result<String> {
    if name.is_empty()
        || name.len() > 63
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(SqlError::new(
            ErrorKind::InvalidQuery,
            "invalid reconstruction identifier",
        ));
    }
    Ok(format!("\"{name}\""))
}

#[cfg(test)]
mod tests;
