//! Bounded native PostgreSQL execution on the caller's Tokio runtime.
//!
//! Importing this crate performs no IO. Open is eager; it never creates a database.
//! Driver types remain private. Share an Engine through Arc, not a cloned database.
//! Only acknowledged, reset ordinary/session completions may reuse a connection;
//! errors, cancellation and unclassifiable session state retire their lease.
//! Prepared-statement caching remains disabled. See the codec and lease policy below.
//!
//! Cancellation loses observation: a write or commit may already have taken effect.
//! Cleanup retains admission through bounded transport disposal while the runtime lives.
//! Outside a runtime, Drop disposes sockets by RAII, without claiming remote rollback
//! acknowledgement. Drain application tasks and await close before runtime shutdown.
//! Result caps bound retained first-party copies, not driver packet/server allocations.
//! Native date/decimal/UUID/array codecs and automatic retries are excluded.
//!
//! ```no_run
//! use stackpress_sql::Engine;
//! use stackpress_sql_postgres::{PostgresDatabase, PostgresOptions};
//! # async fn application(secret: String) -> stackpress_sql::Result<()> {
//! let database = PostgresDatabase::open(PostgresOptions::new(
//!     "db.internal", "application", "application", secret,
//! )).await?;
//! let engine = Engine::new(database);
//! engine.close().await?;
//! # Ok(())
//! # }
//! ```

//! A database owner cannot be cloned; share its Engine explicitly.
//!
//! ```compile_fail
//! use stackpress_sql_postgres::PostgresDatabase;
//! fn clone_owner(database: &PostgresDatabase) { let _: PostgresDatabase = database.clone(); }
//! ```
//!
//! A transaction retains one exclusive mutable borrow across each awaited query.
//!
//! ```compile_fail
//! use stackpress_sql::{TransactionSession, Query, QueryLimits};
//! async fn overlap(session: &mut dyn TransactionSession, query: Query) {
//!     let first = session.run(query.clone(), QueryLimits::default());
//!     let second = session.run(query, QueryLimits::default());
//!     first.await.unwrap();
//!     second.await.unwrap();
//! }
//! ```
//!
//! Completion consumes session ownership when the future is constructed.
//!
//! ```compile_fail
//! use stackpress_sql::TransactionSession;
//! fn reuse(session: Box<dyn TransactionSession>) {
//!     let completion = session.commit();
//!     drop(session);
//!     drop(completion);
//! }
//! ```

#![doc = include_str!("../CODECS.md")]

mod collect;
mod database;
mod execute;
mod json;
mod lease;
mod options;
#[cfg(test)]
#[allow(dead_code)]
mod probe;
mod reuse;
mod session;

pub use database::PostgresDatabase;
pub use options::{PostgresOptions, PostgresTls};

#[cfg(test)]
mod tests;
