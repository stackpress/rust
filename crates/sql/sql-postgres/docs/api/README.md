# PostgreSQL adapter API

The crate exports exactly three public types. Engine, Database, TransactionSession,
QueryLimits, SqlValue, Row and SqlError belong to the [SQL core](../../../sql/docs/api/README.md).

| Surface | Reference |
| --- | --- |
| `PostgresOptions` | [Fields, defaults, validation and secret handling](options.md) |
| `PostgresTls` | [TLS modes and serialized shapes](tls.md) |
| `PostgresDatabase` | [Eager open, trait methods and lease policy](database.md) |
| Native values | [Strict codec matrix and statement outcomes](codecs.md) |
| Budgets and failures | [Limits, deadlines and disclosure](errors-and-limits.md) |

Use [getting started](../getting-started.md) for a complete opening/query/closing example.
No public driver handles, migration engine or generic connection-URL parser are exposed.
