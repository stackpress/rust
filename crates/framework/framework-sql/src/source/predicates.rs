//! Source filter order, bound values, implicit active predicate and positive hasnt behavior.
use super::membership::{flatten, flatten_has, membership};
use super::text::source_text;
use super::{NativeStore, SourceResult as Result, StoreQuery};
use stackpress_framework_schema::source::{Record, SourceSchema, Value};
use stackpress_sql::{Compare, DialectId, Expr, Identifier};
impl<S: SourceSchema<Value = Value, Record = Record>> NativeStore<S> {
    /// Source conjunctive filter set; invalid/empty scalar conditions are silently skipped.
    pub fn where_clause(&self, query: &StoreQuery, dialect: DialectId) -> Result<Option<Expr>> {
        let mut filters = Vec::new();
        for field in self.model.fields.iter().filter(|field| field.active) {
            if !query.eq.contains_key(&field.name) {
                filters.push(Expr::eq(
                    Identifier::qualified([self.table(), &field.column]),
                    true,
                ));
            }
        }
        if let Some(q) = query
            .q
            .as_ref()
            .filter(|q| stackpress_framework_schema::source::truthy(q))
        {
            let q = source_text(q, self.schema.offset_minutes());
            let filters_q = self
                .model
                .fields
                .iter()
                .filter(|f| f.searchable)
                .map(|field| {
                    Expr::column(Identifier::qualified([self.table(), &field.column]))
                        .compare(Compare::Like, Expr::value(format!("%{q}%")))
                })
                .collect::<Vec<_>>();
            if !filters_q.is_empty() {
                filters.push(Expr::any(filters_q));
            }
        }
        for (map, compare, like) in [
            (&query.eq, Compare::Eq, false),
            (&query.like, Compare::Like, true),
            (&query.ne, Compare::Ne, false),
            (&query.ge, Compare::Ge, false),
            (&query.le, Compare::Le, false),
        ] {
            for (expression, value) in flatten(map) {
                if compare == Compare::Eq
                    && self
                        .model
                        .fields
                        .iter()
                        .any(|f| f.active && f.name == expression)
                    && value.as_f64() == Some(-1.0)
                {
                    continue;
                }
                if let Some(filter) = self.where_values(&expression, &value, compare, like)? {
                    filters.push(filter);
                }
            }
        }
        // These source helpers bypass the ordinary scalar selector lookup. An unknown
        // column therefore reaches the database instead of silently widening the query.
        for (map, require_all) in [(&query.has, true), (&query.hasnt, false)] {
            // G53 fixes only has arrays. Positive hasnt retains source index flattening.
            let values = if require_all {
                flatten_has(map)
            } else {
                flatten(map)
            };
            for (expression, value) in values {
                let (column, json) = expression.split_once(':').unwrap_or((&expression, ""));
                let identifier = Identifier::qualified(column.split('.'));
                let json = json
                    .split('.')
                    .filter(|part| !part.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let values = value.as_array().cloned().unwrap_or_else(|| vec![value]);
                let members = values
                    .iter()
                    .map(|value| membership(identifier.clone(), &json, value, dialect))
                    .collect::<Result<Vec<_>>>()?;
                if !members.is_empty() {
                    // Keep the exact scalar AST; several has values must all be present.
                    filters.push(if require_all && members.len() > 1 {
                        Expr::all(members)
                    } else {
                        Expr::any(members)
                    });
                }
            }
        }
        Ok((!filters.is_empty()).then(|| Expr::all(filters)))
    }

    /// Source OR grouping for scalar/array filter values, including per-field storage codecs.
    fn where_values(
        &self,
        expression: &str,
        value: &Value,
        compare: Compare,
        like: bool,
    ) -> Result<Option<Expr>> {
        let Some(selector) = self.selectors(&[expression.into()]).into_iter().next() else {
            return Ok(None);
        };
        let schema = self.sibling(&selector.model);
        let column = schema
            .as_ref()
            .and_then(|schema| schema.columns().iter().find(|c| c.name() == selector.field));
        let values = value
            .as_array()
            .cloned()
            .unwrap_or_else(|| vec![value.clone()]);
        let mut filters = Vec::new();
        for value in values {
            let value = if like && value.is_string() {
                Value::String(format!(
                    "%{}%",
                    value.as_str().unwrap_or_default().to_owned()
                ))
            } else {
                value
            };
            let serialized = if selector.json.is_empty()
                && let Some(column) = &column
            {
                column.serialize(
                    Some(&value),
                    self.schema.seed(),
                    true,
                    self.schema.offset_minutes(),
                )?
            } else {
                Some(value)
            };
            let Some(value) =
                serialized.filter(|value| !value.is_null() && value.as_str() != Some(""))
            else {
                continue;
            };
            // Source uses the base store's toSqlValue even for related/extracted values;
            // switching to the joined model codec here would change coercion semantics.
            let scalar = self.to_sql_value(&selector.field, &value)?;
            filters.push(selector.operand()?.compare(compare, Expr::value(scalar)));
        }
        Ok((!filters.is_empty()).then(|| Expr::any(filters)))
    }
}
