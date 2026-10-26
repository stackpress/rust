//! Explicit pool ownership and exclusive transaction composition.

use std::future::Future;
use std::pin::Pin;

use crate::{DialectId, Query, QueryLimits, QueryPlan, QueryResult, Render, Result, Row, SqlError};

/// A driver future that can cross executor threads without widening borrowed lifetimes.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Row, SqlError, SqlFuture};
/// fn ready_id<'a>(row: &'a Row) -> SqlFuture<'a, i64> {
///     Box::pin(async move { row.try_get("id") })
/// }
/// # tokio::runtime::Builder::new_current_thread()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// let row = Row::new(vec![("id".into(), 7.into())]);
/// assert_eq!(ready_id(&row).await?, 7);
/// #         Ok::<(), stackpress_sql::SqlError>(())
/// #     })?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub type SqlFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

/// Transaction configuration: one ordinary deferred transaction, no savepoints.
///
/// The empty options value currently selects an ordinary deferred transaction.
/// Savepoints, configurable isolation and callback syntax are outside this API.
///
/// # Examples
///
/// ```
/// use stackpress_sql::TransactionOptions;
/// let options = TransactionOptions::default();
/// let _: TransactionOptions = options;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct TransactionOptions {}

/// Driver-independent, thread-shareable database ownership.
///
/// Adapter implementers own pooling, admission, preparation/arity, native bindings,
/// output collection limits, and cancellation cleanup. [`crate::Engine`] validates/renderers
/// first but direct trait callers must also receive correct validation. Run returns
/// complete owned results and releases normal leases; begin pins an exclusive session.
/// No automatic retry is permitted. See [`crate::TransactionSession`] for completion/drop.
///
/// # Errors
/// Native/admission/closed/arity/decode/limit errors retain their typed kind and source.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{
///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
/// };
/// # mod support {
/// #     include!(concat!(
/// #         env!("CARGO_MANIFEST_DIR"),
/// #         "/examples/support/mod.rs"
/// #     ));
/// # }
/// # tokio::runtime::Builder::new_current_thread()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// let database = support::RecordingDatabase::default();
/// assert_eq!(database.dialect(), DialectId::Sqlite);
/// let query =
///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
/// let result = database.run(query.clone(), QueryLimits::default()).await?;
/// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
/// let mut session = database.begin(TransactionOptions::default()).await?;
/// session.run(query, QueryLimits::default()).await?;
/// session.commit().await?;
/// let session = database.begin(TransactionOptions::default()).await?;
/// session.rollback().await?;
/// database.close().await?;
/// #         Ok::<(), stackpress_sql::SqlError>(())
/// #     })?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub trait Database: Send + Sync + 'static {
    /// Return a stable native renderer identity for this adapter's lifetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn dialect(&self) -> DialectId;
    /// Execute one statement, returning all owned data before releasing its lease.
    ///
    /// # Errors
    /// Propagates adapter validation, admission or native failures with stable error kinds.
    /// Cancellation follows the ownership and cleanup contract on this trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>;
    /// Acquire one exclusive native session; cancellation must clean up or retire it.
    ///
    /// # Errors
    /// Propagates adapter validation, admission or native failures with stable error kinds.
    /// Cancellation follows the ownership and cleanup contract on this trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>>;
    /// Reject new work and wait for held leases. Cancellation may be re-awaited.
    ///
    /// # Errors
    /// Propagates adapter validation, admission or native failures with stable error kinds.
    /// Cancellation follows the ownership and cleanup contract on this trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn close(&self) -> SqlFuture<'_, ()>;
}

