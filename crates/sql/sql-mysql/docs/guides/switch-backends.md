# Choose a backend using configuration

Compile every selectable adapter into the application, then return the same `Engine` service
from application-owned startup code. Routes and operations can continue to accept `&Engine`.
Changing configuration selects an already compiled backend; it cannot load an uncompiled plugin.

## Compile the selectable adapters

Use these dependencies in addition to Tokio. Adjust local paths for your consumer:

```toml
[dependencies]
stackpress-sql = { path = "crates/sql/sql" }
stackpress-sql-sqlite = { path = "crates/sql/sql-sqlite" }
stackpress-sql-postgres = { path = "crates/sql/sql-postgres" }
stackpress-sql-mysql = { path = "crates/sql/sql-mysql" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt"] }
```

Alternatively enable facade features `sql-sqlite`, `sql-postgres` and `sql-mysql` and use
`stackpress::sql` / `stackpress::sql_sqlite` / `stackpress::sql_postgres` /
`stackpress::sql_mysql`. Each adapter feature also enables the core SQL facade.

## Own the configuration schema

This complete application-owned schema accepts connection metadata without a password.
Resolve the selected database's password from your secret source and pass it separately to
`open_database` after deserializing with `serde_json::from_str::<AppConfig>`.

```rust
use serde::Deserialize;
use stackpress_sql::{Engine, Result};
use stackpress_sql_mysql::{MysqlDatabase, MysqlOptions, MysqlTls};
use stackpress_sql_postgres::{PostgresDatabase, PostgresOptions, PostgresTls};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AppConfig {
    database: DatabaseConfig,
}

#[derive(Deserialize)]
#[serde(
    tag = "adapter",
    content = "options",
    rename_all = "snake_case",
    deny_unknown_fields
)]
enum DatabaseConfig {
    Sqlite {
        path: PathBuf,
        #[serde(default)]
        create_if_missing: bool,
    },
    Postgres {
        host: String,
        database: String,
        user: String,
        #[serde(default)]
        port: Option<u16>,
        #[serde(default)]
        tls: PostgresTls,
    },
    Mysql {
        host: String,
        database: String,
        user: String,
        #[serde(default)]
        port: Option<u16>,
        #[serde(default)]
        tls: MysqlTls,
    },
}

async fn open_database(config: AppConfig, password: String) -> Result<Engine> {
    // Each branch constructs the same service type from a compiled adapter.
    match config.database {
        DatabaseConfig::Sqlite {
            path,
            create_if_missing,
        } => Ok(Engine::new(
            SqliteDatabase::open(SqliteOptions::file(path).create_if_missing(create_if_missing))
                .await?,
        )),
        DatabaseConfig::Postgres {
            host,
            database,
            user,
            port,
            tls,
        } => {
            let mut options = PostgresOptions::new(host, database, user, password);
            if let Some(port) = port {
                options.port = port;
            }
            options.tls = tls;
            Ok(Engine::new(PostgresDatabase::open(options).await?))
        }
        DatabaseConfig::Mysql {
            host,
            database,
            user,
            port,
            tls,
        } => {
            let mut options = MysqlOptions::new(host, database, user, password);
            if let Some(port) = port {
                options.port = port;
            }
            options.tls = tls;
            Ok(Engine::new(MysqlDatabase::open(options).await?))
        }
    }
}
```

The match constructs real adapter options and returns one core type. Constructors supply
the omitted finite pool/deadline defaults; open validates them before IO. This example's
schema exposes connection identity and TLS only: add application fields explicitly if you
need to configure the [pool budgets](../api/options.md). Unknown fields or adapter names
fail deserialization. There is no package-owned `DatabaseConfig` parser.

## Change only the database section

The SQLite selection is:

```json
{
  "database": {
    "adapter": "sqlite",
    "options": { "path": "data/profiles.sqlite", "create_if_missing": false }
  }
}
```

For MySQL, change that section to:

```json
{
  "database": {
    "adapter": "mysql",
    "options": {
      "host": "db.internal",
      "port": 3306,
      "database": "profiles",
      "user": "profiles_app",
      "tls": { "mode": "verify_identity" }
    }
  }
}
```

The application passes the returned Engine into its existing service registration. Neither
JSON example contains a password. Direct deserialization of `MysqlOptions` is a different
schema and requires its `password` field; its serialization includes the secret.

A backend switch does not copy data, create schema, migrate SQL or make dialect capabilities
identical. Keep operations within the selected dialect's supported builders and strict
[wire values](../api/codecs.md). Generated stores retain the [backend boundaries verified by the shared suite](testing.md#generated-stores).
