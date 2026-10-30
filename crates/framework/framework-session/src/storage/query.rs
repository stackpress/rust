//! Source selector/where compilation using checked SQL identifiers and bound values.

use super::{ModelKind, ProfileStore, Result, SourceStatement, Store};
use crate::columns::Value;
use crate::columns::{self, Record};
use serde::{Deserialize, Serialize};
use stackpress_sql::{
    Compare, DialectId, Expr, Identifier, Join, JoinKind, JsonPath, JsonScalar, Order, Select,
    Selector, TableRef,
};

/// Source select/filter envelope. Empty and missing maps have the same source defaults.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StoreQuery {
    /// Source contains-search; only Profile has declared searchable columns.
    pub q: Option<String>,
    /// Requested source paths; defaults to all base columns.
    pub columns: Vec<String>,
    /// Equal values; active=-1 explicitly removes the implicit active=true predicate.
    pub eq: Record,
    /// Not-equal values (arrays are source OR groups).
    pub ne: Record,
    /// Greater-or-equal values.
    pub ge: Record,
    /// Less-or-equal values.
    pub le: Record,
    /// Contains-search values; % and _ retain SQL wildcard semantics.
    pub like: Record,
    /// JSON-array membership.
    pub has: Record,
    /// Source currently compiles this as membership too, preserved deliberately.
    pub hasnt: Record,
    /// Source ASC/DESC map, case insensitive; invalid directions are skipped.
    pub sort: Record,
    /// Source offset.
    pub skip: u64,
    /// Source default 50; zero means unlimited.
    pub take: u64,
}
impl Default for StoreQuery {
    /// Preserve original generated select defaults rather than the framework public-query cap.
    fn default() -> Self {
        Self {
            q: None,
            columns: vec!["*".into()],
            eq: Record::new(),
            ne: Record::new(),
            ge: Record::new(),
            le: Record::new(),
            like: Record::new(),
            has: Record::new(),
            hasnt: Record::new(),
            sort: Record::new(),
            skip: 0,
            take: 50,
        }
    }
}
impl StoreQuery {
    /// Construct a focused equal-field query, retaining all ordinary defaults.
    pub fn eq(name: impl Into<String>, value: impl Into<Value>) -> Self {
        let mut query = Self::default();
        query.eq.insert(name.into(), value.into());
        query
    }
    /// Explicitly include inactive rows, as the original eq.active=-1 sentinel does.
    pub fn all(mut self) -> Self {
        self.eq.insert("active".into(), (-1).into());
        self
    }
}

/// Resolved source selector with finite table/column identity and nested output path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreSelector {
    /// Original expression after wildcard expansion.
    pub expression: String,
    /// Native output alias.
    pub alias: String,
    /// Source field name, not snake-case SQL spelling.
    pub field: String,
    /// SQL column spelling.
    pub column: String,
    /// Constant base/relation alias.
    pub table: String,
    /// Source nested output path.
    pub path: Vec<String>,
    /// JSON extraction path for filtering and ordering.
    pub json: Vec<String>,
    /// True for the Auth store's Profile edge.
    pub profile: bool,
}
impl StoreSelector {
    /// Checked qualified SQL column.
    fn identifier(&self) -> Identifier {
        Identifier::qualified([self.table.clone(), self.column.clone()])
    }
    /// Extract JSON only for predicates/sort; source selection still reads the whole column.
    fn operand(&self) -> Result<Expr> {
        let column = self.identifier();
        if self.json.is_empty() {
            return Ok(Expr::column(column));
        }
        Ok(Expr::json(column, path(&self.json)?))
    }
}

impl<M: ModelKind> Store<M> {
    /// Resolve one source expression, ignoring unknown paths as the generated store does.
    /// Profile has no reverse store relations even though Idea declares reverse metadata.
    pub fn paths(&self, expression: &str) -> Vec<StoreSelector> {
        self.selectors(&[expression.into()])
    }

