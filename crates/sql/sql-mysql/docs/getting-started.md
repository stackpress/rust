# Read a bound value from MySQL

This walkthrough opens an existing database, sends one bound name and prints its decoded
value. It creates no table or database. Use a development database to which you already have
permission to connect.

## Prepare a consumer

Use Rust 1.96 and a Tokio binary. These path entries are relative to the repository root;
adjust them relative to your consumer's `Cargo.toml`:

```toml
[dependencies]
stackpress-sql = { path = "crates/sql/sql" }
stackpress-sql-mysql = { path = "crates/sql/sql-mysql" }
tokio = { version = "1", features = ["macros", "rt"] }
```

Set `APP_DB_HOST`, `APP_DB_NAME`, `APP_DB_USER` and `APP_DB_PASSWORD` in your application's
environment using your secret-handling process. The default port is 3306; TLS verifies
the server identity by default. For a private CA or an explicitly plaintext local server,
choose the policy in the [TLS reference](api/tls.md) before opening. These application
variables are separate from the repository's native-test gates.

## Query and close

Put this program in `src/main.rs`:

```rust
use stackpress_sql::{DialectId, Engine, Result, Sql};
use stackpress_sql_mysql::{MysqlDatabase, MysqlOptions};

async fn read_name(options: MysqlOptions) -> Result<()> {
    let engine = Engine::new(MysqlDatabase::open(options).await?);
    let work: Result<()> = async {
        // Keep the name in an owned bind, separate from SQL text.
        let rows = engine
            .fetch(&Sql::literal(DialectId::Mysql, "SELECT ? AS name").bind("Ada"))
            .await?;
        let name = rows[0].try_get::<String>("name")?;
        assert_eq!(name, "Ada");
        println!("profile-name: {name}");
        Ok(())
    }
    .await;

    // Await close on either path, retaining any primary error.
    let cleanup = engine.close().await;
    match (work, cleanup) {
        (Ok(()), cleanup) => cleanup,
        (Err(primary), Ok(())) => Err(primary),
        (Err(primary), Err(cleanup)) => Err(primary.with_cleanup(cleanup)),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // The application resolves secrets; the adapter never reads these variables.
    let options = MysqlOptions::new(
        std::env::var("APP_DB_HOST")?,
        std::env::var("APP_DB_NAME")?,
        std::env::var("APP_DB_USER")?,
        std::env::var("APP_DB_PASSWORD")?,
    );
    read_name(options).await?;
    Ok(())
}
```

The named helper owns the Engine lifecycle. Engine renders the explicit MySQL statement,
checks its binds and returns owned rows. The completion match awaits close even if the query
fails, preserving a secondary cleanup error.

## Verify the result

Run `cargo run` in your consumer. With a reachable, correctly authenticated database, it prints:

```text
profile-name: Ada
```

That output confirms a real prepared bind and result decode. The example then closes its own
pool while the runtime remains alive. It does not verify TLS against every server or install
an application schema.

## What happened

`MysqlOptions::new` selected explicit connection fields and finite defaults.
`MysqlDatabase::open` validated them and connected eagerly; `Engine::new` wrapped that owner
in the core execution API. No operation retries automatically.

Read [concepts](concepts.md) before adding concurrency, and use
[configuration switching](guides/switch-backends.md) to keep operation code on `Engine`.
