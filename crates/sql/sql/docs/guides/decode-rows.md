# Decode owned rows into application data

A query result should remain usable after its connection lease ends. `Row` owns its cells;
application code can turn those cells into a DTO with strict, fallible lookups.

## Decode a profile

Define the application type and one conversion function. This program checks optional text
without converting an integer into a string:

```rust
use stackpress_sql::{Result, Row, SqlValue};

#[derive(Debug, PartialEq)]
struct Profile {
    id: i64,
    display_name: Option<String>,
}

fn decode_profile(row: &Row) -> Result<Profile> {
    Ok(Profile {
        id: row.try_get("id")?,
        display_name: row.try_get("display_name")?,
    })
}

fn main() -> Result<()> {
    let row = Row::new(vec![
        ("id".into(), 7.into()),
        ("display_name".into(), SqlValue::Null),
    ]);
    assert_eq!(
        decode_profile(&row)?,
        Profile {
            id: 7,
            display_name: None
        }
    );
    Ok(())
}
```

`Option<String>` accepts SQL NULL as None and delegates every non-NULL value to strict text
decoding. Missing columns, incompatible types, and integer overflow return `ErrorKind::Decode`.
Decode failure does not silently replace an invalid cell with a default.

## Handle joins and metadata

Alias duplicate names in the projection when possible. Otherwise, use `try_get_at` by ordinal;
`try_get` rejects both missing and ambiguous names. Inspect `cells()` for ordered values and
`columns()` for borrowed metadata. A native declared type is diagnostic, not a coercion rule.

For a custom scalar, implement `FromSqlValue::from_sql(&SqlValue) -> Result<Self>`. Return owned
contents and a typed Decode error for incompatible storage. Row constructors and metadata
attachment are also available for adapter authors; applications normally receive rows from IO.

You can now build a DTO without a live connection or generated mapping layer. See
[rows and decoding](../api/rows.md) and [strict values](../api/values.md) for exact supported types.
