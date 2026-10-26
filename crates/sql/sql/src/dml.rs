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
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
/// let query = Select::new(["id"])
///     .select("name")
///     .from(TableRef::new("profile").alias("p"))
///     .join(Join {
///         kind: JoinKind::Cross,
///         table: "category".into(),
///         on: None,
///     })
///     .sort(Sort {
///         expression: Expr::column("id"),
///         order: Order::Desc,
///     })
///     .limit(10)
///     .offset(2)
///     .distinct();
/// let data = query.build();
/// assert_eq!(data.selectors.len(), 2);
/// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
/// assert_eq!(data.joins.len(), 1);
/// assert!(data.filters.is_empty());
/// assert_eq!(data.sort[0].order, Order::Desc);
/// assert_eq!(
///     (data.limit, data.offset, data.distinct),
///     (Some(10), Some(2), true)
/// );
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains("CROSS JOIN")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Order {
    /// Ascending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Order, Select};
    /// let query = Select::default().order_by("id", Order::Asc);
    /// assert_eq!(query.build().sort.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    #[default]
    Asc,
    /// Descending order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Order, Select};
    /// let query = Select::default().order_by("id", Order::Desc);
    /// assert_eq!(query.build().sort.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Desc,
}

/// A checked join family; CROSS never carries an ON clause.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
/// let query = Select::new(["id"])
///     .select("name")
///     .from(TableRef::new("profile").alias("p"))
///     .join(Join {
///         kind: JoinKind::Cross,
///         table: "category".into(),
///         on: None,
///     })
///     .sort(Sort {
///         expression: Expr::column("id"),
///         order: Order::Desc,
///     })
///     .limit(10)
///     .offset(2)
///     .distinct();
/// let data = query.build();
/// assert_eq!(data.selectors.len(), 2);
/// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
/// assert_eq!(data.joins.len(), 1);
/// assert!(data.filters.is_empty());
/// assert_eq!(data.sort[0].order, Order::Desc);
/// assert_eq!(
///     (data.limit, data.offset, data.distinct),
///     (Some(10), Some(2), true)
/// );
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains("CROSS JOIN")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JoinKind {
    /// Matching rows only.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Render, Select};
    /// let join = Join {
    ///     kind: JoinKind::Inner,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// Select::default()
    ///     .from("profile")
    ///     .join(join)
    ///     .render(DialectId::Postgres)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Inner,
    /// All rows from the left input.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Render, Select};
    /// let join = Join {
    ///     kind: JoinKind::Left,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// Select::default()
    ///     .from("profile")
    ///     .join(join)
    ///     .render(DialectId::Postgres)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Left,
    /// All rows from the right input (SQLite 3.39+).
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Render, Select};
    /// let join = Join {
    ///     kind: JoinKind::Right,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// Select::default()
    ///     .from("profile")
    ///     .join(join)
    ///     .render(DialectId::Postgres)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Right,
    /// All rows from both inputs (SQLite 3.39+).
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Render, Select};
    /// let join = Join {
    ///     kind: JoinKind::Full,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// Select::default()
    ///     .from("profile")
    ///     .join(join)
    ///     .render(DialectId::Postgres)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Full,
    /// Cartesian product, without an ON predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Join, JoinKind, Render, Select};
    /// let join = Join {
    ///     kind: JoinKind::Cross,
    ///     table: "parent".into(),
    ///     on: None,
    /// };
    /// Select::default()
    ///     .from("profile")
    ///     .join(join)
    ///     .render(DialectId::Postgres)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Cross,
}

/// An ordered relation join with an explicit typed condition.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
/// let query = Select::new(["id"])
///     .select("name")
///     .from(TableRef::new("profile").alias("p"))
///     .join(Join {
///         kind: JoinKind::Cross,
///         table: "category".into(),
///         on: None,
///     })
///     .sort(Sort {
///         expression: Expr::column("id"),
///         order: Order::Desc,
///     })
///     .limit(10)
///     .offset(2)
///     .distinct();
/// let data = query.build();
/// assert_eq!(data.selectors.len(), 2);
/// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
/// assert_eq!(data.joins.len(), 1);
/// assert!(data.filters.is_empty());
/// assert_eq!(data.sort[0].order, Order::Desc);
/// assert_eq!(
///     (data.limit, data.offset, data.distinct),
///     (Some(10), Some(2), true)
/// );
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains("CROSS JOIN")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Join {
    /// The join's finite syntax choice.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Join, JoinKind};
    /// let value = Join {
    ///     kind: JoinKind::Inner,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// assert_eq!(value.kind, JoinKind::Inner);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub kind: JoinKind,
    /// The joined table and optional alias.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Join, JoinKind};
    /// let value = Join {
    ///     kind: JoinKind::Inner,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// assert_eq!(value.table.name.segments(), ["parent"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub table: TableRef,
    /// Required for ordinary joins and forbidden for CROSS.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Join, JoinKind};
    /// let value = Join {
    ///     kind: JoinKind::Inner,
    ///     table: "parent".into(),
    ///     on: Some(Expr::eq("id", 7)),
    /// };
    /// assert!(value.on.is_some());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub on: Option<Expr>,
}

