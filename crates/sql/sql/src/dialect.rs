//! Explicit renderer capabilities, independent from the presence of a native driver.

use crate::identifier::{invalid, unsupported};
use crate::{DialectId, Result};

/// Observable renderer differences. These flags make no claim about installed adapters.
///
/// Read these flags through [`crate::DialectId::capabilities`]. A true flag still requires
/// a valid definition and a compatible native server. Rendering never acquires IO.
///
/// # Examples
///
/// ```
/// use stackpress_sql::DialectId;
/// assert!(DialectId::Sqlite.capabilities().returning);
/// assert!(DialectId::Postgres.capabilities().cascade);
/// assert!(!DialectId::Mysql.capabilities().transactional_ddl);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DialectCapabilities {
    /// INSERT/UPDATE/DELETE RETURNING projections.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.returning, true);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub returning: bool,
    /// DDL can participate in an ordinary rollback-capable transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.transactional_ddl, true);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub transactional_ddl: bool,
    /// Native alteration of field type/default/nullability and primary/foreign keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.alter_constraints, false);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub alter_constraints: bool,
    /// Native FULL OUTER JOIN.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.full_join, true);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub full_join: bool,
    /// Native DROP TABLE/TRUNCATE CASCADE.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.cascade, false);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub cascade: bool,
    /// Native CREATE INDEX IF NOT EXISTS.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.idempotent_indexes, true);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub idempotent_indexes: bool,
    /// Numeric JSON equality distinguishes integer and real storage kinds.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.distinct_json_number_kinds, true);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub distinct_json_number_kinds: bool,
    /// Native text can retain a NUL byte.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// let value = DialectId::Sqlite.capabilities();
    /// assert_eq!(value.text_nul, true);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub text_nul: bool,
}

impl DialectId {
    /// Inspect the explicit rendering matrix without connecting to a database.
    /// PostgreSQL/MySQL rendering does not supply production network adapters.
    ///
    /// | Capability | SQLite | PostgreSQL | MySQL |
    /// | --- | --- | --- | --- |
    /// | DML RETURNING | yes | yes | no |
    /// | Transactional DDL / execute_plan | yes | yes | no |
    /// | Native field/key/FK alteration | no | yes | yes |
    /// | FULL OUTER JOIN | yes | yes | no |
    /// | DROP/TRUNCATE CASCADE | no | yes | no |
    /// | CREATE INDEX IF NOT EXISTS | yes | yes | no |
    /// | Distinct JSON integer/real equality | yes | no | no |
    /// | Text containing NUL | yes | no | yes |
    ///
    /// SQLite RIGHT/FULL joins require SQLite 3.39 or later; RETURNING requires 3.35
    /// or later. These flags describe renderer support; they do not probe server versions.
    /// MySQL idempotent CREATE keeps indexes inline in CREATE TABLE IF NOT EXISTS.
    /// See [`crate::Create::render`] and [`crate::Alter::render`] for additional operation-specific limits.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::DialectId;
    /// assert!(DialectId::Sqlite.capabilities().returning);
    /// assert!(DialectId::Postgres.capabilities().cascade);
    /// assert!(!DialectId::Mysql.capabilities().transactional_ddl);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
