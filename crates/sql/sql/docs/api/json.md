# JSON documents, scalars, and paths

These types keep JSON values and path components distinct from SQL syntax. Source:
[json.rs](../../src/json.rs); dialect rendering is owned by the query renderer.

```rust
use stackpress_sql::{JsonPath, JsonPathOptions, JsonScalar, JsonSegment, JsonValue, Result};

fn main() -> Result<()> {
    let document = JsonValue::Object(vec![(
        "enabled".into(),
        JsonValue::Scalar(JsonScalar::Bool(false)),
    )]);
    assert_eq!(document.to_json()?, r#"{"enabled":false}"#);
    let keys = JsonPath::parse("settings.01", JsonPathOptions::default())?;
    assert!(matches!(&keys.segments()[1], JsonSegment::Key(key) if key == "01"));
    let index = JsonPath::new([JsonSegment::Key("tags".into()), JsonSegment::Index(0)]);
    assert_eq!(index.segments().len(), 2);
    Ok(())
}
```

Encoding preserves false, and convenience parsing keeps `01` a literal key. Array access uses
the explicit Index variant.

Contents: [Values](#1-jsonscalar-and-jsonvalue) · [Paths](#2-paths-and-options) ·
[Query behavior](#3-query-semantics)

## 1. JsonScalar and JsonValue

Both types are Clone/Debug/PartialEq. `JsonScalar` variants are `Null`, `Bool(bool)`,
`Integer(i64)`, `Real(f64)`, and `String(String)`. JSON null is not SQL NULL; booleans are
not integer 0/1. Reals must be finite.

`JsonValue` variants are `Scalar(JsonScalar)`, `Array(Vec<JsonValue>)`, and
`Object(Vec<(String, JsonValue)>)`. Object order is retained and duplicate keys are rejected.
`to_json(&self)` takes no additional parameters.

**Returns** `Result<String>` with bounded JSON text. Encoding accepts depth through 64, at most
10,000 nodes, and at most 1 MiB encoded text. Invalid duplicate keys/nonfinite numbers and Limit
errors are returned rather than overwritten or coerced. Use `SqlValue::json` for explicit text
encoding, or `WriteValue::Json` for dialect-aware write validation.

## 2. Paths and options

`JsonSegment` is Clone/Debug/Eq/PartialEq, with `Key(String)` and `Index(u32)`. Index must not
exceed i32::MAX when validated. Keys retain punctuation and numeric-looking text.

`JsonPathOptions` is Copy/Clone/Debug/Eq/PartialEq/Default; its only public field is
`separator: char`, default `'.'`. Parsing is key-only. A control, alphanumeric, quote,
backslash, bracket, or `$` separator is invalid.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `JsonPath::new(segments)` | `impl IntoIterator<Item = JsonSegment>` | Own typed keys/indices; empty selects root. |
| `JsonPath::parse(path, options)` | `&str`, `JsonPathOptions` | Split a nonempty key-only convenience path. |
| `JsonPath::segments(&self)` | none | Inspect components in order. |

**Returns** `new` returns Self with render-time validation; `parse` returns `Result<JsonPath>`
and checks invalid separators, empty components, depth/size, and index bounds. `segments` returns
`&[JsonSegment]`. JsonPath is Clone/Debug/Default/Eq/PartialEq with private components; Default
is the root. Paths allow at most 64 components and 16 KiB encoded path text.

## 3. Query semantics

`Expr::json` extracts; `json_eq` / `where_json` match exact scalar kinds; `json_contains` /
`where_json_contains` test direct membership in the selected array. Missing paths and SQL NULL
do not match JSON null. Membership does not perform recursive object containment.

SQLite distinguishes JSON integer and real kinds; PostgreSQL/MySQL equate numbers 1 and 1.0.
PostgreSQL JSONB rejects NUL strings/keys. SQLite reads JSON as text and the row decoder does not
automatically parse arbitrary text. See [Query JSON](../guides/query-json.md) for the write/read
flow and [strict values](values.md) for explicit encoders.
