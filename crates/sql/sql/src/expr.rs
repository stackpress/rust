//! Bounded expression trees and selectors shared by every DML builder.

use crate::identifier::{invalid, quote, unsupported};
use crate::{
    DialectId, Identifier, JsonPath, JsonScalar, Query, QueryLimits, Render, Result, Sql, SqlValue,
    TrustedSql,
};

/// Typed comparison operators; caller strings never enter the operator position.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compare {
    /// Equality (NULL uses IS NULL).
    Eq,
    /// Inequality (NULL uses IS NOT NULL).
    Ne,
    /// Strictly less than.
    Lt,
    /// Less than or equal.
    Le,
    /// Strictly greater than.
    Gt,
    /// Greater than or equal.
    Ge,
    /// SQL LIKE, with an explicit escape character through [`Expr::Like`].
    Like,
}

impl Compare {
    /// Map a finite operator to native syntax.
    fn sql(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "<>",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
            Self::Like => "LIKE",
        }
    }
}

/// Detached typed expression. Empty groups and more than 64 nested levels are errors.
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A separately checked identifier.
    Column(Identifier),
    /// A bound scalar; no value becomes executable text.
    Value(SqlValue),
    /// A binary comparison with explicit operands.
    Compare(Box<Expr>, Compare, Box<Expr>),
    /// Parenthesized conjunction in insertion/bind order.
    All(Vec<Expr>),
    /// Parenthesized disjunction in insertion/bind order.
    Any(Vec<Expr>),
    /// Boolean negation.
    Not(Box<Expr>),
    /// A NULL test; the boolean selects IS NOT NULL.
    IsNull(Box<Expr>, bool),
    /// A nonempty IN list; the boolean selects NOT IN.
    In(Box<Expr>, Vec<SqlValue>, bool),
    /// LIKE with a bound pattern and a single explicit escape character.
    Like(Box<Expr>, String, char),
    /// Extract a scalar/document at a typed path; missing becomes SQL NULL.
    JsonExtract(Identifier, JsonPath),
    /// Match JSON type as well as value, distinguishing missing/null/bool/number/text.
    JsonEq(Identifier, JsonPath, JsonScalar),
    /// Direct scalar membership in the array at the selected path, never recursive.
    JsonContains(Identifier, JsonPath, JsonScalar),
    /// Explicit audited native fragment with dialect identity and owned binds.
    Trusted(DialectId, TrustedSql, Vec<SqlValue>),
}

impl Expr {
    /// Refer to one literal column or an explicitly qualified identifier.
    pub fn column(name: impl Into<Identifier>) -> Self {
        Self::Column(name.into())
    }

    /// Keep a scalar value separate from syntax.
    pub fn value(value: impl Into<SqlValue>) -> Self {
        Self::Value(value.into())
    }

    /// Compare this expression to another typed operand.
    pub fn compare(self, operator: Compare, right: Self) -> Self {
        Self::Compare(Box::new(self), operator, Box::new(right))
    }

    /// Compare one column with a bound value; NULL renders as IS NULL.
    pub fn eq(column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        Self::column(column).compare(Compare::Eq, Self::value(value))
    }

    /// Compose a conjunction; an empty collection is rejected at rendering.
    pub fn all(expressions: impl IntoIterator<Item = Self>) -> Self {
        Self::All(expressions.into_iter().collect())
    }

    /// Compose a disjunction without widening an empty group to all rows.
    pub fn any(expressions: impl IntoIterator<Item = Self>) -> Self {
        Self::Any(expressions.into_iter().collect())
    }

    /// Describe a JSON scalar extraction without parsing a SQL-looking selector string.
    pub fn json(column: impl Into<Identifier>, path: JsonPath) -> Self {
        Self::JsonExtract(column.into(), path)
    }

    /// Compare exact JSON scalar types, preserving JSON null versus missing/SQL NULL.
    pub fn json_eq(column: impl Into<Identifier>, path: JsonPath, value: JsonScalar) -> Self {
        Self::JsonEq(column.into(), path, value)
    }

