//! Bounded default field selection from an already recursively projected finite include view.
use crate::{DataError, DataErrorKind, DataResult, codec_error};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
/// A finite selection trie; these paths are compiler-owned and never become SQL fragments.
#[derive(Default)]
struct Paths {
    /// A selected whole node retains all approved base fields alongside explicit nested relations.
    full: bool,
    /// Finite requested child paths.
    children: BTreeMap<String, Paths>,
}
/// Preserve the distinction between generated view wrappers and arbitrary user fields named row/state.
#[derive(Clone, Copy)]
enum Node<'a> {
    /// Generated eager wrapper, whose row/edge members have fixed protocol meaning.
    View(&'a Value),
    /// Approved user data, where row/state/value may be ordinary field names.
    Field(&'a Value),
}
/// Invalid/missing projection state must never be mistaken for an absent optional relation.
fn invalid() -> DataError {
    DataError::new("default selection does not match its loaded public view")
        .with_kind(DataErrorKind::Decode)
}
/// Select compiled approved paths, or the base public row when the model declares no default selection.
/// No SQL or lazy load occurs; an unloaded selected relation is an explicit error.
pub fn select<T: Serialize>(view: &T, paths: &[&str], base_fallback: bool) -> DataResult<Value> {
    // Bound the already public eager view before constructing a JSON selection tree.
    let mut buffer = crate::RecordBuffer {
        limit: 16 * 1024 * 1024,
        ..Default::default()
    };
    serde_json::to_writer(&mut buffer, view).map_err(codec_error)?;
    let value: Value = serde_json::from_slice(&buffer.bytes).map_err(codec_error)?;
    if paths.is_empty() && base_fallback {
        return value.get("row").cloned().ok_or_else(invalid);
    }
    // A whole-node flag prevents a redundant narrower path from discarding approved siblings.
    let mut root = Paths::default();
    for path in paths {
        if path.len() > 1024 || path.split('.').count() > 32 {
            return Err(invalid());
        }
        let mut cursor = &mut root;
        for part in path.split('.') {
            if part.is_empty() {
                return Err(invalid());
            }
            cursor = cursor.children.entry(part.into()).or_default();
        }
        cursor.full = true;
    }
    if paths.is_empty() {
        return Ok(serde_json::json!({}));
    }
    project(Node::View(&value), &root, 0)
}
/// Unwrap exactly one generated eager-relation state; user data objects never enter this path.
fn loaded(value: &Value) -> DataResult<&Value> {
    if value.get("state").and_then(Value::as_str) != Some("loaded") {
        return Err(invalid());
    }
    value.get("value").ok_or_else(invalid)
}
/// Flatten a selected full view using only its projected row and explicitly loaded child edges.
fn full(node: Node<'_>, depth: usize) -> DataResult<Value> {
    if depth > 40 {
        return Err(invalid());
    }
    let value = match node {
        Node::Field(value) => return Ok(value.clone()),
        Node::View(value) => value,
    };
    if value.is_null() {
        return Ok(Value::Null);
    }
    if let Some(values) = value.as_array() {
        return values
            .iter()
            .map(|value| full(Node::View(value), depth + 1))
            .collect::<DataResult<Vec<_>>>()
            .map(Value::Array);
    }
    let mut row = value
        .get("row")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(invalid)?;
    for (name, relation) in value
        .as_object()
        .ok_or_else(invalid)?
        .iter()
        .filter(|(name, _)| name.as_str() != "row")
    {
        // Unselected child edges are omitted, rather than fabricated as empty/None values.
        if relation.get("state").and_then(Value::as_str) == Some("loaded") {
            // Convert the reserved internal edge spelling back to its logical source field.
            let name = crate::stores::source_relation_member(name);
            row.insert(
                name.to_owned(),
                full(Node::View(loaded(relation)?), depth + 1)?,
            );
        }
    }
    Ok(Value::Object(row))
}
/// Apply one finite trie to records/arrays, keeping loaded optional None and list cardinality intact.
fn project(node: Node<'_>, paths: &Paths, depth: usize) -> DataResult<Value> {
    if depth > 40 {
        return Err(invalid());
    }
    // Whole user fields are copied as data; only generated views interpret eager edge wrappers.
    if paths.full && (paths.children.is_empty() || matches!(node, Node::Field(_))) {
        return full(node, depth);
    }
    let value = match node {
        Node::View(value) | Node::Field(value) => value,
    };
    if value.is_null() {
        return Ok(Value::Null);
    }
    if let Some(values) = value.as_array() {
        return values
            .iter()
            .map(|value| {
                project(
                    match node {
                        Node::View(_) => Node::View(value),
                        Node::Field(_) => Node::Field(value),
                    },
                    paths,
                    depth + 1,
                )
            })
            .collect::<DataResult<Vec<_>>>()
            .map(Value::Array);
    }
    let mut output = if paths.full {
        value
            .get("row")
            .and_then(Value::as_object)
            .cloned()
            .ok_or_else(invalid)?
    } else {
        serde_json::Map::new()
    };
    // Resolve base fields before relation wrappers, retaining nullable/list cardinality recursively.
    for (name, nested) in &paths.children {
        if paths.full && output.contains_key(name) {
            continue;
        }
        let selected = match node {
            Node::Field(value) => Node::Field(value.get(name).ok_or_else(invalid)?),
            Node::View(value) => {
                if let Some(field) = value.get("row").and_then(|row| row.get(name)) {
                    Node::Field(field)
                } else {
                    Node::View(loaded(
                        value
                            .get(crate::stores::relation_member(name))
                            .ok_or_else(invalid)?,
                    )?)
                }
            }
        };
        output.insert(name.clone(), project(selected, nested, depth + 1)?);
    }
    Ok(Value::Object(output))
}
