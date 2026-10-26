# Store JSON and query a path

JSON text needs explicit encoding, while path comparisons need a distinction between missing,
JSON null, and SQL NULL. Keep the document and path typed until rendering.

## Encode a document and match one array member

Use `WriteValue::Json` to let the selected dialect validate structured content. Construct array
indices explicitly; a numeric-looking object key remains a key.

```rust
use stackpress_sql::{
    DialectId, Insert, JsonPath, JsonScalar, JsonSegment, JsonValue, Render, Result, Select,
    WriteValue,
};

fn main() -> Result<()> {
    let document = JsonValue::Object(vec![(
        "tags".into(),
        JsonValue::Array(vec![JsonValue::Scalar(JsonScalar::String(
            "reviewer".into(),
        ))]),
    )]);
    let insert = Insert::new("profile")
        .columns(["data"])
        .values([WriteValue::Json(document)]);
    assert_eq!(insert.render(DialectId::Sqlite)?.binds().len(), 1);

    let tags = JsonPath::new([JsonSegment::Key("tags".into())]);
    let lookup = Select::new(["id"]).from("profile").where_json_contains(
        "data",
        tags,
        JsonScalar::String("reviewer".into()),
    );
    let query = lookup.render(DialectId::Sqlite)?;
    assert!(!query.binds().is_empty());
    Ok(())
}
```

The membership test searches direct scalar members of `data.tags`; it does not recursively
search nested objects. Paths and compared values remain data. `where_json` instead compares
one scalar at a path, and `Expr::json` projects an extraction through a `Selector`.

## Preserve the distinctions

`JsonPath::parse("settings.theme", JsonPathOptions::default())` is a key-only convenience.
It rejects empty components; use `JsonPath::default()` for the root and `JsonSegment::Index`
for array positions. Options apply per call, with no global separator state.

`JsonScalar::Null` matches JSON null, not an absent path or a SQL NULL document. Boolean true
is not integer 1. SQLite distinguishes JSON integer/real kinds; PostgreSQL/MySQL treat 1 and
1.0 numerically equal. PostgreSQL JSONB rejects NUL keys/strings. `SqlValue::json` produces
text explicitly; it does not make the row decoder automatically parse returned text.

You can now encode a document and target a precise JSON value without writing path SQL.
See [JSON values and paths](../api/json.md) for budgets and [expressions](../api/expressions.md)
for projection and comparison shapes.
