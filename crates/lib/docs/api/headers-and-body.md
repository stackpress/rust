# Headers and Body API

`Headers` owns checked case-insensitive multi-value fields; `Body` preserves buffered
representations. Both are clonable data, with no transport ownership. Sources:
[headers.rs](../../src/headers.rs), [body.rs](../../src/body.rs).

```rust
use stackpress_lib::{Body, Headers, ParseLimits, Result, Value};

let mut headers = Headers::from_pairs([("accept", "application/json")])?;
```

Run method snippets independently inside a function returning `Result<()>`, with this setup.
Response-owned headers require [headers_mut](response.md#3-guarded-mutable-access) while output is
open.

Contents: [Headers](#1-headers) · [Body](#2-body)

## 1. Headers

Names are normalized to lowercase. Insertion order of names and occurrence order of values are
retained; `Set-Cookie` is never comma-folded. Mutation checks default limits (1 MiB aggregate
name/value bytes, 10,000 occurrences), independently of a request/response policy.

### 1.1. Headers::new

`Headers::new() -> Self`

Create an empty field collection; `Default` is equivalent.

```rust
let headers = Headers::new();
assert!(headers.is_empty());
```

**Returns** An empty owned `Headers`, without validation or I/O.

### 1.2. Headers::from_pairs

`Headers::from_pairs<K: AsRef<str>, V: AsRef<str>>(pairs: impl IntoIterator<Item = (K, V)>) -> Result<Self>`

Create headers from repeated name/value pairs; each pair appends an occurrence.

```rust
let headers = Headers::from_pairs([("Set-Cookie", "a=1"), ("set-cookie", "b=2")])?;
assert_eq!(headers.get_all("SET-COOKIE").unwrap().len(), 2);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `pairs` | `impl IntoIterator<Item = (K, V)>` | Names/values convertible to borrowed strings. |

**Returns** New headers, or an error for invalid names/values or default aggregate limits. No
partial collection is returned.

### 1.3. Headers::get

`get(&self, name: &str) -> Option<&str>`

Read the first occurrence without folding values.

```rust
assert_eq!(headers.get("ACCEPT"), Some("application/json"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Case-insensitive field name. |

**Returns** A borrowed first value, or `None` when absent.

### 1.4. Headers::get_all

`get_all(&self, name: &str) -> Option<&[String]>`

Read all occurrences in their supplied order.

```rust
assert_eq!(headers.get_all("accept").unwrap().len(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Case-insensitive field name. |

**Returns** A borrowed value slice, or `None` when absent.

### 1.5. Headers::has

`has(&self, name: &str) -> bool`

Test normalized name presence, including an empty value.

```rust
assert!(headers.has("Accept"));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Case-insensitive field name. |

**Returns** A boolean; no value parsing occurs.

### 1.6. Headers::remove

`remove(&mut self, name: &str) -> Option<Vec<String>>`

Remove all occurrences while retaining other names' order.

```rust
let old = headers.remove("ACCEPT");
assert_eq!(old.unwrap(), ["application/json"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Case-insensitive field name. |

**Returns** The owned removed list, or `None` when absent; no validation error.

### 1.7. Headers::len

`len(&self) -> usize`

Count distinct normalized names, not occurrences.

```rust
assert_eq!(headers.len(), 1);
```

**Returns** The distinct name count.

### 1.8. Headers::is_empty

`is_empty(&self) -> bool`

Test whether any normalized names are present.

```rust
assert!(!headers.is_empty());
```

**Returns** True only when there are no names.

### 1.9. Headers::entries

`entries(&self) -> impl Iterator<Item = (&String, &Vec<String>)>`

Inspect normalized names and their value lists in insertion order.

```rust
let names: Vec<_> = headers.entries().map(|(name, _)| name.as_str()).collect();
assert_eq!(names, ["accept"]);
```

**Returns** A borrowing iterator; it exposes no writable field storage.

### 1.10. Headers::set

`set(&mut self, name: &str, value: &str) -> Result<&mut Self>`

Replace all occurrences of a field.

```rust
headers.set("accept", "text/plain")?;
assert_eq!(headers.get_all("accept").unwrap().len(), 1);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Nonempty ASCII token, at most 8,192 bytes. |
| `value` | `&str` | Value without control bytes except horizontal tab; CR/LF and DEL are rejected. |

**Returns** The same mutable headers for chaining. Invalid framing or exceeded default
byte/occurrence budgets returns an error and preserves the collection.

### 1.11. Headers::append

`append(&mut self, name: &str, value: &str) -> Result<&mut Self>`

Append a separate occurrence to an existing or new field.

```rust
headers.append("accept", "text/plain")?;
assert_eq!(headers.get_all("accept").unwrap().len(), 2);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Nonempty ASCII token, at most 8,192 bytes. |
| `value` | `&str` | Value without control bytes except horizontal tab; CR/LF and DEL are rejected. |

**Returns** The same mutable headers for chaining. Invalid framing or exceeded default
byte/occurrence budgets returns an error and preserves the collection.

## 2. Body

Public variants are `Empty`, `Text(String)`, `Bytes(Vec<u8>)`, and `Value(Value)`. Default is
`Empty`; it differs from `Value(Value::Null)`. Cloning copies ordinary content and shares only
explicit `SharedNative` payloads inside rich values.

### 2.1. Body::type_name

`type_name(&self) -> &'static str`

Inspect representation without coercing it.

```rust
assert_eq!(Body::Bytes(vec![0, 255]).type_name(), "bytes");
assert_eq!(Body::Value(Value::Bytes(vec![0])).type_name(), "byte-value");
```

**Returns** A label: `empty`, `string`, `bytes`; rich values use `null`, `boolean`, `number`,
`string`, `array`, `object`, `date`, `byte-value`, `file`, or `shared`. No payload is formatted.

### 2.2. Body::validate

`validate(&self, limits: &ParseLimits) -> Result<()>`

Check a buffer before accepting it at a boundary.

```rust
Body::Bytes(vec![0, 255]).validate(&ParseLimits::default())?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `&ParseLimits` | Text/raw bytes check byte length; rich values check recursive budgets too. |

**Returns** Unit on success, or a limit error. `Empty` always validates; validation does not convert
non-JSON values to JSON.

## Related

[Requests and responses guide](../guides/requests-and-responses.md) · [Session](session.md) ·
[Response](response.md) · [Data](data.md) · [API index](README.md)
