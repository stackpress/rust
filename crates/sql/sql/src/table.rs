//! Explicit destructive/rename table statements, detached from execution.

use crate::expr::RenderContext;
use crate::identifier::invalid;
use crate::{DialectId, Query, Render, Result};

/// Checked single-table operations. Applications choose when to execute them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TableCommand {
    /// Drop a literal table; SQLite rejects cascade.
    Drop {
        /// One literal table name.
        table: String,
        /// Explicit idempotence choice.
        if_exists: bool,
        /// Requested cascading native semantics, unsupported on SQLite.
        cascade: bool,
    },
    /// Rename a table without changing its data.
    Rename {
        /// Existing literal name.
        from: String,
        /// Distinct target literal name.
        to: String,
    },
    /// Remove rows using SQLite DELETE; it never resets the autoincrement sequence.
    Truncate {
        /// One literal table name.
        table: String,
        /// SQLite rejects cascade instead of silently discarding it.
        cascade: bool,
    },
}

impl TableCommand {
    /// Describe an explicit non-idempotent table drop without executing it.
    pub fn drop_table(table: impl Into<String>) -> Self {
        Self::Drop {
            table: table.into(),
            if_exists: false,
            cascade: false,
        }
    }

    /// Describe a literal table rename.
    pub fn rename(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::Rename {
            from: from.into(),
            to: to.into(),
        }
    }

    /// Describe a DELETE-style truncate without cascade or sequence reset.
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
