# Update data and nested fields

An operation often needs both a literal field name and a path into an object. `Nest` keeps those
choices separate, so a field named `profile.name` does not accidentally become a nested write.

## Write, then verify

Start inside a function returning `stackpress_lib::Result<()>`. Create a store and add one nested
field:

```rust
use stackpress_lib::{Nest, Value};

let mut data = Nest::new();
data.set_path("profile.name", "Ada")?;
assert_eq!(data.path_as::<String>("profile.name")?, Some("Ada".into()));
```

`set_path` creates the missing object containers; `path_as` deserializes the present value into the
requested type. A missing value returns `Ok(None)`, while an incompatible present value returns an
error.

Now add a literal key and a present null:

```rust
data.set("profile.name", "literal")?
    .set("middle_name", Value::Null)?;
assert_eq!(
    data.get("profile.name").and_then(Value::as_str),
    Some("literal")
);
assert!(data.has("middle_name"));
assert!(!data.has("missing"));
```

`get` reads the literal dotted key. `has` checks presence, so null remains different from absence.
`path` and `has_path` return absence for malformed paths too; use `path_as` or a parsed `DataPath`
when you need a parse failure.

## Append without shifting identities

Use typed segments for literal punctuation or append operations. Dot strings have no escape or `[]`
grammar:

```rust
use stackpress_lib::{DataPath, PathSegment};

let next_tag = DataPath::from_segments([PathSegment::Key("tags".into()), PathSegment::Append])?;
data.set_path(&next_tag, "rust")?
    .set_path(&next_tag, "events")?;
data.remove_path("tags.0")?;
assert_eq!(data.path_as::<String>("tags.1")?, Some("events".into()));
```

The first two writes produce a dense array. Removing index 0 converts that container to a
numeric-keyed object, preserving index 1. Appending to sparse data uses one past its highest
canonical numeric key. Large sparse indices do not allocate an array up to that index.

## Overlay and export deliberately

`merge` replaces whole top-level fields from the incoming store. It does not recursively merge
nested configuration:

```rust
data.merge(Nest::from_object(
    serde_json::json!({"profile": {"active": true}}),
)?)?;
assert!(data.path("profile.name").is_none());
let json = data.to_json()?;
```

The new `profile` replaces the old object. JSON export succeeds only for JSON-compatible values
within the store's chosen limits; bytes, dates, files, and shared natives must be converted
explicitly first.

You can now distinguish literal keys from paths, preserve missing/null, and update sparse
collections without shifting identities. See [data API](../api/data.md) for limits, cloning,
visitors, and collection helpers, or [Parsing input](parsing-input.md) for bracket paths at input
boundaries.