/// One ordered sort expression.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
/// let query = Select::new(["id"])
///     .select("name")
///     .from(TableRef::new("profile").alias("p"))
///     .join(Join {
///         kind: JoinKind::Cross,
///         table: "category".into(),
///         on: None,
///     })
///     .sort(Sort {
///         expression: Expr::column("id"),
///         order: Order::Desc,
///     })
///     .limit(10)
///     .offset(2)
///     .distinct();
/// let data = query.build();
/// assert_eq!(data.selectors.len(), 2);
/// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
/// assert_eq!(data.joins.len(), 1);
/// assert!(data.filters.is_empty());
/// assert_eq!(data.sort[0].order, Order::Desc);
/// assert_eq!(
///     (data.limit, data.offset, data.distinct),
///     (Some(10), Some(2), true)
/// );
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains("CROSS JOIN")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Sort {
    /// A typed column or expression, with bound data if needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Order, Sort};
    /// let value = Sort {
    ///     expression: Expr::column("id"),
    ///     order: Order::Asc,
    /// };
    /// assert!(matches!(value.expression, Expr::Column(_)));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub expression: Expr,
    /// Explicit direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Order, Sort};
    /// let value = Sort {
    ///     expression: Expr::column("id"),
    ///     order: Order::Asc,
    /// };
    /// assert_eq!(value.order, Order::Asc);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub order: Order,
}

/// Immutable inspection snapshot borrowed from a [`crate::Select`].
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
/// let query = Select::new(["id"])
///     .select("name")
///     .from(TableRef::new("profile").alias("p"))
///     .join(Join {
///         kind: JoinKind::Cross,
///         table: "category".into(),
///         on: None,
///     })
///     .sort(Sort {
///         expression: Expr::column("id"),
///         order: Order::Desc,
///     })
///     .limit(10)
///     .offset(2)
///     .distinct();
/// let data = query.build();
/// assert_eq!(data.selectors.len(), 2);
/// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
/// assert_eq!(data.joins.len(), 1);
/// assert!(data.filters.is_empty());
/// assert_eq!(data.sort[0].order, Order::Desc);
/// assert_eq!(
///     (data.limit, data.offset, data.distinct),
///     (Some(10), Some(2), true)
/// );
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains("CROSS JOIN")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SelectData {
    /// Ordered projections; default construction selects `*`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert_eq!(value.selectors.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub selectors: Vec<Selector>,
    /// Required source relation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert!(value.table.is_some());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub table: Option<TableRef>,
    /// Ordered joins.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert!(value.joins.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub joins: Vec<Join>,
    /// Ordered predicates combined by AND.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert!(value.filters.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub filters: Vec<Expr>,
    /// Ordered sorting.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert_eq!(value.sort.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub sort: Vec<Sort>,
    /// None leaves SQL unbounded; Some(0) means zero rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert_eq!(value.limit, Some(10));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub limit: Option<u64>,
    /// Optional row offset; offset alone uses SQLite LIMIT -1.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert_eq!(value.offset, None);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub offset: Option<u64>,
    /// Whether duplicate projected rows are removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").order("id").limit(10);
    /// let value = query.build();
    /// assert_eq!(value.distinct, false);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub distinct: bool,
}

