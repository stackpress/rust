# stackpress-sql-sqlite

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Execute Stackpress SQL through a native Tokio/SQLx SQLite adapter with finite pool and query budgets. Applications own the database file, runtime, transactions and shutdown.

```rust
use stackpress_sql_sqlite::SqliteOptions;
let options = SqliteOptions::file("application.sqlite").create_if_missing(true);
let _ = options;
```

Options alone open no file. Pass them to `SqliteDatabase::open` inside a Tokio runtime, then wrap the database in `stackpress_sql::Engine`.

Licence: [Apache-2.0](LICENSE).
