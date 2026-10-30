//! Source selector/where compilation using checked SQL identifiers and bound values.

use super::Value;
use super::membership::{flatten, membership, path};
use super::{Column, Schema};
use super::{ModelKind, Record, Result, Store};
use crate::sql::SourceStatement;
use serde::{Deserialize, Serialize};
use stackpress_framework_session::profile::ProfileSchema;
use stackpress_sql::{
    Compare, DialectId, Expr, Identifier, Join, JoinKind, Order, Select, Selector, TableRef,
};

/// Source select/filter envelope. Empty and missing maps have the same source defaults.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StoreQuery {
    /// Source contains-search; Application declares name and website; Session ignores it.
    pub q: Option<Value>,
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
    /// Model providing the selected scalar codec.
    pub model: &'static str,
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

    /// Resolve all forward stored relations, including Session.application.profile.
    /// Reverse Idea arrays have no generated @relation and therefore no store join.
    fn resolve(&self, expression: &str) -> (Vec<Column>, &'static str, String, Vec<Edge>) {
        let (field, _) = expression.split_once(':').unwrap_or((expression, ""));
        let mut parts = field.split('.').peekable();
        let mut model = M::NAME;
        let mut table = M::TABLE.to_owned();
        let mut edges = Vec::new();
        let mut last_relation = None;
        while let Some(part) = parts.next() {
            let columns = model_columns(model);
            if part == "*" {
                return (columns, model, table, edges);
            }
            if let Some(column) = columns.iter().find(|column| column.name == part) {
                return (vec![column.clone()], model, table, edges);
            }
            let target = match (model, part) {
                ("Application" | "Session", "profile") => {
                    Some(("Profile", "profile", "profile_id"))
                }
                ("Session", "application") => {
                    Some(("Application", "application", "application_id"))
                }
                _ => None,
            };
            let Some((next, target, local)) = target else {
                break;
            };
            // Source paths retain the last valid relation even if the next segment
            // is unknown. Its non-column selector reaches SQL and fails there.
            last_relation = Some((
                Column {
                    name: if part == "profile" {
                        "profile"
                    } else {
                        "application"
                    },
                    sql: if part == "profile" {
                        "profile"
                    } else {
                        "application"
                    },
                    kind: super::Kind::String,
                    nullable: false,
                    required_message: None,
                    minimum: None,
                    default: None,
                },
                model,
                table.clone(),
            ));
            let alias = format!("{table}__{target}");
            edges.push(Edge {
                from: table,
                alias: alias.clone(),
                table: target,
                local,
            });
            model = next;
            table = alias;
            if parts.peek().is_none() {
                break;
            }
        }
        if let Some((column, model, table)) = last_relation {
            (vec![column], model, table, edges)
        } else {
            (Vec::new(), model, table, edges)
        }
    }
    /// Expand source wildcards and de-duplicate expressions in supplied order.
    pub fn selectors(&self, expressions: &[String]) -> Vec<StoreSelector> {
        let mut selectors: Vec<StoreSelector> = Vec::new();
        for expression in expressions {
            let (_, json) = expression.split_once(':').unwrap_or((expression, ""));
            let (columns, model, table, _) = self.resolve(expression);
            for column in columns {
                let expanded = expression.replacen('*', column.name, 1);
                if selectors
                    .iter()
                    .any(|selector| selector.expression == expanded)
                {
                    continue;
                }
                let output_path: Vec<String> = expanded
                    .replace(':', ".")
                    .split('.')
                    .map(str::to_owned)
                    .collect();
                let alias = output_path
                    .iter()
                    .map(|part| match part.as_str() {
                        "profileId" => "profile_id",
                        "applicationId" => "application_id",
                        _ => part,
                    })
                    .collect::<Vec<_>>()
                    .join("__");
                selectors.push(StoreSelector {
                    expression: expanded,
                    alias,
                    field: column.name.into(),
                    column: column.sql.into(),
                    table: table.clone(),
                    path: output_path,
                    json: json
                        .split('.')
                        .filter(|part| !part.is_empty())
                        .map(str::to_owned)
                        .collect(),
                    model,
                });
            }
        }
        selectors
    }
    /// Required inner joins arise from selections and every filter/sort path, without
    /// adding active/expiry predicates to the joined Application or Profile.
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
        let mut edges: Vec<Edge> = Vec::new();
        for expression in expressions {
            for edge in self.resolve(&expression).3 {
                if !edges.contains(&edge) {
                    edges.push(edge);
                }
            }
        }
        edges
            .into_iter()
            .map(|edge| Join {
                kind: JoinKind::Inner,
                table: TableRef::new(edge.table).alias(edge.alias.clone()),
                on: Some(
                    Expr::column(Identifier::qualified([edge.from, edge.local.into()])).compare(
                        Compare::Eq,
                        Expr::column(Identifier::qualified([edge.alias, "id".into()])),
                    ),
                ),
            })
            .collect()
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
        if M::NAME == "Application"
            && let Some(q) = query.q.as_ref().filter(|q| truthy(q))
        {
            // JS template interpolation follows the original value's truthiness. In
            // particular [] is truthy and becomes an empty contains-search, not omission.
            let q = source_text(q, self.offset);
            filters.push(Expr::any(["name", "website"].map(|column| {
                Expr::column(Identifier::qualified([M::TABLE, column]))
                    .compare(Compare::Like, Expr::value(format!("%{q}%")))
            })));
        }
        for (map, compare, like) in [
            (&query.eq, Compare::Eq, false),
            (&query.like, Compare::Like, true),
            (&query.ne, Compare::Ne, false),
            (&query.ge, Compare::Ge, false),
            (&query.le, Compare::Le, false),
        ] {
            for (expression, value) in flatten(map) {
                if std::ptr::eq(map, &query.eq)
                    && expression == "active"
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
        let column = model_columns(selector.model)
            .into_iter()
            .find(|column| column.name == selector.field);
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
                column.serialize(Some(&value), "", true, self.offset)?
            } else {
                Some(value)
            };
            let Some(value) =
                serialized.filter(|value| !value.is_null() && value.as_str() != Some(""))
            else {
                continue;
            };
            // Source selects the column's SQL conversion even for extracted JSON values.
            // Source uses the base store's toSqlValue, even for a relation field.
            let scalar = self.to_sql_value(&selector.field, &value)?;
            filters.push(selector.operand()?.compare(compare, Expr::value(scalar)));
        }
        Ok((!filters.is_empty()).then(|| Expr::any(filters)))
    }
}

