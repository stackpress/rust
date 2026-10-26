//! Read-only edit planning, incremental budgets and an infallible commit phase.

use super::nest::{canonical_index, make_object};
use super::value::Usage;
use super::{DataMap, DataPath, ParseLimits, PathSegment, Value};
use crate::{Error, Result};

/// Resolved keys and final container shapes; no append arithmetic remains at commit.
pub(super) struct Step {
    /// Literal key selected against the original immutable tree.
    key: String,
    /// Canonical numeric keys can edit dense vectors without materializing a map.
    index: Option<usize>,
    /// The planner proved this container's final keys form a dense numeric array.
    array: bool,
}

/// Original container shape, before any descendant or membership change is committed.
#[derive(Clone, Copy)]
enum ContainerBefore {
    /// Setting below a missing parent reserves a new object before possible normalization.
    Missing,
    /// Existing literal keys already contribute their UTF-8 bytes to usage.
    Object,
    /// Existing dense indices contribute no key bytes until an object conversion.
    Array,
}

/// One key's membership outcome; replacement and descendant edits leave membership unchanged.
#[derive(Clone, Copy)]
enum MembershipChange {
    /// Replacement or descendant mutation retains this key's membership.
    Unchanged,
    /// A new child contributes exactly one new member.
    Added,
    /// Removing an existing leaf subtracts exactly one member.
    Removed,
}

/// Independent added/removed totals allow representation changes to cancel exactly.
#[derive(Default)]
pub(super) struct Change {
    added: Usage,
    removed: Usage,
}
impl Change {
    /// Reserve incoming payload/containers/keys without touching the current tree.
    pub(super) fn add(&mut self, usage: Usage) -> Result<()> {
        // Accumulate prospective payload and container costs before committing any edit.
        self.added.nodes = checked_add(self.added.nodes, usage.nodes)?;
        self.added.bytes = checked_add(self.added.bytes, usage.bytes)?;
        Ok(())
    }
    /// Account for replaced payload or keys removed by array normalization.
    pub(super) fn remove(&mut self, usage: Usage) -> Result<()> {
        // Record replaced costs separately so representation changes can cancel newly introduced
        // keys.
        self.removed.nodes = checked_add(self.removed.nodes, usage.nodes)?;
        self.removed.bytes = checked_add(self.removed.bytes, usage.bytes)?;
        Ok(())
    }
    /// Check only the net delta; newly added keys can disappear during normalization.
    pub(super) fn project(&self, current: Usage, limits: &ParseLimits) -> Result<Usage> {
        // Apply the net change and enforce the final budget without rescanning unrelated siblings.
        let usage = Usage {
            nodes: project(current.nodes, self.added.nodes, self.removed.nodes)?,
            bytes: project(current.bytes, self.added.bytes, self.removed.bytes)?,
        };
        // Check the final aggregate, including newly created containers and sparse index-key bytes.
        if usage.nodes > limits.max_entries || usage.bytes > limits.max_bytes {
            return Err(Error::limit("nested data byte/entry limit exceeded"));
        }
        Ok(usage)
    }
}

/// Reject bookkeeping overflow while the original tree is still untouched.
fn checked_add(left: usize, right: usize) -> Result<usize> {
    // Overflow is a preflight failure; it must never wrap into an apparently small allowed usage.
    left.checked_add(right)
        .ok_or_else(|| Error::limit("value usage overflow"))
}
/// Apply a net change without an artificial intermediate overflow/underflow.
fn project(current: usize, added: usize, removed: usize) -> Result<usize> {
    // Choose the direction of the net delta before arithmetic, avoiding artificial intermediate
    // overflow.

    // Newly created then normalized keys cancel before comparing against current totals.
    if added >= removed {
        current.checked_add(added - removed)
    } else {
        current.checked_sub(removed - added)
    }
    .ok_or_else(|| Error::limit("value usage arithmetic overflow"))
}

/// Plan at most one bounded path; untouched siblings are neither copied nor measured.
pub(super) fn prepare(
    root: &Value,
    path: &DataPath,
    setting: bool,
    incoming: Usage,
    limits: &ParseLimits,
) -> Result<(Vec<Step>, Change)> {
    // Keep the plan and its resource delta together for one immutable inspection pass.
    let mut preparation = Preparation {
        setting,
        limits,
        steps: Vec::with_capacity(path.0.len()),
        change: Change::default(),
    };
    if setting {
        preparation.change.add(incoming)?;
    }
    if !preparation.inspect(Some(root), &path.0, true)? {
        // Missing removal cannot alter a container or its cached totals.
        preparation.steps.clear();
    }
    Ok((preparation.steps, preparation.change))
}

