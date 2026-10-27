//! Snapshot reads and finite breadth-first include loading through one borrowed executor.
use super::query::ordered;
use super::{
    Exposure, Page, ReadQuery, RelationSpec, StoreSpec, Visibility, count, invalid, limit, select,
};
use crate::{DataError, DataErrorKind, DataResult, DataSession, codec_error, encode_value};
use serde_json::Value;
use stackpress_sql::{DialectId, Expr, Order, QueryLimits, Render, Select, Sort, SqlValue};
use std::collections::{BTreeMap, VecDeque};

/// Lowerable aggregate include budgets. Executor per-statement limits always remain authoritative.
#[derive(Clone, Copy, Debug)]
pub struct RelationLimits {
    /// Maximum requested relation paths, at most sixteen.
    pub paths: usize,
    /// Maximum explicit relation depth, at most three.
    pub depth: usize,
    /// Total related rows across every requested path.
    pub rows: usize,
    /// Logical encoded bytes of the complete result.
    pub bytes: usize,
    /// Total statements including base page and optional count.
    pub statements: usize,
    /// Deduplicated lookup keys across requested path levels.
    pub keys: usize,
}
impl Default for RelationLimits {
    /// Finite initial contract; callers may only lower these limits.
    fn default() -> Self {
        Self {
            paths: 16,
            depth: 3,
            rows: 10000,
            bytes: 16 * 1024 * 1024,
            statements: 128,
            keys: 10000,
        }
    }
}
/// A finite parsed include trie, built in full before the first database effect.
#[derive(Clone, Default)]
struct Paths(BTreeMap<String, Paths>);
/// Resolve an immutable catalog table, never a request-chosen database identifier.
fn target<'a>(catalog: &'a [&StoreSpec], table: &str) -> DataResult<&'a StoreSpec> {
    catalog
        .iter()
        .copied()
        .find(|spec| spec.table == table)
        .ok_or_else(invalid)
}
/// Validate each relation segment against its actual model and exposure, merging shared prefixes.
fn paths(
    catalog: &[&StoreSpec],
    spec: &StoreSpec,
    query: &ReadQuery,
    exposure: Exposure,
) -> DataResult<Paths> {
    let mut root = Paths::default();
    for path in &query.includes {
        let mut spec = spec;
        let mut cursor = &mut root;
        for part in path.split('.') {
            let relation = spec
                .relations
                .iter()
                .find(|relation| {
                    relation.name == part && (exposure == Exposure::Trusted || relation.public)
                })
                .ok_or_else(invalid)?;
            cursor = cursor.0.entry(part.into()).or_default();
            spec = target(catalog, relation.target)?;
        }
    }
    Ok(root)
}
/// Internal acyclic arena: children are appended after parents, even for repeated self-relations.
struct Node<'a> {
    /// Validated stored fields.
    row: Value,
    /// Exact catalog owner.
    spec: &'a StoreSpec,
    /// Requested loaded edges; unrequested fields stay absent.
    edges: BTreeMap<String, Vec<usize>>,
}
/// One batched level/path job across every parent of that model at this path.
struct Job {
    /// Arena parent indexes.
    parents: Vec<usize>,
    /// Shared remaining path trie.
    paths: Paths,
}
/// Produce a stable typed-scalar grouping key; null is handled as absence before this call.
fn identity(value: &Value) -> DataResult<String> {
    if !matches!(value, Value::String(_) | Value::Number(_) | Value::Bool(_)) {
        return Err(invalid());
    }
    serde_json::to_string(value).map_err(codec_error)
}
/// Construct one finite relation query with exact predicate overhead and complete key ordering.
fn relation_select(
    spec: &StoreSpec,
    relation: &RelationSpec,
    values: Vec<SqlValue>,
    remaining: usize,
    dialect: DialectId,
) -> DataResult<Select> {
    let mut query = spec
        .visible(spec.base(), Visibility::Active)?
        .and_where(Expr::In(
            Box::new(Expr::column(spec.column(relation.foreign)?.column)),
            values,
            false,
        ));
    for key in spec.key {
        let field = spec
            .fields
            .iter()
            .find(|field| field.storage.wire == *key)
            .ok_or_else(invalid)?;
        query = query.sort(Sort {
            expression: ordered(field, dialect)?,
            order: Order::Asc,
        });
    }
    // One sentinel row detects an aggregate overflow; publishing a truncated list is forbidden.
    Ok(query.limit(remaining.saturating_add(1) as u64))
}
/// Partition before IO using the actual executor's bind/byte/SQL ceilings, without retrying queries.
fn chunks(
    spec: &StoreSpec,
    relation: &RelationSpec,
    values: Vec<SqlValue>,
    limits: QueryLimits,
    dialect: DialectId,
    remaining: usize,
) -> DataResult<Vec<Select>> {
    // Every active destination spends one bind before the first relation key.
    let overhead = usize::from(spec.active.is_some());
    let capacity = limits.binds.min(1000).saturating_sub(overhead);
    if capacity == 0 && !values.is_empty() {
        return Err(limit());
    }
    let mut queries = Vec::new();
    let mut current = Vec::new();
    // Greedily form finite candidate plans; rendering is pure and never probes the database.
    for value in values {
        let mut candidate = current.clone();
        candidate.push(value.clone());
        let query = relation_select(spec, relation, candidate.clone(), remaining, dialect)?;
        let fits = candidate.len() <= capacity
            && query
                .render(dialect)
                .and_then(|query| query.validate(limits))
                .is_ok();
        // Finish the previous validated chunk before checking this key on its own.
        if !fits {
            if current.is_empty() {
                return Err(limit());
            }
            queries.push(relation_select(
                spec,
                relation,
                std::mem::take(&mut current),
                remaining,
                dialect,
            )?);
            // A single key must fit independently; no query is submitted as a budget probe.
            relation_select(spec, relation, vec![value.clone()], remaining, dialect)?
                .render(dialect)?
                .validate(limits)?;
        }
        current.push(value);
    }
    if !current.is_empty() {
        queries.push(relation_select(
            spec, relation, current, remaining, dialect,
        )?);
    }
    Ok(queries)
}
/// Read one complete bounded base page and requested relations through one exact executor.
pub async fn search(
    db: &mut DataSession<'_>,
    catalog: &[&StoreSpec],
    spec: &StoreSpec,
    query: ReadQuery,
    exposure: Exposure,
    visibility: Visibility,
    limits: RelationLimits,
) -> DataResult<Page<Value>> {
    // Caller limits may lower the contract, never silently expand a global ceiling.
    let defaults = RelationLimits::default();
    if limits.paths > defaults.paths
        || limits.depth > defaults.depth
        || limits.rows > defaults.rows
        || limits.bytes > defaults.bytes
        || limits.statements > defaults.statements
        || limits.keys > defaults.keys
    {
        return Err(invalid());
    }
    if query.includes.len() > limits.paths
        || query
            .includes
            .iter()
            .any(|path| path.split('.').count() > limits.depth)
    {
        return Err(limit());
    }
    // Compile all requested paths and the base page before the first database call.
    let select = select(spec, &query, db.dialect(), exposure, visibility)?;
    let paths = paths(catalog, spec, &query, exposure)?;
    // One transaction alone is not a repeatable snapshot on every backend.
    if query.total || !paths.0.is_empty() {
        db.require_transaction()?;
        if db.dialect() != DialectId::Sqlite {
            return Err(DataError::new(
                "consistent multi-statement reads require the verified SQLite snapshot",
            )
            .with_kind(DataErrorKind::Unsupported));
        }
    }
    let mut statements = 1 + usize::from(query.total);
    if statements > limits.statements {
        return Err(limit());
    }
    // Count establishes the same caller-owned SQLite snapshot subsequently used by rows/includes.
    let total = if query.total {
        Some(count(db, spec, &query, exposure, visibility).await?)
    } else {
        None
    };
    let rows = db.fetch(&select).await?;
    let mut nodes = Vec::new();
    let mut bytes = 0usize;
    // Validate stored records before retaining them in the finite arena.
    for row in rows {
        let row = (spec.decode)(row)?;
        bytes = bytes.saturating_add(serde_json::to_vec(&row).map_err(codec_error)?.len());
        if bytes > limits.bytes {
            return Err(limit());
        }
        nodes.push(Node {
            row,
            spec,
            edges: BTreeMap::new(),
        });
    }
    let base_len = nodes.len();
    let mut jobs = VecDeque::from([Job {
        parents: (0..base_len).collect(),
        paths,
    }]);
    let mut related = 0usize;
    let mut keys_used = 0usize;
    // A job spans every parent at this path: parent count changes binds, not one query per row.
    while let Some(job) = jobs.pop_front() {
        let Some(first) = job.parents.first() else {
            continue;
        };
        let source = nodes[*first].spec;
        for (name, nested) in job.paths.0 {
            let relation = source
                .relations
                .iter()
                .find(|relation| relation.name == name)
                .ok_or_else(invalid)?;
            let destination = target(catalog, relation.target)?;
            // Group by canonical scalar identity and skip nullable absent foreign keys.
            let mut keys = BTreeMap::new();
            for parent in &job.parents {
                let value = nodes[*parent].row.get(relation.local).ok_or_else(invalid)?;
                if !value.is_null() {
                    keys.entry(identity(value)?)
                        .or_insert(encode_value(value, destination.column(relation.foreign)?)?);
                }
            }
            // Deduplicated keys consume one shared budget across every requested path.
            keys_used = keys_used.saturating_add(keys.len());
            if keys_used > limits.keys {
                return Err(limit());
            }
            let queries = chunks(
                destination,
                relation,
                keys.into_values().collect(),
                db.limits(),
                db.dialect(),
                limits.rows.saturating_sub(related),
            )?;
            let mut groups: BTreeMap<String, Vec<usize>> = BTreeMap::new();
            let mut children = Vec::new();
            // Statement/row/byte ceilings accumulate across chunks rather than resetting per query.
            for query in queries {
                statements = statements.saturating_add(1);
                if statements > limits.statements {
                    return Err(limit());
                }
                for row in db.fetch(&query).await? {
                    related = related.saturating_add(1);
                    if related > limits.rows {
                        return Err(limit());
                    }
                    let row = (destination.decode)(row)?;
                    bytes =
                        bytes.saturating_add(serde_json::to_vec(&row).map_err(codec_error)?.len());
                    if bytes > limits.bytes {
                        return Err(limit());
                    }
                    let key = identity(row.get(relation.foreign).ok_or_else(invalid)?)?;
                    let index = nodes.len();
                    groups.entry(key).or_default().push(index);
                    children.push(index);
                    nodes.push(Node {
                        row,
                        spec: destination,
                        edges: BTreeMap::new(),
                    });
                }
            }
            // Attach complete groups to the original parents without disturbing base/child order.
            for parent in &job.parents {
                let value = nodes[*parent].row.get(relation.local).ok_or_else(invalid)?;
                let children = if value.is_null() {
                    Vec::new()
                } else {
                    groups.get(&identity(value)?).cloned().unwrap_or_default()
                };
                if !relation.many
                    && (children.len() > 1 || relation.required && children.is_empty())
                {
                    return Err(
                        DataError::new("required or unique relation integrity failed")
                            .with_kind(DataErrorKind::Decode),
                    );
                }
                nodes[*parent].edges.insert(name.clone(), children);
            }
            // One job per path level batches all parents, rather than recursing/fetching per row.
            if !nested.0.is_empty() {
                jobs.push_back(Job {
                    parents: children,
                    paths: nested,
                });
            }
        }
    }
    // Reverse arena order is topological even for repeated bounded self-model paths.
    let mut views: BTreeMap<usize, (Value, usize)> = BTreeMap::new();
    let mut cached_bytes = 0usize;
    for (index, node) in nodes.into_iter().enumerate().rev() {
        // Children already exist in the cache; bound the expanded parent before cloning them.
        let mut object = serde_json::Map::new();
        let mut size = serde_json::to_vec(&node.row).map_err(codec_error)?.len();
        object.insert("row".into(), node.row);
        for (name, children) in node.edges {
            let relation = node
                .spec
                .relations
                .iter()
                .find(|relation| relation.name == name)
                .ok_or_else(invalid)?;
            for child in &children {
                size = size.saturating_add(views.get(child).ok_or_else(invalid)?.1);
            }
            if size > limits.bytes {
                return Err(limit());
            }
            let values = children
                .iter()
                .map(|child| {
                    views
                        .get(child)
                        .map(|(value, _)| value.clone())
                        .ok_or_else(invalid)
                })
                .collect::<DataResult<Vec<_>>>()?;
            let value = if relation.many {
                Value::Array(values)
            } else {
                values.into_iter().next().unwrap_or(Value::Null)
            };
            object.insert(
                super::relation_member(&name).to_owned(),
                serde_json::json!({"state":"loaded","value":value}),
            );
        }
        let value = Value::Object(object);
        let size = serde_json::to_vec(&value).map_err(codec_error)?.len();
        cached_bytes = cached_bytes.saturating_add(size);
        if cached_bytes > limits.bytes.saturating_mul(4) {
            return Err(limit());
        }
        views.insert(index, (value, size));
    }
    let mut rows = Vec::new();
    let mut final_bytes = 0usize;
    // Publish only original base rows, checking aggregate expanded bytes once more.
    for index in 0..base_len {
        let (row, size) = views.remove(&index).ok_or_else(invalid)?;
        final_bytes = final_bytes.saturating_add(size);
        if final_bytes > limits.bytes {
            return Err(limit());
        }
        rows.push(row);
    }
    Ok(Page {
        rows,
        total,
        skip: query.skip,
        take: query.take,
    })
}
