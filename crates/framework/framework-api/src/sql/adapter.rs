//! Adapt source search operators to the selected SQL dialect under owner decision G46.

use stackpress_sql::{DialectId, Query, Render, Sql, TrustedSql};
use std::ops::Deref;

/// A detached source statement. The underlying SQL builder remains inspectable through Deref.
/// Under G46, PostgreSQL uses ILIKE; SQLite and MySQL use their supported LIKE operator.
#[derive(Clone, Debug)]
pub struct SourceStatement<T>(pub(crate) T);
impl<T> Deref for SourceStatement<T> {
    type Target = T;
    /// Borrow the checked builder for build/assignment inspection.
    fn deref(&self) -> &T {
        &self.0
    }
}
impl<T: Render> Render for SourceStatement<T> {
    /// Render checked identifiers and binds, adapting PostgreSQL search spelling under G46.
    fn render(&self, dialect: DialectId) -> stackpress_sql::Result<Query> {
        let query = self.0.render(dialect)?;
        // G46 retains ILIKE only on PostgreSQL. SQLite's ASCII matching and MySQL's default
        // case-insensitive collation use LIKE without rebuilding the rendered query.
        let text = match dialect {
            DialectId::Postgres => query.sql().replace(" LIKE ", " ILIKE "),
            DialectId::Sqlite | DialectId::Mysql => return Ok(query),
        };
        // The owned builders contain only fixed schema identifiers and bound data. No caller
        // value enters SQL text, so this operator substitution cannot alter a literal value.
        let mut sql = Sql::trusted(dialect, TrustedSql::audited(text));
        for value in query.binds() {
            sql = sql.bind(value.clone());
        }
        sql.render(dialect)
    }
}