/// Owned reusable SELECT, independent from a database or transaction.
///
/// Builders retain owned data and defer validation to [`crate::Render::render`].
/// [`Self::build`] borrows an inspection snapshot and performs no validation.
/// [`Self::where_eq`] and [`Self::and_where`] append predicates joined with AND
/// in call order. SELECT may omit filters.
/// `limit(0)` requests zero rows; absent limit leaves the adapter output budget in force.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
/// let query = Select::new(["id"])
///     .select("name")
///     .from(TableRef::new("profile").alias("p"))
///     .join(Join {
///         kind: JoinKind::Cross,
///         table: "category".into(),
///         on: None,
///     })
///     .sort(Sort {
///         expression: Expr::column("id"),
///         order: Order::Desc,
///     })
///     .limit(10)
///     .offset(2)
///     .distinct();
/// let data = query.build();
/// assert_eq!(data.selectors.len(), 2);
/// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
/// assert_eq!(data.joins.len(), 1);
/// assert!(data.filters.is_empty());
/// assert_eq!(data.sort[0].order, Order::Desc);
/// assert_eq!(
///     (data.limit, data.offset, data.distinct),
///     (Some(10), Some(2), true)
/// );
/// assert!(
///     query
///         .render(DialectId::Sqlite)?
///         .sql()
///         .contains("CROSS JOIN")
/// );
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
///
/// A discarded builder loses its configured operation.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// stackpress_sql::Select::new(["id"]).limit(1);
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use = "use the configured value or operation"]
pub struct Select {
    /// All mutable builder state; only consumed fluent calls can change it.
    data: SelectData,
}

impl Default for Select {
    /// Select all columns; a table must still be supplied before rendering.
    fn default() -> Self {
        Self::new(["*"])
    }
}

impl Select {
    /// Select an explicit list. Only the exact string `*` is special syntax.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::new(["id", "name"]);
    /// assert_eq!(query.build().selectors.len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
        }
    }

    /// Inspect immutable data; cloning the builder produces independent state.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile").limit(0);
    /// assert_eq!(query.build().limit, Some(0));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn build(&self) -> &SelectData {
        &self.data
    }

    /// Add another typed projection in SQL/bind order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::new(["id"]).select("name");
    /// assert_eq!(query.build().selectors.len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn select(mut self, selector: impl Into<Selector>) -> Self {
        self.data.selectors.push(selector.into());
        self
    }

    /// Set a literal or explicitly qualified/aliased source relation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// let query = Select::default().from("profile");
    /// assert!(query.build().table.is_some());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn from(mut self, table: impl Into<TableRef>) -> Self {
        self.data.table = Some(table.into());
        self
    }

    /// Request duplicate-row elimination.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// assert!(Select::default().distinct().build().distinct);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn distinct(mut self) -> Self {
        self.data.distinct = true;
        self
    }

    /// Append a typed join, validated as a whole before rendering returns.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
    /// let query = Select::new(["id"])
    ///     .select("name")
    ///     .from(TableRef::new("profile").alias("p"))
    ///     .join(Join {
    ///         kind: JoinKind::Cross,
    ///         table: "category".into(),
    ///         on: None,
    ///     })
    ///     .sort(Sort {
    ///         expression: Expr::column("id"),
    ///         order: Order::Desc,
    ///     })
    ///     .limit(10)
    ///     .offset(2)
    ///     .distinct();
    /// let data = query.build();
    /// assert_eq!(data.selectors.len(), 2);
    /// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
    /// assert_eq!(data.joins.len(), 1);
    /// assert!(data.filters.is_empty());
    /// assert_eq!(data.sort[0].order, Order::Desc);
    /// assert_eq!(
    ///     (data.limit, data.offset, data.distinct),
    ///     (Some(10), Some(2), true)
    /// );
    /// assert!(
    ///     query
    ///         .render(DialectId::Sqlite)?
    ///         .sql()
    ///         .contains("CROSS JOIN")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn join(mut self, join: Join) -> Self {
        self.data.joins.push(join);
        self
    }

    /// Append a bound equality predicate, joined with AND in call order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Select, SqlValue};
    /// let query = Select::default()
    ///     .from("profile")
    ///     .where_eq("id", 7)
    ///     .where_eq("active", true);
    /// let rendered = query.render(DialectId::Sqlite)?;
    /// assert_eq!(
    ///     rendered.sql(),
    ///     "SELECT * FROM \"profile\" WHERE \"id\" = ? AND \"active\" = ?"
    /// );
    /// assert_eq!(rendered.binds(), &[SqlValue::Integer(7), SqlValue::Bool(true)]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_eq(self, column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        self.and_where(Expr::eq(column, value))
    }

    /// Append a typed condition while preserving group parentheses.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Select};
    /// let query = Select::default()
    ///     .from("profile")
    ///     .and_where(Expr::eq("id", 7));
    /// assert_eq!(query.build().filters.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn and_where(mut self, expression: Expr) -> Self {
        self.data.filters.push(expression);
        self
    }

    /// Append exact typed JSON scalar equality at a checked path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, JsonPath, JsonPathOptions, JsonScalar, Render, Select};
    /// let path = JsonPath::parse("tags", JsonPathOptions::default())?;
    /// let query = Select::default().from("profile").where_json(
    ///     "data",
    ///     path,
    ///     JsonScalar::String("admin".into()),
    /// );
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.render(DialectId::Sqlite)?.binds().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_json(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_eq(column, path, value))
    }

    /// Append direct scalar membership in a JSON array.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, JsonPath, JsonPathOptions, JsonScalar, Render, Select};
    /// let path = JsonPath::parse("tags", JsonPathOptions::default())?;
    /// let query = Select::default().from("profile").where_json_contains(
    ///     "data",
    ///     path,
    ///     JsonScalar::String("admin".into()),
    /// );
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.render(DialectId::Sqlite)?.binds().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_json_contains(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_contains(column, path, value))
    }

    /// Append a column sort with an explicit direction.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Order, Select};
    /// let query = Select::default().order_by("id", Order::Desc);
    /// assert_eq!(query.build().sort[0].order, Order::Desc);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn order_by(mut self, column: impl Into<Identifier>, order: Order) -> Self {
        self.data.sort.push(Sort {
            expression: Expr::column(column),
            order,
        });
        self
    }

    /// Append the default ascending column sort.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Order, Select};
    /// assert_eq!(
    ///     Select::default().order("id").build().sort[0].order,
    ///     Order::Asc
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn order(self, column: impl Into<Identifier>) -> Self {
        self.order_by(column, Order::Asc)
    }

    /// Append an explicit expression sort.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Join, JoinKind, Order, Render, Select, Sort, TableRef};
    /// let query = Select::new(["id"])
    ///     .select("name")
    ///     .from(TableRef::new("profile").alias("p"))
    ///     .join(Join {
    ///         kind: JoinKind::Cross,
    ///         table: "category".into(),
    ///         on: None,
    ///     })
    ///     .sort(Sort {
    ///         expression: Expr::column("id"),
    ///         order: Order::Desc,
    ///     })
    ///     .limit(10)
    ///     .offset(2)
    ///     .distinct();
    /// let data = query.build();
    /// assert_eq!(data.selectors.len(), 2);
    /// assert_eq!(data.table.as_ref().unwrap().alias.as_deref(), Some("p"));
    /// assert_eq!(data.joins.len(), 1);
    /// assert!(data.filters.is_empty());
    /// assert_eq!(data.sort[0].order, Order::Desc);
    /// assert_eq!(
    ///     (data.limit, data.offset, data.distinct),
    ///     (Some(10), Some(2), true)
    /// );
    /// assert!(
    ///     query
    ///         .render(DialectId::Sqlite)?
    ///         .sql()
    ///         .contains("CROSS JOIN")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn sort(mut self, sort: Sort) -> Self {
        self.data.sort.push(sort);
        self
    }

    /// Set a row limit; zero is retained rather than treated as absent.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// assert_eq!(Select::default().limit(0).build().limit, Some(0));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn limit(mut self, limit: u64) -> Self {
        self.data.limit = Some(limit);
        self
    }

    /// Set a row offset, checked against SQLite's signed integer range.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Select;
    /// assert_eq!(Select::default().offset(0).build().offset, Some(0));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
