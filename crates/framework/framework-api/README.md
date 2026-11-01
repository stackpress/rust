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

PostgreSQL executes Application q/like and Session like with mixed ASCII case and counts,
using the real Profile/Application/Session `install()` paths and source codecs. API Session
ignores q, following its [source query contract](src/models/query.rs). This is direct-store
evidence; the SQLite test additionally covers the registered application-search event.

MySQL successfully creates the owned database, but Profile `install()` returns
**Unsupported: dialect does not support transactional DDL plans** before seeding or searches.
That opted-in case stays failing. The [SQL Engine contract](../../sql/sql/src/engine.rs) is
preserved, with no hand-written schema or plan bypass. The exact owned database is dropped
on this failure as well as on success. MySQL execution and arbitrary collation/Unicode,
JSON-query, aggregate-provider or production-load behaviour are not established by these tests.
