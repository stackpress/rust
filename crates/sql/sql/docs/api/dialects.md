# Dialects and capability flags

`DialectId` selects rendering independently of adapter availability. Read capabilities without
opening a database. Source: [dialect.rs](../../src/dialect.rs) and [query.rs](../../src/query.rs).

```rust
use stackpress_sql::DialectId;

fn main() {
    let sqlite = DialectId::Sqlite.capabilities();
    assert!(sqlite.returning);
    assert!(!sqlite.alter_constraints);
    assert!(!DialectId::Mysql.capabilities().transactional_ddl);
}
```

Flags describe renderer support; they neither inspect native versions nor guarantee a supplied
definition is valid. See the runnable [capabilities example](../../examples/capabilities.rs).

## 1. DialectId

Variants are `Sqlite`, `Postgres`, and `Mysql`. The enum is Copy, Clone, Debug, Eq, and PartialEq.
`capabilities(self)` takes no additional parameters and is a const function.

**Returns** A copied `DialectCapabilities` value. SQLite/PostgreSQL quote names with double quotes;
MySQL uses backticks. SQLite/MySQL placeholders are `?`; PostgreSQL uses `$1`, `$2`, and so on.
A rendered Query retains this identity and rejects rendering for a different dialect.

## 2. DialectCapabilities

This Copy/Clone/Debug/Eq/PartialEq struct exposes every flag as a public bool:

| Field | SQLite | PostgreSQL | MySQL |
| --- | --- | --- | --- |
| `returning` | true | true | false |
| `transactional_ddl` | true | true | false |
| `alter_constraints` | false | true | true |
| `full_join` | true | true | false |
| `cascade` | false | true | false |
| `idempotent_indexes` | true | true | false |
| `distinct_json_number_kinds` | true | false | false |
| `text_nul` | true | false | true |

`alter_constraints` covers native field type/default/nullability and primary/foreign-key changes.
`cascade` covers native DROP/TRUNCATE CASCADE. The JSON flag distinguishes integer from real
equality, not general JSON availability.

SQLite RETURNING requires 3.35 or later, RIGHT/FULL JOIN 3.39 or later. MySQL idempotent CREATE
keeps indexes inline in CREATE TABLE IF NOT EXISTS. PostgreSQL identifier segments are limited
to 63 UTF-8 bytes; MySQL segments to 64 characters. All names also pass common validation.

The matrix has operation-specific limits: see [Create](create.md), [Alter](alter.md),
[table commands](table-command.md), and [JSON](json.md). SQLite's production adapter is
[documented separately](../../../sql-sqlite/docs/README.md). Network renderer tests do not
supply production PostgreSQL/MySQL adapters.
