//! Explicit pool ownership and exclusive transaction composition.

use std::future::Future;
use std::pin::Pin;

use crate::{DialectId, Query, QueryLimits, QueryPlan, QueryResult, Render, Result, Row, SqlError};

/// A driver future that can cross executor threads without widening borrowed lifetimes.
pub type SqlFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Transaction configuration. M0 supports one ordinary deferred transaction, no savepoints.
#[derive(Clone, Copy, Debug, Default)]
pub struct TransactionOptions {}

/// Driver-independent, thread-shareable database ownership.
pub trait Database: Send + Sync + 'static {
    /// Return the native renderer identity.
    fn dialect(&self) -> DialectId;
    /// Execute one statement, returning all owned data before releasing its lease.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>;
    /// Acquire one exclusive native session; cancellation must clean up or retire it.
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>>;
    /// Reject new work and wait for held leases. Cancellation may be re-awaited.
    fn close(&self) -> SqlFuture<'_, ()>;
}

/// An exclusive native transaction; implementations must roll back uncommitted Drop.
///
/// Returned rows own their data. Commit/rollback consume the session; commit cancellation
/// can have an unknown outcome and must never cause an automatic replay.
pub trait TransactionSession: Send + 'static {
    /// Run on this session's connection. The mutable borrow lasts across await.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>;
    /// Consume the session, acknowledging commit or returning its native error.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()>;
    /// Consume the session and explicitly acknowledge rollback.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()>;
}

/// One owned database and policy. Share with `Arc<Engine>` deliberately; no implicit Clone.
pub struct Engine {
    /// One runtime-selected adapter; sharing requires an explicit outer Arc.
    database: Box<dyn Database>,
    /// Copied into each transaction so its query policy cannot drift mid-operation.
    limits: QueryLimits,
}

impl Engine {
    /// Compare explicit storage definitions without querying the adapter or inferring renames.
    pub fn diff(
        &self,
        source: &crate::TableDefinition,
        target: &crate::TableDefinition,
    ) -> Result<crate::SchemaDiff> {
        crate::SchemaDiff::between(source, target)
    }
    /// Execute a wholly validated DDL plan on one owned transaction.
    ///
    /// Results are owned and ordered exactly like [`QueryPlan::queries`]. Validation
    /// precedes acquisition; the first statement error stops execution and rolls back,
    /// preserving both the original error and any secondary cleanup error. Cancellation
    /// drops the owned session and initiates the adapter's rollback-on-drop contract.
    /// Commit cancellation/failure can have an unknown outcome and is never retried.
    ///
    /// # Errors
    /// Returns validation/acquisition/statement/commit failures. Dialects without
    /// transactional DDL (MySQL) are rejected before any connection is acquired.
    pub async fn execute_plan(&self, plan: &QueryPlan) -> Result<Vec<QueryResult>> {
        plan.validate(self.database.dialect(), self.limits)?;
        self.database.dialect().require(
            self.database.dialect().capabilities().transactional_ddl,
            "dialect does not support transactional DDL plans",
        )?;
        let mut transaction = self.begin().await?;
        let results = match transaction.execute_plan(plan).await {
            Ok(results) => results,
            Err(primary) => return Err(transaction.rollback_after(primary).await),
        };
        // Commit only after every statement has returned a complete owned outcome.
        transaction.commit().await?;
        Ok(results)
    }

    /// Inspect the adapter's dialect without acquiring a native connection.
    pub fn dialect(&self) -> DialectId {
        self.database.dialect()
    }

    /// Construct a detached select; no executor is captured by the returned value.
    pub fn select(
        &self,
        columns: impl IntoIterator<Item = impl Into<crate::Selector>>,
    ) -> crate::Select {
        crate::Select::new(columns)
    }

    /// Construct a detached insert that may also be executed on an explicit transaction.
    pub fn insert(&self, table: impl Into<crate::Identifier>) -> crate::Insert {
        crate::Insert::new(table)
    }

    /// Construct a detached update requiring an explicit filter/all-rows choice.
    pub fn update(&self, table: impl Into<crate::Identifier>) -> crate::Update {
        crate::Update::new(table)
    }

    /// Construct a detached delete requiring an explicit filter/all-rows choice.
    pub fn delete(&self, table: impl Into<crate::Identifier>) -> crate::Delete {
        crate::Delete::new(table)
    }

    /// Describe table creation without installation or schema introspection.
    pub fn create(&self, table: impl Into<String>) -> crate::Create {
        crate::Create::new(table)
    }

    /// Describe explicit table alterations without migration history or effects.
    pub fn alter(&self, table: impl Into<String>) -> crate::Alter {
        crate::Alter::new(table)
    }