    /// Test direct scalar membership at an array path.
    pub fn json_contains(column: impl Into<Identifier>, path: JsonPath, value: JsonScalar) -> Self {
        Self::JsonContains(column.into(), path, value)
    }
}

/// A projection with explicit wildcard/alias/expression semantics.
#[derive(Clone, Debug, PartialEq)]
pub struct Selector {
    /// None selects `*` (optionally qualified); Some renders the expression.
    expression: Option<Expr>,
    /// Qualification for a wildcard only.
    wildcard: Option<Identifier>,
    /// Optional one-segment output name.
    alias: Option<String>,
}

impl Selector {
    /// Project an expression; its values remain bound data.
    pub fn expression(expression: Expr) -> Self {
        Self {
            expression: Some(expression),
            wildcard: None,
            alias: None,
        }
    }

    /// Select all columns, optionally from an explicitly qualified table/alias.
    pub fn wildcard(table: Option<Identifier>) -> Self {
        Self {
            expression: None,
            wildcard: table,
            alias: None,
        }
    }

    /// Name the result column; aliases are validated and quoted separately.
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Render projection syntax while preserving bind order.
    pub(crate) fn render(&self, context: &mut RenderContext) -> Result<String> {
        let mut sql = match (&self.expression, &self.wildcard) {
            (Some(expression), _) => context.expression(expression, 0)?,
            (None, Some(table)) => format!("{}.*", table.quoted()?),
            (None, None) => "*".into(),
        };
        if let Some(alias) = &self.alias {
            if self.expression.is_none() {
                return Err(invalid("wildcards cannot have aliases"));
            }
            sql.push_str(&format!(" AS {}", quote(alias)?));
        }
        Ok(sql)
    }
}

impl From<&str> for Selector {
    /// Recognize only the exact wildcard; every other string is a single literal name.
    fn from(value: &str) -> Self {
        if value == "*" {
            Self::wildcard(None)
        } else {
            Self::expression(Expr::column(value))
        }
    }
}

impl From<String> for Selector {
    /// Treat owned text as a name, except the exact wildcard.
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<Identifier> for Selector {
    /// Project an explicitly qualified column.
    fn from(value: Identifier) -> Self {
        Self::expression(Expr::Column(value))
    }
}

/// Per-render state; never global and never retained by a connection or builder.
pub(crate) struct RenderContext {
    /// Bind order follows the final SQL traversal exactly.
    binds: Vec<SqlValue>,
    /// Total expression nodes visited across every clause.
    nodes: usize,
    /// Cumulative retained bind bytes, checked before retaining each next argument.
    bytes: usize,
}

impl RenderContext {
    /// Reject other dialects until M2 provides their real renderers.
    pub(crate) fn new(dialect: DialectId) -> Result<Self> {
        if dialect != DialectId::Sqlite {
            return Err(unsupported("SQL dialect renderer is not implemented"));
        }
        Ok(Self {
            binds: Vec::new(),
            nodes: 0,
            bytes: 0,
        })
    }

    /// Validate a scalar before retaining another parameter.
    pub(crate) fn bind(&mut self, value: SqlValue) -> Result<String> {
        self.reserve_bind(&value)?;
        self.binds.push(value);
        Ok("?".into())
    }

    /// Validate borrowed input before cloning it into the retained argument list.
    pub(crate) fn bind_ref(&mut self, value: &SqlValue) -> Result<String> {
        self.reserve_bind(value)?;
        self.binds.push(value.clone());
        Ok("?".into())
    }

    /// Reserve count/bytes atomically so failed binds leave accounting unchanged.
    fn reserve_bind(&mut self, value: &SqlValue) -> Result<()> {
        value.validate()?;
        let limits = QueryLimits::default();
        let bytes = self
            .bytes
            .checked_add(value.retained_bytes())
            .ok_or_else(render_limit)?;
        if self.binds.len() >= limits.binds || bytes > limits.bytes {
            return Err(render_limit());
        }
        self.bytes = bytes;
        Ok(())
    }

