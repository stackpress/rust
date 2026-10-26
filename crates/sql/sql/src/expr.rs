//! Bounded expression trees and selectors shared by every DML builder.

use crate::identifier::{invalid, unsupported};
use crate::{
    DialectId, Identifier, JsonPath, JsonScalar, Query, QueryLimits, Render, Result, Sql, SqlValue,
    TrustedSql,
};

/// Typed comparison operators; caller strings never enter the operator position.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
/// let filter = Expr::all([
///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
/// ]);
/// let query = Select::default()
///     .from("profile")
///     .and_where(filter)
///     .render(DialectId::Sqlite)?;
/// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Compare {
    /// Equality (NULL uses IS NULL).
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Eq, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Eq,
    /// Inequality (NULL uses IS NOT NULL).
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Ne, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Ne,
    /// Strictly less than.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Lt, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Lt,
    /// Less than or equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Le, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Le,
    /// Strictly greater than.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Gt, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Gt,
    /// Greater than or equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Ge, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Ge,
    /// SQL LIKE, with an explicit escape character through [`crate::Expr::Like`].
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::column("id").compare(Compare::Like, Expr::value(7));
    /// Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
///
/// Rendering permits at most 64 nested levels and 10,000 expression nodes.
/// Those checks apply to rendering; constructing a tree and derived Clone/Debug/PartialEq
/// or ordinary recursive destruction do not run the renderer's validation.
/// Empty groups/IN lists are invalid. JSON equality distinguishes missing, JSON null,
/// boolean, number and text; PostgreSQL/MySQL numeric equality treats 1 and 1.0
/// as equal (see [`crate::DialectCapabilities::distinct_json_number_kinds`]).
/// Trusted PostgreSQL fragments with binds must precede other bound expressions
/// because caller-authored `$n` placeholders are never silently renumbered.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
/// let filter = Expr::all([
///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
/// ]);
/// let query = Select::default()
///     .from("profile")
///     .and_where(filter)
///     .render(DialectId::Sqlite)?;
/// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A separately checked identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::Column("id".into());
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Column(Identifier),
    /// A bound scalar; no value becomes executable text.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::Value(7.into());
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Value(SqlValue),
    /// A binary comparison with explicit operands.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::Compare(
    ///     Box::new(Expr::column("id")),
    ///     Compare::Eq,
    ///     Box::new(Expr::value(7)),
    /// );
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Compare(Box<Expr>, Compare, Box<Expr>),
    /// Parenthesized conjunction in insertion/bind order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::All(vec![Expr::eq("id", 7)]);
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    All(Vec<Expr>),
    /// Parenthesized disjunction in insertion/bind order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::Any(vec![Expr::eq("id", 7), Expr::eq("id", 8)]);
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Any(Vec<Expr>),
    /// Boolean negation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::Not(Box::new(Expr::eq("active", true)));
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Not(Box<Expr>),
    /// A NULL test; the boolean selects IS NOT NULL.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::IsNull(Box::new(Expr::column("name")), false);
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    IsNull(Box<Expr>, bool),
    /// A nonempty IN list; the boolean selects NOT IN.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::In(
    ///     Box::new(Expr::column("id")),
    ///     vec![7.into(), 8.into()],
    ///     false,
    /// );
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    In(Box<Expr>, Vec<SqlValue>, bool),
    /// LIKE with a bound pattern and a single explicit escape character.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector};
    /// let expression = Expr::Like(Box::new(Expr::column("name")), "Ada%".into(), '\\');
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Like(Box<Expr>, String, char),
    /// Extract a scalar/document at a typed path; missing becomes SQL NULL.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, JsonPath, Render, Select, Selector};
    /// let expression = Expr::JsonExtract("data".into(), JsonPath::default());
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    JsonExtract(Identifier, JsonPath),
    /// Match JSON type as well as value, distinguishing missing/null/bool/number/text.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, JsonPath, JsonScalar, Render, Select, Selector};
    /// let expression = Expr::JsonEq("data".into(), JsonPath::default(), JsonScalar::Null);
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    JsonEq(Identifier, JsonPath, JsonScalar),
    /// Direct scalar membership in the array at the selected path, never recursive.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, JsonPath, JsonScalar, Render, Select, Selector};
    /// let expression = Expr::JsonContains("data".into(), JsonPath::default(), JsonScalar::Integer(7));
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    JsonContains(Identifier, JsonPath, JsonScalar),
    /// Explicit audited native fragment with dialect identity and owned binds.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Render, Select, Selector, TrustedSql};
    /// let expression = Expr::Trusted(
    ///     DialectId::Sqlite,
    ///     TrustedSql::audited("? = ?"),
    ///     vec![7.into(), 7.into()],
    /// );
    /// Select::new([Selector::expression(expression)])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Trusted(DialectId, TrustedSql, Vec<SqlValue>),
}

