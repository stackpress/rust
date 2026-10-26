//! Explicit renderer capabilities, independent from the presence of a native driver.

use crate::identifier::{invalid, unsupported};
use crate::{DialectId, Result};

/// Observable renderer differences. These flags make no claim about installed adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialectCapabilities {
    /// INSERT/UPDATE/DELETE RETURNING projections.
    pub returning: bool,
    /// DDL can participate in an ordinary rollback-capable transaction.
    pub transactional_ddl: bool,
    /// Native alteration of field type/default/nullability and primary/foreign keys.
    pub alter_constraints: bool,
    /// Native FULL OUTER JOIN.
    pub full_join: bool,
    /// Native DROP TABLE/TRUNCATE CASCADE.
    pub cascade: bool,
    /// Native CREATE INDEX IF NOT EXISTS.
    pub idempotent_indexes: bool,
    /// Numeric JSON equality distinguishes integer and real storage kinds.
    pub distinct_json_number_kinds: bool,
    /// Native text can retain a NUL byte.
    pub text_nul: bool,
}

impl DialectId {
    /// Inspect the explicit rendering matrix without connecting to a database.
    /// PostgreSQL/MySQL rendering does not supply production network adapters.
    pub const fn capabilities(self) -> DialectCapabilities {
        match self {
            Self::Sqlite => DialectCapabilities {
                returning: true,
                transactional_ddl: true,
                alter_constraints: false,
                full_join: true,
                cascade: false,
                idempotent_indexes: true,
                distinct_json_number_kinds: true,
                text_nul: true,
            },
            Self::Postgres => DialectCapabilities {
                returning: true,
                transactional_ddl: true,
                alter_constraints: true,
                full_join: true,
                cascade: true,
                idempotent_indexes: true,
                distinct_json_number_kinds: false,
                text_nul: false,
            },
            Self::Mysql => DialectCapabilities {
                returning: false,
                transactional_ddl: false,
                alter_constraints: true,
                full_join: false,
                cascade: false,
                idempotent_indexes: false,
                distinct_json_number_kinds: false,
                text_nul: true,
            },
        }
    }

    /// Quote one bounded segment without interpreting literal dots or embedded quotes.
    pub(crate) fn quote(self, name: &str) -> Result<String> {
        // Keep common validation centralized, then reject native truncation/length hazards.
        crate::identifier::quote(name)?;
        if (self == Self::Postgres && name.len() > 63)
            || (self == Self::Mysql && name.chars().count() > 64)
        {
            return Err(invalid("SQL identifier exceeds dialect limit"));
        }
        let quote = if self == Self::Mysql { '`' } else { '"' };
        Ok(format!(
            "{quote}{}{quote}",
            name.replace(quote, &format!("{quote}{quote}"))
        ))
    }

    /// Validate that a renderer capability exists before returning any plan/query.
    pub(crate) fn require(self, supported: bool, message: &'static str) -> Result<()> {
        if !supported {
            return Err(unsupported(message));
        }
        Ok(())
    }
}
