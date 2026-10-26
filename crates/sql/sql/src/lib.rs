//! Build SQL, inspect its text and binds, then choose an executor explicitly.
//!
//! `stackpress-sql` supplies six detached builder families, SQLite/PostgreSQL/MySQL
//! renderers, owned rows and database/transaction contracts. The library has no
//! normal dependencies and requires no runtime to build or render a query.
//!
//! # Start with one query
//!
//! Import [`crate::Render`] to turn a builder into a checked [`crate::Query`]. This example runs
//! without a database:
//!
//! ```
//! use stackpress_sql::{DialectId, Render, Select, SqlValue};
//!
//! let select = Select::new(["id", "name"])
//!     .from("profile")
//!     .where_eq("name", "Ada")
//!     .order("id")
//!     .limit(10);
//! let query = select.render(DialectId::Sqlite)?;
//!
//! assert_eq!(
//!     query.sql(),
//!     "SELECT \"id\", \"name\" FROM \"profile\" WHERE \"name\" = ? ORDER BY \"id\" ASC LIMIT 10"
//! );
//! assert_eq!(query.binds(), &[SqlValue::Text("Ada".into())]);
//! assert_eq!(select.build().limit, Some(10));
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```
//!
//! The builder owns the description; `build()` borrows inspectable data. Rendering
//! quotes identifiers and keeps values in a separate bind list. It performs no IO.
//! Reuse the same builder with `render(DialectId::Postgres)` to get `$1` placeholders.
//!
//! From the repository root, run a complete example with:
//!
//! ```text
//! cargo run -p stackpress-sql --example build_inspect
//! ```
//!
//! It prints all six builder families' SQL and deliberate fixture binds. Applications
//! should choose explicitly whether inspection may disclose private bind data.
//!
//! # Choose the next task
//!
//! Each entry links to runnable API examples. The standalone programs can also be
//! run with `cargo run -p stackpress-sql --example <name>`.
//!
//! | Task | API | Standalone example |
//! | --- | --- | --- |
//! | Read and write rows | [`crate::Select`], [`crate::Insert`], [`crate::Update`], [`crate::Delete`] | `build_inspect` |
//! | Compose predicates, JSON paths and aliases | [`crate::Expr`], [`crate::JsonPath`], [`crate::Selector`], [`crate::TableRef`] | `build_inspect` |
//! | Compare renderer support | [`crate::DialectId::capabilities`], [`crate::DialectCapabilities`] | `capabilities` |
//! | Inspect CREATE/ALTER statements | [`crate::Create`], [`crate::Alter`], [`crate::QueryPlan`] | `build_inspect` |
//! | Compare explicit schemas and reconcile a rename | [`crate::TableDefinition`], [`crate::SchemaDiff`] | `schema_diff` |
//! | Execute a whole plan or compose a transaction | [`crate::Engine::execute_plan`], [`crate::Transaction`] | `execute_plan` |
//! | Decode an owned result into an application DTO | [`crate::Row`], [`crate::FromSqlValue`], [`crate::QueryResult`] | See [`crate::Row::try_get`] |
//! | Use caller-authored SQL | [`crate::Sql`], [`crate::TrustedSql`] | See [`crate::Sql::literal`] |
//!
//! # Execute deliberately
//!
//! Create an [`crate::Engine`] with a [`crate::Database`] adapter. Native SQLite is provided by the
//! separate `stackpress-sql-sqlite` crate. PostgreSQL/MySQL renderers have opt-in
//! native validation tests; production network adapters remain outside this crate.
//! The `execute_plan` example uses a recording adapter and an existing dev-only
//! Tokio runtime to run transaction orchestration without a server. It does not
//! interpret SQL or prove storage behavior.
//!
//! [`crate::Engine::execute`] and [`crate::Engine::fetch`] render and validate before asking the
//! adapter for IO. Their results own their data after the connection lease releases.
//! A mutation requires a filter or an explicit `all_rows()` choice:
//!
//! ```
//! use stackpress_sql::{Delete, DialectId, ErrorKind, Render, Update};
//!
//! assert_eq!(
//!     Delete::new("profile").render(DialectId::Sqlite).unwrap_err().kind(),
//!     ErrorKind::InvalidQuery
//! );
//! let update = Update::new("profile").set("active", false).all_rows();
//! assert!(update.render(DialectId::Sqlite)?.binds().len() == 1);
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```
//!
//! [`crate::Engine::begin`] creates an exclusive, non-Clone [`crate::Transaction`]. Pass it by
//! `&mut` across composed async functions, and consume it with `commit` or `rollback`.
//! Uncommitted drop initiates adapter rollback. A cancelled write may have executed;
//! commit cancellation can leave an unknown outcome. No operation retries implicitly.
//!
//! [`crate::Engine::execute_plan`] validates every statement before acquisition, uses one
//! transaction, returns ordered [`crate::QueryResult`] values, and rolls back at the first
//! failure while retaining the original error and any rollback failure. The
//! transaction method retains caller ownership and leaves completion to the caller.
//! MySQL plans can be inspected but atomic plan execution is rejected up front.
//!
//! # Validate before effects
//!
//! Renderers return typed errors for invalid definitions, unsupported capabilities,
//! nonfinite numbers, malformed identifiers and bounded-input overflow. Adapters own
//! prepared-parameter arity, native execution and bounded result collection.
//! [`crate::QueryLimits`] documents the default budgets. There are no crate feature flags.
//!
//! Ordinary strings are literal identifiers, including dots or function-looking
//! text. Use [`crate::Identifier::qualified`] for qualification and [`crate::Selector::expression`]
//! for expressions. [`crate::TrustedSql::audited`] asserts trusted authorship; it is not
//! sanitization. Raw SQL, custom types and defaults keep their dialect identity.
//!
//! JSON paths are typed data. JSON null, missing paths and SQL NULL remain distinct.
//! Membership means a scalar directly in the selected array, never recursive object
//! containment. PostgreSQL/MySQL compare JSON numbers 1 and 1.0 equally; SQLite can
//! distinguish their kinds. Dates use explicit [`crate::SqlValue::utc_text`] validation.
//!
//! Schemas are caller-supplied snapshots. Diff does not inspect a live database or
//! infer renames. SQLite reconstruction, physical column reordering, migration
//! history, callbacks/middleware and automatic DTO generation are outside this API.
//! Native SQLite type/key/FK changes return [`crate::ErrorKind::Unsupported`] before a plan
//! is exposed. See [`crate::Alter`] for network renderer limits and primary-name assumptions.
//!
//! Builders require an explicit dialect and cannot be awaited:
//!
//! ```compile_fail
//! use stackpress_sql::Select;
//! async fn invalid() { Select::default().from("profile").await; }
//! ```
//!
//! ```compile_fail
//! use stackpress_sql::{Render, Select};
//! let _ = Select::default().from("profile").render();
//! ```

