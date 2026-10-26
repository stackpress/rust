//! Detached SELECT/INSERT/UPDATE/DELETE data and native dialect rendering.
//!
//! Builders never acquire connections. Render validates the complete statement first;
//! unfiltered mutations require an explicit all-rows choice.

use crate::expr::{RenderContext, join_sql};
use crate::identifier::{invalid, quote};
use crate::{
    DialectId, Expr, Identifier, JsonPath, JsonScalar, Query, Render, Result, Selector, SqlValue,
    TableRef,
};

/// Explicit ordering direction, ascending by default.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Order {
    /// Ascending order.
    #[default]
    Asc,
    /// Descending order.
    Desc,
}

/// A checked join family; CROSS never carries an ON clause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoinKind {
    /// Matching rows only.
    Inner,
    /// All rows from the left input.
    Left,
    /// All rows from the right input (SQLite 3.39+).
    Right,
    /// All rows from both inputs (SQLite 3.39+).
    Full,
    /// Cartesian product, without an ON predicate.
    Cross,
}

/// An ordered relation join with an explicit typed condition.
#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    /// The join's finite syntax choice.
    pub kind: JoinKind,
    /// The joined table and optional alias.
    pub table: TableRef,
    /// Required for ordinary joins and forbidden for CROSS.
    pub on: Option<Expr>,
}

/// One ordered sort expression.
#[derive(Clone, Debug, PartialEq)]
pub struct Sort {
    /// A typed column or expression, with bound data if needed.
    pub expression: Expr,
    /// Explicit direction.
    pub order: Order,
}

/// Immutable inspection snapshot borrowed from a [`Select`].
#[derive(Clone, Debug, PartialEq)]
pub struct SelectData {
    /// Ordered projections; default construction selects `*`.
    pub selectors: Vec<Selector>,
    /// Required source relation.
    pub table: Option<TableRef>,
    /// Ordered joins.
    pub joins: Vec<Join>,
    /// Ordered predicates combined by AND.
    pub filters: Vec<Expr>,
    /// Ordered sorting.
    pub sort: Vec<Sort>,
    /// None leaves SQL unbounded; Some(0) means zero rows.
    pub limit: Option<u64>,
    /// Optional row offset; offset alone uses SQLite LIMIT -1.
    pub offset: Option<u64>,
    /// Whether duplicate projected rows are removed.
    pub distinct: bool,
}

/// Owned reusable SELECT, independent from a database or transaction.
#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    /// All mutable builder state; only consumed fluent calls can change it.
    data: SelectData,
    /// Location of the M0 replacement-style equality predicate, separate from additive filters.
    equality_index: Option<usize>,
}

impl Default for Select {
    /// Select all columns; a table must still be supplied before rendering.
    fn default() -> Self {
        Self::new(["*"])
    }
}

impl Select {
    /// Select an explicit list. Only the exact string `*` is special syntax.
    pub fn new(columns: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        Self {
            data: SelectData {
                selectors: columns.into_iter().map(Into::into).collect(),
                table: None,
                joins: Vec::new(),
                filters: Vec::new(),
                sort: Vec::new(),
                limit: None,
                offset: None,
                distinct: false,
            },
            equality_index: None,
        }
    }

    /// Inspect immutable data; cloning the builder produces independent state.
    pub fn build(&self) -> &SelectData {
        &self.data
    }

    /// Add another typed projection in SQL/bind order.
    pub fn select(mut self, selector: impl Into<Selector>) -> Self {
        self.data.selectors.push(selector.into());
        self
    }

    /// Set a literal or explicitly qualified/aliased source relation.
    pub fn from(mut self, table: impl Into<TableRef>) -> Self {
        self.data.table = Some(table.into());
        self
    }

    /// Request duplicate-row elimination.
    pub fn distinct(mut self) -> Self {
        self.data.distinct = true;
        self
    }

    /// Append a typed join, validated as a whole before rendering returns.
    pub fn join(mut self, join: Join) -> Self {
        self.data.joins.push(join);
        self
    }

    /// Set the M0 equality condition, replacing its previous value if present.
    /// Use [`Self::and_where`] with [`Expr::eq`] to append multiple equalities.
    pub fn where_eq(mut self, column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        let expression = Expr::eq(column, value);
        if let Some(index) = self.equality_index {
            // Preserve replacement without discarding other predicate kinds.
            self.data.filters[index] = expression;
        } else {
            self.equality_index = Some(self.data.filters.len());
            self.data.filters.push(expression);
        }
        self
    }

