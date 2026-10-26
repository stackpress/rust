//! Same-lease native preparation, typed binding, streaming collection, and finite IO.

use crate::collect::OutputBudget;
use sqlx::{Either, Executor, PgConnection, SqlSafeStr, Statement};
use stackpress_sql::{
    DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlValue,
};
use std::future::poll_fn;
use std::time::Duration;

/// Preserve native typed causes without copying native SQL/authentication messages.
pub(crate) fn driver_error(cause: sqlx::Error) -> SqlError {
    let kind = match &cause {
        sqlx::Error::PoolTimedOut => ErrorKind::PoolTimedOut,
        sqlx::Error::PoolClosed | sqlx::Error::WorkerCrashed => ErrorKind::Closed,
        sqlx::Error::ColumnDecode { .. }
        | sqlx::Error::Decode(_)
        | sqlx::Error::ColumnIndexOutOfBounds { .. }
        | sqlx::Error::ColumnNotFound(_) => ErrorKind::Decode,
        sqlx::Error::Configuration(_) => ErrorKind::Configuration,
        sqlx::Error::Encode(_) | sqlx::Error::InvalidArgument(_) => ErrorKind::InvalidQuery,
        _ => ErrorKind::Driver,
    };
    SqlError::with_source(kind, "PostgreSQL operation failed", cause)
}
/// Reject mismatched dialects and excess query data before native acquisition or IO.
pub(crate) fn validate(query: &Query, limits: QueryLimits) -> Result<()> {
    if query.dialect() != DialectId::Postgres {
        return Err(SqlError::new(
            ErrorKind::Unsupported,
            "PostgreSQL requires its own dialect",
        ));
    }
    query.validate(limits)
}
/// Bound the entire prepare/bind/stream/decode operation on its existing lease.
pub(crate) async fn timed_run(
    connection: &mut PgConnection,
    query: Query,
    limits: QueryLimits,
    timeout: Duration,
) -> Result<QueryResult> {
    validate(&query, limits)?;
    tokio::time::timeout(timeout, run(connection, query, limits))
        .await
        .map_err(|cause| {
            SqlError::with_source(
                ErrorKind::Driver,
                "PostgreSQL statement deadline exceeded; outcome uncertain",
                cause,
            )
        })?
}
/// Execute only adapter-owned transaction control, with a finite loss-of-observation boundary.
pub(crate) async fn control(
    connection: &mut PgConnection,
    command: &'static str,
    timeout: Duration,
) -> Result<()> {
    tokio::time::timeout(timeout, connection.execute(command))
        .await
        .map_err(|cause| {
            SqlError::with_source(
                ErrorKind::Driver,
                "PostgreSQL transaction deadline exceeded; outcome uncertain",
                cause,
            )
        })?
        .map(|_| ())
        .map_err(driver_error)
}
/// Prepare metadata and execute the same SQL on the exclusively borrowed socket.
async fn run(
    connection: &mut PgConnection,
    query: Query,
    limits: QueryLimits,
) -> Result<QueryResult> {
    // This is the only dynamic SQL audit marker. Query has separate, owned binds;
    // the core renderer already validated its single trusted ordinary statement.
    let statement = connection
        .prepare(sqlx::AssertSqlSafe(query.sql()).into_sql_str())
        .await
        .map_err(driver_error)?;
    let types = match statement.parameters() {
        Some(Either::Left(types)) => types,
        _ => {
            return Err(SqlError::new(
                ErrorKind::Driver,
                "PostgreSQL parameter types unavailable",
            ));
        }
    };
    check_arity(types.len(), query.binds().len())?;
    let mut bound = statement.query().persistent(false);
    // Metadata selects native parameter widths and typed NULLs before execution.
    for (value, native_type) in query.binds().iter().zip(types) {
        bound = bind(bound, value, native_type)?;
    }
    // Poll rows incrementally; never allocate an unbounded fetch_all collection.
    let mut stream = connection.fetch_many(bound);
    let mut result = QueryResult::default();
    let mut budget = OutputBudget::new(limits);
    while let Some(item) = poll_fn(|context| stream.as_mut().poll_next(context)).await {
        match item.map_err(driver_error)? {
            Either::Left(outcome) => {
                // Native command completion is authoritative for this statement only.
                result.affected = outcome.rows_affected();
                // PostgreSQL has no statement-local insert ID; RETURNING rows carry keys.
            }
            Either::Right(native) => result.rows.push(budget.decode(&native)?),
        }
    }
    Ok(result)
}
/// Report a native parameter count mismatch before executing any bound statement.
fn check_arity(expected: usize, actual: usize) -> Result<()> {
    if expected != actual {
        return Err(SqlError::new(
            ErrorKind::Arity { expected, actual },
            "SQL bind arity mismatch",
        ));
    }
    Ok(())
}