mod ddl;
mod dialect;
mod diff;
mod dml;
mod engine;
mod error;
mod expr;
mod identifier;
mod json;
mod json_render;
mod network_ddl;
mod query;
mod schema;
mod table;
mod value;

pub use ddl::{Alter, AlterData, AlterOperation, Create, CreateData, QueryPlan};
pub use dialect::DialectCapabilities;
pub use diff::{SchemaDiff, TableDefinition};
pub use dml::{
    Delete, Insert, InsertData, Join, JoinKind, MutationData, Order, Select, SelectData, Sort,
    Update, WriteValue,
};
pub use engine::{
    Database, Engine, SqlFuture, Transaction, TransactionOptions, TransactionSession,
};
pub use error::{ErrorKind, Result, SqlError};
pub use expr::{Compare, Expr, Selector};
pub use identifier::{Identifier, TableRef};
pub use json::{JsonPath, JsonPathOptions, JsonScalar, JsonSegment, JsonValue};
pub use query::{DialectId, Query, QueryLimits, Render, Sql, TrustedSql};
pub use schema::{DefaultValue, Field, ForeignKey, Index, Length, ReferentialAction, SqlType};
pub use table::TableCommand;
pub use value::{ColumnMetadata, ColumnMetadataRef, FromSqlValue, QueryResult, Row, SqlValue};