/// let insert = Insert::new("profile")
///     .columns(["name"])
///     .values([WriteValue::from("Ada")])
///     .values([WriteValue::from("Grace")])
///     .returning(["id"]);
/// let data = insert.build();
/// assert_eq!(data.table.segments(), ["profile"]);
/// assert_eq!(data.columns, ["name"]);
/// assert_eq!(data.rows.len(), 2);
/// assert_eq!(data.returning.len(), 1);
/// assert!(!data.default_values);
/// assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum WriteValue {
    /// Explicit structured JSON: bound text is cast to the dialect's native JSON storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, JsonScalar, JsonValue, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["value"])
    ///     .values([WriteValue::Json(JsonValue::Scalar(JsonScalar::Null))]);
    /// insert.render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Json(crate::JsonValue),
    /// Owned native bind, including SQL NULL.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["value"])
    ///     .values([WriteValue::Value("Ada".into())]);
    /// insert.render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Value(SqlValue),
    /// Omit/use the column's default; SQLite allows this only for a whole omitted column.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["value"])
    ///     .values([WriteValue::Default]);
    /// insert.render(DialectId::Postgres)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Default,
    /// A typed expression, useful for updates such as `count = count + ...` via trusted SQL.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Expr, Insert, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["value"])
    ///     .values([WriteValue::Expression(Expr::value(7))]);
    /// insert.render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Expression(Expr),
}

impl<T: Into<SqlValue>> From<T> for WriteValue {
    /// Preserve one scalar as a bind rather than SQL text.
    fn from(value: T) -> Self {
        Self::Value(value.into())
    }
}

