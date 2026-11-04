//! Source selector/path traversal and owning joins from the immutable generated catalog.
use super::membership::{flatten, path};
use super::{NativeStore, SourceResult as Result, StoreQuery};
use stackpress_framework_schema::{
    Field, ValueType,
    source::{Record, SourceSchema, Value},
};
use stackpress_sql::{Compare, Expr, Identifier, Join, JoinKind, TableRef};
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
    pub model: String,
}
impl StoreSelector {
    /// Checked qualified SQL column.
    pub(super) fn identifier(&self) -> Identifier {
        Identifier::qualified([self.table.clone(), self.column.clone()])
    }
    /// Extract JSON only for predicates/sort; source selection still reads the whole column.
    pub(super) fn operand(&self) -> Result<Expr> {
        let column = self.identifier();
        if self.json.is_empty() {
            return Ok(Expr::column(column));
        }
        Ok(Expr::json(column, path(&self.json)?))
    }
}

impl<S: SourceSchema<Value = Value, Record = Record>> NativeStore<S> {
    /// Resolve one source expression, ignoring unknown paths as the generated store does.
    /// Profile has no reverse store relations even though Idea declares reverse metadata.
    /// A valid relation followed by an unknown child retains its last relation selector;
    /// the database reports that non-column path rather than silently dropping the filter.
    pub fn paths(&self, expression: &str) -> Vec<StoreSelector> {
        self.selectors(&[expression.into()])
    }

    /// Resolve all forward stored relations, including Session.application.profile.
    /// Reverse Idea arrays have no generated @relation and therefore no store join.
    fn resolve(&self, expression: &str) -> (Vec<Field>, String, String, Vec<Edge>) {
        let (field, _) = expression.split_once(':').unwrap_or((expression, ""));
        let mut model = &self.model;
        let mut table = self.table().to_owned();
        let mut edges = Vec::new();
        let mut last = None;
        for part in field.split('.') {
            let stored: Vec<_> = model
                .fields
                .iter()
                .filter(|f| !matches!(f.value_type, ValueType::Model(_)))
                .cloned()
                .collect();
            if part == "*" {
                return (stored, model.name.clone(), table, edges);
            }
            if let Some(column) = stored.iter().find(|f| f.name == part) {
                return (vec![column.clone()], model.name.clone(), table, edges);
            }
            let Some(relation_field) = model
                .fields
                .iter()
                .find(|f| f.name == part && f.relation.is_some())
            else {
                break;
            };
            let ValueType::Model(target) = &relation_field.value_type else {
                break;
            };
            let Some(next) = self.catalog.models.iter().find(|m| &m.name == target) else {
                break;
            };
            let relation = relation_field
                .relation
                .as_ref()
                .expect("checked owning relation");
            let local = model
                .fields
                .iter()
                .find(|f| f.name == relation.local)
                .expect("checked local key");
            let foreign = next
                .fields
                .iter()
                .find(|f| f.name == relation.foreign)
                .expect("checked foreign key");
            last = Some((
                vec![relation_field.clone()],
                model.name.clone(),
                table.clone(),
            ));
            let target = self.table_of(next);
            let alias = format!("{table}__{target}");
            edges.push(Edge {
                from: table,
                alias: alias.clone(),
                table: target.to_owned(),
                local: local.column.clone(),
                foreign: foreign.column.clone(),
                required: relation_field.required,
            });
            table = alias;
            model = next;
        }
        // The original keeps a terminal/unknown-child relation selector; SQL may reject its virtual column.
        let (fields, model, table) =
            last.unwrap_or_else(|| (Vec::new(), model.name.clone(), table));
        (fields, model, table, edges)
    }
    /// Expand source wildcards and de-duplicate expressions in supplied order.
    pub fn selectors(&self, expressions: &[String]) -> Vec<StoreSelector> {
        let mut selectors: Vec<StoreSelector> = Vec::new();
        for expression in expressions {
            let (_, json) = expression.split_once(':').unwrap_or((expression, ""));
            let (columns, model, table, _) = self.resolve(expression);
            for column in columns {
                let expanded = expression.replacen('*', &column.name, 1);
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
                    .map(|part| snake(part))
                    .collect::<Vec<_>>()
                    .join("__");
                selectors.push(StoreSelector {
                    expression: expanded,
                    alias,
                    field: column.name,
                    column: column.column.clone(),
                    table: table.clone(),
                    path: output_path,
                    json: json
                        .split('.')
                        .filter(|part| !part.is_empty())
                        .map(str::to_owned)
                        .collect(),
                    model: model.clone(),
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
                kind: if edge.required {
                    JoinKind::Inner
                } else {
                    JoinKind::Left
                },
                table: TableRef::new(edge.table).alias(edge.alias.clone()),
                on: Some(
                    Expr::column(Identifier::qualified([edge.from, edge.local])).compare(
                        Compare::Eq,
                        Expr::column(Identifier::qualified([edge.alias, edge.foreign])),
                    ),
                ),
            })
            .collect()
    }
}
/// One checked owning edge, resolved from the generated catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Edge {
    from: String,
    alias: String,
    table: String,
    local: String,
    foreign: String,
    required: bool,
}
/// Match source snake-case output aliases without altering stored graph identities.
fn snake(value: &str) -> String {
    let mut out = String::new();
    for (i, c) in value.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_')
        }
        out.extend(c.to_lowercase());
    }
    out
}