impl Expr {
    /// Refer to one literal column or an explicitly qualified identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::all([
    ///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
    ///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
    /// ]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn column(name: impl Into<Identifier>) -> Self {
        Self::Column(name.into())
    }

    /// Keep a scalar value separate from syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::all([
    ///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
    ///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
    /// ]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn value(value: impl Into<SqlValue>) -> Self {
        Self::Value(value.into())
    }

    /// Compare this expression to another typed operand.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::all([
    ///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
    ///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
    /// ]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn compare(self, operator: Compare, right: Self) -> Self {
        Self::Compare(Box::new(self), operator, Box::new(right))
    }

    /// Compare one column with a bound value; NULL renders as IS NULL.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::all([
    ///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
    ///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
    /// ]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn eq(column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        Self::column(column).compare(Compare::Eq, Self::value(value))
    }

    /// Compose a conjunction; an empty collection is rejected at rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::all([
    ///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
    ///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
    /// ]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn all(expressions: impl IntoIterator<Item = Self>) -> Self {
        Self::All(expressions.into_iter().collect())
    }

    /// Compose a disjunction without widening an empty group to all rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Compare, DialectId, Expr, Render, Select};
    /// let filter = Expr::all([
    ///     Expr::column("age").compare(Compare::Ge, Expr::value(18)),
    ///     Expr::any([Expr::eq("active", true), Expr::eq("role", "admin")]),
    /// ]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(filter)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[18.into(), true.into(), "admin".into()]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn any(expressions: impl IntoIterator<Item = Self>) -> Self {
        Self::Any(expressions.into_iter().collect())
    }

    /// Describe a JSON scalar extraction without parsing a SQL-looking selector string.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, JsonPath, JsonSegment, Render, Select, Selector};
    /// let path = JsonPath::new([JsonSegment::Key("name".into())]);
    /// let selector = Selector::expression(Expr::json("data", path)).alias("name");
    /// let query = Select::new([selector])
    ///     .from("profile")
    ///     .render(DialectId::Sqlite)?;
    /// assert!(query.sql().contains("json_extract"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn json(column: impl Into<Identifier>, path: JsonPath) -> Self {
        Self::JsonExtract(column.into(), path)
    }

    /// Compare exact JSON scalar types, preserving JSON null versus missing/SQL NULL.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, JsonPath, JsonScalar, JsonSegment, Render, Select};
    /// let path = JsonPath::new([JsonSegment::Key("tags".into())]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(Expr::json_eq("data", path, JsonScalar::Null))
    ///     .render(DialectId::Sqlite)?;
    /// assert!(query.sql().contains("json_"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn json_eq(column: impl Into<Identifier>, path: JsonPath, value: JsonScalar) -> Self {
        Self::JsonEq(column.into(), path, value)
    }

    /// Test direct scalar membership at an array path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, JsonPath, JsonScalar, JsonSegment, Render, Select};
    /// let path = JsonPath::new([JsonSegment::Key("tags".into())]);
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(Expr::json_contains("data", path, JsonScalar::Integer(7)))
    ///     .render(DialectId::Sqlite)?;
    /// assert!(query.sql().contains("json_"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn json_contains(column: impl Into<Identifier>, path: JsonPath, value: JsonScalar) -> Self {
        Self::JsonContains(column.into(), path, value)
    }
}

