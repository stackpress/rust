//! Runtime-independent SQL ownership and execution contracts.
//!
//! This M0 foundation provides a small SQLite SELECT renderer and explicitly trusted
//! single statements. Schema builders and network dialect renderers are later milestones.
//! Adapters own IO; applications explicitly share an [`Engine`] through `Arc` and pass
//! [`Transaction`] by mutable borrow. No host, driver or runtime is required here.
//!
//! ```
//! use stackpress_sql::{DialectId, Render, Select};
//! let query = Select::new(["name"]).from("profile").where_eq("id", 7_i64);
//! let rendered = query.render(DialectId::Sqlite)?;
//! assert_eq!(rendered.sql(), "SELECT \"name\" FROM \"profile\" WHERE \"id\" = ?");
//! assert_eq!(rendered.binds().len(), 1);
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```

mod engine;
mod error;
mod query;
mod value;

pub use engine::{
    Database, Engine, SqlFuture, Transaction, TransactionOptions, TransactionSession,
};
pub use error::{ErrorKind, Result, SqlError};
pub use query::{DialectId, Query, QueryLimits, Render, Select, Sql, TrustedSql};
pub use value::{FromSqlValue, QueryResult, Row, SqlValue};