/// Immutable inspection data for one or many explicit-column insert rows.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
/// let insert = Insert::new("profile")
///     .columns(["name"])
///     .values([WriteValue::from("Ada")])
///     .values([WriteValue::from("Grace")])
///     .returning(["id"]);
/// let data = insert.build();
/// assert_eq!(data.table.segments(), ["profile"]);
/// assert_eq!(data.columns, ["name"]);
/// assert_eq!(data.rows.len(), 2);
/// assert_eq!(data.returning.len(), 1);
/// assert!(!data.default_values);
/// assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct InsertData {
    /// Literal destination table.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let query = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// let value = query.build();
    /// assert_eq!(value.table.segments(), ["profile"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub table: Identifier,
    /// Stable column order shared by every row.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let query = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// let value = query.build();
    /// assert_eq!(value.columns, ["name"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub columns: Vec<String>,
    /// Each row must have exactly the column count.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let query = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// let value = query.build();
    /// assert_eq!(value.rows.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub rows: Vec<Vec<WriteValue>>,
    /// Explicit RETURNING list, empty when not requested.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let query = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// let value = query.build();
    /// assert!(value.returning.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub returning: Vec<Selector>,
    /// Explicit `DEFAULT VALUES`, mutually exclusive with columns/rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let query = Insert::new("profile").columns(["name"]).values(["Ada"]);
    /// let value = query.build();
    /// assert_eq!(value.default_values, false);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub default_values: bool,
}

/// Owned INSERT builder with stable bulk column and bind order.
///
/// Call [`Self::columns`] once to set stable column order, then [`Self::values`]
/// for each row. Every row must have that arity. Use [`Self::default_values`] for
/// a single all-default row. SQLite rejects per-cell DEFAULT; JSON writes can
/// use [`crate::WriteValue::Json`] to request the native JSON cast. RETURNING is unsupported
/// on MySQL. Construction/build are inspection only; rendering validates the shape.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
/// let insert = Insert::new("profile")
///     .columns(["name"])
///     .values([WriteValue::from("Ada")])
///     .values([WriteValue::from("Grace")])
///     .returning(["id"]);
/// let data = insert.build();
/// assert_eq!(data.table.segments(), ["profile"]);
/// assert_eq!(data.columns, ["name"]);
/// assert_eq!(data.rows.len(), 2);
/// assert_eq!(data.returning.len(), 1);
/// assert!(!data.default_values);
/// assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
///
/// A discarded builder loses its configured operation.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// stackpress_sql::Insert::new("items").default_values();
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use = "use the configured value or operation"]
pub struct Insert {
    data: InsertData,
}

