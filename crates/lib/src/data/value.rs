//! Rich values keep dates, bytes, files and explicitly shared natives distinct.

use super::{DataMap, ParseLimits};

use crate::{Error, Result};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Cached payload totals; object keys count as bytes, array indices do not.
#[derive(Clone, Copy, Default)]
pub(crate) struct Usage {
    /// Containers and scalar leaves both consume one node.
    pub(crate) nodes: usize,
    /// UTF-8 keys/text and exact buffered bytes consume the payload budget.
    pub(crate) bytes: usize,
}

/// A checked UTC instant; presentation/time-zone conversion is an adapter concern.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Timestamp {
    /// Whole signed seconds from the Unix epoch.
    pub seconds: i64,
    /// Fractional nanoseconds, always less than one billion.
    nanoseconds: u32,
}
impl Timestamp {
    /// Construct an instant without accepting an invalid fractional component.
    ///
    /// # Errors
    /// Returns invalid input when nanoseconds is at least one billion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Timestamp;
    /// let instant = Timestamp::new(0, 123)?;
    /// assert_eq!(instant.nanoseconds(), 123);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(seconds: i64, nanoseconds: u32) -> Result<Self> {
        // Nanoseconds are a bounded fractional component; reject invalid instants before
        // constructing one.
        if nanoseconds >= 1_000_000_000 {
            return Err(Error::invalid("invalid timestamp nanoseconds"));
        }
        Ok(Self {
            seconds,
            nanoseconds,
        })
    }
    /// Inspect the validated fractional component.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Timestamp;
    /// assert_eq!(Timestamp::new(0, 123)?.nanoseconds(), 123);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn nanoseconds(&self) -> u32 {
        // The private fraction was validated at construction and cannot be overwritten externally.
        self.nanoseconds
    }
}

/// Buffered file bytes and untrusted metadata; construction never writes to disk.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileData {
    /// Exact file bytes, including invalid UTF-8 and an empty file.
    pub data: Vec<u8>,
    /// Untrusted supplied filename, never a permitted filesystem path.
    pub name: String,
    /// Supplied media type or application/octet-stream.
    pub mimetype: String,
}
impl FileData {
    /// Own a file buffer and its metadata, with no host-side effects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::FileData;
    /// let file = FileData::new(vec![0, 255], "upload.bin", "application/octet-stream");
    /// assert_eq!(file.data, [0, 255]);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(
        data: impl Into<Vec<u8>>,
        name: impl Into<String>,
        mimetype: impl Into<String>,
    ) -> Self {
        // Take ownership of buffered bytes/metadata only; an untrusted filename grants no
        // filesystem permission.
        Self {
            data: data.into(),
            name: name.into(),
            mimetype: mimetype.into(),
        }
    }
}

/// Explicit shared ownership of an opaque, immutable-access native payload.
///
/// Clone shares this named payload only. Interior synchronization is deliberately
/// chosen by the caller's payload type; no unchecked mutable reference is exposed.
#[derive(Clone)]
pub struct SharedNative(Arc<dyn Any + Send + Sync>);
impl SharedNative {
    /// Select explicit native sharing for an in-process typed payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::SharedNative;
    /// let native = SharedNative::new(7u32);
    /// assert_eq!(native.downcast_ref::<u32>(), Some(&7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new<T: Any + Send + Sync>(value: T) -> Self {
        // Shared construction is the explicit boundary where native ownership becomes Arc
        // ownership.
        Self(Arc::new(value))
    }
    /// Borrow a payload only when its concrete type matches.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::SharedNative;
    /// let native = SharedNative::new(7u32);
    /// assert!(native.downcast_ref::<String>().is_none());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        // A type mismatch is None; the native payload is never reinterpreted or exposed mutably.
        self.0.downcast_ref()
    }
    /// Compare shared identity without observing the private payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::SharedNative;
    /// let native = SharedNative::new(7u32);
    /// // The Shared name makes the allocation retained by Clone explicit.
    /// assert!(native.ptr_eq(&native.clone()));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn ptr_eq(&self, other: &Self) -> bool {
        // Compare Arc allocation identity without invoking native equality or observing private
        // content.
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl fmt::Debug for SharedNative {
    /// Redact the payload instead of inferring a serializable/debuggable native type.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not call a native payload's Debug or infer a wire representation from its concrete
        // type.
        f.write_str("SharedNative(..)")
    }
}

