# Read a bound value from SQLite

A query builder alone cannot show that data survives a real database connection. Here you
will create a temporary file, install one table, insert Ada with a bind and read her name
back. The temporary directory stays alive until the pool has closed.

## Prepare a consumer

Use the repository's Rust 1.96 toolchain and a small Tokio binary. Add path dependencies on
`stackpress-sql` and `stackpress-sql-sqlite`, plus Tokio with `macros` and `rt`, and `tempfile`.
The path entries below are relative to the repository root:

```toml
[dependencies]
stackpress-sql = { path = "crates/sql/sql" }
stackpress-sql-sqlite = { path = "crates/sql/sql-sqlite" }
tokio = { version = "1", features = ["macros", "rt"] }
tempfile = "3"
```

Resolve the path entries relative to your consumer's `Cargo.toml`. The examples in these
docs use a disposable consumer outside the workspace with equivalent absolute paths.

## Create, query and close

Put this program in the consumer's `src/main.rs`. `create_if_missing(true)` opts into file
creation; `execute_plan` performs the separate table installation.

```rust
use stackpress_sql::{Create, DialectId, Engine, Field, Insert, Select, SqlType, SqlValue};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let engine = Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(root.path().join("profiles.sqlite")).create_if_missing(true),
        )
        .await?,
    );

    // Keep installation and application queries explicit.
    let work: stackpress_sql::Result<()> = async {
        let plan = Create::new("profile")
            .add_field(Field::new("id", SqlType::Integer).not_null())
            .add_field(Field::new("name", SqlType::Text).not_null())
            .primary_key(["id"])
            .render(DialectId::Sqlite)?;
        engine.execute_plan(&plan).await?;
        engine
            .execute(
                &Insert::new("profile")
                    .columns(["id", "name"])
                    .values([SqlValue::Integer(1), "Ada".into()]),
            )
            .await?;
        let rows = engine
            .fetch(&Select::new(["name"]).from("profile").where_eq("id", 1))
            .await?;
        assert_eq!(rows.len(), 1);
        let name = rows[0].try_get::<String>("name")?;
        assert_eq!(name, "Ada");
        println!("profile-name: {name}");
        Ok(())
    }
    .await;

    // Close on both paths while this runtime and the temporary directory still exist.
    let cleanup = engine.close().await;
    match (work, cleanup) {
        (Ok(()), cleanup) => cleanup?,
        (Err(primary), Ok(())) => return Err(primary.into()),
        (Err(primary), Err(cleanup)) => return Err(primary.with_cleanup(cleanup).into()),
    }
    Ok(())
}
```

The insert owns two binds, and the select owns the ID bind. Neither value is pasted into SQL.
The final match preserves a query failure while awaiting cleanup. If both fail,
`with_cleanup` retains the secondary failure without replacing the original cause.

## Verify the result

Run the consumer with `cargo run`. Its output is:

```text
profile-name: Ada
```

This confirms that the adapter opened a real file, executed the table plan and decoded the
stored name. `Engine::close().await` finishes before the runtime and temporary directory drop.

## What happened

`SqliteOptions` selected a file and opted into creation. `SqliteDatabase::open` validated
that policy and opened a native pool eagerly; it did not install schema. Engine rendered
each explicit operation for SQLite and returned owned rows.

You now have a complete file lifecycle. Read [concepts](concepts.md) before adding concurrent
work or transactions, and use the [service guide](guides/server-service.md) to share one
Engine with server operations. Builder and value details live in the
[SQL core docs](../../sql/docs/README.md).
