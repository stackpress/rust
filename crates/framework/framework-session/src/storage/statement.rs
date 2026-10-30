//! Preserve source SQL operators while delegating identifier/bind rendering to stackpress-sql.

use stackpress_sql::{DialectId, Query, Render, Sql, TrustedSql};
use std::ops::Deref;

/// A detached source statement. The underlying SQL builder remains inspectable through Deref.
/// Source ILIKE is deliberately preserved even on a native dialect that rejects it.
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
    /// Render checked identifiers and binds first, then retain the source operator spelling.
    fn render(&self, dialect: DialectId) -> stackpress_sql::Result<Query> {
        let query = self.0.render(dialect)?;
        // The owned builders contain only fixed schema identifiers and bound data. No caller
        // value enters SQL text, so this operator substitution cannot alter a literal value.
        let mut sql = Sql::trusted(
            dialect,
            TrustedSql::audited(query.sql().replace(" LIKE ", " ILIKE ")),
        );
        for value in query.binds() {
            sql = sql.bind(value.clone());
        }
        sql.render(dialect)
    }
}