impl Insert {
    /// Name a destination; columns/values must be supplied or default_values requested.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let insert = Insert::new("profile");
    /// assert_eq!(insert.build().table.segments(), ["profile"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["name"])
    ///     .values([WriteValue::from("Ada")])
    ///     .values([WriteValue::from("Grace")])
    ///     .returning(["id"]);
    /// let data = insert.build();
    /// assert_eq!(data.table.segments(), ["profile"]);
    /// assert_eq!(data.columns, ["name"]);
    /// assert_eq!(data.rows.len(), 2);
    /// assert_eq!(data.returning.len(), 1);
    /// assert!(!data.default_values);
    /// assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn build(&self) -> &InsertData {
        &self.data
    }

    /// Set explicit stable columns; duplicate/empty names are render errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Insert;
    /// let insert = Insert::new("profile").columns(["name", "id"]);
    /// assert_eq!(insert.build().columns, ["name", "id"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn columns(mut self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.data.columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Append one row in column order; bulk inserts call this repeatedly.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["name"])
    ///     .values([WriteValue::from("Ada")])
    ///     .values([WriteValue::from("Grace")])
    ///     .returning(["id"]);
    /// let data = insert.build();
    /// assert_eq!(data.table.segments(), ["profile"]);
    /// assert_eq!(data.columns, ["name"]);
    /// assert_eq!(data.rows.len(), 2);
    /// assert_eq!(data.returning.len(), 1);
    /// assert!(!data.default_values);
    /// assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn values(mut self, values: impl IntoIterator<Item = impl Into<WriteValue>>) -> Self {
        self.data
            .rows
            .push(values.into_iter().map(Into::into).collect());
        self
    }

    /// Explicitly insert a row using database defaults for every column.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render};
    /// let insert = Insert::new("profile").default_values();
    /// assert!(insert.build().default_values);
    /// assert!(
    ///     insert
    ///         .render(DialectId::Sqlite)?
    ///         .sql()
    ///         .contains("DEFAULT VALUES")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn default_values(mut self) -> Self {
        self.data.default_values = true;
        self
    }

    /// Replace RETURNING projections with an explicit ordered list.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render, WriteValue};
    /// let insert = Insert::new("profile")
    ///     .columns(["name"])
    ///     .values([WriteValue::from("Ada")])
    ///     .values([WriteValue::from("Grace")])
    ///     .returning(["id"]);
    /// let data = insert.build();
    /// assert_eq!(data.table.segments(), ["profile"]);
    /// assert_eq!(data.columns, ["name"]);
    /// assert_eq!(data.rows.len(), 2);
    /// assert_eq!(data.returning.len(), 1);
    /// assert!(!data.default_values);
    /// assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn returning(mut self, selectors: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        self.data.returning = selectors.into_iter().map(Into::into).collect();
        self
    }

    /// Request every returned column.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Insert, Render};
    /// let insert = Insert::new("profile")
    ///     .columns(["name"])
    ///     .values(["Ada"])
    ///     .returning_all();
    /// assert!(
    ///     insert
    ///         .render(DialectId::Postgres)?
    ///         .sql()
    ///         .ends_with("RETURNING *")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
                let columns = join_sql(
                    included
                        .iter()
                        .map(|index| dialect.quote(&data.columns[*index])),
                    ", ",
                )?;
                // Both a row and the full VALUES body are admitted incrementally. DEFAULT
                // and trusted expressions can grow SQL without consuming the bind budget.
                let rows = join_sql(
                    data.rows.iter().map(|row| {
                        let values = join_sql(
                            included
                                .iter()
                                .map(|index| render_write(&row[*index], &mut context)),
                            ", ",
                        )?;
                        Ok(format!("({values})"))
                    }),
                    ", ",
                )?;
                sql.push_str(&format!(" ({columns}) VALUES {rows}"));
            }
        }
        append_returning(&mut sql, &data.returning, &mut context)?;
        context.finish(sql)
    }
}

/// Shared inspectable mutation scope, including the explicit all-rows authorization.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Delete, DialectId, Render};
/// let query = Delete::new("profile").where_eq("id", 7).returning(["id"]);
/// assert_eq!(query.build().filters.len(), 1);
/// assert!(!query.build().all_rows);
/// assert_eq!(query.build().returning.len(), 1);
/// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct MutationData {
    /// Literal or explicitly qualified target.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Delete;
    /// let query = Delete::new("profile").where_eq("id", 7);
    /// let value = query.build();
    /// assert_eq!(value.table.segments(), ["profile"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub table: Identifier,
    /// Bound predicates in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Delete;
    /// let query = Delete::new("profile").where_eq("id", 7);
    /// let value = query.build();
    /// assert_eq!(value.filters.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub filters: Vec<Expr>,
    /// Explicit permission for a mutation without predicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Delete;
    /// let query = Delete::new("profile").where_eq("id", 7);
    /// let value = query.build();
    /// assert_eq!(value.all_rows, false);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub all_rows: bool,
    /// Explicit RETURNING projections.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Delete;
    /// let query = Delete::new("profile").where_eq("id", 7);
    /// let value = query.build();
    /// assert!(value.returning.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
///
/// Rendering requires a nonempty filter or an explicit [`Self::all_rows`] choice.
/// An empty/invalid filter still errors after all_rows; it never widens the mutation.
/// RETURNING is unsupported on MySQL. Builder methods retain owned input;
/// [`Self::build`] borrows data without validation or execution.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Render, Update};
/// let query = Update::new("profile")
///     .set("name", "Ada")
///     .where_eq("id", 7)
///     .returning(["id"]);
/// assert_eq!(query.build().filters.len(), 1);
/// assert!(!query.build().all_rows);
/// assert_eq!(query.build().returning.len(), 1);
/// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
///
/// A discarded builder loses its configured operation.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// stackpress_sql::Update::new("items").all_rows();
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use = "use the configured value or operation"]
pub struct Update {
    /// Shared target, predicates and returned projections.
    scope: MutationData,
    /// Stable unique assignment names and explicit cell states.
    assignments: Vec<(String, WriteValue)>,
}

