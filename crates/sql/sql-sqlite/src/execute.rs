//! Same-lease prepared arity checking, binding, native execution and bounded decoding.

use std::future::poll_fn;

use sqlx::sqlite::SqliteStatement;
use sqlx::{Connection, Either, Executor, SqlSafeStr, SqliteConnection, Statement};
use stackpress_sql::{
    DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlValue,
};

use crate::collect::OutputBudget;

/// Map native categories while retaining the original error without echoing it publicly.
pub(crate) fn driver_error(error: sqlx::Error) -> SqlError {
    let kind = match &error {
        sqlx::Error::PoolTimedOut => ErrorKind::PoolTimedOut,
        sqlx::Error::PoolClosed | sqlx::Error::WorkerCrashed => ErrorKind::Closed,
        sqlx::Error::ColumnDecode { .. }
        | sqlx::Error::Decode(_)
        | sqlx::Error::ColumnIndexOutOfBounds { .. }
        | sqlx::Error::ColumnNotFound(_) => ErrorKind::Decode,
        sqlx::Error::Encode(_) | sqlx::Error::InvalidArgument(_) => ErrorKind::InvalidQuery,
        sqlx::Error::Configuration(_) => ErrorKind::Configuration,
        sqlx::Error::Database(native)
            if native.code().is_some_and(|code| {
                code.parse::<i32>()
                    .is_ok_and(|code| matches!(code & 0xff, 5 | 6))
            }) =>
        {
            ErrorKind::Busy
        }
        _ => ErrorKind::Driver,
    };
    SqlError::with_source(kind, "SQLite operation failed", error)
}

/// The only dynamic SQLx audit marker. Query construction has already separated binds,
/// rejected batches/control commands and required trusted authorship for raw text.
fn audited_sql(query: &Query) -> sqlx::SqlStr {
    sqlx::AssertSqlSafe(query.sql()).into_sql_str()
}

/// Prepare and execute on the same connection so native arity cannot drift between leases.
pub(crate) async fn run(
    connection: &mut SqliteConnection,
    query: Query,
    limits: QueryLimits,
) -> Result<QueryResult> {
    if query.dialect() != DialectId::Sqlite {
        return Err(SqlError::new(
            ErrorKind::Unsupported,
            "SQLite requires its own dialect",
        ));
    }
    query.validate(limits)?;
    refresh_schema(connection).await?;
    // sqlite3_changes() is sticky across SELECT/DDL. Snapshot the native cumulative
    // counter on this exclusive lease so an unchanged statement cannot inherit it.
    let before = NativeCounters::read(connection).await?;
    let statement = connection
        .prepare(audited_sql(&query))
        .await
        .map_err(driver_error)?;
    validate_arity(&statement, query.binds().len())?;
    let mut bound = statement.query();
    for value in query.binds() {
        bound = match value {
            SqlValue::Null => bound.bind(Option::<i64>::None),
            SqlValue::Bool(value) => bound.bind(*value),
            SqlValue::Integer(value) => bound.bind(*value),
            SqlValue::Real(value) => bound.bind(*value),
            SqlValue::Text(value) => bound.bind(value.clone()),
            SqlValue::Blob(value) => bound.bind(value.clone()),
        };
    }
    // Poll the boxed native Stream directly: no additional stream utility dependency.
    let mut stream = connection.fetch_many(bound);
    let mut result = QueryResult::default();
    let mut budget = OutputBudget::new(limits);
    while let Some(item) = poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
        match item.map_err(driver_error)? {
            // Native completion acknowledges execution. Read SQLite's 64-bit SQL
            // changes() below rather than SQLx's sqlite3_changes() integer count.
            Either::Left(_) => {}
            Either::Right(native) => result.rows.push(budget.decode(native)?),
        }
    }
    drop(stream);
    let after = NativeCounters::read(connection).await?;
    if before.changes != after.changes {
        result.affected = u64::try_from(after.statement_changes).map_err(|error| {
            SqlError::with_source(ErrorKind::Driver, "invalid SQLite change count", error)
        })?;
    }
    // A connection's last ID is sticky. Only a changed native ID during a statement
    // that actually changed data establishes a statement-local observation. Reused
    // explicit IDs and WITHOUT ROWID tables yield None; RETURNING is authoritative.
    if result.affected > 0 && before.insert_rowid != after.insert_rowid {
        result.insert_id = Some(after.insert_rowid);
    }
    Ok(result)
}

/// Refresh native schema and discard driver metadata after another connection's DDL.
async fn refresh_schema(connection: &mut SqliteConnection) -> Result<()> {
    // Preparing against an old schema can interpret a new quoted column as a string.
    // Stepping this bounded schema read refreshes SQLite's view before native prepare.
    connection
        .execute("SELECT 1 FROM main.sqlite_schema LIMIT 0")
        .await
        .map_err(driver_error)?;
    // SQLite's automatic recompile does not update SQLx's saved column metadata.
    // Invalidate it on every statement, accepting reprepare cost without global cache state.
    connection
        .clear_cached_statements()
        .await
        .map_err(driver_error)
}

/// Connection-local native observations, captured only under one exclusive lease.
struct NativeCounters {
    /// SQLite's SQL changes() function returns the exact 64-bit statement count.
    statement_changes: i64,
    /// Cumulative change counter, used to distinguish a sticky changes() result.
    changes: i64,
    /// Last rowid, compared before/after rather than reused blindly on every outcome.
    insert_rowid: i64,
}

impl NativeCounters {
    /// Read native counters in one round trip; no SQL-text result detection.
    async fn read(connection: &mut SqliteConnection) -> Result<Self> {
        let (statement_changes, changes, insert_rowid) =
            sqlx::query_as("SELECT changes(), total_changes(), last_insert_rowid()")
                .fetch_one(connection)
                .await
                .map_err(driver_error)?;
        Ok(Self {
            statement_changes,
            changes,
            insert_rowid,
        })
    }
}

/// Reject missing/excess parameters using metadata from the actual native preparation.
fn validate_arity(statement: &SqliteStatement, actual: usize) -> Result<()> {
    let expected = match statement.parameters() {
        Some(Either::Right(count)) => count,
        Some(Either::Left(types)) => types.len(),
        None => {
            return Err(SqlError::new(
                ErrorKind::Driver,
                "SQLite parameter metadata is unavailable",
            ));
        }
    };
    // SQLite silently substitutes NULL for absent binds, so mismatch is a first-party error.
    if expected != actual {
        return Err(SqlError::new(
            ErrorKind::Arity { expected, actual },
            "SQL bind arity mismatch",
        ));
    }
    Ok(())
}