    /// Describe a checked table drop; execution remains explicit.
    pub fn drop_table(&self, table: impl Into<String>) -> crate::TableCommand {
        crate::TableCommand::drop_table(table)
    }

    /// Describe a checked rename; no SQL is executed by this convenience method.
    pub fn rename_table(
        &self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> crate::TableCommand {
        crate::TableCommand::rename(from, to)
    }

    /// Describe SQLite DELETE-style truncate; its sequence is not reset.
    pub fn truncate(&self, table: impl Into<String>) -> crate::TableCommand {
        crate::TableCommand::truncate(table)
    }

    /// Own a native adapter with the default finite query budgets.
    pub fn new(database: impl Database) -> Self {
        Self {
            database: Box::new(database),
            limits: QueryLimits::default(),
        }
    }

    /// Replace per-statement budgets before sharing this Engine.
    pub fn with_limits(mut self, limits: QueryLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Render and execute explicitly, releasing a normal lease before returning.
    ///
    /// # Errors
    /// Returns render, input/output limit, admission/acquisition or native failures.
    /// A cancelled or failed write may already have effects; no retry is attempted.
    pub async fn execute(&self, query: &(impl Render + Sync)) -> Result<QueryResult> {
        let query = query.render(self.database.dialect())?;
        query.validate(self.limits)?;
        self.database.run(query, self.limits).await
    }

    /// Return owned rows from one explicit statement.
    pub async fn fetch(&self, query: &(impl Render + Sync)) -> Result<Vec<Row>> {
        Ok(self.execute(query).await?.rows)
    }

    /// Acquire one transaction. Explicitly pass it to composed functions by `&mut`.
    ///
    /// # Errors
    /// Returns admission, acquisition or native begin failure without publishing a guard.
    pub async fn begin(&self) -> Result<Transaction> {
        let session = self.database.begin(TransactionOptions::default()).await?;
        Ok(Transaction {
            session,
            dialect: self.database.dialect(),
            limits: self.limits,
        })
    }

    /// Await idempotent close; callers supply their own deadline and may re-await.
    pub async fn close(&self) -> Result<()> {
        self.database.close().await
    }
}

/// Exclusive transaction ownership, including across await; completion consumes it.
pub struct Transaction {
    /// One exclusive driver-owned session; never cloned or implicitly shared.
    session: Box<dyn TransactionSession>,
    /// Renderer identity retained from the originating Engine.
    dialect: DialectId,
    /// The Engine's budgets frozen when the transaction began.
    limits: QueryLimits,
}

impl Transaction {
    /// Execute a validated plan inside this caller-owned transaction, in source order.
    ///
    /// Returns one owned outcome per statement. This method never commits or consumes
    /// the transaction; on failure the caller chooses explicit rollback/rollback_after.
    /// Cancelling this borrowed future retains caller ownership; the adapter's active
    /// statement cancellation policy still applies. Dropping the owner initiates rollback.
    ///
    /// # Errors
    /// Rejects a mismatched/invalid/nontransactional-DDL plan before the first run.
    pub async fn execute_plan(&mut self, plan: &QueryPlan) -> Result<Vec<QueryResult>> {
        plan.validate(self.dialect, self.limits)?;
        self.dialect.require(
            self.dialect.capabilities().transactional_ddl,
            "dialect does not support transactional DDL plans",
        )?;
        let mut results = Vec::with_capacity(plan.queries().len());
        for query in plan.queries() {
            // A single mutable session borrow keeps every statement on the same lease.
            results.push(self.session.run(query.clone(), self.limits).await?);
        }
        Ok(results)
    }

    /// Execute on the already leased connection, without implicit pool reacquisition.
    pub async fn execute(&mut self, query: &(impl Render + Sync)) -> Result<QueryResult> {
        let query = query.render(self.dialect)?;
        query.validate(self.limits)?;
        self.session.run(query, self.limits).await
    }

    /// Return owned rows while retaining exclusive transaction ownership.
    pub async fn fetch(&mut self, query: &(impl Render + Sync)) -> Result<Vec<Row>> {
        Ok(self.execute(query).await?.rows)
    }

    /// Consume and commit. Lost acknowledgement means an unknown outcome; never retry blindly.
    pub async fn commit(self) -> Result<()> {
        self.session.commit().await
    }

    /// Consume and explicitly roll back, preserving a native error if cleanup fails.
    pub async fn rollback(self) -> Result<()> {
        self.session.rollback().await
    }

    /// Roll back after a failed statement, returning the original failure and retaining
    /// any secondary rollback error through [`SqlError::cleanup_error`].
    pub async fn rollback_after(self, primary: SqlError) -> SqlError {
        match self.rollback().await {
            Ok(()) => primary,
            Err(cleanup) => primary.with_cleanup(cleanup),
        }
    }
}