impl Update {
    /// Name the target without granting permission to update all rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Update;
    /// let query = Update::new("profile");
    /// assert_eq!(query.build().table.segments(), ["profile"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn new(table: impl Into<Identifier>) -> Self {
        Self {
            scope: MutationData::new(table),
            assignments: Vec::new(),
        }
    }

    /// Inspect target/filter/returning state.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Update};
    /// let query = Update::new("profile")
    ///     .set("name", "Ada")
    ///     .where_eq("id", 7)
    ///     .returning(["id"]);
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.build().all_rows);
    /// assert_eq!(query.build().returning.len(), 1);
    /// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn build(&self) -> &MutationData {
        &self.scope
    }

    /// Inspect ordered assignments without exposing mutable builder state.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Update, WriteValue};
    /// let query = Update::new("profile").set("name", "Ada");
    /// assert_eq!(
    ///     query.assignments(),
    ///     &[("name".into(), WriteValue::Value("Ada".into()))]
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn assignments(&self) -> &[(String, WriteValue)] {
        &self.assignments
    }

    /// Append an assignment; repeated names are rejected at render time.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Update, WriteValue};
    /// let query = Update::new("profile").set("name", "Ada");
    /// assert_eq!(
    ///     query.assignments(),
    ///     &[("name".into(), WriteValue::Value("Ada".into()))]
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn set(mut self, column: impl Into<String>, value: impl Into<WriteValue>) -> Self {
        self.assignments.push((column.into(), value.into()));
        self
    }

    /// Append a bound equality predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Update;
    /// let query = Update::new("profile")
    ///     .set("name", "Ada")
    ///     .where_eq("id", 7)
    ///     .where_eq("active", true);
    /// assert_eq!(query.build().filters.len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_eq(self, column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        self.and_where(Expr::eq(column, value))
    }

    /// Append a typed predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Expr, Update};
    /// let query = Update::new("profile")
    ///     .set("name", "Ada")
    ///     .and_where(Expr::eq("id", 7));
    /// assert_eq!(query.build().filters.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn and_where(mut self, expression: Expr) -> Self {
        self.scope.filters.push(expression);
        self
    }

    /// Match an exact JSON scalar at a checked path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, JsonPath, JsonPathOptions, JsonScalar, Render, Update};
    /// let path = JsonPath::parse("tags", JsonPathOptions::default())?;
    /// let query = Update::new("profile").set("name", "Ada").where_json(
    ///     "data",
    ///     path,
    ///     JsonScalar::String("admin".into()),
    /// );
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.render(DialectId::Sqlite)?.binds().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_json(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_eq(column, path, value))
    }

    /// Match direct JSON array membership.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, JsonPath, JsonPathOptions, JsonScalar, Render, Update};
    /// let path = JsonPath::parse("tags", JsonPathOptions::default())?;
    /// let query = Update::new("profile")
    ///     .set("name", "Ada")
    ///     .where_json_contains("data", path, JsonScalar::String("admin".into()));
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.render(DialectId::Sqlite)?.binds().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_json_contains(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_contains(column, path, value))
    }

    /// Explicitly authorize an update without predicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Update};
    /// let query = Update::new("profile").set("name", "Ada").all_rows();
    /// assert!(query.build().all_rows);
    /// query.render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn all_rows(mut self) -> Self {
        self.scope.all_rows = true;
        self
    }

    /// Replace the ordered RETURNING list.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Update};
    /// let query = Update::new("profile")
    ///     .set("name", "Ada")
    ///     .where_eq("id", 7)
    ///     .returning(["id"]);
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.build().all_rows);
    /// assert_eq!(query.build().returning.len(), 1);
    /// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn returning(mut self, selectors: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        self.scope.returning = selectors.into_iter().map(Into::into).collect();
        self
    }

    /// Return every updated column.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Update};
    /// let query = Update::new("profile")
    ///     .set("name", "Ada")
    ///     .where_eq("id", 7)
    ///     .returning_all();
    /// assert!(
    ///     query
    ///         .render(DialectId::Postgres)?
    ///         .sql()
    ///         .ends_with("RETURNING *")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
        // Bound generated text even for assignments with no ordinary bind values.
        let assignments = join_sql(
            self.assignments.iter().map(|(column, value)| {
                Ok(format!(
                    "{} = {}",
                    dialect.quote(column)?,
                    render_write(value, &mut context)?
                ))
            }),
            ", ",
        )?;
        let mut sql = format!(
            "UPDATE {} SET {}",
            self.scope.table.quoted(dialect)?,
            assignments
        );
        append_filters(&mut sql, &self.scope.filters, &mut context)?;
        append_returning(&mut sql, &self.scope.returning, &mut context)?;
        context.finish(sql)
    }
}

