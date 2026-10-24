//! Object-root nested storage with atomic checked mutation and ordinary borrowing.
//!
//! Literal keys use `get/has/set/remove`; path operations use
//! `path/path_as/has_path/set_path/remove_path`. Every path operation accepts
//! a dot string or a reusable DataPath through the sealed IntoPath adapter.
//! Reads return None/false for malformed paths; typed reads and writes return errors.
//!
//! ```
//! use stackpress_lib::{DataPath, Nest, Result};
//! # fn example() -> Result<()> {
//! let mut data = Nest::new();
//! // A literal dotted key and a nested dot path address different entries.
//! data.set("literal.key", true)?
//!     .set_path("profile.name", "Ada")?;
//! let name = DataPath::dot("profile.name")?;
//! assert!(data.has("literal.key") && data.has_path(&name));
//! assert_eq!(data.path_as::<String>(&name)?, Some("Ada".into()));
//! data.remove("literal.key")?;
//! data.remove_path("profile.name")?;
//! #     Ok(())
//! # }
//! ```

use super::mutation::{Change, commit, prepare};
use super::value::Usage;
use super::{DataMap, DataPath, IntoPath, ParseLimits, PathSegment, Value};
use crate::{Error, Flow, Result};
use serde::{Serialize, Serializer, de::DeserializeOwned};
use std::fmt;
use std::future::Future;

/// Inputs deliberately convertible to an object-root store.
pub trait IntoNest {
    /// Validate an object root and its finite value budgets.
    ///
    /// # Errors
    /// Rejects non-object input or exceeded value limits.
    ///
    /// ```
    /// use stackpress_lib::data::IntoNest;
    /// let data = serde_json::json!({"id": 7}).into_nest()?;
    /// assert_eq!(data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    fn into_nest(self) -> Result<Nest>;
    /// Validate against a caller's deliberate construction budget.
    ///
    /// # Errors
    /// Rejects input incompatible with the selected policy.
    ///
    /// ```
    /// use stackpress_lib::{ParseLimits, data::IntoNest};
    /// let data = serde_json::json!({"id": 7}).into_nest_with(ParseLimits::default())?;
    /// assert!(data.has("id"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    fn into_nest_with(self, limits: ParseLimits) -> Result<Nest>
    where
        Self: Sized,
    {
        // Apply the final caller-selected budget instead of retaining a looser input policy.
        Nest::from_value(self.into_nest()?.into_value(), limits)
    }
}
impl IntoNest for Nest {
    /// Move an already checked store.
    fn into_nest(self) -> Result<Nest> {
        // Already checked owned storage can move without duplicating its payload.
        Ok(self)
    }
}
impl IntoNest for serde_json::Value {
    /// Check a supplied parsed JSON object.
    fn into_nest(self) -> Result<Nest> {
        // Parsed JSON still needs the object-root and finite-budget contract.
        Nest::from_object(self)
    }
    /// Apply chosen limits before constructing/copying parsed JSON nodes.
    fn into_nest_with(self, limits: ParseLimits) -> Result<Nest> {
        // Apply the final caller-selected budget instead of retaining a looser input policy.
        Nest::from_value(Value::from_json_with(self, &limits)?, limits)
    }
}
impl IntoNest for Value {
    /// Check an explicitly rich object.
    fn into_nest(self) -> Result<Nest> {
        // Rich input keeps its native variants while enforcing the same object-root contract.
        Nest::from_value(self, ParseLimits::default())
    }
    /// Validate rich values using the selected invocation limits.
    fn into_nest_with(self, limits: ParseLimits) -> Result<Nest> {
        // Apply the final caller-selected budget instead of retaining a looser input policy.
        Nest::from_value(self, limits)
    }
}

/// Result of visiting a nested collection, separate from application status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisitOutcome {
    /// Every entry was visited.
    Completed,
    /// A visitor cooperatively stopped iteration.
    Aborted,
    /// The selected collection was missing or empty.
    Empty,
}