/// Owned data. Only the explicitly named Shared variant aliases after Clone.
///
/// Missing is represented by Option at lookup, distinct from present Null.
/// Serialize accepts JSON variants and rejects all non-JSON variants deliberately.
/// It checks default budgets and all variants before calling the serializer, then
/// borrows the tree. A serializer's own later failure may still leave partial output.
#[derive(Clone, Debug)]
pub enum Value {
    /// A present null value.
    Null,
    /// A boolean, including false.
    Bool(bool),
    /// A finite validated JSON number.
    Number(serde_json::Number),
    /// UTF-8 text.
    String(String),
    /// An ordered sequence without sparse holes.
    Array(Vec<Value>),
    /// Insertion-ordered literal string keys.
    Object(DataMap<String, Value>),
    /// A typed UTC instant, never silently stringified.
    DateTime(Timestamp),
    /// Arbitrary bytes, never silently decoded.
    Bytes(Vec<u8>),
    /// Buffered file data and metadata.
    File(FileData),
    /// Visibly shared in-process native state, never a wire value.
    Shared(SharedNative),
}

impl Value {
    /// Construct a finite floating-point number.
    ///
    /// # Errors
    /// Rejects NaN and infinity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert!(Value::number(f64::NAN).is_err());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn number(value: f64) -> Result<Self> {
        // Reject nonfinite floating values before they can enter the JSON-number variant.
        serde_json::Number::from_f64(value)
            .map(Self::Number)
            .ok_or_else(|| Error::invalid("number must be finite"))
    }
    /// Inspect a text value without converting another variant.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::from("Ada").as_str(), Some("Ada"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        // Borrow only actual text; dates, numbers and native values are not implicitly stringified.
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }
    /// Inspect a boolean without truthiness coercion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::from(false).as_bool(), Some(false));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        // A false boolean remains Some(false); other variants do not participate in truthiness.
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }
    /// Inspect an exactly representable signed integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::from(-1).as_i64(), Some(-1));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        // Delegate exact signed representability to the validated number type, preserving mismatch
        // as None.
        if let Self::Number(v) = self {
            v.as_i64()
        } else {
            None
        }
    }
    /// Inspect an exactly representable unsigned integer.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::from(7u64).as_u64(), Some(7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        // Negative/fractional numbers cannot be treated as unsigned identifiers by coercion.
        if let Self::Number(v) = self {
            v.as_u64()
        } else {
            None
        }
    }
    /// Inspect a numeric value as floating point.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::number(1.5)?.as_f64(), Some(1.5));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        // Only a numeric variant has a floating view; textual numbers require explicit parsing.
        if let Self::Number(v) = self {
            v.as_f64()
        } else {
            None
        }
    }
    /// Inspect an array, preserving the distinction from an object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::from(vec![1.into()]).as_array().unwrap().len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_array(&self) -> Option<&[Value]> {
        // Borrow dense order without exposing mutation that could bypass a Nest budget cache.
        if let Self::Array(v) = self {
            Some(v)
        } else {
            None
        }
    }
    /// Inspect literal object keys.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// let value = Value::try_from(serde_json::json!({"id": 7}))?;
    /// assert!(value.as_object().unwrap().contains_key("id"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_object(&self) -> Option<&DataMap<String, Value>> {
        // Literal object identity stays distinct from arrays and opaque native objects.
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }
    /// Inspect raw bytes without lossy UTF-8 conversion.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(
    ///     Value::Bytes(vec![0, 255]).as_bytes(),
    ///     Some([0, 255].as_slice())
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_bytes(&self) -> Option<&[u8]> {
        // Byte access never performs lossy UTF-8 conversion or changes the stored representation.
        if let Self::Bytes(v) = self {
            Some(v)
        } else {
            None
        }
    }
    /// Inspect an explicitly shared native payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// let value = Value::from(stackpress_lib::SharedNative::new(7u32));
    /// assert_eq!(value.as_shared().unwrap().downcast_ref::<u32>(), Some(&7));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_shared(&self) -> Option<&SharedNative> {
        // Only the visibly Shared variant exposes a native downcast handle.
        if let Self::Shared(v) = self {
            Some(v)
        } else {
            None
        }
    }
    /// Report a literal object, not any arbitrary native object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert!(Value::try_from(serde_json::json!({}))?.is_object());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_object(&self) -> bool {
        // Use the enum shape rather than treating every container/native value as a generic object.
        matches!(self, Self::Object(_))
    }
    /// Report a present null, not a missing field.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert!(Value::Null.is_null());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_null(&self) -> bool {
        // Missing belongs to Option at lookup; this predicate recognizes only a present null.
        matches!(self, Self::Null)
    }

    /// Validate node count/depth/bytes before copying, mutation or wire conversion.
    ///
    /// # Errors
    /// Returns a path-qualified resource-limit failure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// Value::from("bounded").validate(&ParseLimits::default())?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn validate(&self, limits: &ParseLimits) -> Result<()> {
        // Keep one authoritative traversal for both public checking and cached mutation totals.
        self.measure(limits).map(|_| ())
    }

    /// Check bounds and collect totals in the same traversal, without cloning payloads.
    pub(crate) fn measure(&self, limits: &ParseLimits) -> Result<Usage> {
        // Iterative traversal bounds depth without first recursively processing it.
        let mut pending = vec![(self, 0usize, String::new())];
        let (mut count, mut bytes) = (0usize, 0usize);
        while let Some((value, depth, path)) = pending.pop() {
            count = count
                .checked_add(1)
                .ok_or_else(|| Error::limit("node count overflow"))?;
            // The hard recursion ceiling applies even if a caller requests a larger workload depth.
            if depth > limits.max_depth.min(64) || count > limits.max_entries {
                return Err(Error::limit("value depth/entry limit exceeded").at_path(path));
            }
            // Charge owned payload bytes and schedule children; fixed-size scalar variants need
            // only their node cost.
            let payload = match value {
                Self::String(v) => v.len(),
                Self::Bytes(v) => v.len(),
                Self::File(v) => v
                    .data
                    .len()
                    .saturating_add(v.name.len())
                    .saturating_add(v.mimetype.len()),
                Self::Array(values) => {
                    // Reject excessive fan-out before allocating one pending traversal entry per
                    // child.
                    if values.len() > limits.max_entries.saturating_sub(count) {
                        return Err(Error::limit("value entry limit exceeded").at_path(path));
                    }
                    for (i, child) in values.iter().enumerate() {
                        pending.push((child, depth + 1, qualify(&path, &i.to_string())));
                    }
                    0
                }
                Self::Object(values) => {
                    if values.len() > limits.max_entries.saturating_sub(count) {
                        return Err(Error::limit("value entry limit exceeded").at_path(path));
                    }
                    for (key, child) in values {
                        bytes = bytes.saturating_add(key.len());
                        // Carry each literal key into the child diagnostic while keeping its
                        // payload borrowed.
                        pending.push((child, depth + 1, qualify(&path, key)));
                    }
                    0
                }
                _ => 0,
            };
            // Saturating totals cannot wrap an oversized payload into an accepted small number.
            bytes = bytes.saturating_add(payload);
            if bytes > limits.max_bytes {
                return Err(Error::limit("value byte limit exceeded").at_path(path));
            }
        }
        Ok(Usage {
            nodes: count,
            bytes,
        })
    }

    /// Convert deliberately to JSON, rejecting dates/bytes/files/shared natives.
    ///
    /// # Errors
    /// Returns the exact nested path of an unsupported variant or resource limit.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{ParseLimits, Value};
    /// assert_eq!(Value::from(false).to_json()?, serde_json::json!(false));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn to_json(&self) -> Result<serde_json::Value> {
        // Apply the default wire budget before recursively producing a deliberate JSON
        // representation.
        self.to_json_with(&ParseLimits::default())
    }

    /// Convert JSON-compatible values using an explicit finite wire budget.
    ///
    /// # Errors
    /// Rejects exceeded limits or non-JSON variants at their nested path.
    /// The whole tree's budgets are checked before conversion checks variants in
    /// insertion/index order; a budget failure therefore wins over unsupported data.
    /// No partial JSON tree is returned and the source value remains unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_lib::{ParseLimits, Value};
    /// let value = Value::from("bounded");
    /// assert_eq!(
    ///     value.to_json_with(&ParseLimits::default())?,
    ///     serde_json::json!("bounded")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn to_json_with(&self, limits: &ParseLimits) -> Result<serde_json::Value> {
        // A store/caller's deliberate larger policy must not be silently replaced by defaults.
        self.validate(limits)?;
        // Recursive conversion is safe only after the same hard depth ceiling is checked.
        self.json_at("")
    }

    /// Convert a validated tree while carrying a useful error path.
    pub(super) fn json_at(&self, path: &str) -> Result<serde_json::Value> {
        // Carry relative field/index context into every child so unsupported variants identify
        // their location.
        Ok(match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(v) => (*v).into(),
            Self::Number(v) => serde_json::Value::Number(v.clone()),
            Self::String(v) => v.clone().into(),
            // Serialize array positions in order; an unsupported child fails the entire conversion.
            Self::Array(values) => serde_json::Value::Array(
                values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v.json_at(&qualify(path, &i.to_string())))
                    .collect::<Result<_>>()?,
            ),
            // Preserve literal keys and insertion order while qualifying each child conversion.
            Self::Object(values) => serde_json::Value::Object(
                values
                    .iter()
                    .map(|(k, v)| Ok((k.clone(), v.json_at(&qualify(path, k))?)))
                    .collect::<Result<_>>()?,
            ),
            _ => {
                // Dates, raw bytes, files and native values require an explicit caller-selected
                // wire form.
                return Err(Error::invalid(
                    "non-JSON value requires an explicit wire representation",
                )
                .at_path(path));
            }
        })
    }

    /// Convert a JSON tree after validating its bounds iteratively.
    pub(crate) fn from_json_with(value: serde_json::Value, limits: &ParseLimits) -> Result<Self> {
        // Validate raw JSON before recursively constructing rich values.
        let mut pending = vec![(&value, 0usize)];
        let (mut count, mut bytes) = (0usize, 0usize);
        while let Some((node, depth)) = pending.pop() {
            count = count.saturating_add(1);
            if depth > limits.max_depth.min(64) || count > limits.max_entries {
                return Err(Error::limit("JSON depth/entry limit exceeded"));
            }
            match node {
                serde_json::Value::String(v) => bytes = bytes.saturating_add(v.len()),
                serde_json::Value::Array(values) => {
                    if values.len() > limits.max_entries.saturating_sub(count) {
                        return Err(Error::limit("JSON entry limit exceeded"));
                    }
                    // Raw parsed JSON is inspected before the bounded recursive rich-value
                    // construction phase.
                    pending.extend(values.iter().map(|v| (v, depth + 1)));
                }
                serde_json::Value::Object(values) => {
                    if values.len() > limits.max_entries.saturating_sub(count) {
                        return Err(Error::limit("JSON entry limit exceeded"));
                    }
                    for (k, v) in values {
                        bytes = bytes.saturating_add(k.len());
                        pending.push((v, depth + 1));
                    }
                }
                _ => {}
            }
            if bytes > limits.max_bytes {
                return Err(Error::limit("JSON byte limit exceeded"));
            }
        }
        // Only a fully validated raw tree reaches recursive conversion; no partial rich result is
        // returned.
        Ok(Self::from_validated_json(value))
    }

    /// Construct only after the depth guard has established a bounded tree.
    fn from_validated_json(value: serde_json::Value) -> Self {
        // Depth and payload bounds were already proved; now move ordinary JSON nodes without
        // copying them.
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(v) => Self::Bool(v),
            serde_json::Value::Number(v) => Self::Number(v),
            serde_json::Value::String(v) => Self::String(v),
            serde_json::Value::Array(v) => {
                Self::Array(v.into_iter().map(Self::from_validated_json).collect())
            }
            serde_json::Value::Object(v) => Self::Object(
                v.into_iter()
                    .map(|(k, v)| (k, Self::from_validated_json(v)))
                    .collect(),
            ),
        }
    }
}

