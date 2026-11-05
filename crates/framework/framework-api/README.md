# API source-search execution

The API crate exposes source Application/Session models and stores behind `sql`.
[SQLite tests](tests/sql.rs) and [dialect adapters](src/sql/adapter.rs) pin PostgreSQL ILIKE
versus SQLite/MySQL LIKE. The new [live executable](tests/live_search.rs) runs actual actions
using the [shared test-only fixture](../framework-session/tests/live/support.rs).

Set `STACKPRESS_TEST_POSTGRES_URL` or `STACKPRESS_TEST_MYSQL_URL` privately, then run
`cargo nextest run -p stackpress-framework-api --test live_search --features sql --locked --offline --test-threads 2 --no-fail-fast --success-output final --failure-output final`.
Unset gates are ignored/skipped. The [live-search guide](../framework-session/docs/live-search.md)
specifies the localhost/port/identity restrictions, exact-name CREATE/DROP ownership and
parent/admin watchdogs. No listener or public fixture API is added.

PostgreSQL and MySQL execute Application q/like and Session like with mixed ASCII case and counts,
using the real Profile/Application/Session `install()` paths and source codecs. API Session
ignores q, following its [source query contract](src/models/query.rs). This is direct-store
evidence; the SQLite test additionally covers the registered application-search event.

G48 uses [explicit schema setup](../framework-sql/docs/api/setup.md): MySQL runs source
install/upgrade statements in order, accepting non-atomic partial state; PostgreSQL/SQLite
retain atomic setup. The ordinary [SQL Engine contract](../../sql/sql/src/engine.rs) and
MySQL transaction-DDL refusal stay unchanged. Tests also execute real store upgrade,
purge with `cascade=true` (forwarded only on PostgreSQL under G48) and dependency-ordered
uninstall. Exact owned databases are dropped on success and
failure. Arbitrary collation/Unicode, JSON-query, aggregate-provider or production-load
behaviour is not established by these tests.

`eq` takes one value per field (G53). Arrays are misuse and retain the original indexed-path semantics: `{"eq":{"id":["a","b"]}}`
produces indexed-path AND predicates rather than an OR group. The [maintained SQL test](tests/sql.rs)
pins that behavior against upstream `stackpress-sql/src/transform/store/select.ts:567–570`.

## Shipped Idea source

The original `schema.idea` is available as `stackpress_framework_api::SCHEMA_SOURCE`;
`codegen::SOURCE` remains an alias. Applications import `stackpress-api/schema.idea`
through the opt-in [package loader](../../idea/idea-transformer/docs/guides/package-imports.md).
That source imports `stackpress-session/schema.idea`, retaining the complete Profile/Auth
declarations. The aggregate catalog additionally resolves `stackpress/stackpress.idea`.
[Composition tests](../framework/tests/package_sources.rs) prove nested imports and G19
application extensions. [Generated families](docs/source-models.md) now replace the old
three-model Typed projection: config()/normalize() retain the full four-model composition,
and register_with<M> binds the application's generated family. Existing schema/store paths
remain constructor adapters over generated defaults. Portable UI and native schemas/stores
are emitted into separate partitions by the [checkpoint tool](../../../examples/api-checkpoint/tool/src/main.rs).
