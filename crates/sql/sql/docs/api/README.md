# SQL API reference

Use this index when you know the surface you need. These pages cover only public exports;
linked source includes the detailed runnable Rustdoc. Start with [Getting started](../getting-started.md)
for a first query, or [Concepts](../concepts.md) for the execution model.

## Describe and render

Builders own data and perform no IO. DML/raw/table commands render one Query; schema builders
render ordered QueryPlans.

| Surface | Public items |
| --- | --- |
| [Queries and trusted SQL](query.md) | Render, Query, QueryLimits, Sql, TrustedSql. |
| [Dialects](dialects.md) | DialectId, DialectCapabilities. |
| [Identifiers](identifiers.md) | Identifier, TableRef. |
| [Expressions](expressions.md) | Compare, Expr, Selector. |
| [JSON](json.md) | JsonScalar, JsonValue, JsonSegment, JsonPathOptions, JsonPath. |
| [Select](select.md) | Select, SelectData, Join, JoinKind, Sort, Order. |
| [Insert](insert.md) | Insert, InsertData, WriteValue. |
| [Update](update.md) | Update, MutationData. |
| [Delete](delete.md) | Delete; shared MutationData. |

## Describe schema changes

These are caller-supplied definitions, not native catalog snapshots or applied migration state.
Operation-specific native restrictions still apply after common definition validation.

| Surface | Public items |
| --- | --- |
| [Schema values](schema.md) | SqlType, Length, DefaultValue, Field, Index, ReferentialAction, ForeignKey. |
| [Create](create.md) | Create, CreateData, Create::definition. |
| [Alter](alter.md) | Alter, AlterData, AlterOperation. |
| [Plans](query-plan.md) | QueryPlan. |
| [Table commands](table-command.md) | TableCommand. |
| [Schema comparison](schema-diff.md) | TableDefinition, SchemaDiff. |

## Execute and decode

Execution is explicit through an adapter. Rows/results own their data; the SQLite adapter's
[native lifecycle docs](../../../sql-sqlite/docs/README.md) own its pooling and cancellation details.

| Surface | Public items |
| --- | --- |
| [Values](values.md) | SqlValue and strict input conversions. |
| [Rows](rows.md) | Row, FromSqlValue, ColumnMetadata, ColumnMetadataRef, QueryResult. |
| [Engine](engine.md) | Engine ownership, builders, execution, plans, shutdown, diff. |
| [Transaction](transaction.md) | Exclusive composition and consuming completion. |
| [Adapter contracts](database.md) | Database, TransactionSession, SqlFuture, TransactionOptions. |
| [Errors](errors.md) | Result, ErrorKind, SqlError. |

Related: [task guides](../README.md#choose-a-task), [public exports](../../src/lib.rs),
[runnable examples](../../examples/).