    /// Freeze the SQL/parameter pair after enforcing the default input budgets.
    pub(crate) fn finish(self, sql: String) -> Result<Query> {
        Query::bound(DialectId::Sqlite, sql, self.binds)
    }

    /// Render all selectors with a finite list bound and deterministic bind order.
    pub(crate) fn selectors(&mut self, selectors: &[Selector]) -> Result<String> {
        if selectors.is_empty() {
            return Err(invalid("empty SQL selector list"));
        }
        if selectors.len() > 1000 {
            return Err(crate::SqlError::new(
                crate::ErrorKind::Limit,
                "SQL selector limit exceeded",
            ));
        }
        join_sql(selectors.iter().map(|selector| selector.render(self)), ", ")
    }

    /// Render a bounded expression recursively, carrying one shared node/bind budget.
    pub(crate) fn expression(&mut self, expression: &Expr, depth: usize) -> Result<String> {
        self.nodes += 1;
        if depth > 64 || self.nodes > 10_000 {
            return Err(crate::SqlError::new(
                crate::ErrorKind::Limit,
                "SQL expression limit exceeded",
            ));
        }
        match expression {
            Expr::Column(name) => name.quoted(),
            Expr::Value(value) => self.bind_ref(value),
            Expr::Compare(left, operator, right) => {
                // SQL equality to NULL is a null test, not an always-unknown comparison.
                if matches!(right.as_ref(), Expr::Value(SqlValue::Null))
                    && matches!(operator, Compare::Eq | Compare::Ne)
                {
                    return Ok(format!(
                        "{} IS {}NULL",
                        self.operand(left, depth + 1)?,
                        if *operator == Compare::Ne { "NOT " } else { "" }
                    ));
                }
                Ok(format!(
                    "{} {} {}",
                    self.operand(left, depth + 1)?,
                    operator.sql(),
                    self.operand(right, depth + 1)?
                ))
            }
            Expr::All(values) => self.group(values, " AND ", depth),
            Expr::Any(values) => self.group(values, " OR ", depth),
            Expr::Not(value) => Ok(format!("NOT ({})", self.expression(value, depth + 1)?)),
            Expr::IsNull(value, negate) => Ok(format!(
                "{} IS {}NULL",
                self.operand(value, depth + 1)?,
                if *negate { "NOT " } else { "" }
            )),
            Expr::In(value, values, negate) => {
                if values.is_empty() {
                    return Err(invalid("empty SQL IN list"));
                }
                let operand = self.operand(value, depth + 1)?;
                let arguments = values
                    .iter()
                    .map(|value| self.bind_ref(value))
                    .collect::<Result<Vec<_>>>()?;
                Ok(format!(
                    "{operand} {}IN ({})",
                    if *negate { "NOT " } else { "" },
                    arguments.join(", ")
                ))
            }
            Expr::Like(value, pattern, escape) => {
                if escape.is_control() {
                    return Err(invalid("invalid SQL LIKE escape"));
                }
                let operand = self.operand(value, depth + 1)?;
                let pattern = self.bind(pattern.as_str().into())?;
                let escape = self.bind(escape.to_string().into())?;
                Ok(format!("{operand} LIKE {pattern} ESCAPE {escape}"))
            }
            Expr::JsonExtract(column, path) => self.json_call("json_extract", column, path),
            Expr::JsonEq(column, path, value) => {
                let kind = value.kind()?;
                let native_type = self.json_call("json_type", column, path)?;
                let kind_bind = self.bind(kind.into())?;
                // JSON null has a type, whereas an absent path returns SQL NULL.
                if matches!(value, JsonScalar::Null) {
                    return Ok(format!("({native_type} = {kind_bind})"));
                }
                let extracted = self.json_call("json_extract", column, path)?;
                let value_bind = self.bind(value.bind())?;
                Ok(format!(
                    "({native_type} = {kind_bind} AND {extracted} = {value_bind})"
                ))
            }
            Expr::JsonContains(column, path, value) => self.json_contains(column, path, value),
            Expr::Trusted(dialect, text, values) => {
                if text.text().trim().is_empty() {
                    return Err(invalid("empty trusted SQL expression"));
                }
                // Validate a complete wrapper to prevent fragment batches or quote/comment escapes.
                let query = Sql::trusted(
                    *dialect,
                    TrustedSql::audited(format!("SELECT ({})", text.text())),
                );
                query.render(DialectId::Sqlite)?;
                for value in values {
                    self.bind_ref(value)?;
                }
                Ok(format!("({})", text.text()))
            }
        }
    }