    /// Expand source wildcards and de-duplicate by expression in first registration order.
    pub fn selectors(&self, expressions: &[String]) -> Vec<StoreSelector> {
        let profile_store = ProfileStore::new(self.seed.clone());
        let mut selectors = Vec::new();
        for expression in expressions {
            let (field_path, json) = expression.split_once(':').unwrap_or((expression, ""));
            let profile = M::NAME == "Auth" && field_path.starts_with("profile.");
            let field = if profile {
                field_path.strip_prefix("profile.").unwrap_or(field_path)
            } else {
                field_path
            };
            // Source stops after the first recognized scalar; child segments are retained in
            // the output path even though no further column/relation traversal takes place.
            let first = field.split('.').next().unwrap_or_default();
            let schema = if profile {
                &profile_store.schema
            } else {
                &self.schema
            };
            for column in schema
                .columns
                .iter()
                .filter(|column| first == "*" || column.name == first)
            {
                let expanded = expression.replacen('*', column.name, 1);
                if selectors
                    .iter()
                    .any(|selector: &StoreSelector| selector.expression == expanded)
                {
                    continue;
                }
                let mut output_path: Vec<String> = expanded
                    .replace(':', ".")
                    .split('.')
                    .map(str::to_owned)
                    .collect();
                if output_path.is_empty() {
                    output_path.push(column.name.into());
                }
                let alias = output_path
                    .iter()
                    .map(|part| {
                        if part == "profileId" {
                            "profile_id"
                        } else {
                            part
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("__");
                selectors.push(StoreSelector {
                    expression: expanded,
                    alias,
                    field: column.name.into(),
                    column: column.sql.into(),
                    table: if profile {
                        "auth__profile".into()
                    } else {
                        M::TABLE.into()
                    },
                    path: output_path,
                    json: json
                        .split('.')
                        .filter(|part| !part.is_empty())
                        .map(str::to_owned)
                        .collect(),
                    profile,
                });
            }
        }
        selectors
    }

    /// Determine whether any selected/filter/sort path requires the original inner join.
    pub fn joins(&self, query: &StoreQuery) -> Vec<Join> {
        let mut expressions = query.columns.clone();
        for map in [
            &query.eq,
            &query.ne,
            &query.ge,
            &query.le,
            &query.like,
            &query.has,
            &query.hasnt,
            &query.sort,
        ] {
            expressions.extend(flatten(map).keys().cloned());
        }
        if !self
            .selectors(&expressions)
            .iter()
            .any(|selector| selector.profile)
        {
            return Vec::new();
        }
        vec![Join {
            kind: JoinKind::Inner,
            table: TableRef::new("profile").alias("auth__profile"),
            on: Some(
                Expr::column(Identifier::qualified(["auth", "profile_id"])).compare(
                    Compare::Eq,
                    Expr::column(Identifier::qualified(["auth__profile", "id"])),
                ),
            ),
        }]
    }

    /// Compile selected columns, relation joins, source filters and optional pagination.
    pub fn select(
        &self,
        query: &StoreQuery,
        dialect: DialectId,
    ) -> Result<SourceStatement<Select>> {
        let selectors = self.selectors(&query.columns).into_iter().map(|selector| {
            Selector::expression(Expr::column(selector.identifier())).alias(selector.alias)
        });
        let mut select = Select::new(selectors).from(M::TABLE);
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

    /// Source conjunctive filter set; invalid/empty scalar conditions are silently skipped.
    pub fn where_clause(&self, query: &StoreQuery, dialect: DialectId) -> Result<Option<Expr>> {
        let mut filters = Vec::new();
        if !query.eq.contains_key("active") {
            filters.push(Expr::eq(Identifier::qualified([M::TABLE, "active"]), true));
        }
        if M::NAME == "Profile"
            && let Some(q) = query.q.as_ref().filter(|q| !q.is_empty())
        {
            filters.push(
                Expr::column(Identifier::qualified([M::TABLE, "name"]))
                    .compare(Compare::Like, Expr::value(format!("%{q}%"))),
            );
        }
        for (map, compare, like) in [
            (&query.eq, Compare::Eq, false),
            (&query.like, Compare::Like, true),
            (&query.ne, Compare::Ne, false),
            (&query.ge, Compare::Ge, false),
            (&query.le, Compare::Le, false),
        ] {
            for (expression, value) in flatten(map) {
                if std::ptr::eq(map, &query.eq) && expression == "active" && value == -1 {
                    continue;
                }
                if let Some(filter) = self.where_values(&expression, &value, compare, like)? {
                    filters.push(filter);
                }
            }
        }
        // These source helpers bypass the ordinary scalar selector lookup. An unknown
        // column therefore reaches the database instead of silently widening the query.
        for map in [&query.has, &query.hasnt] {
            for (expression, value) in flatten(map) {
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
                    filters.push(Expr::any(members));
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
        let profile = ProfileStore::new(self.seed.clone());
        let schema = if selector.profile {
            &profile.schema
        } else {
            &self.schema
        };
        let Some(column) = schema.column(&selector.field) else {
            return Ok(None);
        };
        let values = value
            .as_array()
            .cloned()
            .unwrap_or_else(|| vec![value.clone()]);
        let mut filters = Vec::new();
        for value in values {
            let value = if like && value.is_string() {
                Value::String(format!("%{}%", columns::string(&value)))
            } else {
                value
            };
            let serialized = if selector.json.is_empty() {
                column.serialize(Some(&value), &schema.seed, true, schema.offset)?
            } else {
                Some(value)
            };
            let Some(value) =
                serialized.filter(|value| !value.is_null() && value.as_str() != Some(""))
            else {
                continue;
            };
            // Source selects the column's SQL conversion even for extracted JSON values.
            let scalar = if selector.profile {
                profile.to_sql_value(&selector.field, &value)?
            } else {
                self.to_sql_value(&selector.field, &value)?
            };
            filters.push(selector.operand()?.compare(compare, Expr::value(scalar)));
        }
        Ok((!filters.is_empty()).then(|| Expr::any(filters)))
    }
}

/// Source flatten traverses plain nested objects, retaining arrays as OR/membership values.
pub(crate) fn flatten(value: &Record) -> Record {
    let mut result = Record::new();
    for (name, value) in value {
        if let Value::Object(object) = value {
            for (child, value) in flatten(object) {
                result.insert(format!("{name}.{child}"), value);
            }
        } else {
            result.insert(name.clone(), value.clone());
        }
    }
    result
}

/// Convert a JSON path to the SQL crate's checked path representation.
fn path(parts: &[String]) -> Result<JsonPath> {
    Ok(JsonPath::new(parts.iter().map(|part| {
        if !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) {
            part.parse::<u32>()
                .map(stackpress_sql::JsonSegment::Index)
                .unwrap_or_else(|_| stackpress_sql::JsonSegment::Key(part.clone()))
        } else {
            stackpress_sql::JsonSegment::Key(part.clone())
        }
    })))
}

/// Native JSON membership accepts the source scalar family, with unsupported shapes explicit.
fn json_scalar(value: &Value) -> Result<JsonScalar> {
    Ok(match value {
        Value::Null => JsonScalar::Null,
        Value::Bool(value) => JsonScalar::Bool(*value),
        Value::String(value) => JsonScalar::String(value.clone()),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                JsonScalar::Integer(value)
            } else {
                JsonScalar::Real(value.as_f64().unwrap_or_default())
            }
        }
        _ => {
            return Err(super::ActionError::new(
                500,
                "JSON membership requires a scalar",
                Record::new(),
            ));
        }
    })
}

/// SQLite source json_each compares extracted SQL scalars: null never equals null,
/// and Boolean/number equality follows SQLite, rather than adding a JSON type guard.
fn membership(
    identifier: Identifier,
    json: &[String],
    value: &Value,
    dialect: DialectId,
) -> Result<Expr> {
    if dialect != DialectId::Sqlite {
        return Ok(Expr::json_contains(
            identifier,
            path(json)?,
            json_scalar(value)?,
        ));
    }
    use stackpress_sql::Render;
    let rendered = Select::new([Selector::expression(Expr::column(identifier))])
        .from("__source")
        .render(dialect)?;
    // The selected expression is a checked identifier, with no aliases, binds or clauses.
    let column = rendered
        .sql()
        .strip_prefix("SELECT ")
        .and_then(|sql| sql.strip_suffix(" FROM \"__source\""))
        .ok_or_else(|| {
            super::ActionError::new(500, "native column rendering failed", Record::new())
        })?;
    let mut path = String::from("$");
    for part in json {
        if !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) {
            path.push_str(&format!("[{part}]"));
        } else {
            path.push('.');
            path.push_str(
                &serde_json::to_string(part).map_err(|cause| {
                    super::ActionError::new(500, cause.to_string(), Record::new())
                })?,
            );
        }
    }
    let scalar = match value {
        Value::Null => stackpress_sql::SqlValue::Null,
        Value::Bool(value) => stackpress_sql::SqlValue::Bool(*value),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                stackpress_sql::SqlValue::Integer(value)
            } else {
                stackpress_sql::SqlValue::Real(value.as_f64().unwrap_or_default())
            }
        }
        Value::String(value) => stackpress_sql::SqlValue::Text(value.clone()),
        _ => {
            return Err(super::ActionError::new(
                500,
                "JSON membership requires a scalar",
                Record::new(),
            ));
        }
    };
    Ok(Expr::Trusted(
        dialect,
        stackpress_sql::TrustedSql::audited(format!(
            "EXISTS (SELECT 1 FROM json_each({column}, ?) WHERE value = ?)"
        )),
        vec![path.into(), scalar],
    ))
}