    /// Append a typed condition while preserving group parentheses.
    pub fn and_where(mut self, expression: Expr) -> Self {
        self.data.filters.push(expression);
        self
    }

    /// Append exact typed JSON scalar equality at a checked path.
    pub fn where_json(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_eq(column, path, value))
    }

    /// Append direct scalar membership in a JSON array.
    pub fn where_json_contains(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_contains(column, path, value))
    }

    /// Append a column sort with an explicit direction.
    pub fn order_by(mut self, column: impl Into<Identifier>, order: Order) -> Self {
        self.data.sort.push(Sort {
            expression: Expr::column(column),
            order,
        });
        self
    }

    /// Append the default ascending column sort.
    pub fn order(self, column: impl Into<Identifier>) -> Self {
        self.order_by(column, Order::Asc)
    }

    /// Append an explicit expression sort.
    pub fn sort(mut self, sort: Sort) -> Self {
        self.data.sort.push(sort);
        self
    }

    /// Set a row limit; zero is retained rather than treated as absent.
    pub fn limit(mut self, limit: u64) -> Self {
        self.data.limit = Some(limit);
        self
    }

    /// Set a row offset, checked against SQLite's signed integer range.
    pub fn offset(mut self, offset: u64) -> Self {
        self.data.offset = Some(offset);
        self
    }
}

impl Render for Select {
    /// Render every clause and reject invalid/oversize input before returning a query.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        let mut context = RenderContext::new(dialect)?;
        let data = &self.data;
        let table = data
            .table
            .as_ref()
            .ok_or_else(|| invalid("SELECT requires a table"))?;
        let mut sql = format!(
            "SELECT {}{} FROM {}",
            if data.distinct { "DISTINCT " } else { "" },
            context.selectors(&data.selectors)?,
            table.quoted(dialect)?
        );
        // Render joins in their original order before filters so parameter order matches SQL.
        if data.joins.len() > 64 {
            return Err(invalid("too many SQL joins"));
        }
        for join in &data.joins {
            dialect.require(
                join.kind != JoinKind::Full || dialect.capabilities().full_join,
                "dialect does not support FULL JOIN",
            )?;
            let kind = match join.kind {
                JoinKind::Inner => "INNER",
                JoinKind::Left => "LEFT",
                JoinKind::Right => "RIGHT",
                JoinKind::Full => "FULL",
                JoinKind::Cross => "CROSS",
            };
            if (join.kind == JoinKind::Cross) == join.on.is_some() {
                return Err(invalid("invalid SQL join condition"));
            }
            sql.push_str(&format!(" {kind} JOIN {}", join.table.quoted(dialect)?));
            if let Some(on) = &join.on {
                sql.push_str(&format!(" ON {}", context.expression(on, 0)?));
            }
        }
        append_filters(&mut sql, &data.filters, &mut context)?;
        if !data.sort.is_empty() {
            let sort = join_sql(
                data.sort.iter().map(|sort| {
                    Ok(format!(
                        "{} {}",
                        context.expression(&sort.expression, 0)?,
                        if sort.order == Order::Asc {
                            "ASC"
                        } else {
                            "DESC"
                        }
                    ))
                }),
                ", ",
            )?;
            sql.push_str(&format!(" ORDER BY {sort}"));
        }
        if let Some(limit) = data.limit {
            sql.push_str(&format!(" LIMIT {}", page_value(limit)?));
        }
        if let Some(offset) = data.offset {
            // SQLite requires LIMIT with OFFSET; -1 means unbounded, not an invented row cap.
            if data.limit.is_none() {
                match dialect {
                    DialectId::Sqlite => sql.push_str(" LIMIT -1"),
                    DialectId::Mysql => sql.push_str(" LIMIT 18446744073709551615"),
                    DialectId::Postgres => {}
                }
            }
            sql.push_str(&format!(" OFFSET {}", page_value(offset)?));
        }
        context.finish(sql)
    }
}