/// Required forward relation resolved from source metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Edge {
    from: String,
    alias: String,
    table: &'static str,
    local: &'static str,
}
/// Always obtain Profile from its public owner; API owns only its two field catalogs.
pub(super) fn model_columns(model: &str) -> Vec<Column> {
    match model {
        "Application" => Schema::<super::ApplicationModel>::new().columns.clone(),
        "Session" => Schema::<super::SessionModel>::new().columns.clone(),
        _ => ProfileSchema::default().columns.clone(),
    }
}

/// Source truthiness applies before interpolation; empty containers remain truthy.
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64() != Some(0.0),
        Value::String(value) => !value.is_empty(),
        _ => true,
    }
}
/// JS query interpolation/computed-property keys differ from the column JSON codec.
/// Arrays join, null entries are empty and objects use their ordinary object spelling.
pub(crate) fn source_text(value: &Value, offset: i32) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Object(_) => "[object Object]".into(),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                if value.is_null() {
                    String::new()
                } else {
                    source_text(value, offset)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Date { millis, display } => display.clone().unwrap_or_else(|| {
            stackpress_framework_session::codecs::date_string(
                *millis,
                offset,
                "Coordinated Universal Time",
            )
            .unwrap_or_else(|_| "Invalid Date".into())
        }),
        Value::Number(value) => value
            .as_f64()
            .map(number_text)
            .unwrap_or_else(|| value.to_string()),
        _ => value.to_string(),
    }
}
/// Preserve JS's zero and exponent spelling after its one-number-family conversion.
fn number_text(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }
    // JS uses plain decimal for [1e-6,1e21) and an explicit positive exponent otherwise.
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        let scientific = format!("{value:e}");
        if let Some((mantissa, exponent)) = scientific.split_once('e')
            && !exponent.starts_with('-')
        {
            return format!("{mantissa}e+{exponent}");
        }
        return scientific;
    }
    value.to_string()
}