    /// Preserve nested predicate operands instead of relying on SQL associativity.
    fn operand(&mut self, expression: &Expr, depth: usize) -> Result<String> {
        let sql = self.expression(expression, depth)?;
        if matches!(
            expression,
            Expr::Compare(..) | Expr::IsNull(..) | Expr::In(..) | Expr::Like(..) | Expr::Not(..)
        ) {
            return Ok(format!("({sql})"));
        }
        Ok(sql)
    }

    /// Retain group parentheses so mixed AND/OR conditions cannot change scope.
    fn group(&mut self, values: &[Expr], separator: &str, depth: usize) -> Result<String> {
        if values.is_empty() {
            return Err(invalid("empty SQL predicate group"));
        }
        let parts = join_sql(
            values.iter().map(|value| self.expression(value, depth + 1)),
            separator,
        )?;
        Ok(format!("({parts})"))
    }

    /// Bind paths as data for both extraction and JSON type checks.
    fn json_call(
        &mut self,
        function: &str,
        column: &Identifier,
        path: &JsonPath,
    ) -> Result<String> {
        let column = column.quoted()?;
        let path = self.bind(path.sqlite_path()?.into())?;
        Ok(format!("{function}({column}, {path})"))
    }

    /// Restrict membership to direct array elements with exact scalar types.
    fn json_contains(
        &mut self,
        column: &Identifier,
        path: &JsonPath,
        value: &JsonScalar,
    ) -> Result<String> {
        let kind = value.kind()?;
        let array_type = self.json_call("json_type", column, path)?;
        let column = column.quoted()?;
        let path = self.bind(path.sqlite_path()?.into())?;
        let kind = self.bind(kind.into())?;
        // The table-valued function contributes only immediate children, never descendants.
        let mut predicate = format!("\"__sp_json\".\"type\" = {kind}");
        if !matches!(value, JsonScalar::Null) {
            predicate.push_str(&format!(
                " AND \"__sp_json\".\"value\" = {}",
                self.bind(value.bind())?
            ));
        }
        Ok(format!(
            "({array_type} = 'array' AND EXISTS (SELECT 1 FROM json_each({column}, {path}) AS \"__sp_json\" WHERE {predicate}))"
        ))
    }
}

/// Join rendered fragments incrementally so wide qualified selectors/groups stay bounded.
pub(crate) fn join_sql(
    parts: impl IntoIterator<Item = Result<String>>,
    separator: &str,
) -> Result<String> {
    let mut joined = String::new();
    for (index, part) in parts.into_iter().enumerate() {
        let part = part?;
        let separator = if index == 0 { "" } else { separator };
        let length = joined
            .len()
            .checked_add(separator.len())
            .and_then(|length| length.checked_add(part.len()))
            .ok_or_else(render_limit)?;
        if length > QueryLimits::default().sql_bytes {
            return Err(render_limit());
        }
        joined.push_str(separator);
        joined.push_str(&part);
    }
    Ok(joined)
}

/// Use one typed, input-free rendering resource failure.
fn render_limit() -> crate::SqlError {
    crate::SqlError::new(
        crate::ErrorKind::Limit,
        "SQL rendering input limit exceeded",
    )
}