/// Owned DELETE; an absent/invalid predicate never silently widens mutation scope.
///
/// Rendering requires a nonempty filter or an explicit [`Self::all_rows`] choice.
/// An empty/invalid filter still errors after all_rows; it never widens the mutation.
/// RETURNING is unsupported on MySQL. Builder methods retain owned input;
/// [`Self::build`] borrows data without validation or execution.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Delete, DialectId, Render};
/// let query = Delete::new("profile").where_eq("id", 7).returning(["id"]);
/// assert_eq!(query.build().filters.len(), 1);
/// assert!(!query.build().all_rows);
/// assert_eq!(query.build().returning.len(), 1);
/// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
///
/// A discarded builder loses its configured operation.
///
/// ```compile_fail
/// #![deny(unused_must_use)]
/// stackpress_sql::Delete::new("items").all_rows();
/// ```
#[derive(Clone, Debug, PartialEq)]
#[must_use = "use the configured value or operation"]
pub struct Delete {
    scope: MutationData,
}

impl Delete {
    /// Name a target, initially requiring a predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Delete;
    /// let query = Delete::new("profile");
    /// assert_eq!(query.build().table.segments(), ["profile"]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn new(table: impl Into<Identifier>) -> Self {
        Self {
            scope: MutationData::new(table),
        }
    }

    /// Inspect all detached mutation state.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, DialectId, Render};
    /// let query = Delete::new("profile").where_eq("id", 7).returning(["id"]);
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.build().all_rows);
    /// assert_eq!(query.build().returning.len(), 1);
    /// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn build(&self) -> &MutationData {
        &self.scope
    }

    /// Append a bound equality predicate.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Delete;
    /// let query = Delete::new("profile")
    ///     .where_eq("id", 7)
    ///     .where_eq("active", true);
    /// assert_eq!(query.build().filters.len(), 2);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_eq(self, column: impl Into<Identifier>, value: impl Into<SqlValue>) -> Self {
        self.and_where(Expr::eq(column, value))
    }

    /// Append a typed predicate, preserving its group semantics.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, Expr};
    /// let query = Delete::new("profile").and_where(Expr::eq("id", 7));
    /// assert_eq!(query.build().filters.len(), 1);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn and_where(mut self, expression: Expr) -> Self {
        self.scope.filters.push(expression);
        self
    }

    /// Match an exact JSON scalar at a checked path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, DialectId, JsonPath, JsonPathOptions, JsonScalar, Render};
    /// let path = JsonPath::parse("tags", JsonPathOptions::default())?;
    /// let query = Delete::new("profile").where_json("data", path, JsonScalar::String("admin".into()));
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.render(DialectId::Sqlite)?.binds().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_json(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_eq(column, path, value))
    }

    /// Match direct scalar membership in a JSON array.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, DialectId, JsonPath, JsonPathOptions, JsonScalar, Render};
    /// let path = JsonPath::parse("tags", JsonPathOptions::default())?;
    /// let query = Delete::new("profile").where_json_contains(
    ///     "data",
    ///     path,
    ///     JsonScalar::String("admin".into()),
    /// );
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.render(DialectId::Sqlite)?.binds().is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn where_json_contains(
        self,
        column: impl Into<Identifier>,
        path: JsonPath,
        value: JsonScalar,
    ) -> Self {
        self.and_where(Expr::json_contains(column, path, value))
    }

    /// Explicitly authorize deleting every row.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, DialectId, Render};
    /// let query = Delete::new("profile").all_rows();
    /// assert!(query.build().all_rows);
    /// query.render(DialectId::Sqlite)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn all_rows(mut self) -> Self {
        self.scope.all_rows = true;
        self
    }

    /// Replace the ordered RETURNING list.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, DialectId, Render};
    /// let query = Delete::new("profile").where_eq("id", 7).returning(["id"]);
    /// assert_eq!(query.build().filters.len(), 1);
    /// assert!(!query.build().all_rows);
    /// assert_eq!(query.build().returning.len(), 1);
    /// assert!(query.render(DialectId::Sqlite)?.sql().contains("RETURNING"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn returning(mut self, selectors: impl IntoIterator<Item = impl Into<Selector>>) -> Self {
        self.scope.returning = selectors.into_iter().map(Into::into).collect();
        self
    }

    /// Return every deleted column.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{Delete, DialectId, Render};
    /// let query = Delete::new("profile").where_eq("id", 7).returning_all();
    /// assert!(
    ///     query
    ///         .render(DialectId::Postgres)?
    ///         .sql()
    ///         .ends_with("RETURNING *")
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
            let encoded = crate::json::encode_for_dialect(value, context.dialect)?;
            let marker = context.bind(encoded.into())?;
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
