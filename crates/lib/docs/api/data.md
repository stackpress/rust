# Data API

Owned rich values, object-root stores, checked paths, ordered collection aliases, and explicit JSON
boundaries. Sources: [data module](../../src/data/mod.rs), [Value](../../src/data/value.rs),
[Nest](../../src/data/nest.rs), [paths](../../src/data/path.rs).

```rust
use stackpress_lib::data::{
    IntoNest, VisitOutcome, make_array, make_object, should_be_an_array,
};
use stackpress_lib::{
    DataMap, DataPath, DataSet, FileData, Flow, IntoPath, Nest, ParseLimits, PathSegment,
    Result, SharedNative, Timestamp, Value,
};

let mut data = Nest::from_object(serde_json::json!({
    "profile": {"name": "Ada"}, "tags": ["rust", "events"], "active": false,
}))?;
```

Examples run independently inside an async function returning `Result<()>`, with these imports and a
fresh `data` from the setup. [Data and paths](../guides/data-and-paths.md) explains the usage flow.

Contents: [Limits and value shapes](#1-limits-and-value-shapes) ·
[Value operations](#2-value-operations) ·
[Native and buffered values](#3-native-and-buffered-values) · [Paths](#4-paths) ·
[Nest construction and inspection](#5-nest-construction-and-inspection) ·
[Nest reads](#6-nest-reads) · [Nest mutations and JSON](#7-nest-mutations-and-json) ·
[Input adaptation and ordered collections](#8-input-adaptation-and-ordered-collections)

## 1. Limits and value shapes

Limits apply to already-parsed values as well as boundary text. Fixed-size scalars use a node; each
container and leaf also uses a node. Object key bytes, UTF-8 text, bytes, and file data/name/MIME
count toward the byte budget. Opaque shared payload internals are not traversed or byte-counted.

| ParseLimits field | Type | Default / meaning |
| --- | --- | --- |
| `max_bytes` | `usize` | 1,048,576: raw buffered input or aggregate owned variable payload bytes. |
| `max_depth` | `usize` | 64: root is depth 0; hard implementation ceiling remains 64. |
| `max_entries` | `usize` | 10,000: value nodes, plus parser-specific input-work/part checks. |
| `max_part_headers` | `usize` | 8,192: header bytes per multipart part. |

`ParseLimits` is `Clone + Copy + Debug + Eq + PartialEq`; `Default` chooses these values. Custom
fields are public. Empty `Nest::with_limits` construction itself is infallible, including a
zero-entry policy; call validation when you need to establish whether an empty root fits such a
policy.

`Value` variants are `Null`, `Bool(bool)`, `Number(serde_json::Number)`, `String(String)`,
`Array(Vec<Value>)`, `Object(DataMap<String, Value>)`, `DateTime(Timestamp)`, `Bytes(Vec<u8>)`,
`File(FileData)`, and `Shared(SharedNative)`. Missing is an `Option` at lookup, never a variant.
Clone deeply copies ordinary data; only Shared retains its Arc identity. Value Debug can show
ordinary payloads; it is not the redacted Nest/Request wrapper view.

## 2. Value operations

Accessors only accept their actual variant and never perform truthiness or text coercion. JSON
conversion rejects dates, bytes, files, and shared native values even when empty.

### 2.1. Value::number

`Value::number(value: f64) -> Result<Self>`

Construct a finite JSON number from floating point.

```rust
assert_eq!(Value::number(1.5)?.as_f64(), Some(1.5));
assert!(Value::number(f64::NAN).is_err());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `f64` | Finite floating-point input. |

**Returns** A Number value, or invalid-input error for NaN/infinity.

### 2.2. Value::as_str

`as_str(&self) -> Option<&str>`

Borrow actual text.

```rust
assert_eq!(Value::from("Ada").as_str(), Some("Ada"));
```

**Returns** Borrowed text, or None for another variant.

### 2.3. Value::as_bool

`as_bool(&self) -> Option<bool>`

Inspect a boolean without coercion.

```rust
assert_eq!(Value::from(false).as_bool(), Some(false));
```

**Returns** The boolean, including false, or None.

### 2.4. Value::as_i64

`as_i64(&self) -> Option<i64>`

Inspect a number representable as a signed integer.

```rust
assert_eq!(Value::from(-7).as_i64(), Some(-7));
```

**Returns** An exactly representable i64, or None for a different variant or incompatible number.

### 2.5. Value::as_u64

`as_u64(&self) -> Option<u64>`

Inspect a number representable as an unsigned integer.

```rust
assert_eq!(Value::from(7u64).as_u64(), Some(7));
```

**Returns** An exactly representable u64, or None for a different variant, negative, or incompatible
number.

### 2.6. Value::as_f64

`as_f64(&self) -> Option<f64>`

Inspect the floating view of a JSON number.

```rust
assert_eq!(Value::number(1.5)?.as_f64(), Some(1.5));
```

**Returns** The numeric floating view, or None for another variant; integer precision is governed by
f64 conversion.

### 2.7. Value::as_array

`as_array(&self) -> Option<&[Value]>`

Borrow a dense array without exposing store mutation.

```rust
let value = Value::Array(vec!["rust".into()]);
assert_eq!(value.as_array().unwrap().len(), 1);
```

**Returns** A borrowed array slice, or None.

### 2.8. Value::as_object

`as_object(&self) -> Option<&DataMap<String, Value>>`

Borrow literal object fields.

```rust
assert!(data.value().as_object().unwrap().contains_key("profile"));
```

**Returns** A borrowed object map, or None for arrays and other variants.

### 2.9. Value::as_bytes

`as_bytes(&self) -> Option<&[u8]>`

Borrow exact bytes without text decoding.

```rust
let value = Value::Bytes(vec![0, 255]);
assert_eq!(value.as_bytes(), Some([0, 255].as_slice()));
```

**Returns** A borrowed byte slice only for Bytes, or None.

### 2.10. Value::as_shared

`as_shared(&self) -> Option<&SharedNative>`

Borrow an explicitly shared native handle.

```rust
let value = Value::from(SharedNative::new(7u32));
assert_eq!(value.as_shared().unwrap().downcast_ref::<u32>(), Some(&7));
```

**Returns** The borrowed SharedNative only for Shared, or None.

### 2.11. Value::is_object

`is_object(&self) -> bool`

Test the actual object variant.

```rust
assert!(data.value().is_object());
```

**Returns** True only for Object.

### 2.12. Value::is_null

`is_null(&self) -> bool`

Test a present null value.

```rust
assert!(Value::Null.is_null());
```

**Returns** True only for Null, independently of missing lookup state.

### 2.13. Value::validate

`validate(&self, limits: &ParseLimits) -> Result<()>`

Measure nodes, depth, and payload bytes without wire conversion.

```rust
data.value().validate(&ParseLimits::default())?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `&ParseLimits` | Caller-selected finite policy, depth capped at 64. |

**Returns** Unit, or a path-qualified limit error. Non-JSON variants may validate successfully;
shared-native internals are not traversed.

### 2.14. Value::to_json

`to_json(&self) -> Result<serde_json::Value>`

Convert deliberately using default limits.

```rust
assert_eq!(Value::from(false).to_json()?, serde_json::json!(false));
```

**Returns** Owned JSON preserving types and key order. Default budget failures or non-JSON variants
return an error with a relative data path; no implicit string/base64/native conversion occurs.

### 2.15. Value::to_json_with

`to_json_with(&self, limits: &ParseLimits) -> Result<serde_json::Value>`

Convert under an explicit policy when defaults do not fit the workload.

```rust
let limits = ParseLimits {
    max_bytes: 2 * 1024 * 1024,
    ..Default::default()
};
let json = data.value().to_json_with(&limits)?;
assert!(json.is_object());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `&ParseLimits` | Selected conversion limits; hard depth ceiling remains 64. |

**Returns** Owned JSON, or budget/unsupported-value error qualified to the failing child.

### 2.16. Value conversions

`Value::from(input) / Value::try_from(json) / serde serialization`

Use exact owned conversions for known Rust values. `From` is implemented for `String`, `&str`,
`bool`, `i64`, `i32`, `u64`, `u32`, `Vec<Value>`, `DataMap<String, Value>`, `SharedNative`,
`Timestamp`, and `FileData`. Use explicit variants for raw bytes and null.

```rust
let value = Value::try_from(serde_json::json!({"name": "Ada"}))?;
let encoded = serde_json::to_string(&value)?;
let decoded: Value = serde_json::from_str(&encoded)?;
assert!(decoded.is_object());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `input` | `One of the listed From types` | Owned value or copied borrowed text. |
| `json` | `serde_json::Value` | For TryFrom; raw input still needs default-budget validation. |

**Returns** From produces Value infallibly without tree validation; TryFrom validates default tree
limits. Serialize uses `to_json()` and reports non-JSON/default-limit failures through the
serializer. Deserialize reads JSON-compatible values under default limits and cannot create native
variants.

## 3. Native and buffered values

These values retain explicit types without choosing a wire format. Timestamp is
Clone/Copy/Debug/Eq/PartialEq; FileData is Clone/Debug/Eq/PartialEq; SharedNative is Clone with
redacted Debug and immutable payload access.

| Field | Type | Meaning |
| --- | --- | --- |
| `Timestamp.seconds` | `i64` | Public signed epoch seconds; fractional nanoseconds remain private. |
| `FileData.data` | `Vec<u8>` | Exact owned file bytes, including empty/non-UTF-8. |
| `FileData.name` | `String` | Untrusted filename; no filesystem permission or I/O. |
| `FileData.mimetype` | `String` | Supplied metadata; FileData::new does not infer/validate it. |

### 3.1. Timestamp::new

`Timestamp::new(seconds: i64, nanoseconds: u32) -> Result<Self>`

Construct an epoch instant with a checked fractional component.

```rust
let instant = Timestamp::new(-1, 123)?;
assert_eq!(instant.seconds, -1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `seconds` | `i64` | Signed whole epoch seconds. |
| `nanoseconds` | `u32` | Fractional component below 1,000,000,000. |

**Returns** A Timestamp, or invalid-input error when the fraction is at least one billion. No
calendar/time-zone interpretation is performed.

### 3.2. Timestamp::nanoseconds

`nanoseconds(&self) -> u32`

Read the checked fractional component.

```rust
assert_eq!(Timestamp::new(0, 123)?.nanoseconds(), 123);
```

**Returns** The fraction, always below one billion.

### 3.3. FileData::new

`FileData::new(data: impl Into<Vec<u8>>, name: impl Into<String>, mimetype: impl Into<String>) -> Self`

Own a buffer and metadata without writing or validating a filesystem path.

```rust
let file = FileData::new(vec![0, 255], "upload.bin", "application/octet-stream");
assert_eq!(file.data, [0, 255]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `data` | `impl Into<Vec<u8>>` | Exact owned/copyable file buffer. |
| `name` | `impl Into<String>` | Untrusted supplied filename. |
| `mimetype` | `impl Into<String>` | Explicit supplied media metadata; no default argument. |

**Returns** A FileData, infallibly. Enclosing Value/Nest validation applies payload budgets;
multipart parsing supplies the fallback MIME when needed.

### 3.4. SharedNative::new

`SharedNative::new<T: Any + Send + Sync>(value: T) -> Self`

Choose explicit Arc sharing for an in-process payload.

```rust
let shared = SharedNative::new(String::from("local service"));
assert!(shared.ptr_eq(&shared.clone()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `T: Any + Send + Sync` | Owned static native payload; any interior synchronization belongs to its type. |

**Returns** A shared native handle. Clone shares allocation identity; the library exposes no mutable
downcast and never serializes the payload.

### 3.5. SharedNative::downcast_ref

`downcast_ref<T: Any>(&self) -> Option<&T>`

Borrow the payload using its exact concrete type.

```rust
let shared = SharedNative::new(7u32);
assert_eq!(shared.downcast_ref::<u32>(), Some(&7));
```

**Returns** A borrowed T on exact type match, otherwise None. T is a type parameter; downcasting
does not convert types or expose unchecked mutation.

### 3.6. SharedNative::ptr_eq

`ptr_eq(&self, other: &Self) -> bool`

Compare allocation identity without observing native equality.

```rust
let shared = SharedNative::new(7u32);
assert!(shared.ptr_eq(&shared.clone()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `other` | `&SharedNative` | Handle whose Arc identity is compared. |

**Returns** True only for handles sharing the same allocation, independently of payload equality.

## 4. Paths

`PathSegment::{Key(String), Index(usize), Append}` distinguishes literal keys, numeric identity, and
next-index insertion. Paths are immutable Clone/Debug/Eq/PartialEq values. Dot text has no escaping
or bracket-append syntax; use explicit segments for those cases.

String paths are at most 16 KiB before parsing, at most 64 segments, and at most 8 KiB aggregate key
bytes. Canonical decimal text (`0` or non-leading-zero digits) becomes Index; overflow is an error.
`01` and negative spellings stay keys. Display quotes literal punctuation/numeric keys for human
clarity; its output is not a general round-trip path grammar.

### 4.1. DataPath::dot

`DataPath::dot(text: &str) -> Result<Self>`

Parse a reusable dot-separated path.

```rust
let path = DataPath::dot("profile.name")?;
assert_eq!(data.path_as::<String>(&path)?, Some("Ada".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Nonempty dot path; no empty components. |

**Returns** A checked DataPath, or input/limit error for empty components, numeric overflow,
source/key size, or segment depth.

### 4.2. DataPath::separated

`DataPath::separated(text: &str, separator: char) -> Result<Self>`

Parse the same component vocabulary with a caller-selected literal separator.

```rust
let path = DataPath::separated("profile/name", '/')?;
assert_eq!(data.path_as::<String>(&path)?, Some("Ada".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `text` | `&str` | Nonempty separated path. |
| `separator` | `char` | Literal component delimiter, not a regex. |

**Returns** A checked path, with the same empty-component, index, and size/depth failures as dot
parsing.

### 4.3. DataPath::from_segments

`DataPath::from_segments(segments: impl IntoIterator<Item = PathSegment>) -> Result<Self>`

Preserve literal punctuation/numeric keys or request append explicitly.

```rust
let path = DataPath::from_segments([PathSegment::Key("tags".into()), PathSegment::Append])?;
data.set_path(&path, "async")?;
assert_eq!(data.path_as::<String>("tags.2")?, Some("async".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `segments` | `impl IntoIterator<Item = PathSegment>` | At least one and at most 64; nonempty Key payloads totaling at most 8 KiB. |

**Returns** A checked immutable path, or input/limit error. Consumption stops when a bound is
exceeded; indices do not allocate sparse arrays.

### 4.4. DataPath::segments

`segments(&self) -> &[PathSegment]`

Borrow checked segments for inspection.

```rust
let path = DataPath::dot("tags.0")?;
assert_eq!(path.segments().len(), 2);
```

**Returns** A borrowed immutable slice; it cannot bypass path validation.

### 4.5. IntoPath::into_path

`into_path(self) -> Result<Cow<'a, DataPath>>`

Adapt an accepted path form; the trait is sealed.

```rust
let parsed = DataPath::dot("profile.name")?;
let borrowed = (&parsed).into_path()?;
assert_eq!(borrowed.segments().len(), 2);
```

**Returns** Borrowed Cow for `&DataPath`, owned Cow for moved DataPath, or newly parsed owned Cow
for `&str` and `&String`. Malformed string input returns a path error. Owned String itself is not an
IntoPath implementation.

## 5. Nest construction and inspection

Nest keeps an object root, retained limits, and private mutation bookkeeping. Clone copies ordinary
values and shares only SharedNative. Default equals new. Debug lists top-level keys rather than
values; Serialize uses the selected store limits, with no Nest Deserialize implementation.

### 5.1. Nest::new

`Nest::new() -> Self`

Create an independent empty object store with default limits.

```rust
let data = Nest::new();
assert!(data.is_empty());
```

**Returns** An empty Nest, with a root node and no fields.

### 5.2. Nest::with_limits

`Nest::with_limits(limits: ParseLimits) -> Self`

Retain a chosen policy for later writes and output conversion.

```rust
let data = Nest::with_limits(ParseLimits {
    max_bytes: 4096,
    ..Default::default()
});
assert_eq!(data.limits().max_bytes, 4096);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Policy retained by value; constructor does not validate an empty root against it. |

**Returns** An empty Nest infallibly. Even this empty root consumes one node when validated.

### 5.3. Nest::from_object

`Nest::from_object(value: serde_json::Value) -> Result<Self>`

Construct from parsed object-root JSON using default limits.

```rust
let data = Nest::from_object(serde_json::json!({"name": "Ada"}))?;
assert!(data.has("name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `serde_json::Value` | Object root, with no second scalar coercion. |

**Returns** A checked store, or a non-object-root/default-budget error.

### 5.4. Nest::from_value

`Nest::from_value(value: Value, limits: ParseLimits) -> Result<Self>`

Accept rich object-root data under an explicit policy.

```rust
let mut fields = DataMap::new();
fields.insert("native".into(), Value::from(SharedNative::new(7u32)));
let data = Nest::from_value(Value::Object(fields), ParseLimits::default())?;
assert!(data.has("native"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `value` | `Value` | Must be Object; children may be rich in-process values. |
| `limits` | `ParseLimits` | Retained tree policy. |

**Returns** A checked Nest, or root-shape/limit error. No JSON conversion is required.

### 5.5. Nest::value

`value(&self) -> &Value`

Borrow the immutable object root.

```rust
assert!(data.value().is_object());
```

**Returns** A borrowed root; no writable bypass exists.

### 5.6. Nest::into_value

`into_value(self) -> Value`

Consume the store and move the root.

```rust
let root = data.into_value();
assert!(root.is_object());
```

**Returns** The owned object Value; store policy is discarded.

### 5.7. Nest::limits

`limits(&self) -> ParseLimits`

Inspect this store's retained policy.

```rust
assert_eq!(data.limits().max_depth, 64);
```

**Returns** A copy of ParseLimits; changing it does not change this store.

### 5.8. Nest::len

`len(&self) -> usize`

Count top-level fields, not recursive nodes.

```rust
assert_eq!(data.len(), 3);
```

**Returns** The number of root keys.

### 5.9. Nest::is_empty

`is_empty(&self) -> bool`

Check top-level field presence.

```rust
assert!(!data.is_empty());
```

**Returns** True for an empty object root.

### 5.10. Nest::entries

`entries(&self) -> &DataMap<String, Value>`

Borrow root entries in insertion order.

```rust
assert!(data.entries().contains_key("profile"));
```

**Returns** An immutable root map.

### 5.11. Nest::keys

`keys(&self) -> impl Iterator<Item = &String>`

Iterate root keys without copying them.

```rust
assert_eq!(data.keys().next().map(String::as_str), Some("profile"));
```

**Returns** A borrowing iterator in insertion order.

### 5.12. Nest::values

`values(&self) -> impl Iterator<Item = &Value>`

Iterate root values without cloning native handles.

```rust
assert_eq!(data.values().count(), 3);
```

**Returns** A borrowing iterator aligned with key order.

## 6. Nest reads

Literal operations never parse punctuation. Path operations accept IntoPath forms; their optional
reads deliberately collapse malformed/missing/scalar traversal into absence. Typed reads retain
parse and conversion errors.

### 6.1. Nest::get

`get(&self, key: &str) -> Option<&Value>`

Read one literal root field.

```rust
assert_eq!(data.get("active").and_then(Value::as_bool), Some(false));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `key` | `&str` | Exact top-level key, including dots/brackets. |

**Returns** A borrowed present value, including Null, or None.

### 6.2. Nest::has

`has(&self, key: &str) -> bool`

Check literal root presence without truthiness.

```rust
assert!(data.has("active"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `key` | `&str` | Exact top-level name. |

**Returns** True for any present value, including false/zero/null.

### 6.3. Nest::path

`path<'p>(&self, path: impl IntoPath<'p>) -> Option<&Value>`

Read a nested value with convenient absence semantics.

```rust
assert_eq!(
    data.path("profile.name").and_then(Value::as_str),
    Some("Ada")
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl IntoPath<'p>` | Dot string or checked path; Append is not a readable element. |

**Returns** Borrowed value, or None for malformed/missing paths or traversal through a scalar.

### 6.4. Nest::has_path

`has_path<'p>(&self, path: impl IntoPath<'p>) -> bool`

Check path presence using the same optional read policy.

```rust
assert!(data.has_path("profile.name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl IntoPath<'p>` | Accepted path form. |

**Returns** True for a present value; false for malformed/missing/unreadable paths.

### 6.5. Nest::path_as

`path_as<'p, T: DeserializeOwned>(&self, path: impl IntoPath<'p>) -> Result<Option<T>>`

Deserialize a present JSON-compatible value into a real target type.

```rust
assert_eq!(
    data.path_as::<Vec<String>>("tags")?,
    Some(vec!["rust".into(), "events".into()])
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl IntoPath<'p>` | Accepted path form; T is the caller-selected DeserializeOwned type. |

**Returns** Ok(None) when lookup is absent. A present null is deserialized into T (for example T =
Option<String> gives Some(None)). Invalid path syntax, incompatible T, non-JSON variants, or
selected-budget failures return path-qualified errors.

## 7. Nest mutations and JSON

Result-producing mutations validate before committing; returned errors preserve the tree. Traversing
an existing scalar is an error. Root stays Object; nonempty dense canonical numeric children
normalize to Array. Removing an array member preserves remaining numeric identities through object
conversion; append uses one past the highest canonical index. Merge is incoming-wins at the top
level, not recursive.

### 7.1. Nest::set

`set(&mut self, key: &str, value: impl Into<Value>) -> Result<&mut Self>`

Set a literal top-level field, even if its name contains dots.

```rust
data.set("literal.name", "Ada")?;
assert!(data.has("literal.name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `key` | `&str` | Nonempty literal key within path/key bounds. |
| `value` | `impl Into<Value>` | Owned compatible value. |

**Returns** The mutable Nest for chaining, or path/value/budget error with no mutation.

### 7.2. Nest::set_path

`set_path<'p>(&mut self, path: impl IntoPath<'p>, value: impl Into<Value>) -> Result<&mut Self>`

Set a checked nested field; create missing containers without overwriting existing scalars.

```rust
data.set_path("profile.name", "Grace")?;
assert_eq!(
    data.path_as::<String>("profile.name")?,
    Some("Grace".into())
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl IntoPath<'p>` | Nonempty dot/explicit path; Append chooses next numeric identity. |
| `value` | `impl Into<Value>` | Incoming subtree validated at its final insertion depth. |

**Returns** The mutable store, or malformed path, scalar traversal, index arithmetic, or resulting
budget error. All returned errors preserve previous data.

### 7.3. Nest::remove

`remove(&mut self, key: &str) -> Result<Option<Value>>`

Remove a literal root field.

```rust
assert!(data.remove("active")?.is_some());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `key` | `&str` | Nonempty literal field name. |

**Returns** The moved removed value or None if absent; invalid path/key/structural-budget checks
return an error without mutation.

### 7.4. Nest::remove_path

`remove_path<'p>(&mut self, path: impl IntoPath<'p>) -> Result<Option<Value>>`

Remove a nested field without shifting array identities.

```rust
data.remove_path("tags.0")?;
assert_eq!(data.path_as::<String>("tags.1")?, Some("events".into()));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl IntoPath<'p>` | Nonempty nested path. |

**Returns** The moved removed value or None for a missing target. Malformed paths, scalar traversal,
append overflow, or a resulting sparse-object budget failure return an error with no mutation.

### 7.5. Nest::merge

`merge(&mut self, other: Nest) -> Result<&mut Self>`

Overlay entire root fields; existing keys keep their positions and new keys append in incoming
order.

```rust
data.merge(Nest::from_object(
    serde_json::json!({"profile": {"active": true}}),
)?)?;
assert!(data.path("profile.name").is_none());
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `other` | `Nest` | Consumed incoming object store, rechecked under receiver limits. |

**Returns** The mutable receiver, or combined value/budget error without a partial overlay.

### 7.6. Nest::clear

`clear(&mut self)`

Remove all root fields while retaining this store's selected limits.

```rust
data.clear();
assert!(data.is_empty());
```

**Returns** Unit; root is reset to an empty object. There is no Result-producing validation step.

### 7.7. Nest::to_json

`to_json(&self) -> Result<serde_json::Value>`

Convert an object-root store under its own policy.

```rust
let json = data.to_json()?;
assert_eq!(json["profile"]["name"], "Ada");
```

**Returns** Owned JSON, or selected-budget/non-JSON-variant error qualified to its child. Unlike
Value::to_json, it retains the store's chosen policy.

### 7.8. Nest::stringify

`stringify(&self, pretty: bool) -> Result<String>`

Render validated JSON as compact or pretty text.

```rust
let text = data.stringify(false)?;
assert!(text.contains("Ada"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pretty` | `bool` | true selects serde pretty output; false selects compact output. |

**Returns** Owned JSON text; conversion/serialization errors return Err without changing the store.

### 7.9. Nest::visit

`async visit<'a, 'p, F, Fut>(&'a self, path: impl IntoPath<'p>, visitor: F) -> Result<VisitOutcome> where F: FnMut(String, &'a Value) -> Fut + Send, Fut: Future<Output = Result<Flow>> + Send`

Visit only the immediate children of the selected collection, sequentially. Array keys are decimal
indices; object keys retain insertion order. A visitor receives an owned key and a borrowed child.

```rust
async fn inspect(_key: String, value: &Value) -> Result<Flow> {
    assert!(value.as_str().is_some());
    Ok(Flow::Continue)
}
assert_eq!(data.visit("tags", inspect).await?, VisitOutcome::Completed);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `path` | `impl IntoPath<'p>` | Collection path. |
| `visitor` | `F` | Sequential Send callback; may return Continue/Abort or an error. |

**Returns** `VisitOutcome::{Completed, Aborted, Empty}`; Empty covers missing and empty collections.
Malformed path, scalar target, or visitor failure returns an error. Aborting/error preserves prior
visitor side effects; no later children run.

## 8. Input adaptation and ordered collections

`IntoNest` implementations are supplied for Nest, object-root Value, and object-root
serde_json::Value. The trait is public and unsealed; custom implementations may add conversion
behavior. Request builder inputs require additional Send/static ownership bounds.

### 8.1. IntoNest::into_nest

`into_nest(self) -> Result<Nest>`

Convert owned data into a checked object-root store.

```rust
let data = serde_json::json!({"name": "Ada"}).into_nest()?;
assert!(data.has("name"));
```

**Returns** Nest moves unchanged; built-in Value/JSON conversion uses default limits and rejects
non-object roots. Custom implementations may return their own conversion errors.

### 8.2. IntoNest::into_nest_with

`into_nest_with(self, limits: ParseLimits) -> Result<Nest> where Self: Sized`

Convert built-in inputs using a selected final policy. The trait default revalidates the result of
into_nest.

```rust
let data = serde_json::json!({"name": "Ada"}).into_nest_with(ParseLimits::default())?;
assert!(data.has("name"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Final store policy; built-in Value/JSON overrides avoid imposing the default policy first. |

**Returns** An object-root Nest under the selected policy, or conversion/root/budget error. A custom
implementation using the default method may also encounter its into_nest policy first.

`DataMap<K, V>` aliases `indexmap::IndexMap<K, V>`; `DataSet<T>` aliases `indexmap::IndexSet<T>`.
They add no wrapper methods or Nest budgets. Their constructors, `insert`, borrowed `get`, indexed
`get_index`, `len`, `clear`, and iterators are the underlying collection API. Use `shift_remove`
when removal must retain the relative order of survivors.

```rust
let mut map: DataMap<String, u64> = DataMap::new();
map.insert("Ada".into(), 7);
assert_eq!(map.get("Ada"), Some(&7));
let set: DataSet<_> = ["rust", "events", "rust"].into_iter().collect();
assert_eq!(set.iter().copied().collect::<Vec<_>>(), ["rust", "events"]);
```

Map entries and unique set values retain insertion order. Iterator `find`, `filter`, and `map`
provide
ordinary Rust collection operations; no callable store/controller interface exists.

### 8.3. should_be_an_array

`should_be_an_array(object: &DataMap<String, Value>) -> bool`

Check whether a nonempty object has every canonical numeric key 0 through len−1.

```rust
let object = make_object(vec!["rust".into(), "events".into()]);
assert!(should_be_an_array(&object));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `object` | `&DataMap<String, Value>` | Borrowed object keys; insertion order does not determine the result. |

**Returns** True only for a nonempty complete canonical index set; empty/sparse/noncanonical keys
return false.

### 8.4. make_array

`make_array(object: DataMap<String, Value>) -> Vec<Value>`

Move values out in lexicographic key order. This explicit helper differs from Nest's numeric
normalization.

```rust
let object = [("b".into(), 2.into()), ("a".into(), 1.into())]
    .into_iter()
    .collect();
assert_eq!(make_array(object)[0].as_i64(), Some(1));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `object` | `DataMap<String, Value>` | Consumed literal-keyed map. |

**Returns** Owned values sorted by string key; keys are discarded. Lexicographic numeric-looking
keys put `10` before `2`; no normalization predicate is required.

### 8.5. make_object

`make_object(array: Vec<Value>) -> DataMap<String, Value>`

Move array values under their original decimal index names.

```rust
let object = make_object(vec!["rust".into(), "events".into()]);
assert_eq!(object["1"].as_str(), Some("events"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `array` | `Vec<Value>` | Consumed dense values. |

**Returns** An ordered map keyed `"0"`, `"1"`, etc. Empty input yields an empty map; no value
cloning or deep merge occurs.

## Related

[Data and paths guide](../guides/data-and-paths.md) · [Parsers](parsers.md) · [Request](request.md)
· [Response](response.md) · [API index](README.md)