/// Explicit write-cell state: a bound NULL differs from a requested database default.
#[derive(Clone, Debug, PartialEq)]
pub enum WriteValue {
    /// Explicit structured JSON: bound text is cast to the dialect's native JSON storage.
    Json(crate::JsonValue),
    /// Owned native bind, including SQL NULL.
    Value(SqlValue),
    /// Omit/use the column's default; SQLite allows this only for a whole omitted column.
    Default,
    /// A typed expression, useful for updates such as `count = count + ...` via trusted SQL.
    Expression(Expr),
}

impl<T: Into<SqlValue>> From<T> for WriteValue {
    /// Preserve one scalar as a bind rather than SQL text.
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

/// Immutable inspection data for one or many explicit-column insert rows.
#[derive(Clone, Debug, PartialEq)]
pub struct InsertData {
    /// Literal destination table.
    pub table: Identifier,
    /// Stable column order shared by every row.
    pub columns: Vec<String>,
    /// Each row must have exactly the column count.
    pub rows: Vec<Vec<WriteValue>>,
    /// Explicit RETURNING list, empty when not requested.
    pub returning: Vec<Selector>,
    /// Explicit `DEFAULT VALUES`, mutually exclusive with columns/rows.
    pub default_values: bool,
}

/// Owned INSERT builder with stable bulk column and bind order.
#[derive(Clone, Debug, PartialEq)]
pub struct Insert {
    data: InsertData,
}

impl Insert {
    /// Name a destination; columns/values must be supplied or default_values requested.
    pub fn new(table: impl Into<Identifier>) -> Self {
        Self {
            data: InsertData {
                table: table.into(),
                columns: Vec::new(),
                rows: Vec::new(),
                returning: Vec::new(),
                default_values: false,
            },
        }
    }

    /// Inspect the detached insert state.
    pub fn build(&self) -> &InsertData {
        &self.data
    }

    /// Set explicit stable columns; duplicate/empty names are render errors.
    pub fn columns(mut self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data.columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Append one row in column order; bulk inserts call this repeatedly.
    pub fn values(mut self, values: impl IntoIterator<Item = impl Into<WriteValue>>) -> Self {
        self.data
            .rows
            .push(values.into_iter().map(Into::into).collect());
        self
    }

    /// Explicitly insert a row using database defaults for every column.
    pub fn default_values(mut self) -> Self {
        self.data.default_values = true;
        self
    }

    /// Replace RETURNING projections with an explicit ordered list.
    pub fn returning(mut self, selectors: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        self.data.returning = selectors.into_iter().map(Into::into).collect();
        self
    }

    /// Request every returned column.
    pub fn returning_all(self) -> Self {
        self.returning(["*"])
    }
}

impl Render for Insert {
    /// Validate all rows and defaults before constructing one bound statement.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        let mut context = RenderContext::new(dialect)?;
        let data = &self.data;
        let mut sql = format!("INSERT INTO {}", data.table.quoted(dialect)?);
        if data.default_values {
            if !data.columns.is_empty() || !data.rows.is_empty() {
                return Err(invalid("DEFAULT VALUES conflicts with insert data"));
            }
            sql.push_str(if dialect == DialectId::Mysql {
                " () VALUES ()"
            } else {
                " DEFAULT VALUES"
            });
        } else {
            checked_columns(&data.columns)?;
            if data.rows.is_empty() || data.rows.len() > 1000 {
                return Err(invalid("INSERT requires a bounded nonempty row list"));
            }
            for row in &data.rows {
                if row.len() != data.columns.len() {
                    return Err(invalid("INSERT row/column arity mismatch"));
                }
            }
            // SQLite has no per-cell DEFAULT token. Entire defaulted columns can be omitted.
            let included = (0..data.columns.len())
                .filter(|index| {
                    dialect != DialectId::Sqlite
                        || !data
                            .rows
                            .iter()
                            .all(|row| matches!(row[*index], WriteValue::Default))
                })
                .collect::<Vec<_>>();
            if included.is_empty() {
                if data.rows.len() != 1 {
                    return Err(crate::identifier::unsupported(
                        "SQLite cannot bulk insert DEFAULT VALUES",
                    ));
                }
                sql.push_str(" DEFAULT VALUES");
            } else {
                let columns = included
                    .iter()
                    .map(|index| dialect.quote(&data.columns[*index]))
                    .collect::<Result<Vec<_>>>()?;
                let mut rows = Vec::new();
                for row in &data.rows {
                    let values = included
                        .iter()
                        .map(|index| render_write(&row[*index], &mut context))
                        .collect::<Result<Vec<_>>>()?;
                    rows.push(format!("({})", values.join(", ")));
                }
                sql.push_str(&format!(
                    " ({}) VALUES {}",
                    columns.join(", "),
                    rows.join(", ")
                ));
            }
        }
        append_returning(&mut sql, &data.returning, &mut context)?;
        context.finish(sql)
    }
}

/// Shared inspectable mutation scope, including the explicit all-rows authorization.
#[derive(Clone, Debug, PartialEq)]
pub struct MutationData {
    /// Literal or explicitly qualified target.
    pub table: Identifier,
    /// Bound predicates in insertion order.
    pub filters: Vec<Expr>,
    /// Explicit permission for a mutation without predicates.
    pub all_rows: bool,
    /// Explicit RETURNING projections.
    pub returning: Vec<Selector>,
}

impl MutationData {
    /// Initialize a safe mutation requiring a filter by default.
    fn new(table: impl Into<Identifier>) -> Self {
        Self {
            table: table.into(),
            filters: Vec::new(),
            all_rows: false,
            returning: Vec::new(),
        }
    }

