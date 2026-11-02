# Run the live source searches

The [session executable](../tests/live_search.rs) mirrors the SQLite G46 regression through
real Profile/Auth actions. The [API executable](../../framework-api/tests/live_search.rs)
uses the same owned-database harness for Application/Session actions. These are opt-in tests,
separate from renderer snapshots and from aggregate command-provider integration.

## Select the local fixture

Set `STACKPRESS_TEST_POSTGRES_URL` or `STACKPRESS_TEST_MYSQL_URL` privately to the documented
local passwordless endpoint. This harness accepts localhost or 127.0.0.1, PostgreSQL port
5432 with user postgres and database postgres, or MySQL port 3306 with user root and
database mysql. It never prints the URL. The exact documented URL forms are the only accepted
forms; no alternate credentials, query options or ambient connection fields are adopted.

Run `cargo nextest run -p stackpress-framework-session -p stackpress-framework-api --test live_search --features stackpress-framework-session/sql,stackpress-framework-api/sql --locked --offline --test-threads 2 --no-fail-fast --success-output final --failure-output final`.
An unset variable produces ignored/skipped cases, never an early-return pass. Always retain
the successful-output records so every owned database name has its matching drop confirmation.

## Ownership and deadlines

The [shared test-only owner](../tests/live/support.rs) reuses the existing SQL discovery and
child-watchdog helpers. The adapter database-owner structs are private to their executables;
no public testing API or crate is introduced. Each enabled case creates one unique
`stackpress_g46_*` database with CREATE DATABASE, without IF NOT EXISTS. CREATE acknowledgement
arms cleanup before the administrative socket closes. Only that generated exact name can be
dropped; successful DROP disarms cleanup before socket close. No database is enumerated,
adopted, swept or altered outside that owned name, and no role/global setting is changed.

The parent kills and reaps a child after 30 seconds, then performs exact-name cleanup on
success, failure or unwind. Administrative operations have independent 9-second IO and
10-second wall deadlines. The forced-stall control must fail with the watchdog's exact
timeout result. No listener is started; only the already-running local database is contacted.

## Current execution evidence

Both PostgreSQL and MySQL execute Profile q/like, Auth-to-Profile like, Application q/like and API Session
like successfully, including mixed ASCII case, ordered identities and counts. All tables use
the stores' real `install()` paths and source codecs. API Session has no q search fields and
continues to ignore q, as declared in its [query contract](../../framework-api/src/models/query.rs).
The proof does not claim arbitrary Unicode matching, other collations, every JSON operator,
every possible schema upgrade or production-load acceptance. The same cases also exercise
real store upgrade with a nullable integer column, purge with `cascade=true` and
dependency-ordered uninstall. G48 omits MySQL CASCADE at these callers; PostgreSQL
retains the flag and SQLite retains its DELETE mapping. These standalone actions
do not discover or tear down other tables: the test uninstalls children before parents.

G48 selects [explicit schema setup](../../framework-sql/docs/api/setup.md) for the actual
store install/upgrade actions: MySQL executes the same statements in order outside a
transaction and can retain partial state after failure; PostgreSQL/SQLite stay atomic.
The ordinary Engine::execute_plan and MySQL transaction-DDL refusals remain unchanged.
No handwritten table replaces source DDL. All successful and failing cases confirm the
drop of their exact owned database. The [aggregate live setup proof](../../framework/tests/live_setup.rs)
separately tests real adjacent generated upgrades, partial failures and cancellation.
