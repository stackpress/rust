//! Explicit destructive/rename table statements, detached from execution.

use crate::expr::RenderContext;
use crate::identifier::invalid;
use crate::{DialectId, Query, Render, Result};

/// Checked single-table operations. Applications choose when to execute them.
///
/// Construction has no effects. Render validates literal names and flags; CASCADE
/// is PostgreSQL-only. SQLite truncate uses DELETE and retains the sequence.
/// PostgreSQL/MySQL use native TRUNCATE, subject to their transaction semantics.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Render, TableCommand};
/// let drop = TableCommand::drop_table("profile").render(DialectId::Sqlite)?;
/// assert_eq!(drop.sql(), "DROP TABLE \"profile\"");
/// let rename = TableCommand::rename("profile", "person").render(DialectId::Postgres)?;
/// assert!(rename.sql().contains("RENAME TO"));
/// let truncate = TableCommand::truncate("person").render(DialectId::Sqlite)?;
/// assert_eq!(truncate.sql(), "DELETE FROM \"person\"");
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TableCommand {
    /// Drop a literal table; SQLite rejects cascade.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, TableCommand};
    /// let command = TableCommand::Drop {
    ///     table: "profile".into(),
    ///     if_exists: true,
    ///     cascade: false,
    /// };
    /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Drop {
        /// One literal table name.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Drop {
        ///     table: "profile".into(),
        ///     if_exists: true,
        ///     cascade: false,
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        table: String,
        /// Explicit idempotence choice.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Drop {
        ///     table: "profile".into(),
        ///     if_exists: true,
        ///     cascade: false,
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        if_exists: bool,
        /// Requested cascading native semantics, unsupported on SQLite.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Drop {
        ///     table: "profile".into(),
        ///     if_exists: true,
        ///     cascade: false,
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        cascade: bool,
    },
    /// Rename a table without changing its data.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, TableCommand};
    /// let command = TableCommand::Rename {
    ///     from: "profile".into(),
    ///     to: "person".into(),
    /// };
    /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Rename {
        /// Existing literal name.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Rename {
        ///     from: "profile".into(),
        ///     to: "person".into(),
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        from: String,
        /// Distinct target literal name.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Rename {
        ///     from: "profile".into(),
        ///     to: "person".into(),
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        to: String,
    },
    /// Remove rows using SQLite DELETE; it never resets the autoincrement sequence.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, TableCommand};
    /// let command = TableCommand::Truncate {
    ///     table: "profile".into(),
    ///     cascade: false,
    /// };
    /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Truncate {
        /// One literal table name.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Truncate {
        ///     table: "profile".into(),
        ///     cascade: false,
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        table: String,
        /// SQLite rejects cascade instead of silently discarding it.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{DialectId, Render, TableCommand};
        /// let command = TableCommand::Truncate {
        ///     table: "profile".into(),
        ///     cascade: false,
        /// };
        /// assert!(!command.render(DialectId::Postgres)?.sql().is_empty());
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        cascade: bool,
    },
}

impl TableCommand {
    /// Describe an explicit non-idempotent table drop without executing it.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, TableCommand};
    /// let drop = TableCommand::drop_table("profile").render(DialectId::Sqlite)?;
    /// assert_eq!(drop.sql(), "DROP TABLE \"profile\"");
    /// let rename = TableCommand::rename("profile", "person").render(DialectId::Postgres)?;
    /// assert!(rename.sql().contains("RENAME TO"));
    /// let truncate = TableCommand::truncate("person").render(DialectId::Sqlite)?;
    /// assert_eq!(truncate.sql(), "DELETE FROM \"person\"");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn drop_table(table: impl Into<String>) -> Self {
        Self::Drop {
            table: table.into(),
            if_exists: false,
            cascade: false,
        }
    }

    /// Describe a literal table rename.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, TableCommand};
    /// let drop = TableCommand::drop_table("profile").render(DialectId::Sqlite)?;
    /// assert_eq!(drop.sql(), "DROP TABLE \"profile\"");
    /// let rename = TableCommand::rename("profile", "person").render(DialectId::Postgres)?;
    /// assert!(rename.sql().contains("RENAME TO"));
    /// let truncate = TableCommand::truncate("person").render(DialectId::Sqlite)?;
    /// assert_eq!(truncate.sql(), "DELETE FROM \"person\"");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn rename(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Rename {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Describe a DELETE-style truncate without cascade or sequence reset.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, TableCommand};
    /// let drop = TableCommand::drop_table("profile").render(DialectId::Sqlite)?;
    /// assert_eq!(drop.sql(), "DROP TABLE \"profile\"");
    /// let rename = TableCommand::rename("profile", "person").render(DialectId::Postgres)?;
    /// assert!(rename.sql().contains("RENAME TO"));
    /// let truncate = TableCommand::truncate("person").render(DialectId::Sqlite)?;
    /// assert_eq!(truncate.sql(), "DELETE FROM \"person\"");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn truncate(table: impl Into<String>) -> Self {
        Self::Truncate {
            table: table.into(),
            cascade: false,
        }
    }
}

impl Render for TableCommand {
    /// Reject invalid names/cascade before returning a single validated statement.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        let context = RenderContext::new(dialect)?;
        let sql = match self {
            Self::Drop {
                table,
                if_exists,
                cascade,
            } => {
                let table = dialect.quote(table)?;
                dialect.require(
                    !*cascade || dialect.capabilities().cascade,
                    "dialect DROP cascade is unsupported",
                )?;
                format!(
                    "DROP TABLE {}{table}{}",
                    if *if_exists { "IF EXISTS " } else { "" },
                    if *cascade { " CASCADE" } else { "" }
                )
            }
            Self::Rename { from, to } => {
                let source = dialect.quote(from)?;
                let target = dialect.quote(to)?;
                if from.eq_ignore_ascii_case(to) {
                    return Err(invalid("table rename requires distinct names"));
                }
                format!("ALTER TABLE {source} RENAME TO {target}")
            }
            Self::Truncate { table, cascade } => {
                let table = dialect.quote(table)?;
                dialect.require(
                    !*cascade || dialect.capabilities().cascade,
                    "dialect truncate cascade is unsupported",
                )?;
                if dialect == DialectId::Sqlite {
                    format!("DELETE FROM {table}")
                } else {
                    format!(
                        "TRUNCATE TABLE {table}{}",
                        if *cascade { " CASCADE" } else { "" }
                    )
                }
            }
        };
        context.finish(sql)
    }
}