    /// Reject accidental whole-table work even if all other syntax is valid.
    fn validate_scope(&self) -> Result<()> {
        if self.filters.is_empty() && !self.all_rows {
            return Err(invalid("unfiltered mutation requires all_rows"));
        }
        Ok(())
    }
}

/// Owned UPDATE; empty assignments and implicit whole-table mutation are errors.
#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    /// Shared target, predicates and returned projections.
    scope: MutationData,
    /// Stable unique assignment names and explicit cell states.
    assignments: Vec<(String, WriteValue)>,
}

impl Update {
    /// Name the target without granting permission to update all rows.
    pub fn new(table: impl Into<Identifier>) -> Self {
        Self {
            scope: MutationData::new(table),
            assignments: Vec::new(),
        }
    }

    /// Inspect target/filter/returning state.
    pub fn build(&self) -> &MutationData {
        &self.scope
    }

    /// Inspect ordered assignments without exposing mutable builder state.
    pub fn assignments(&self) -> &[(String, WriteValue)] {
        &self.assignments
    }

    /// Append an assignment; repeated names are rejected at render time.
    pub fn set(mut self, column: impl Into<String>, value: impl Into<WriteValue>) -> Self {
        self.assignments.push((column.into(), value.into()));
        self
    }

    /// Append a bound equality predicate.
    pub fn where_eq(self, column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        self.and_where(Expr::eq(column, value))
    }

    /// Append a typed predicate.
    pub fn and_where(mut self, expression: Expr) -> Self {
        self.scope.filters.push(expression);
        self
    }

    /// Match an exact JSON scalar at a checked path.
    pub fn where_json(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_eq(column, path, value))
    }

    /// Match direct JSON array membership.
    pub fn where_json_contains(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_contains(column, path, value))
    }

    /// Explicitly authorize an update without predicates.
    pub fn all_rows(mut self) -> Self {
        self.scope.all_rows = true;
        self
    }

    /// Replace the ordered RETURNING list.
    pub fn returning(mut self, selectors: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        self.scope.returning = selectors.into_iter().map(Into::into).collect();
        self
    }

    /// Return every updated column.
    pub fn returning_all(self) -> Self {
        self.returning(["*"])
    }
}

