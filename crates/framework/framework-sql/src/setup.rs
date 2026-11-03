//! Explicit source schema setup: non-atomic on MySQL, atomic on PostgreSQL/SQLite (G48).
//!
//! Applications choose when to call these functions. Nothing runs at registration;
//! ordinary Engine/Transaction execution and the SQLite migration runner are unchanged.

use stackpress_sql::{DialectId, Engine, Query, QueryPlan, QueryResult, Result};

/// Execute a real store DDL plan with the owner-selected backend setup semantics.
///
/// # Errors
/// Preserves SQL errors unchanged. MySQL failures retain observed non-atomic progress;
/// cancellation may leave earlier statements applied. Other dialects retain execute_plan's
/// all-or-nothing contract and rollback error context. Neither path retries.
pub async fn execute_plan(engine: &Engine, plan: &QueryPlan) -> Result<Vec<QueryResult>> {
    if engine.dialect() == DialectId::Mysql {
        // Retain plan-wide validation, including nonempty plans, before opting out of atomicity.
        plan.validate(engine.dialect(), engine.limits())?;
        engine.execute_non_atomic(plan.queries()).await
    } else {
        engine.execute_plan(plan).await
    }
}

/// Execute source command statements in order, accepting MySQL's implicit commits.
///
/// Empty scripts are valid. PostgreSQL/SQLite retain one transaction and rollback on
/// failure. On MySQL earlier statements can remain after failure or cancellation;
/// a missing acknowledgement never proves that the failing statement had no effect.
/// # Errors
/// Retains the original SQL kind/cause/progress and atomic rollback context, without retries.
pub async fn execute(engine: &Engine, queries: &[Query]) -> Result<Vec<QueryResult>> {
    if engine.dialect() == DialectId::Mysql {
        return engine.execute_non_atomic(queries).await;
    }
    let mut transaction = engine.begin().await?;
    let mut results = Vec::with_capacity(queries.len());
    // Keep all statements on the caller's atomic path, retaining rollback failures as secondary.
    for query in queries {
        match transaction.execute(query).await {
            Ok(result) => results.push(result),
            Err(error) => return Err(transaction.rollback_after(error).await),
        }
    }
    transaction.commit().await?;
    Ok(results)
}