/// An exclusive native transaction; implementations must roll back uncommitted Drop.
///
/// Returned rows own their data. Commit/rollback consume the session; commit cancellation
/// can have an unknown outcome and must never cause an automatic replay.
///
/// Implementers keep one native connection for the session lifetime. Uncommitted
/// Drop must initiate rollback or retire the connection so later callers cannot see
/// its pending work. Never publish a lease after a cancelled statement before cleanup.
/// Completion futures own their sessions; commit acknowledgement loss is unknown
/// outcome. Methods enforce the same binding/result budgets as [`crate::Database::run`].
///
/// # Examples
///
/// ```
/// use stackpress_sql::{
///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
/// };
/// # mod support {
/// #     include!(concat!(
/// #         env!("CARGO_MANIFEST_DIR"),
/// #         "/examples/support/mod.rs"
/// #     ));
/// # }
/// # tokio::runtime::Builder::new_current_thread()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// let database = support::RecordingDatabase::default();
/// assert_eq!(database.dialect(), DialectId::Sqlite);
/// let query =
///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
/// let result = database.run(query.clone(), QueryLimits::default()).await?;
/// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
/// let mut session = database.begin(TransactionOptions::default()).await?;
/// session.run(query, QueryLimits::default()).await?;
/// session.commit().await?;
/// let session = database.begin(TransactionOptions::default()).await?;
/// session.rollback().await?;
/// database.close().await?;
/// #         Ok::<(), stackpress_sql::SqlError>(())
/// #     })?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub trait TransactionSession: Send + 'static {
    /// Run on this session's connection. The mutable borrow lasts across await.
    ///
    /// # Errors
    /// Propagates adapter validation, admission or native failures with stable error kinds.
    /// Cancellation follows the ownership and cleanup contract on this trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>;
    /// Consume the session, acknowledging commit or returning its native error.
    ///
    /// # Errors
    /// Propagates adapter validation, admission or native failures with stable error kinds.
    /// Cancellation follows the ownership and cleanup contract on this trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()>;
    /// Consume the session and explicitly acknowledge rollback.
    ///
    /// # Errors
    /// Propagates adapter validation, admission or native failures with stable error kinds.
    /// Cancellation follows the ownership and cleanup contract on this trait.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{
    ///     Database, DialectId, QueryLimits, Render, Sql, SqlError, TransactionOptions,
    /// };
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// let database = support::RecordingDatabase::default();
    /// assert_eq!(database.dialect(), DialectId::Sqlite);
    /// let query =
    ///     Sql::literal(database.dialect(), "SELECT 7 AS id").render(database.dialect())?;
    /// let result = database.run(query.clone(), QueryLimits::default()).await?;
    /// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    /// let mut session = database.begin(TransactionOptions::default()).await?;
    /// session.run(query, QueryLimits::default()).await?;
    /// session.commit().await?;
    /// let session = database.begin(TransactionOptions::default()).await?;
    /// session.rollback().await?;
    /// database.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()>;
}

/// One owned database and policy. Share with `Arc<Engine>` deliberately; no implicit Clone.
///
/// An engine owns one [`crate::Database`] and finite [`crate::QueryLimits`]. Builders are detached;
/// executing takes a borrow and returns owned results. Share it deliberately with
/// `Arc<Engine>`. There is no implicit retry, current transaction, or raw mutable
/// connection getter. Examples use a recording adapter that does not interpret SQL;
/// use stackpress-sql-sqlite for native SQLite storage.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Create, Engine, Field, Render, SqlError, SqlType};
/// # mod support {
/// #     include!(concat!(
/// #         env!("CARGO_MANIFEST_DIR"),
/// #         "/examples/support/mod.rs"
/// #     ));
/// # }
/// # tokio::runtime::Builder::new_current_thread()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
/// let recorder = support::RecordingDatabase::default();
/// let state = recorder.state.clone();
/// let engine = Engine::new(recorder);
/// let plan = Create::new("profile")
///     .add_field(Field::new("id", SqlType::Integer))
///     .render(engine.dialect())?;
/// let outcomes = engine.execute_plan(&plan).await?;
/// assert_eq!(outcomes.len(), 1);
/// assert_eq!(state.lock().unwrap().committed.len(), 1);
/// engine.close().await?;
/// #         Ok::<(), stackpress_sql::SqlError>(())
/// #     })?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub struct Engine {
    /// One runtime-selected adapter; sharing requires an explicit outer Arc.
    database: Box<dyn Database>,
    /// Copied into each transaction so its query policy cannot drift mid-operation.
    limits: QueryLimits,
}