/// Join literal keys for diagnostics; it is not used to interpret data paths.
pub(crate) fn qualify(parent: &str, key: &str) -> String {
    // Diagnostic joining never feeds back into path parsing, so literal key punctuation is not
    // reinterpreted.
    if parent.is_empty() {
        key.to_owned()
    } else {
        format!("{parent}.{key}")
    }
}

impl TryFrom<serde_json::Value> for Value {
    type Error = Error;
    /// Validate a supplied parsed JSON tree using default finite limits.
    fn try_from(value: serde_json::Value) -> Result<Self> {
        // External parsed JSON still needs the framework's default finite-tree contract.
        Self::from_json_with(value, &ParseLimits::default())
    }
}
impl Serialize for Value {
    /// Never erase a non-JSON variant through an accidental serializer fallback.
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        // Complete rich-to-JSON validation first; a generic serializer must never silently erase a
        // native variant.
        super::json::Json::checked(self, &ParseLimits::default())
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Value {
    /// External input creates only bounded JSON-compatible variants.
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        // Only JSON-compatible variants can enter through serde; native payloads require explicit
        // in-process construction.
        Self::try_from(serde_json::Value::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
impl From<String> for Value {
    /// Own UTF-8 text.
    fn from(v: String) -> Self {
        // Move owned text without creating shared backing storage.
        Self::String(v)
    }
}
impl From<&str> for Value {
    /// Copy borrowed UTF-8 text.
    fn from(v: &str) -> Self {
        // Copy borrowed text into ordinary owned data.
        Self::String(v.into())
    }
}
impl From<bool> for Value {
    /// Preserve false without truthiness coercion.
    fn from(v: bool) -> Self {
        // Preserve a false scalar as data instead of interpreting it as absence.
        Self::Bool(v)
    }
}
impl From<i64> for Value {
    /// Preserve a signed integer exactly.
    fn from(v: i64) -> Self {
        // The integer conversion is exact and never passes through floating-point representation.
        Self::Number(v.into())
    }
}
impl From<i32> for Value {
    /// Preserve an ordinary signed integer exactly.
    fn from(v: i32) -> Self {
        // The integer conversion is exact and never passes through floating-point representation.
        Self::Number(v.into())
    }
}
impl From<u64> for Value {
    /// Preserve an unsigned integer exactly.
    fn from(v: u64) -> Self {
        // Unsigned values retain the full representable JSON integer range.
        Self::Number(v.into())
    }
}
impl From<u32> for Value {
    /// Preserve an ordinary unsigned integer exactly.
    fn from(v: u32) -> Self {
        // Unsigned values retain exact integer representation.
        Self::Number(v.into())
    }
}
impl From<Vec<Value>> for Value {
    /// Own an array of rich values.
    fn from(v: Vec<Value>) -> Self {
        // Move the array and its child ownership without adding aliases.
        Self::Array(v)
    }
}
impl From<DataMap<String, Value>> for Value {
    /// Own an insertion-ordered object.
    fn from(v: DataMap<String, Value>) -> Self {
        // Move insertion-ordered literal fields without normalizing numeric keys implicitly.
        Self::Object(v)
    }
}
impl From<SharedNative> for Value {
    /// Keep native sharing visible in the variant.
    fn from(v: SharedNative) -> Self {
        // Native identity remains visibly Shared after conversion into the rich enum.
        Self::Shared(v)
    }
}
impl From<Timestamp> for Value {
    /// Keep dates distinct from strings/numbers.
    fn from(v: Timestamp) -> Self {
        // Keep an instant typed rather than inferring a string/date wire encoding.
        Self::DateTime(v)
    }
}
impl From<FileData> for Value {
    /// Keep file bytes and metadata together.
    fn from(v: FileData) -> Self {
        // Buffered files retain byte data and metadata, never an inferred filesystem resource.
        Self::File(v)
    }
}
