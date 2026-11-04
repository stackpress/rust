//! Detached SELECT assembly; predicates and selector traversal retain their own source rules.
use super::membership::flatten;
use super::{NativeStore, SourceResult as Result, SourceStatement, StoreQuery};
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
use stackpress_sql::{DialectId, Expr, Order, Select, Selector};
impl<S: SourceSchema<Value = Value, Record = Record>> NativeStore<S> {
    /// Compile selected columns, relation joins, source filters and optional pagination.
    pub fn select(
        &self,
        query: &StoreQuery,
        dialect: DialectId,
    ) -> Result<SourceStatement<Select>> {
        let selectors = self.selectors(&query.columns).into_iter().map(|selector| {
            Selector::expression(Expr::column(selector.identifier())).alias(selector.alias)
        });
        let mut select = Select::new(selectors).from(self.table());
        for join in self.joins(query) {
            select = select.join(join);
        }
        if let Some(filter) = self.where_clause(query, dialect)? {
            select = select.and_where(filter);
        }
        if query.skip != 0 {
            select = select.offset(query.skip);
        }
        if query.take != 0 {
            select = select.limit(query.take);
        }
        for (expression, direction) in flatten(&query.sort) {
            let Some(direction) = direction.as_str() else {
                continue;
            };
            let order = match direction.to_ascii_lowercase().as_str() {
                "asc" => Order::Asc,
                "desc" => Order::Desc,
                _ => continue,
            };
            if let Some(selector) = self.selectors(&[expression]).first() {
                // Sort expressions use the SQL crate's typed expression form, including JSON.
                select = select.sort(stackpress_sql::Sort {
                    expression: selector.operand()?,
                    order,
                });
            }
        }
        Ok(SourceStatement(select))
    }
}