impl Engine {
    /// Compare explicit storage definitions without querying the adapter or inferring renames.
    ///
    /// # Errors
    /// Returns definition validation, same-table or unsupported physical-reordering errors; no IO.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Engine, Field, SchemaDiff, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let source = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .definition()?;
    /// assert_eq!(engine.diff(&source, &source)?, SchemaDiff::NoChanges);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn diff(
        &self,
        source: &crate::TableDefinition,
        target: &crate::TableDefinition,
    ) -> Result<crate::SchemaDiff> {
        crate::SchemaDiff::between(source, target)
    }
    /// Execute a wholly validated DDL plan on one owned transaction.
    ///
    /// Results are owned and ordered exactly like [`crate::QueryPlan::queries`]. Budgets
    /// apply per statement; the returned vector retains all outcomes until the caller drops it.
    /// Validation
    /// precedes acquisition; the first statement error stops execution and rolls back,
    /// preserving both the original error and any secondary cleanup error. Cancellation
    /// drops the owned session and initiates the adapter's rollback-on-drop contract.
    /// Commit cancellation/failure can have an unknown outcome and is never retried.
    ///
    /// # Errors
    /// Returns validation/acquisition/statement/commit failures. Dialects without
    /// transactional DDL (MySQL) are rejected before any connection is acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Engine, Field, Render, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let plan = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .render(engine.dialect())?;
    /// let outcomes = engine.execute_plan(&plan).await?;
    /// assert_eq!(outcomes.len(), 1);
    /// assert_eq!(state.lock().unwrap().committed.len(), 1);
    /// engine.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Engine, Field, Render, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let plan = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .render(engine.dialect())?;
    /// let outcomes = engine.execute_plan(&plan).await?;
    /// assert_eq!(outcomes.len(), 1);
    /// assert_eq!(state.lock().unwrap().committed.len(), 1);
    /// engine.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn dialect(&self) -> DialectId {
        self.database.dialect()
    }

    /// Construct a detached select; no executor is captured by the returned value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine.select(["id"]).from("profile");
    /// assert!(query.build().selectors.len() == 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn select(
        &self,
        columns: impl IntoIterator<Item = impl Into<crate::Selector>>,
    ) -> crate::Select {
        crate::Select::new(columns)
    }

    /// Construct a detached insert that may also be executed on an explicit transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine.insert("profile").columns(["name"]).values(["Ada"]);
    /// assert!(query.build().rows.len() == 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn insert(&self, table: impl Into<crate::Identifier>) -> crate::Insert {
        crate::Insert::new(table)
    }

    /// Construct a detached update requiring an explicit filter/all-rows choice.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine
    ///     .update("profile")
    ///     .set("name", "Ada")
    ///     .where_eq("id", 7);
    /// assert!(query.assignments().len() == 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn update(&self, table: impl Into<crate::Identifier>) -> crate::Update {
        crate::Update::new(table)
    }

    /// Construct a detached delete requiring an explicit filter/all-rows choice.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine.delete("profile").where_eq("id", 7);
    /// assert!(query.build().filters.len() == 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn delete(&self, table: impl Into<crate::Identifier>) -> crate::Delete {
        crate::Delete::new(table)
    }

    /// Describe table creation without installation or schema introspection.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Field, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine
    ///     .create("profile")
    ///     .add_field(Field::new("id", SqlType::Integer));
    /// assert!(query.build()?.fields.len() == 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn create(&self, table: impl Into<String>) -> crate::Create {
        crate::Create::new(table)
    }

    /// Describe explicit table alterations without migration history or effects.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Field, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine
    ///     .alter("profile")
    ///     .add_field(Field::new("active", SqlType::Boolean));
    /// assert!(query.build()?.operations.len() == 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn alter(&self, table: impl Into<String>) -> crate::Alter {
        crate::Alter::new(table)
    }

    /// Describe a checked table drop; execution remains explicit.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Render, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine.drop_table("profile");
    /// assert!(
    ///     query
    ///         .render(engine.dialect())?
    ///         .sql()
    ///         .starts_with("DROP TABLE")
    /// );
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn drop_table(&self, table: impl Into<String>) -> crate::TableCommand {
        crate::TableCommand::drop_table(table)
    }

    /// Describe a checked rename; no SQL is executed by this convenience method.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Render, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine.rename_table("profile", "person");
    /// assert!(query.render(engine.dialect())?.sql().contains("RENAME TO"));
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn rename_table(
        &self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> crate::TableCommand {
        crate::TableCommand::rename(from, to)
    }

    /// Describe native truncate; SQLite uses DELETE without resetting its sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Render, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = engine.truncate("profile");
    /// assert!(
    ///     query
    ///         .render(engine.dialect())?
    ///         .sql()
    ///         .starts_with("DELETE FROM")
    /// );
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn truncate(&self, table: impl Into<String>) -> crate::TableCommand {
        crate::TableCommand::truncate(table)
    }

    /// Own a native adapter with the default finite query budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Engine, Field, Render, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let plan = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .render(engine.dialect())?;
    /// let outcomes = engine.execute_plan(&plan).await?;
    /// assert_eq!(outcomes.len(), 1);
    /// assert_eq!(state.lock().unwrap().committed.len(), 1);
    /// engine.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn new(database: impl Database) -> Self {
        Self {
            database: Box::new(database),
            limits: QueryLimits::default(),
        }
    }

    /// Replace per-statement budgets before sharing this Engine.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, ErrorKind, QueryLimits, Select, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let engine = engine.with_limits(QueryLimits {
    ///     binds: 0,
    ///     ..QueryLimits::default()
    /// });
    /// let query = Select::default().from("profile").where_eq("id", 7);
    /// assert_eq!(
    ///     engine.execute(&query).await.unwrap_err().kind(),
    ///     ErrorKind::Limit
    /// );
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn with_limits(mut self, limits: QueryLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Render and execute explicitly, releasing a normal lease before returning.
    ///
    /// # Errors
    /// Returns render, input/output limit, admission/acquisition or native failures.
    /// A cancelled or failed write may already have effects; no retry is attempted.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Insert, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let query = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// let outcome = engine.execute(&query).await?;
    /// assert_eq!(outcome.affected, 1);
    /// assert_eq!(state.lock().unwrap().committed.len(), 1);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn execute(&self, query: &(impl Render + Sync)) -> Result<QueryResult> {
        let query = query.render(self.database.dialect())?;
        query.validate(self.limits)?;
        self.database.run(query, self.limits).await
    }

    /// Return owned rows from one explicit statement.
    ///
    /// # Errors
    /// Returns the same rendering, admission, driver and output-budget failures as [`Self::execute`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Select, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let rows = engine.fetch(&Select::new(["id"]).from("profile")).await?;
    /// assert_eq!(rows[0].try_get::<i64>("id")?, 7);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn fetch(&self, query: &(impl Render + Sync)) -> Result<Vec<Row>> {
        Ok(self.execute(query).await?.rows)
    }

    /// Acquire one transaction. Explicitly pass it to composed functions by `&mut`.
    ///
    /// # Errors
    /// Returns admission, acquisition or native begin failure without publishing a guard.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let mut transaction = engine.begin().await?;
    /// transaction
    ///     .execute(&Delete::new("profile").where_eq("id", 7))
    ///     .await?;
    /// transaction.rollback().await?;
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn begin(&self) -> Result<Transaction> {
        let session = self.database.begin(TransactionOptions::default()).await?;
        Ok(Transaction {
            session,
            dialect: self.database.dialect(),
            limits: self.limits,
        })
    }

    /// Await idempotent close; callers supply their own deadline and may re-await.
    ///
    /// # Errors
    /// Propagates adapter close failures. Cancellation does not cancel native shutdown;
    /// callers may re-await close and supply their own deadline.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// engine.close().await?;
    /// engine.close().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn close(&self) -> Result<()> {
        self.database.close().await
    }
}

