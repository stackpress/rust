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

## Shipped Idea source

The original `schema.idea` is available as `stackpress_framework_api::SCHEMA_SOURCE`;
`codegen::SOURCE` remains an alias. Applications import `stackpress-api/schema.idea`
through the opt-in [package loader](../../idea/idea-transformer/docs/guides/package-imports.md).
That source imports `stackpress-session/schema.idea`, retaining the complete Profile/Auth
declarations. The aggregate catalog additionally resolves `stackpress/stackpress.idea`.
[Composition tests](../framework/tests/package_sources.rs) prove nested imports and G19
application extensions. API codegen's existing external Profile adaptation remains until
the separately gated G50 generated-model work; composition does not yet replace these
hand-written runtime models or promise full source-model generation.

The native [source-model mechanics](../framework-schema/docs/source-models.md) now live
in framework-schema (`source` feature); `models::{Value,Record,Column,Kind}` re-export
those exact types. Source action errors are shared at `stackpress_framework_sql::source::ActionError`
and retain the existing API/session paths. The unchanged SQLite and opt-in live suites
exercise the extracted codecs. This extraction does not change the runtime model catalog
or API's existing Typed codegen adapter; source generation is a separately released stage.