/// Own only the path-local plan and budget delta while borrowing immutable limits.
struct Preparation<'a> {
    setting: bool,
    limits: &'a ParseLimits,
    steps: Vec<Step>,
    change: Change,
}
impl Preparation<'_> {
    /// Inspect the selected branch before deciding each ancestor's final shape.
    fn inspect(
        &mut self,
        container: Option<&Value>,
        path: &[PathSegment],
        root: bool,
    ) -> Result<bool> {
        // Resolve one selected child, recurse before mutation, then decide this container's final
        // shape.
        let (before, object, array) = match container {
            // Objects and arrays are inspected through immutable borrows; no payload is copied.
            Some(Value::Object(values)) => (ContainerBefore::Object, Some(values), None),
            Some(Value::Array(values)) => (ContainerBefore::Array, None, Some(values)),
            None => {
                // Only self.setting reaches a missing intermediate; it creates one container.
                self.change.add(Usage { nodes: 1, bytes: 0 })?;
                (ContainerBefore::Missing, None, None)
            }
            _ => return Err(Error::invalid("cannot traverse scalar value")),
        };
        let key = resolve(&path[0], object, array)?;
        let index = canonical_index(&key);
        // Read only the chosen child; unrelated sibling payloads do not participate in planning.
        let child = object
            .and_then(|values| values.get(&key))
            .or_else(|| array.and_then(|values| index.and_then(|index| values.get(index))));
        // Reserve this ancestor slot before descending, then fill its final shape on unwind.
        let position = self.steps.len();
        self.steps.push(Step {
            key,
            index,
            array: false,
        });
        let last = path.len() == 1;
        // Only the leaf replaces/removes payload; intermediates retain their existing node
        // identity.
        if last {
            // Measure only the replaced subtree, whose nodes/bytes must be subtracted exactly once.
            if let Some(old) = child {
                self.change.remove(old.measure(self.limits)?)?;
            } else if !self.setting {
                return Ok(false);
            }
        } else {
            // A missing intermediate makes removal a no-op; it must not create empty containers.
            if child.is_none() && !self.setting {
                return Ok(false);
            }
            if !self.inspect(child, &path[1..], false)? {
                return Ok(false);
            }
        }
        let step = &mut self.steps[position];
        let length = object.map_or_else(|| array.map_or(0, Vec::len), DataMap::len);
        let membership = if self.setting && child.is_none() {
            MembershipChange::Added
        } else if !self.setting && last {
            MembershipChange::Removed
        } else {
            MembershipChange::Unchanged
        };
        // Count key membership changes independently of payload replacement.
        let final_length = match membership {
            MembershipChange::Added => checked_add(length, 1)?,
            // A missing leaf returned above, so removal cannot underflow.
            MembershipChange::Removed => length - 1,
            MembershipChange::Unchanged => length,
        };
        // Root remains an object. Deleting an array element deliberately preserves holes.
        step.array = !root
            && (self.setting || (array.is_some() && !last))
            && final_length > 0
            && if array.is_some() {
                !matches!(membership, MembershipChange::Added) || index == Some(length)
            } else {
                // A map becomes an array only when every final canonical numeric key exists.
                (0..final_length).all(|index| {
                    let key = index.to_string();
                    if key == step.key {
                        !matches!(membership, MembershipChange::Removed)
                    } else {
                        object.is_some_and(|values| values.contains_key(&key))
                    }
                })
            };
        // Account representation changes after the recursive payload delta is known.
        account_keys(before, membership, final_length, step, &mut self.change)?;
        Ok(true)
    }
}

/// Resolve append once, rejecting maximum-index overflow before any ancestor changes.
fn resolve(
    segment: &PathSegment,
    object: Option<&DataMap<String, Value>>,
    array: Option<&Vec<Value>>,
) -> Result<String> {
    // Existing numeric identity determines append; sparse key count cannot choose the next index.
    Ok(match segment {
        PathSegment::Key(key) => key.clone(),
        PathSegment::Index(index) => index.to_string(),
        PathSegment::Append => {
            let index = if let Some(values) = array {
                values.len()
            } else {
                object
                    .and_then(|values| values.keys().filter_map(|key| canonical_index(key)).max())
                    .map_or(Ok(0), |index| {
                        index
                            .checked_add(1)
                            .ok_or_else(|| Error::limit("append index overflow"))
                    })?
            };
            index.to_string()
        }
    })
}

