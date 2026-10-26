//! One public Database/TransactionSession behavioral suite, shared by three real adapters.
//! SQL syntax and documented native value/outcome differences are explicit dialect inputs.
use stackpress_sql::{
    Database, DialectId, ErrorKind, Query, QueryLimits, QueryResult, Render, Result, Sql, SqlValue,
    TransactionOptions,
};
use std::sync::Arc;
use std::time::Instant;

/// Construct a trusted test statement while keeping every value a separate bind.
pub(crate) fn query(
    dialect: DialectId,
    sql: &'static str,
    values: impl IntoIterator<Item = SqlValue>,
) -> Query {
    let mut builder = Sql::literal(dialect, sql);
    for value in values {
        builder = builder.bind(value);
    }
    builder.render(dialect).unwrap()
}
/// Choose only syntax, never a weaker assertion, for a scalar parameter target.
fn scalar(dialect: DialectId, value: &SqlValue) -> &'static str {
    match (dialect, value) {
        (DialectId::Postgres, SqlValue::Null | SqlValue::Integer(_)) => {
            "SELECT $1::BIGINT AS value"
        }
        (DialectId::Postgres, SqlValue::Bool(_)) => "SELECT $1::BOOL AS value",
        (DialectId::Postgres, SqlValue::Real(_)) => "SELECT $1::FLOAT8 AS value",
        (DialectId::Postgres, SqlValue::Text(_)) => "SELECT $1::TEXT AS value",
        (DialectId::Postgres, SqlValue::Blob(_)) => "SELECT $1::BYTEA AS value",
        (DialectId::Mysql, SqlValue::Null | SqlValue::Integer(_)) => {
            "SELECT CAST(? AS SIGNED) AS value"
        }
        // MySQL's bare parameter metadata can advertise text; request a binary wire result.
        (DialectId::Mysql, SqlValue::Blob(_)) => "SELECT CAST(? AS BINARY) AS value",
        (_, _) => "SELECT ? AS value",
    }
}
/// Execute through the trait boundary and retain only a complete owned result.
async fn run(
    database: &dyn Database,
    sql: &'static str,
    values: impl IntoIterator<Item = SqlValue>,
) -> Result<QueryResult> {
    database
        .run(
            query(database.dialect(), sql, values),
            QueryLimits::default(),
        )
        .await
}
/// Read a portable strict integer from a constant test statement.
async fn integer(database: &dyn Database, sql: &'static str) -> Result<i64> {
    run(database, sql, []).await?.rows[0].try_get_at(0)
}
#[path = "contract/errors.rs"]
mod errors;
#[path = "contract/lifecycle.rs"]
mod lifecycle;
#[path = "contract/outcomes.rs"]
mod outcomes;
#[path = "contract/transactions.rs"]
mod transactions;
#[path = "contract/values.rs"]
mod values;

/// Run the same ordered contract against a real one-slot owner, then consume its lifecycle.
pub(crate) async fn run_suite(database: Arc<dyn Database>) -> Result<()> {
    let start = Instant::now();
    let dialect = database.dialect();
    values::check(database.as_ref()).await?;
    outcomes::check(database.as_ref()).await?;
    transactions::check(database.as_ref()).await?;
    errors::check(database.as_ref()).await?;
    lifecycle::cancellation(database.as_ref()).await?;
    lifecycle::close(database).await?;
    println!(
        "shared contract {dialect:?}: values/outcomes/transactions/errors/cancellation/close passed in {:?}",
        start.elapsed()
    );
    Ok(())
}