impl Render for Update {
    /// Validate assignment uniqueness and scope before returning SQL.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        let mut context = RenderContext::new(dialect)?;
        self.scope.validate_scope()?;
        let columns = self
            .assignments
            .iter()
            .map(|(column, _)| column.clone())
            .collect::<Vec<_>>();
        checked_columns(&columns)?;
        let assignments = self
            .assignments
            .iter()
            .map(|(column, value)| {
                Ok(format!(
                    "{} = {}",
                    dialect.quote(column)?,
                    render_write(value, &mut context)?
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut sql = format!(
            "UPDATE {} SET {}",
            self.scope.table.quoted(dialect)?,
            assignments.join(", ")
        );
        append_filters(&mut sql, &self.scope.filters, &mut context)?;
        append_returning(&mut sql, &self.scope.returning, &mut context)?;
        context.finish(sql)
    }
}

/// Owned DELETE; an absent/invalid predicate never silently widens mutation scope.
#[derive(Clone, Debug, PartialEq)]
pub struct Delete {
    scope: MutationData,
}

impl Delete {
    /// Name a target, initially requiring a predicate.
    pub fn new(table: impl Into<Identifier>) -> Self {
        Self {
            scope: MutationData::new(table),
        }
    }

    /// Inspect all detached mutation state.
    pub fn build(&self) -> &MutationData {
        &self.scope
    }

    /// Append a bound equality predicate.
    pub fn where_eq(self, column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        self.and_where(Expr::eq(column, value))
    }

    /// Append a typed predicate, preserving its group semantics.
    pub fn and_where(mut self, expression: Expr) -> Self {
        self.scope.filters.push(expression);
        self
    }

    /// Match an exact JSON scalar at a checked path.
    pub fn where_json(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_eq(column, path, value))
    }

    /// Match direct scalar membership in a JSON array.
    pub fn where_json_contains(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_contains(column, path, value))
    }

    /// Explicitly authorize deleting every row.
    pub fn all_rows(mut self) -> Self {
        self.scope.all_rows = true;
        self
    }

    /// Replace the ordered RETURNING list.
    pub fn returning(mut self, selectors: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        self.scope.returning = selectors.into_iter().map(Into::into).collect();
        self
    }

    /// Return every deleted column.
    pub fn returning_all(self) -> Self {
        self.returning(["*"])
    }
}

impl Render for Delete {
    /// Require explicit scope before rendering filters/RETURNING.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        let mut context = RenderContext::new(dialect)?;
        self.scope.validate_scope()?;
        let mut sql = format!("DELETE FROM {}", self.scope.table.quoted(dialect)?);
        append_filters(&mut sql, &self.scope.filters, &mut context)?;
        append_returning(&mut sql, &self.scope.returning, &mut context)?;
        context.finish(sql)
    }
}

/// Reject duplicate/empty column lists before assigning bind positions.
fn checked_columns(columns: &[String]) -> Result<()> {
    if columns.is_empty() || columns.len() > 1000 {
        return Err(invalid("invalid SQL column list"));
    }
    let mut names = std::collections::HashSet::new();
    for column in columns {
        quote(column)?;
        // SQLite identifiers are ASCII case-insensitive even when quoted.
        if !names.insert(column.to_ascii_lowercase()) {
            return Err(invalid("duplicate SQL column"));
        }
    }
    Ok(())
}

/// Render one explicit write state, never substituting NULL for DEFAULT.
fn render_write(value: &WriteValue, context: &mut RenderContext) -> Result<String> {
    match value {
        WriteValue::Json(value) => {
            if context.dialect == DialectId::Postgres {
                crate::json::validate_postgres_json(value)?;
            }
            let marker = context.bind(value.to_json()?.into())?;
            Ok(match context.dialect {
                DialectId::Sqlite => marker,
                DialectId::Postgres => format!("CAST({marker} AS JSONB)"),
                DialectId::Mysql => format!("CAST({marker} AS JSON)"),
            })
        }
        WriteValue::Value(value) => context.bind_ref(value),
        WriteValue::Expression(expression) => context.expression(expression, 0),
        WriteValue::Default if context.dialect != DialectId::Sqlite => Ok("DEFAULT".into()),
        WriteValue::Default => Err(crate::identifier::unsupported(
            "SQLite cannot use per-cell DEFAULT here",
        )),
    }
}

/// Append nonempty predicates without inventing an empty raw condition.
fn append_filters(sql: &mut String, filters: &[Expr], context: &mut RenderContext) -> Result<()> {
    if !filters.is_empty() {
        let parts = join_sql(
            filters.iter().map(|filter| context.expression(filter, 0)),
            " AND ",
        )?;
        sql.push_str(&format!(" WHERE {parts}"));
    }
    Ok(())
}

/// Append an explicitly requested returning clause after filters/values.
fn append_returning(
    sql: &mut String,
    selectors: &[Selector],
    context: &mut RenderContext,
) -> Result<()> {
    if !selectors.is_empty() {
        context.dialect.require(
            context.dialect.capabilities().returning,
            "dialect does not support RETURNING",
        )?;
        sql.push_str(&format!(" RETURNING {}", context.selectors(selectors)?));
    }
    Ok(())
}

/// Reject unsigned pagination outside the actual native signed integer range.
fn page_value(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| invalid("SQL pagination out of range"))
}
