# Queries, rendering, and trusted raw SQL

`Render` produces one checked, owned `Query`; `QueryLimits` sets per-statement budgets.
`Sql` is the explicit caller-authored SQL entry. Source: [query.rs](../../src/query.rs).

```rust
use stackpress_sql::{DialectId, QueryLimits, Render, Result, Sql};

fn main() -> Result<()> {
    let query = Sql::literal(DialectId::Postgres, "SELECT $1")
        .bind(7)
        .render(DialectId::Postgres)?;
    query.validate(QueryLimits::default())?;
    assert_eq!(query.sql(), "SELECT $1");
    assert_eq!(query.binds().len(), 1);
    assert_eq!(query.dialect(), DialectId::Postgres);
    Ok(())
}
```

The raw statement keeps its text and binds separate. Rendering checks its dialect and lexical
subset; the adapter checks native prepared-parameter arity.

Contents: [Render and Query](#1-render-and-query) · [Limits](#2-querylimits) ·
[Raw SQL](#3-raw-statements-and-trust)

## 1. Render and Query

`Render::render(&self, dialect: DialectId) -> Result<Query>` is pure. It rejects invalid shapes,
identifiers, values, resource overflow, and unsupported dialect features before adapter IO.
Implemented by `Select`, `Insert`, `Update`, `Delete`, `TableCommand`, `Sql`, and `Query`.
`Create` and `Alter` instead have inherent methods returning [QueryPlan](query-plan.md).

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `render` | `dialect: DialectId` | Validate and render for the chosen backend. |
| `Query::sql(&self)` | none | Inspect statement text. |
| `Query::binds(&self)` | none | Inspect separate owned values; disclosure is explicit. |
| `Query::dialect(&self)` | none | Inspect the retained identity. |
| `Query::validate(&self, limits)` | `QueryLimits` | Check SQL/bind/retained bind-byte budgets and value representation. |

**Returns** `render` returns an owned `Query` or `SqlError`. Rendering an existing Query clones
it only for the same dialect; mismatch returns Unsupported. Accessors borrow `&str` /
`&[SqlValue]` or copy `DialectId`; `validate` returns `Result<()>` and does not check native arity
or output budgets. Query is Clone and Debug, with private fields and no public unchecked constructor.

## 2. QueryLimits

`QueryLimits` is Copy, Clone, Debug, and Default. Configure its public fields directly:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `sql_bytes` | `usize` | 1,048,576 | UTF-8 statement bytes. |
| `binds` | `usize` | 1,000 | Supplied parameter count. |
| `rows` | `usize` | 10,000 | Retained result rows; zero permits rowless outcomes only. |
| `bytes` | `usize` | 16,777,216 | Retained bind bytes and, separately, result names/values/native declaration bytes. |

Core validates input; adapters enforce output during collection. Limit and nonfinite-value
failures are typed; PostgreSQL NUL text returns Unsupported. Retained-byte accounting excludes
container overhead. Use [Engine::with_limits](engine.md) before sharing. Raising engine limits
does not bypass default renderer construction budgets; plans have no aggregate quota.

## 3. Raw statements and trust

`Sql` and `TrustedSql` are Clone and Debug; TrustedSql also implements Eq/PartialEq. An audited
marker asserts trusted authorship, without escaping, sanitizing, or parsing arbitrary SQL.

```rust
use stackpress_sql::{DialectId, Render, Result, Sql, TrustedSql};

fn main() -> Result<()> {
    let authored = TrustedSql::audited(String::from("SELECT ?"));
    let query = Sql::trusted(DialectId::Sqlite, authored)
        .bind("fixture")
        .render(DialectId::Sqlite)?;
    assert_eq!(query.binds().len(), 1);
    Ok(())
}
```

This string is trusted application code; request values belong in `.bind(...)`. The same marker
can appear in explicitly tagged expression/type/default variants, whose consuming checks differ.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `TrustedSql::audited(sql)` | `impl Into<String>` | Mark owned dynamic text as authored/audited. |
| `Sql::literal(dialect, sql)` | `DialectId`, `&'static str` | Describe static caller-authored SQL. |
| `Sql::trusted(dialect, sql)` | `DialectId`, `TrustedSql` | Describe audited dynamic SQL. |
| `Sql::bind(self, value)` | `impl Into<SqlValue>` | Append one owned value in placeholder order. |

**Returns** Constructors return detached data without validation or IO. `bind` returns Self;
`render` performs validation and returns `Result<Query>`. SQLite/MySQL use anonymous `?`;
PostgreSQL uses native `$n`. Raw SQL is never globally quote-rewritten.

The single-statement gate rejects batches, transaction/connection control, and NUL. Network
backslash-quoted literals and MySQL executable comments are outside the accepted subset. It is
not a complete native grammar checker, permission system, or resource sandbox. InvalidQuery,
Unsupported, or Limit can arise before IO; [adapters](database.md) own arity and execution.

Related: [values](values.md), [errors](errors.md), [concepts](../concepts.md#8-trusted-sql).
