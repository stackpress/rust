# Build and inspect your first query

A query can be reviewed before a connection exists. In this walkthrough you will build a small
profile lookup, print its SQLite SQL, and prove that the name stays in a separate bind list.

## 1. Add the local dependency

Create an edition-2024 binary crate with Rust 1.96 or later. Add the dependency below to its
`Cargo.toml`, replacing the path with your checkout:

```toml
[dependencies]
stackpress-sql = { path = "/path/to/stackpress/rust/crates/sql/sql" }
```

The Cargo package name uses hyphens; the Rust import uses underscores. Query construction and
rendering use only the core crate, so this consumer needs no runtime or database adapter.

## 2. Describe the lookup

Replace `src/main.rs` with this program. Start with the two projected columns, then choose the
table, filter, and maximum row count:

```rust
use stackpress_sql::{DialectId, Render, Result, Select, SqlValue};

fn main() -> Result<()> {
    let lookup = Select::new(["id", "name"])
        .from("profile")
        .where_eq("name", "Ada")
        .limit(5);

    let query = lookup.render(DialectId::Sqlite)?;
    assert_eq!(lookup.build().limit, Some(5));
    assert_eq!(query.binds(), &[SqlValue::Text("Ada".into())]);
    println!("{}", query.sql());
    println!("bind count: {}", query.binds().len());
    Ok(())
}
```

`lookup` owns a description. Importing `Render` makes `.render(...)` available; rendering checks
that description and produces one owned `Query`. `build()` borrows the original state for
inspection. No method in this program opens a database.

## 3. Verify the result

Run the consumer from its own directory:

```sh
cargo run
```

It prints:

```text
SELECT "id", "name" FROM "profile" WHERE "name" = ? LIMIT 5
bind count: 1
```

The placeholder belongs to the SQL text; `Ada` belongs to the bind list. The assertions check
both the retained limit and the exact text value without sending either to a database.

## What happened

You chose SQL structure with typed builders, then selected a dialect explicitly. Re-rendering
the same builder with `DialectId::Postgres` produces `$1`; MySQL uses backtick-quoted names
and `?`. A rendered `Query` keeps its dialect and cannot be reinterpreted as another dialect.

You can now distinguish a builder, its inspectable state, and a rendered statement. Continue
with [Build a query](guides/build-queries.md), read [Concepts](concepts.md), or run the broader
[build_inspect example](../examples/build_inspect.rs) from the repository root.
