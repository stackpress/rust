# Source behaviour and G46 decisions

G43 treats upstream logic as intentional. G46 resolves the ten source-behaviour questions below. The aggregate adds no global guard, default-deny, route ownership arbitration or replacement account policy. G42 preserves multiple matching handlers in priority then registration order, with any handler able to Abort its chain.

## Parity and Rust mechanics

The [source ledger](../proofs/coverage-map.json) records the full upstream package, qualified Rust export owners, command oracles and R01–R20. Nine-stage order includes session auth → email → identity and application admin last. Missing configuration retains each plugin's own source activation rule; view key presence differs from language truthiness.

Static CompiledClient/CompiledModel callbacks replace dynamic imports. Typed config/Send futures, buffered owned request projections and Leptos replace runtime-specific mechanics. Native command/renderer/revision/packaging IO is explicitly supplied. These are mechanics boundaries, not security improvements. Source page/enrollment/CSRF values remain deliberate public projections; native provider graphs are not browser data.

## Ten resolved owner decisions G46

All ten entries are **resolved under owner decision G46**. The only code change is dialect-aware LIKE/ILIKE rendering in the session and API adapters. The [session decision register](../../framework-session/docs/source-behavior.md#owner-decisions-g46) retains exact source anchors and Rust evidence for the seven account/API behaviours.

| Resolved item | Current behaviour / evidence |
| --- | --- |
| PGlite | S11-PGLITE resolved: not required. The standard engines are MySQL, SQLite and PostgreSQL; no PGlite connector is planned. [Ledger](../proofs/coverage-map.json). |
| MIT/Apache metadata | SOURCE-LICENSE resolved: owner-owned. |
| SQLite purge/TRUNCATE | SQLITE-TRUNCATE resolved: keep DELETE FROM "table". [Renderer](../../../sql/sql/src/table.rs), [DDL test](../../../sql/sql-sqlite/src/tests/m2/ddl.rs). |
| Account delete keeps Profile | Intended soft delete via @active. Removed records are deactivated and queries filter active rows by default; the account flow retains Profile. No code change. [Account flow tests](../../framework-session/tests/main/account_flows/sql.rs), [active-filter tests](../../framework-session/tests/main/actions.rs). |
| SQLite ILIKE | PostgreSQL alone retains ILIKE; SQLite and MySQL use LIKE. SQLite ASCII matching and MySQL default _ci collations supply case-insensitive matching. [Session tests](../../framework-session/tests/main/actions.rs), [API tests](../../framework-api/tests/sql.rs). |
| Auth create-return | Follow original source, unchanged. [Action tests](../../framework-session/tests/main/actions.rs). |
| Positive hasnt | Follow original source, unchanged. [Query implementation](../../framework-session/src/storage/query.rs). |
| Optional phone on update | Follow original source, unchanged; G44 optional signup phone remains separate. [Profile update](../../framework-session/src/account/profile.rs). |
| Email sign-in skips send | Follow original source, unchanged. [Email page](../../framework-session/src/auth/pages/email.rs), [integration tests](../../framework-email/tests/cases/integration/mod.rs). |
| Moment admin dates | Follow original source, unchanged. [API roles view](../../framework-api/src/views/roles.rs). |

## Named limits and evidence gaps

These describe what the current proof establishes; they do not imply another host or live provider is implemented.

| Name | Boundary |
| --- | --- |
| API-MATRIX | All 13 original scenarios per mode replay against standalone and aggregate hydrate/CSR bundles. The stdio backend still uses direct API registrars; the separate aggregate source-family SQLite proof covers registered auth/CRUD/joins/admin/export. Not every native edge is replayed through aggregate registration in the browser matrix. |
| API-BUILD-HOST | Closed in G50 M3: the build-host generates and checks the complete four-model source client with Profile.nickname, alongside the unchanged Typed Note fixture. |
| EDGE-REPLAY | Owning source-edge tests remain current, but not every G46 account/API behaviour is reinvoked under aggregate registration. |
| S11-WHATWG | No JS/serverless Request/Response/ReadableStream bridge. |
| NODE-PACKAGING | No npm package.json writer; optional non-awaited app callback retains timing and supervision. Actual bundler/HMR/TTY/listener/process/filesystem integration stays app-owned. |
| LIVE-PROVIDERS | Owning [session/API live tests](../../framework-session/docs/live-search.md) execute direct PostgreSQL/MySQL ASCII searches. [Aggregate live setup](../tests/live_setup.rs) proves direct adjacent generated upgrades and partial failure semantics, not live command-provider event routing, SMTP/SES/sendmail, remote delivery acknowledgement or TCP/TLS flush. |
| INSTALLED-TAURI | Native/WASM compilation and Chromium replay do not establish installed desktop/mobile acceptance. |
| JS-MECHANICS | No duplication of flat JS aliases, dynamic classes/imports, Node streams, TS/CJS/ESM/React/Vite/UnoCSS/reset assets. Qualified Rust owners and explicit bindings apply. |

G48 selects explicit ordered non-atomic MySQL setup; PostgreSQL/SQLite keep atomic setup.
Ordinary transactions and the generated SQLite-only MigrationRunner are unchanged. No
generated MySQL runner, locking/drift protocol or generated-workspace ledger is added.
G48 also resolves MySQL purge: omit CASCADE, which the server rejected with ERROR 1064.
InnoDB rejects parent-table TRUNCATE with ERROR 1701 while a foreign key references it.
Related MySQL catalogs therefore use child-first DELETE in reverse create order;
unrelated catalogs keep TRUNCATE without CASCADE. PostgreSQL retains parent-first
TRUNCATE CASCADE and SQLite retains its DELETE mapping. The general MySQL renderer
still refuses CASCADE. The [SQL reference](api/sql.md) and [native proof](../tests/live/cascade.rs)
pin the mapping and verify that foreign-key constraints remain active.

The [verification page](verification.md) links exact current checks and historical accepted runtime scope. The aggregate adds no hardening beyond the source, and configuration absence remains developer-owned source behaviour.

## Generated model integration

G50 selects compiled SourceModel families over the complete shipped Idea graph. The
aggregate's session/API binding callbacks use the same generated Profile type as model
listeners; portable source UI is a separate partition. Source index creation groups all
unique keys before ordinary keys, and Session foreign keys follow application then
profile declaration order. Existing installations are not changed by generation.
The [source client consumer](../consumers/support/source_family.rs) proves the appended
Profile column through registered auth, CRUD, API joins, admin and ordered export.
