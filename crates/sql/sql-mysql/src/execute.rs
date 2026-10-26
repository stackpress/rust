//! Same-lease native preparation, typed binding, streaming collection, and finite IO.

use sqlx::{Either, Executor, MySqlConnection, SqlSafeStr, Statement};
use std::future::poll_fn;
use std::time::Duration;

use crate::collect::OutputBudget;
use stackpress_sql::{
    DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlValue,
};

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
    SqlError::with_source(kind, "MySQL operation failed", cause)
}
/// Reject mismatched dialects and excess query data before native acquisition or IO.
pub(crate) fn validate(query: &Query, limits: QueryLimits) -> Result<()> {
    if query.dialect() != DialectId::Mysql {
        return Err(SqlError::new(
            ErrorKind::Unsupported,
            "MySQL requires its own dialect",
        ));
    }
    query.validate(limits)
}
/// Bound the entire prepare/bind/stream/decode operation on its existing lease.
pub(crate) async fn timed_run(
    connection: &mut MySqlConnection,
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
                "MySQL statement deadline exceeded; outcome uncertain",
                cause,
            )
        })?
}
/// Execute only adapter-owned transaction control, with a finite loss-of-observation boundary.
pub(crate) async fn control(
    connection: &mut MySqlConnection,
    command: &'static str,
    timeout: Duration,
) -> Result<()> {
    tokio::time::timeout(timeout, async {
        #[cfg(test)]
        crate::probe::at(match command {
            "BEGIN" => crate::probe::Phase::BeforeBegin,
            "COMMIT" => crate::probe::Phase::BeforeCommit,
            _ => crate::probe::Phase::BeforeRollback,
        })
        .await?;
        connection.execute(command).await.map_err(driver_error)?;
        #[cfg(test)]
        crate::probe::at(match command {
            "BEGIN" => crate::probe::Phase::AfterBegin,
            "COMMIT" => crate::probe::Phase::AfterCommit,
            _ => crate::probe::Phase::AfterRollback,
        })
        .await?;
        Ok(())
    })
    .await
    .map_err(|cause| {
        SqlError::with_source(
            ErrorKind::Driver,
            "Mysql transaction deadline exceeded; outcome uncertain",
            cause,
        )
    })?
}
/// Prepare metadata and execute the same SQL on the exclusively borrowed socket.
async fn run(
    connection: &mut MySqlConnection,
    query: Query,
    limits: QueryLimits,
) -> Result<QueryResult> {
    // This is the only dynamic SQL audit marker. Query has separate, owned binds;
    // the core renderer already validated its single trusted ordinary statement.
    #[cfg(test)]
    crate::probe::at(crate::probe::Phase::BeforePrepare).await?;
    let statement = connection
        .prepare(sqlx::AssertSqlSafe(query.sql()).into_sql_str())
        .await
        .map_err(driver_error)?;
    #[cfg(test)]
    crate::probe::at(crate::probe::Phase::AfterPrepare).await?;
    let expected = match statement.parameters() {
        Some(Either::Right(count)) => count,
        Some(Either::Left(types)) => types.len(),
        None => {
            return Err(SqlError::new(
                ErrorKind::Driver,
                "MySQL parameter count unavailable",
            ));
        }
    };
    check_arity(expected, query.binds().len())?;
    let mut bound = statement.query().persistent(false);
    // The native protocol carries each value's type, including an explicit NULL bit.
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
    // Poll rows incrementally; never allocate an unbounded fetch_all collection.
    #[cfg(test)]
    crate::probe::at(crate::probe::Phase::BeforeExecute).await?;
    let mut stream = connection.fetch_many(bound);
    let mut result = QueryResult::default();
    let mut budget = OutputBudget::new(limits);
    while let Some(item) = poll_fn(|context| stream.as_mut().poll_next(context)).await {
        match item.map_err(driver_error)? {
            Either::Left(outcome) => {
                // Native command completion is authoritative for this statement only.
                result.affected = outcome.rows_affected();
                let id = outcome.last_insert_id();
                result.insert_id = if id == 0 {
                    None
                } else {
                    Some(i64::try_from(id).map_err(|cause| {
                        SqlError::with_source(
                            ErrorKind::Decode,
                            "MySQL insert ID exceeds i64",
                            cause,
                        )
                    })?)
                };
            }
            Either::Right(native) => {
                #[cfg(test)]
                crate::probe::at(crate::probe::Phase::AfterRow).await?;
                result.rows.push(budget.decode(&native)?);
            }
        }
    }
    #[cfg(test)]
    crate::probe::at(crate::probe::Phase::AfterExecute).await?;
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

/// Reject implicit-commit DDL in transactions without guessing from a substring.
/// The core gate already bounded/validated quoting and allowed command families.
pub(crate) fn validate_transaction(query: &Query) -> Result<()> {
    let bytes = query.sql().as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        // Skip the exact comment/quote forms admitted by the core MySQL lexical gate.
        if bytes[index..].starts_with(b"/*") {
            index += 2;
            while index + 1 < bytes.len() && !bytes[index..].starts_with(b"*/") {
                index += 1;
            }
            index = (index + 2).min(bytes.len());
        } else if bytes[index] == b'#'
            || (bytes[index..].starts_with(b"--")
                && bytes.get(index + 2).is_none_or(u8::is_ascii_whitespace))
        {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if matches!(bytes[index], b'\'' | b'"' | b'`' | b'[') {
            let close = if bytes[index] == b'[' {
                b']'
            } else {
                bytes[index]
            };
            index += 1;
            while index < bytes.len() {
                if bytes[index] == close {
                    index += 1;
                    if close != b']' && bytes.get(index) == Some(&close) {
                        index += 1;
                    } else {
                        break;
                    }
                } else {
                    index += 1;
                }
            }
        } else if bytes[index].is_ascii_alphabetic() {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            // Reject DDL tokens even behind WITH/EXPLAIN; quoted names/literals are skipped.
            let word = &bytes[start..index];
            if [b"CREATE".as_slice(), b"ALTER", b"DROP"]
                .iter()
                .any(|ddl| word.eq_ignore_ascii_case(ddl))
            {
                return Err(SqlError::new(
                    ErrorKind::Unsupported,
                    "MySQL transaction DDL would implicitly commit",
                ));
            }
        } else {
            index += 1;
        }
    }
    Ok(())
}