/// Encode exactly the prepared native type; never send i64 bytes to an int2/int4 slot.
fn bind<'q>(
    query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    value: &SqlValue,
    native_type: &sqlx::postgres::PgTypeInfo,
) -> Result<sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>> {
    // Exhaustive supported pairs reject unknown/custom types and implicit coercion.
    let name = scalar_type(native_type).ok_or_else(|| {
        SqlError::new(
            ErrorKind::Unsupported,
            "unsupported PostgreSQL bind type; supply an explicit supported cast",
        )
    })?;
    Ok(match (name, value) {
        ("BOOL", SqlValue::Null) => query.bind(Option::<bool>::None),
        ("INT2", SqlValue::Null) => query.bind(Option::<i16>::None),
        ("INT4", SqlValue::Null) => query.bind(Option::<i32>::None),
        ("INT8", SqlValue::Null) => query.bind(Option::<i64>::None),
        ("FLOAT4", SqlValue::Null) => query.bind(Option::<f32>::None),
        ("FLOAT8", SqlValue::Null) => query.bind(Option::<f64>::None),
        ("TEXT" | "VARCHAR", SqlValue::Null) => query.bind(Option::<String>::None),
        ("BYTEA", SqlValue::Null) => query.bind(Option::<Vec<u8>>::None),
        ("BOOL", SqlValue::Bool(value)) => query.bind(*value),
        ("INT2", SqlValue::Integer(value)) => {
            query.bind(i16::try_from(*value).map_err(|cause| {
                SqlError::with_source(
                    ErrorKind::InvalidQuery,
                    "PostgreSQL int2 bind out of range",
                    cause,
                )
            })?)
        }
        ("INT4", SqlValue::Integer(value)) => {
            query.bind(i32::try_from(*value).map_err(|cause| {
                SqlError::with_source(
                    ErrorKind::InvalidQuery,
                    "PostgreSQL int4 bind out of range",
                    cause,
                )
            })?)
        }
        ("INT8", SqlValue::Integer(value)) => query.bind(*value),
        ("FLOAT4", SqlValue::Real(value)) => {
            // Native float4 narrowing must not turn a finite input into infinity.
            let narrowed = *value as f32;
            if !narrowed.is_finite() {
                return Err(SqlError::new(
                    ErrorKind::InvalidQuery,
                    "PostgreSQL float4 bind out of range",
                ));
            }
            query.bind(narrowed)
        }
        ("FLOAT8", SqlValue::Real(value)) => query.bind(*value),
        ("TEXT" | "VARCHAR", SqlValue::Text(value)) => query.bind(value.clone()),
        ("BYTEA", SqlValue::Blob(value)) => query.bind(value.clone()),
        _ => {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "unsupported PostgreSQL bind type; supply an explicit supported cast",
            ));
        }
    })
}

/// Identify only actual PostgreSQL built-in scalar OIDs, never a user-defined lookalike name.
/// These OIDs are fixed in PostgreSQL's built-in pg_type catalog and SQLx 0.9's PgType map;
/// unresolved name-only metadata and domains/custom types deliberately remain unsupported.
pub(crate) fn scalar_type(info: &sqlx::postgres::PgTypeInfo) -> Option<&'static str> {
    // Exact OIDs avoid Type::compatible/type_eq treating a domain as its base type.
    Some(match info.oid()?.0 {
        16 => "BOOL",
        21 => "INT2",
        23 => "INT4",
        20 => "INT8",
        700 => "FLOAT4",
        701 => "FLOAT8",
        25 => "TEXT",
        1043 => "VARCHAR",
        17 => "BYTEA",
        _ => return None,
    })
}