/// Owned object-root data with deep ordinary Clone and explicit Shared identity.
///
/// Debug lists keys only. Missing and present null are different lookup results.
/// Mutations prepare a read-only plan and budget delta before committing in place.
/// Returned errors leave the store unchanged. User-defined Shared native cleanup
/// can panic after a successful commit; tree and cached totals are consistent
/// before that cleanup runs. The framework does not catch application panics.
#[derive(Clone)]
pub struct Nest {
    /// Object invariant is established at construction and preserved by every edit.
    root: Value,
    /// Budgets used for later writes, not just initial parsing.
    limits: ParseLimits,
    /// Totals are changed only after a completely validated in-place edit.
    usage: Usage,
}
impl Default for Nest {
    /// Construct an empty object under default finite limits.
    fn default() -> Self {
        // Use the same finite policy as an ordinary independently constructed store.
        Self::new()
    }
}
impl Nest {
    /// Construct an empty, independent object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// let empty = Nest::new();
    /// assert!(empty.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // No global or aliased storage is introduced by default construction.
        Self::with_limits(ParseLimits::default())
    }
    /// Construct an empty object with explicit workload budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// let data = Nest::with_limits(stackpress_lib::ParseLimits::default());
    /// assert!(data.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn with_limits(limits: ParseLimits) -> Self {
        // An empty root consumes one node; later edits retain these exact workload limits.
        Self {
            root: Value::Object(DataMap::new()),
            limits,
            usage: Usage { nodes: 1, bytes: 0 },
        }
    }
    /// Validate a supplied JSON object; scalar/array roots are rejected.
    ///
    /// # Errors
    /// Returns invalid root shape or resource-limit errors.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// let data = Nest::from_object(serde_json::json!({"id": 7}))?;
    /// assert_eq!(data.path_as::<u64>("id")?, Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_object(value: serde_json::Value) -> Result<Self> {
        // JSON conversion validates representation before establishing the object-root invariant.
        Self::from_value(Value::try_from(value)?, ParseLimits::default())
    }
    /// Validate an owned rich object and retain the selected mutation budgets.
    ///
    /// # Errors
    /// Rejects a non-object root or an oversized/deep tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// let value = Value::try_from(serde_json::json!({"id": 7}))?;
    /// let data = Nest::from_value(value, stackpress_lib::ParseLimits::default())?;
    /// assert!(data.has("id"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_value(value: Value, limits: ParseLimits) -> Result<Self> {
        // Reject an invalid root before caching totals that future edits trust.
        if !value.is_object() {
            return Err(Error::invalid("nested data requires an object root"));
        }
        let usage = value.measure(&limits)?;
        Ok(Self {
            root: value,
            limits,
            usage,
        })
    }
    /// Borrow the complete rich object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.value().is_object());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn value(&self) -> &Value {
        // Borrowing the root does not expose mutation that could stale the cached totals.
        &self.root
    }
    /// Move out the object without copying or erasing Shared identity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.into_value().is_object());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn into_value(self) -> Value {
        // Moving transfers all payloads and drops only the store-specific budget bookkeeping.
        self.root
    }
    /// Inspect the retained parser/mutation budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert_eq!(data.limits().max_depth, 64);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn limits(&self) -> ParseLimits {
        // Return the retained policy by value so callers cannot relax this store in place.
        self.limits
    }
    /// Inspect the top-level entry count.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert_eq!(data.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        // Count top-level fields independently of the cached recursive node count.
        self.entries().len()
    }
    /// Report an empty object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(!data.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        // An empty object root still exists even when it has no top-level fields.
        self.len() == 0
    }
    /// Borrow insertion-ordered literal-key entries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.entries().contains_key("profile"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn entries(&self) -> &DataMap<String, Value> {
        // Construction and edits always retain an object root.
        match &self.root {
            Value::Object(values) => values,
            _ => unreachable!("Nest object invariant"),
        }
    }
    /// Iterate literal keys in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert_eq!(data.keys().next().map(String::as_str), Some("profile"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        // Expose insertion order without copying key strings.
        self.entries().keys()
    }
    /// Iterate values in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.values().next().unwrap().is_object());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        // Borrow payloads so inspecting a native value does not add shared owners.
        self.entries().values()
    }
    /// Borrow a literal top-level key; punctuation is not parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.get("profile.name").is_none());
    /// assert!(data.get("profile").is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn get(&self, key: &str) -> Option<&Value> {
        // Literal punctuation belongs to the key; only the path APIs parse separators.
        self.entries().get(key)
    }
    /// Inspect presence, including a present null.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// data.set("literal.key", Value::Null)?;
    /// assert!(data.has("literal.key"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn has(&self, key: &str) -> bool {
        // A present null must report presence rather than false truthiness.
        self.get(key).is_some()
    }
    /// Borrow a dot string or parsed path; malformed/missing paths return None.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// let path = DataPath::dot("profile.name")?;
    /// assert_eq!(data.path(&path).and_then(Value::as_str), Some("Ada"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn path<'p>(&self, path: impl IntoPath<'p>) -> Option<&Value> {
        // Optional reads collapse malformed/missing paths; typed reads preserve parse failures.
        let path = path.into_path().ok()?;
        self.lookup(&path)
    }
    /// Check path presence, including a present null; malformed paths return false.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.has_path("profile.name"));
    /// assert!(!data.has_path("profile.missing"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn has_path<'p>(&self, path: impl IntoPath<'p>) -> bool {
        // Use the same checked grammar and missing/null distinction as path reads.
        self.path(path).is_some()
    }
    /// Deserialize a selected JSON-compatible value using a string or parsed path.
    ///
    /// # Errors
    /// Rejects malformed paths, unsupported wire variants or type mismatches.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert_eq!(data.path_as::<String>("profile.name")?, Some("Ada".into()));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn path_as<'p, T: DeserializeOwned>(&self, path: impl IntoPath<'p>) -> Result<Option<T>> {
        // Deserialize only a present value; absence stays None and errors retain the selected path.
        let path = path.into_path()?;
        self.lookup(&path)
            .map(|value| {
                serde_json::from_value(value.to_json_with(&self.limits)?).map_err(Error::from)
            })
            .transpose()
            .map_err(|error| qualify_error(error, &path))
    }
    /// Borrow a previously parsed path without allocation.
    fn lookup(&self, path: &DataPath) -> Option<&Value> {
        // Each component narrows the borrow; scalars and append markers cannot be traversed.
        let mut pointer = &self.root;
        for step in &path.0 {
            pointer = match (pointer, step) {
                (Value::Object(values), PathSegment::Key(key)) => values.get(key)?,
                (Value::Object(values), PathSegment::Index(index)) => {
                    values.get(&index.to_string())?
                }
                (Value::Array(values), PathSegment::Index(index)) => values.get(*index)?,
                (Value::Array(values), PathSegment::Key(key)) => {
                    values.get(canonical_index(key)?)?
                }
                _ => return None,
            };
        }
        Some(pointer)
    }
    /// Set a literal top-level key, including a key containing dots.
    ///
    /// # Errors
    /// Rejects an empty key or exceeded mutation budgets.
    ///
    /// # Panics
    /// A final Shared native destructor may panic after tree/totals commit.
    /// Application panics propagate; completed changes are not rolled back.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// data.set("literal.key", false)?;
    /// assert_eq!(
    ///     data.get("literal.key").and_then(Value::as_bool),
    ///     Some(false)
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set(&mut self, key: &str, value: impl Into<Value>) -> Result<&mut Self> {
        // Wrap a literal key explicitly so embedded dots never become nested writes.
        self.set_path(
            &DataPath::from_segments([PathSegment::Key(key.into())])?,
            value,
        )
    }
    /// Stage and atomically commit a nested value mutation.
    ///
    /// # Errors
    /// Rejects traversal through scalars and budget/index overflow.
    ///
    /// # Panics
    /// A final Shared native destructor may panic after tree/totals commit.
    /// Application panics propagate; completed changes are not rolled back.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// let path = DataPath::dot("profile.name")?;
    /// data.set_path(&path, "Grace")?;
    /// assert_eq!(data.path_as::<String>(&path)?, Some("Grace".into()));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set_path<'p>(
        &mut self,
        path: impl IntoPath<'p>,
        value: impl Into<Value>,
    ) -> Result<&mut Self> {
        // Resolve the shared path vocabulary before validating incoming data or touching storage.
        let path = path.into_path()?;
        let value = value.into();
        // New nodes land below the path, so their allowed depth shrinks accordingly.
        if path.0.len() > self.limits.max_depth.min(64) {
            return Err(Error::limit("path depth limit exceeded").at_path(path.to_string()));
        }
        // Measure the incoming subtree at its final depth, not at a misleading new root.
        let incoming = value
            .measure(&ParseLimits {
                max_depth: self.limits.max_depth.min(64) - path.0.len(),
                ..self.limits
            })
            .map_err(|error| qualify_error(error, &path))?;
        // Every fallible structural/arithmetic check finishes before the first write.
        let (plan, change) = prepare(&self.root, &path, true, incoming, &self.limits)
            .map_err(|error| qualify_error(error, &path))?;
        let usage = change
            .project(self.usage, &self.limits)
            .map_err(|error| qualify_error(error, &path))?;
        // All Result-producing work has succeeded; commit moves only the affected branch.
        let previous = commit(&mut self.root, &plan, Some(value));
        // Publish totals only after the matching tree mutation is complete.
        self.usage = usage;
        // A final Shared payload may have a panicking user Drop; its unwind must see consistent
        // totals.
        drop(previous);
        Ok(self)
    }
    /// Remove a parsed path without shifting remaining array element identities.
    ///
    /// # Errors
    /// Rejects traversal through a non-container.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.remove_path("profile.name")?.is_some());
    /// assert!(!data.has_path("profile.name"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn remove_path<'p>(&mut self, path: impl IntoPath<'p>) -> Result<Option<Value>> {
        // Parse before planning so malformed paths cannot even change container representation.
        let path = path.into_path()?;
        // Missing removal returns no plan and cannot change even container representation.
        let (plan, change) = prepare(&self.root, &path, false, Usage::default(), &self.limits)
            .map_err(|error| qualify_error(error, &path))?;
        if plan.is_empty() {
            return Ok(None);
        }
        let usage = change
            .project(self.usage, &self.limits)
            .map_err(|error| qualify_error(error, &path))?;
        let removed = commit(&mut self.root, &plan, None);
        self.usage = usage;
        Ok(removed)
    }
    /// Remove a literal top-level key, without interpreting punctuation.
    ///
    /// # Errors
    /// Rejects invalid keys or budget growth when array identities become map keys.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// data.set("literal.key", true)?;
    /// assert!(data.remove("literal.key")?.is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn remove(&mut self, key: &str) -> Result<Option<Value>> {
        // Keep literal deletion symmetric with literal get/set, including dotted keys.
        self.remove_path(DataPath::from_segments([PathSegment::Key(key.into())])?)
    }
    /// Overlay incoming top-level keys; this deliberately is not recursive merging.
    ///
    /// # Errors
    /// Rejects a combined tree exceeding this store's budgets.
    ///
    /// # Panics
    /// A final Shared native destructor may panic after tree/totals commit.
    /// Application panics propagate; completed changes are not rolled back.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// data.merge(Nest::from_object(
    ///     serde_json::json!({"profile": {"id": 7}}),
    /// )?)?;
    /// assert!(!data.has_path("profile.name"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn merge(&mut self, other: Nest) -> Result<&mut Self> {
        // The incoming root is not inserted: retain our one root node and overlay fields.
        let incoming = other.root.measure(&self.limits)?;
        let mut change = Change::default();
        change.add(Usage {
            nodes: incoming.nodes - 1,
            bytes: incoming.bytes,
        })?;
        for (key, _) in other.entries() {
            if let Some(old) = self.get(key) {
                let old = old.measure(&self.limits)?;
                // An overlapping key keeps its existing position and is counted once.
                change.remove(Usage {
                    nodes: old.nodes,
                    bytes: old.bytes + key.len(),
                })?;
            }
        }
        // Reject combined size before moving any incoming field into the existing map.
        let usage = change.project(self.usage, &self.limits)?;
        // Reserve holders before mutation so no replaced native destructor runs mid-overlay.
        let mut previous = Vec::with_capacity(other.len());
        if let (Value::Object(target), Value::Object(incoming)) = (&mut self.root, other.root) {
            // IndexMap replaces values in place and appends new keys in source order.
            for (key, value) in incoming {
                // Retain replaced values: dropping a final Shared native may execute user code.
                if let Some(old) = target.insert(key, value) {
                    previous.push(old);
                }
            }
        }
        self.usage = usage;
        // All fields and their totals are committed before any replaced native destructor runs.
        drop(previous);
        Ok(self)
    }
    /// Remove all values while retaining this store's budgets.
    ///
    /// # Panics
    /// A final Shared native destructor may panic after tree/totals commit.
    /// Application panics propagate; completed changes are not rolled back.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// data.clear();
    /// assert!(data.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn clear(&mut self) {
        // Reset both representation and cached totals while retaining the chosen limits.
        let previous = std::mem::replace(&mut self.root, Value::Object(DataMap::new()));
        self.usage = Usage { nodes: 1, bytes: 0 };
        // Even a panicking final native destructor cannot leave the empty root with old totals.
        drop(previous);
    }
    /// Convert to JSON only when every nested variant has a deliberate JSON form.
    ///
    /// # Errors
    /// Rejects a non-JSON variant with its qualified path.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert_eq!(data.to_json()?["profile"]["name"], "Ada");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn to_json(&self) -> Result<serde_json::Value> {
        // Wire conversion is explicit and can reject otherwise valid in-process values.
        self.root.to_json_with(&self.limits)
    }
    /// Render compact or pretty JSON without swallowing unsupported values.
    ///
    /// # Errors
    /// Returns value-conversion or serializer failures.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// assert!(data.stringify(false)?.contains("Ada"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn stringify(&self, pretty: bool) -> Result<String> {
        // Complete rich-to-JSON validation before selecting compact versus pretty rendering.
        let value = self.to_json()?;
        Ok(if pretty {
            serde_json::to_string_pretty(&value)?
        } else {
            serde_json::to_string(&value)?
        })
    }
    /// Visit selected array/object values sequentially with cooperative cancellation.
    ///
    /// # Errors
    /// Returns invalid paths/non-collections or the visitor's original error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{DataPath, Nest, Value};
    /// let mut data = Nest::new();
    /// data.set_path("profile.name", "Ada")?;
    /// # async fn example() -> stackpress_lib::Result<()> {
    /// async fn inspect(_: String, _: &Value) -> stackpress_lib::Result<stackpress_lib::Flow> {
    ///     Ok(stackpress_lib::Flow::Continue)
    /// }
    /// let data = Nest::from_object(serde_json::json!({"rows": [1, 2]}))?;
    /// // Each visit finishes before the next starts; Abort would stop the remaining visits.
    /// data.visit("rows", inspect).await?;
    /// #     Ok(())
    /// # }
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub async fn visit<'a, 'p, F, Fut>(
        &'a self,
        path: impl IntoPath<'p>,
        mut visitor: F,
    ) -> Result<VisitOutcome>
    where
        F: FnMut(String, &'a Value) -> Fut + Send,
        Fut: Future<Output = Result<Flow>> + Send,
    {
        // Resolve once, then hold only immutable element borrows across sequential callbacks.
        let path = path.into_path()?;
        let entries: Vec<(String, &Value)> = match self.lookup(&path) {
            None => return Ok(VisitOutcome::Empty),
            Some(Value::Array(values)) => values
                .iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), v))
                .collect(),
            Some(Value::Object(values)) => values.iter().map(|(k, v)| (k.clone(), v)).collect(),
            _ => return Err(Error::invalid("visitor requires a collection")),
        };
        // An existing empty collection has the same empty-visit outcome as an absent one.
        if entries.is_empty() {
            return Ok(VisitOutcome::Empty);
        }
        // Await each visitor before entering the next; an abort/error preserves the visited prefix.
        for (key, value) in entries {
            if visitor(key, value).await? == Flow::Abort {
                return Ok(VisitOutcome::Aborted);
            }
        }
        Ok(VisitOutcome::Completed)
    }
}
impl Serialize for Nest {
    /// Serialize only JSON-compatible values, never Shared native payloads.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        // Delegate to the rich-value serializer so native/date/byte variants never disappear
        // silently.
        // Preserve the store's selected policy instead of Value's independent default wire budget.
        self.to_json()
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
impl fmt::Debug for Nest {
    /// Show structure keys rather than credential-bearing payloads.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Only keys are listed: debugging a request must not print credential-bearing values.
        f.debug_struct("Nest")
            .field("keys", &self.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}
/// Retain a nested conversion failure below the selected human-readable path.
fn qualify_error(error: Error, path: &DataPath) -> Error {
    // Retain a nested failure suffix while replacing internal segment Debug with a readable path.
    let path = match error.path().filter(|path| !path.is_empty()) {
        Some(nested) => format!("{path}.{nested}"),
        None => path.to_string(),
    };
    error.at_path(path)
}

/// Recognize only canonical indices without large sparse allocation.
pub(super) fn canonical_index(key: &str) -> Option<usize> {
    // Leading-zero and nonnumeric keys are identities, not array positions.
    if key == "0"
        || (!key.starts_with('0') && !key.is_empty() && key.bytes().all(|c| c.is_ascii_digit()))
    {
        key.parse().ok()
    } else {
        None
    }
}

/// Check whether an object has exactly the nonempty canonical keys 0..n.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::{DataPath, Nest, Value};
/// let mut data = Nest::new();
/// data.set_path("profile.name", "Ada")?;
/// let map = stackpress_lib::data::make_object(vec![1.into(), 2.into()]);
/// assert!(stackpress_lib::data::should_be_an_array(&map));
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn should_be_an_array(object: &DataMap<String, Value>) -> bool {
    // Nonempty dense canonical keys prove array shape without relying on insertion order.
    !object.is_empty() && (0..object.len()).all(|i| object.contains_key(&i.to_string()))
}
/// Explicit sorted-key value extraction, independent of numeric normalization.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::{DataPath, Nest, Value};
/// let mut data = Nest::new();
/// data.set_path("profile.name", "Ada")?;
/// let map = [("b".into(), 2.into()), ("a".into(), 1.into())]
///     .into_iter()
///     .collect();
/// // This explicit conversion sorts literal keys rather than retaining insertion order.
/// let values = stackpress_lib::data::make_array(map);
/// assert_eq!(values[0].as_i64(), Some(1));
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn make_array(object: DataMap<String, Value>) -> Vec<Value> {
    // This explicit helper sorts literal keys; it is deliberately separate from numeric
    // normalization.
    let mut entries: Vec<_> = object.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.into_iter().map(|(_, v)| v).collect()
}
/// Explicit conversion retaining array index identities as numeric string keys.
///
/// # Examples
///
/// ```
/// # use stackpress_lib::{DataPath, Nest, Value};
/// let mut data = Nest::new();
/// data.set_path("profile.name", "Ada")?;
/// let map = stackpress_lib::data::make_object(vec![1.into(), 2.into()]);
/// assert!(map.contains_key("1"));
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
pub fn make_object(array: Vec<Value>) -> DataMap<String, Value> {
    // Move each element under its original index so sparse deletion never shifts identity.
    array
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i.to_string(), v))
        .collect()
}