/// Adjust key bytes when inserting/removing a map key or changing container form.
fn account_keys(
    before: ContainerBefore,
    membership: MembershipChange,
    length: usize,
    step: &Step,
    change: &mut Change,
) -> Result<()> {
    // Object keys consume bytes; vectors do not, so shape changes need an explicit key adjustment.

    // Map and newly created containers initially charge literal key bytes.
    match before {
        ContainerBefore::Missing | ContainerBefore::Object => {
            account_membership(membership, &step.key, change)?;
            // Only a planner-proven dense object may be materialized as a vector.
            if step.array {
                change.remove(Usage {
                    nodes: 0,
                    bytes: numeric_key_bytes(length)?,
                })?;
            }
        }
        ContainerBefore::Array if !step.array => {
            // A retained dense vector has no key-byte cost; only sparse conversion adds indices.
            // An array-to-map transition materializes retained index keys exactly once.
            let old_length = match membership {
                MembershipChange::Added => length - 1,
                // Reconstruct the existing vector length, already representable as usize.
                MembershipChange::Removed => length + 1,
                MembershipChange::Unchanged => length,
            };
            let bytes = numeric_key_bytes(old_length)?;
            change.add(Usage { nodes: 0, bytes })?;
            account_membership(membership, &step.key, change)?;
        }
        // Retained arrays store no key bytes, including dense appends and replacements.
        ContainerBefore::Array => {}
    }
    Ok(())
}

/// Charge literal key membership once, independently of the container-transition adjustment.
fn account_membership(membership: MembershipChange, key: &str, change: &mut Change) -> Result<()> {
    let usage = Usage {
        nodes: 0,
        bytes: key.len(),
    };
    // Callers select this phase's position so checked arithmetic retains its existing precedence.
    match membership {
        MembershipChange::Added => change.add(usage),
        MembershipChange::Removed => change.remove(usage),
        MembershipChange::Unchanged => Ok(()),
    }
}

/// Sum decimal index lengths in logarithmic time, including the single digit zero.
fn numeric_key_bytes(length: usize) -> Result<usize> {
    // Every index has one digit; each decimal threshold contributes one more digit to its suffix.
    let mut bytes = length;
    let mut threshold = 10usize;
    while threshold < length {
        bytes = checked_add(bytes, length - threshold)?;
        let Some(next) = threshold.checked_mul(10) else {
            break;
        };
        threshold = next;
    }
    Ok(bytes)
}

/// Commit a validated plan without fallible work, callback execution or payload cloning.
pub(super) fn commit(container: &mut Value, steps: &[Step], value: Option<Value>) -> Option<Value> {
    // Only a plan produced from this unchanged tree reaches this non-fallible mutation phase.
    let step = &steps[0];
    let last = steps.len() == 1;
    let setting = value.is_some();
    // Use direct vector replacement/append when the checked index keeps the container dense.
    if let Value::Array(values) = container
        && let Some(index) = step.index
        && (index < values.len() || (index == values.len() && setting))
        && (!last || setting)
    {
        // Dense writes retain the vector allocation; append grows only that vector.
        if index == values.len() {
            values.push(Value::Object(DataMap::new()));
        }
        // Preflight proved this index exists, including the one just appended; no unchecked
        // reference escapes.
        let child = &mut values[index];
        return if last {
            value.map(|value| std::mem::replace(child, value))
        } else {
            commit(child, &steps[1..], value)
        };
    }
    if matches!(container, Value::Array(_)) {
        // Sparse insertion/deletion moves siblings without copying their payloads.
        if let Value::Array(values) = std::mem::replace(container, Value::Null) {
            *container = Value::Object(make_object(values));
        }
    }
    // Preflight rejected scalar traversal, and every fallback array was moved to object form.
    let Value::Object(values) = container else {
        unreachable!("read-only plan proved a container")
    };
    let result = if last {
        if let Some(value) = value {
            values.insert(step.key.clone(), value)
        } else {
            values.shift_remove(&step.key)
        }
    } else {
        let child = values
            // Only missing setting intermediates are created here; all error checks already
            // succeeded.
            .entry(step.key.clone())
            .or_insert_with(|| Value::Object(DataMap::new()));
        commit(child, &steps[1..], value)
    };
    if step.array {
        // The planner proved dense keys; swap removal avoids quadratic shifting.
        let mut array = Vec::with_capacity(values.len());
        for index in 0..values.len() {
            // Lookup by numeric identity retains numeric order while avoiding repeated map shifts.
            if let Some(value) = values.swap_remove(&index.to_string()) {
                array.push(value);
            }
        }
        *container = Value::Array(array);
    }
    result
}
