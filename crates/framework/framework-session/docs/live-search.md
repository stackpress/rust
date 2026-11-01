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

PostgreSQL executes Profile q/like, Auth-to-Profile like, Application q/like and API Session
like successfully, including mixed ASCII case, ordered identities and counts. All tables use
the stores' real `install()` paths and source codecs. API Session has no q search fields and
continues to ignore q, as declared in its [query contract](../../framework-api/src/models/query.rs).
The proof does not claim arbitrary Unicode matching, other collations, every JSON operator,
schema upgrade or production-load acceptance.

MySQL connects and creates its owned database, but both executables stop at Profile
`install()` with **Unsupported: dialect does not support transactional DDL plans**, before
any source search. [Profile install](../src/storage/actions.rs) uses the existing
[Engine plan contract](../../../sql/sql/src/engine.rs); MySQL does not offer transactional DDL.
The test remains failing when opted in. No schema rewrite, manual table or sequential-plan
bypass is substituted. MySQL search execution awaits the separately approved G48 setup work;
this live-search task records the current refusal and does not implement that decision.
All successful and failing cases confirm the drop of their exact owned database.
