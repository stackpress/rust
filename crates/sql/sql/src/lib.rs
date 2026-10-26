//! Runtime-independent SQL ownership and execution contracts.
//!
//! Detached builders render SQLite/PostgreSQL/MySQL CRUD and ordered CREATE/ALTER plans.
//! [`SchemaDiff`] compares explicit [`TableDefinition`] values without native introspection.
//! [`Engine::execute_plan`] validates before acquisition and executes transactional DDL
//! on one session. MySQL transactional DDL is explicitly unsupported. PostgreSQL/MySQL
//! rendering does not provide production network adapters; SQLite has a separate adapter.
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
//!
//! Rename reconciliation is explicit and preserves any additional field changes:
//!
//! ```
//! use stackpress_sql::{Create, DialectId, Field, SqlType};
//! let source = Create::new("profile")
//!     .add_field(Field::new("name", SqlType::Text)).definition()?;
//! let target = Create::new("profile")
//!     .add_field(Field::new("display", SqlType::Text)).definition()?;
//! let diff = source.diff(&target)?.rename_field("name", "display")?;
//! let plan = diff.render(DialectId::Postgres)?.unwrap();
//! assert!(plan.queries()[0].sql().contains("RENAME COLUMN"));
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```
//!
//! A caller selects installation explicitly and receives ordered, owned outcomes:
//!
//! ```
//! use stackpress_sql::{Create, Engine, Field, QueryResult, Result, SqlType};
//! async fn install(database: &Engine) -> Result<Vec<QueryResult>> {
//!     let plan = Create::new("profile")
//!         .add_field(Field::new("id", SqlType::Integer))
//!         .render(database.dialect())?;
//!     database.execute_plan(&plan).await
//! }
//! ```
//!
//! Inspect a whole DDL plan before handing any statement to an executor:
//!
//! ```
//! use stackpress_sql::{Create, DialectId, Field, Index, SqlType};
//! let plan = Create::new("profile")
//!     .add_field(Field::new("id", SqlType::Integer).not_null().auto_increment())
//!     .add_field(Field::new("name", SqlType::Text).default(""))
//!     .primary_key(["id"])
//!     .add_index(Index::new("profile_name", ["name"]).unique())
//!     .render(DialectId::Sqlite)?;
//! assert_eq!(plan.queries().len(), 2);
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```
//!
//! DML is one statement with stable column order and separate owned binds:
//!
//! ```
//! use stackpress_sql::{DialectId, Insert, Render, Update};
//! let insert = Insert::new("profile").columns(["name"])
//!     .values(["Ada"]).values(["Grace"]).returning_all();
//! assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
//! let update = Update::new("profile").set("name", "Ada Lovelace").where_eq("id", 1);
//! assert_eq!(update.render(DialectId::Sqlite)?.binds().len(), 2);
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```
//!
//! JSON paths are explicit data, and JSON null differs from an absent path:
//!
//! ```
//! use stackpress_sql::{DialectId, JsonPath, JsonScalar, JsonSegment, Render, Select};
//! let path = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
//! let query = Select::default().from("profile")
//!     .where_json("data", path, JsonScalar::String("admin".into()));
//! assert!(query.render(DialectId::Sqlite)?.sql().contains("json_type"));
//! # Ok::<(), stackpress_sql::SqlError>(())
//! ```
//!
//! Builders cannot be awaited, and rendering always selects a dialect explicitly:
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