/// Exclusive transaction ownership, including across await; completion consumes it.
///
/// Each query future exclusively borrows the guard across await. Pass `&mut
/// Transaction` to composed functions; do not reacquire through Engine inside a
/// one-connection transaction. Commit/rollback consume ownership. Uncommitted drop
/// initiates adapter rollback; explicit rollback awaits acknowledgement. A failed or
/// cancelled statement may leave the adapter rollback-only. Commit cancellation
/// has an unknown outcome and must not be automatically replayed.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Engine, Insert, SqlError};
/// # mod support {
/// #     include!(concat!(
/// #         env!("CARGO_MANIFEST_DIR"),
/// #         "/examples/support/mod.rs"
/// #     ));
/// # }
/// # tokio::runtime::Builder::new_current_thread()
/// #     .build()
/// #     .unwrap()
/// #     .block_on(async {
/// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
/// let recorder = support::RecordingDatabase::default();
/// let state = recorder.state.clone();
/// let engine = Engine::new(recorder);
/// let mut transaction = engine.begin().await?;
/// let insert = Insert::new("profile").columns(["name"]).values(["Ada"]);
/// transaction.execute(&insert).await?;
/// assert!(state.lock().unwrap().committed.is_empty());
/// transaction.commit().await?;
/// assert_eq!(state.lock().unwrap().committed.len(), 1);
/// #         Ok::<(), stackpress_sql::SqlError>(())
/// #     })?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
///
/// The guard can move between executor threads, but shared concurrent access is unavailable:
///
/// ```compile_fail
/// use stackpress_sql::Transaction;
/// fn require_sync<T: Sync>() {}
/// require_sync::<Transaction>();
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Create, Engine, Field, Render, SqlError, SqlType};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let mut transaction = engine.begin().await?;
    /// let plan = Create::new("profile")
    ///     .add_field(Field::new("id", SqlType::Integer))
    ///     .render(engine.dialect())?;
    /// assert_eq!(transaction.execute_plan(&plan).await?.len(), 1);
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// transaction.rollback().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Errors
    /// Returns rendering, validation, native or output-limit failures. The caller still
    /// owns the transaction and chooses rollback; an adapter may mark it rollback-only.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Insert, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let mut transaction = engine.begin().await?;
    /// let insert = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// transaction.execute(&insert).await?;
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// transaction.commit().await?;
    /// assert_eq!(state.lock().unwrap().committed.len(), 1);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn execute(&mut self, query: &(impl Render + Sync)) -> Result<QueryResult> {
        let query = query.render(self.dialect)?;
        query.validate(self.limits)?;
        self.session.run(query, self.limits).await
    }

    /// Return owned rows while retaining exclusive transaction ownership.
    ///
    /// # Errors
    /// Returns rendering, validation, native or output-limit failures. The caller still
    /// owns the transaction and chooses rollback; an adapter may mark it rollback-only.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Select, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let mut transaction = engine.begin().await?;
    /// let rows = transaction
    ///     .fetch(&Select::new(["id"]).from("profile"))
    ///     .await?;
    /// assert_eq!(rows[0].try_get::<i64>("id")?, 7);
    /// transaction.rollback().await?;
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn fetch(&mut self, query: &(impl Render + Sync)) -> Result<Vec<Row>> {
        Ok(self.execute(query).await?.rows)
    }

    /// Consume and commit. Lost acknowledgement means an unknown outcome; never retry blindly.
    ///
    /// # Errors
    /// Returns the native commit failure. Cancellation or lost acknowledgement may leave
    /// the outcome unknown; no automatic retry is performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, Insert, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let mut transaction = engine.begin().await?;
    /// let insert = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// transaction.execute(&insert).await?;
    /// assert!(state.lock().unwrap().committed.is_empty());
    /// transaction.commit().await?;
    /// assert_eq!(state.lock().unwrap().committed.len(), 1);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn commit(self) -> Result<()> {
        self.session.commit().await
    }

    /// Consume and explicitly roll back, preserving a native error if cleanup fails.
    ///
    /// # Errors
    /// Returns the native rollback failure; the consumed session cannot be reused.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let transaction = engine.begin().await?;
    /// transaction.rollback().await?;
    /// assert_eq!(state.lock().unwrap().rollbacks, 1);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn rollback(self) -> Result<()> {
        self.session.rollback().await
    }

    /// Roll back after a failed statement, returning the original failure and retaining
    /// any secondary rollback error through [`crate::SqlError::cleanup_error`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Engine, ErrorKind, SqlError};
    /// # mod support {
    /// #     include!(concat!(
    /// #         env!("CARGO_MANIFEST_DIR"),
    /// #         "/examples/support/mod.rs"
    /// #     ));
    /// # }
    /// # tokio::runtime::Builder::new_current_thread()
    /// #     .build()
    /// #     .unwrap()
    /// #     .block_on(async {
    /// // Recording fixture from examples/support/mod.rs; it does not interpret SQL.
    /// let recorder = support::RecordingDatabase::default();
    /// let state = recorder.state.clone();
    /// let engine = Engine::new(recorder);
    /// let transaction = engine.begin().await?;
    /// let error = transaction
    ///     .rollback_after(SqlError::new(ErrorKind::Driver, "write failed"))
    ///     .await;
    /// assert_eq!(error.kind(), ErrorKind::Driver);
    /// assert_eq!(state.lock().unwrap().rollbacks, 1);
    /// #         Ok::<(), stackpress_sql::SqlError>(())
    /// #     })?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub async fn rollback_after(self, primary: SqlError) -> SqlError {
        match self.rollback().await {
            Ok(()) => primary,
            Err(cleanup) => primary.with_cleanup(cleanup),
        }
    }
}