/// A projection with explicit wildcard/alias/expression semantics.
///
/// Only the exact string `"*"` becomes a wildcard. Other strings, including dots,
/// commas and `count(*)`, remain literal names. Use typed qualification or an
/// explicit trusted expression. Wildcards cannot carry aliases.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Identifier, Render, Select, Selector, TableRef};
/// let selector =
///     Selector::expression(Expr::column(Identifier::qualified(["p", "id"]))).alias("profile_id");
/// let query = Select::new([selector]).from(TableRef::new("profile").alias("p"));
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains(" AS \"profile_id\"")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Identifier, Render, Select, Selector, TableRef};
    /// let selector =
    ///     Selector::expression(Expr::column(Identifier::qualified(["p", "id"]))).alias("profile_id");
    /// let query = Select::new([selector]).from(TableRef::new("profile").alias("p"));
    /// assert!(
    ///     query
    ///         .render(DialectId::Sqlite)?
    ///         .sql()
    ///         .contains(" AS \"profile_id\"")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn expression(expression: Expr) -> Self {
        Self {
            expression: Some(expression),
            wildcard: None,
            alias: None,
        }
    }

    /// Select all columns, optionally from an explicitly qualified table/alias.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Select, Selector};
    /// let query = Select::new([Selector::wildcard(Some("p".into()))]).from("p");
    /// assert_eq!(
    ///     query.render(DialectId::Sqlite)?.sql(),
    ///     "SELECT \"p\".* FROM \"p\""
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn wildcard(table: Option<Identifier>) -> Self {
        Self {
            expression: None,
            wildcard: table,
            alias: None,
        }
    }

    /// Name the result column; aliases are validated and quoted separately.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Identifier, Render, Select, Selector, TableRef};
    /// let selector =
    ///     Selector::expression(Expr::column(Identifier::qualified(["p", "id"]))).alias("profile_id");
    /// let query = Select::new([selector]).from(TableRef::new("profile").alias("p"));
    /// assert!(
    ///     query
    ///         .render(DialectId::Sqlite)?
    ///         .sql()
    ///         .contains(" AS \"profile_id\"")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    /// Render projection syntax while preserving bind order.
    pub(crate) fn render(&self, context: &mut RenderContext) -> Result<String> {
        let mut sql = match (&self.expression, &self.wildcard) {
            (Some(expression), _) => context.expression(expression, 0)?,
            (None, Some(table)) => format!("{}.*", table.quoted(context.dialect)?),
            (None, None) => "*".into(),
        };
        if let Some(alias) = &self.alias {
            if self.expression.is_none() {
                return Err(invalid("wildcards cannot have aliases"));
            }
            sql.push_str(&format!(" AS {}", context.dialect.quote(alias)?));
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
    /// Renderer identity for quoting and bind numbering.
    pub(crate) dialect: DialectId,
    /// Total expression nodes visited across every clause.
    nodes: usize,
    /// Cumulative retained bind bytes, checked before retaining each next argument.
    bytes: usize,
}

impl RenderContext {
    /// Start isolated quoting, bind numbering and budget state for the chosen renderer.
    pub(crate) fn new(dialect: DialectId) -> Result<Self> {
        Ok(Self {
            binds: Vec::new(),
            dialect,
            nodes: 0,
            bytes: 0,
        })
    }

    /// Validate a scalar before retaining another parameter.
    pub(crate) fn bind(&mut self, value: SqlValue) -> Result<String> {
        self.reserve_bind(&value)?;
        self.binds.push(value);
        Ok(self.marker())
    }

    /// Validate borrowed input before cloning it into the retained argument list.
    pub(crate) fn bind_ref(&mut self, value: &SqlValue) -> Result<String> {
        self.reserve_bind(value)?;
        self.binds.push(value.clone());
        Ok(self.marker())
    }

    /// Number PostgreSQL placeholders from AST bind traversal, never by replacing SQL text.
    fn marker(&self) -> String {
        if self.dialect == DialectId::Postgres {
            format!("${}", self.binds.len())
        } else {
            "?".into()
        }
    }

    /// Reserve count/bytes atomically so failed binds leave accounting unchanged.
    fn reserve_bind(&mut self, value: &SqlValue) -> Result<()> {
        value.validate_bind(self.dialect)?;
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
        Query::bound(self.dialect, sql, self.binds)
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
            Expr::Column(name) => name.quoted(self.dialect),
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
            Expr::JsonExtract(column, path) if self.dialect != DialectId::Sqlite => {
                crate::json_render::extract(self, column, path)
            }
            Expr::JsonExtract(column, path) => self.json_call("json_extract", column, path),
            Expr::JsonEq(column, path, value) if self.dialect != DialectId::Sqlite => {
                crate::json_render::equal(self, column, path, value)
            }
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
            Expr::JsonContains(column, path, value) if self.dialect != DialectId::Sqlite => {
                crate::json_render::contains(self, column, path, value)
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
                if self.dialect == DialectId::Postgres
                    && !values.is_empty()
                    && !self.binds.is_empty()
                {
                    return Err(unsupported(
                        "numbered trusted fragments must precede other binds",
                    ));
                }
                query.render(self.dialect)?;
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
        let column = column.quoted(self.dialect)?;
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
        let column = column.quoted(self.dialect)?;
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
