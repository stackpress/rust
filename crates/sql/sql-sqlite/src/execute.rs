//! Same-lease prepared arity checking, binding, native execution and bounded decoding.

use std::future::poll_fn;

use sqlx::sqlite::{SqliteRow, SqliteStatement};
use sqlx::{
    Column, Either, Executor, Row as NativeRow, SqlSafeStr, SqliteConnection, Statement, TypeInfo,
    ValueRef,
};
use stackpress_sql::{
    DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, Row, SqlError, SqlValue,
};

/// Map native categories while retaining the original error without echoing it publicly.
pub(crate) fn driver_error(error: sqlx::Error) -> SqlError {
    let kind = match &error {
        sqlx::Error::PoolTimedOut => ErrorKind::PoolTimedOut,
        sqlx::Error::PoolClosed | sqlx::Error::WorkerCrashed => ErrorKind::Closed,
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
    // sqlite3_changes() is sticky across SELECT/DDL. Snapshot the native cumulative
    // counter on this exclusive lease so an unchanged statement cannot inherit it.
    let changes_before: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *connection)
        .await
        .map_err(driver_error)?;
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
    let mut retained = 0_usize;
    while let Some(item) = poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
        match item.map_err(driver_error)? {
            Either::Left(outcome) => result.affected = outcome.rows_affected(),
            Either::Right(native) => retain_row(&mut result, &mut retained, native, limits)?,
        }
    }
    drop(stream);
    let changes_after: i64 = sqlx::query_scalar("SELECT total_changes()")
        .fetch_one(&mut *connection)
        .await
        .map_err(driver_error)?;
    if changes_before == changes_after {
        result.affected = 0;
    }
    // SQLx last_insert_rowid can be stale after non-INSERT statements. M0 exposes no
    // implicit ID; callers use INSERT ... RETURNING for statement-local generated keys.
    Ok(result)
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

/// Check a row's retained budgets before storing it in the statement outcome.
fn retain_row(
    result: &mut QueryResult,
    retained: &mut usize,
    native: SqliteRow,
    limits: QueryLimits,
) -> Result<()> {
    if result.rows.len() >= limits.rows {
        return Err(output_limit());
    }
    let row = decode_row(native)?;
    for (name, value) in row.cells() {
        *retained = retained
            .checked_add(name.len())
            .and_then(|bytes| bytes.checked_add(value.retained_bytes()))
            .ok_or_else(output_limit)?;
        if *retained > limits.bytes {
            return Err(output_limit());
        }
    }
    result.rows.push(row);
    Ok(())
}

/// Decode native storage types rather than guessing from SQL prefixes/declared affinity.
fn decode_row(native: SqliteRow) -> Result<Row> {
    let mut cells = Vec::with_capacity(native.len());
    for (index, column) in native.columns().iter().enumerate() {
        let raw = native.try_get_raw(index).map_err(driver_error)?;
        let value = if raw.is_null() {
            SqlValue::Null
        } else {
            match raw.type_info().name() {
                "INTEGER" => SqlValue::Integer(native.try_get(index).map_err(driver_error)?),
                "REAL" => {
                    let value: f64 = native.try_get(index).map_err(driver_error)?;
                    // SQLite can calculate infinity even when all input binds are finite.
                    if !value.is_finite() {
                        return Err(SqlError::new(
                            ErrorKind::Decode,
                            "SQLite real must be finite",
                        ));
                    }
                    SqlValue::Real(value)
                }
                "TEXT" => SqlValue::Text(native.try_get(index).map_err(driver_error)?),
                "BLOB" => SqlValue::Blob(native.try_get(index).map_err(driver_error)?),
                _ => {
                    return Err(SqlError::new(
                        ErrorKind::Decode,
                        "unsupported SQLite storage type",
                    ));
                }
            }
        };
        cells.push((column.name().to_owned(), value));
    }
    Ok(Row::new(cells))
}

/// Return an explicit limit error instead of silently truncating an outcome.
fn output_limit() -> SqlError {
    SqlError::new(ErrorKind::Limit, "SQL retained output limit exceeded")
}
