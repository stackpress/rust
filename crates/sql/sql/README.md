# stackpress-sql

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Build detached SQL queries and render them for SQLite, PostgreSQL or MySQL. Stackpress database adapters implement the owned database and transaction contracts; query construction needs no runtime or database.

```rust
use stackpress_sql::{DialectId, Render, Select};
let query = Select::new(["id"]).from("profile").render(DialectId::Sqlite)?;
assert_eq!(query.sql(), "SELECT \"id\" FROM \"profile\"");
# Ok::<(), stackpress_sql::SqlError>(())
```

Rendering quotes identifiers and keeps bound values separate from SQL text.

Licence: [Apache-2.0](LICENSE).
