//! Bounded observation of this demo's actual SELECTs; never rewrites or executes extra SQL.

use serde_json::{Value, json};
use stackpress_sql::{
    Database, DialectId, Query, QueryLimits, QueryResult, SqlError, SqlFuture, TransactionOptions,
    TransactionSession,
};
use stackpress_sql_sqlite::SqliteDatabase;
use std::sync::{Arc, Mutex};

/// The local demo records at most 32 statements, including synthetic bind values.
pub(crate) type Statements = Arc<Mutex<Vec<Value>>>;

/// Native adapter ownership and the one shared observation list.
pub(crate) struct Observed {
    /// The only native pool; every execution and close delegates to it unchanged.
    pub(crate) database: SqliteDatabase,
    /// Fixed demo evidence, shared with its exact transaction and read only after close.
    pub(crate) statements: Statements,
}
/// Transaction affinity remains with the exact native session, including consuming completion.
struct ObservedSession {
    /// The actual native transaction; no pooled executor is substituted.
    session: Box<dyn TransactionSession>,
    /// The same bounded observation list as the pool owner.
    statements: Statements,
}

/// Refuse poisoned/overflowed recording instead of panicking or silently truncating evidence.
fn record(statements: &Statements, query: &Query) -> stackpress_sql::Result<()> {
    let mut statements = statements.lock().map_err(|_| {
        SqlError::new(
            stackpress_sql::ErrorKind::Driver,
            "demo statement lock is poisoned",
        )
    })?;
    if statements.len() >= 32 {
        return Err(SqlError::new(
            stackpress_sql::ErrorKind::Limit,
            "demo statement budget exceeded",
        ));
    }
    // Bind diagnostics belong only to this local synthetic demo, never a public data handler.
    statements.push(json!({"sql":query.sql(),"binds":format!("{:?}",query.binds())}));
    Ok(())
}

impl Database for Observed {
    /// Preserve the real adapter's declared capability identity.
    fn dialect(&self) -> DialectId {
        self.database.dialect()
    }
    /// Record once before forwarding; recording refusal submits no native statement.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            record(&self.statements, &query)?;
            self.database.run(query, limits).await
        })
    }
    /// Keep transaction affinity while sharing only the observation list.
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            let session = self.database.begin(options).await?;
            Ok(Box::new(ObservedSession {
                session,
                statements: self.statements.clone(),
            }) as Box<dyn TransactionSession>)
        })
    }
    /// Await the real pool's close through the existing Database contract.
    fn close(&self) -> SqlFuture<'_, ()> {
        self.database.close()
    }
}
impl TransactionSession for ObservedSession {
    /// Observe the exact borrowed transaction call without reopening a pool lease.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            record(&self.statements, &query)?;
            self.session.run(query, limits).await
        })
    }
    /// Consume and commit the actual native session.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.session.commit()
    }
    /// Consume and roll back the actual native session.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.session.rollback()
    }
}
