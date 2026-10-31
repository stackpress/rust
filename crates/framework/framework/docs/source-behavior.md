# Source behaviour and open questions

G43 treats upstream logic as intentional. The aggregate composes those behaviours without adding a global guard, default-deny, route ownership arbitration or a replacement account policy. G42 preserves multiple matching handlers in priority then registration order, with any handler able to Abort its chain.

## Parity and Rust mechanics

The [source ledger](../proofs/coverage-map.json) records the full upstream package, qualified Rust export owners, command oracles and R01–R20. Nine-stage order includes session auth → email → identity and application admin last. Missing configuration retains each plugin's own source activation rule; view key presence differs from language truthiness.

Static CompiledClient/CompiledModel callbacks replace dynamic imports. Typed config/Send futures, buffered owned request projections and Leptos replace runtime-specific mechanics. Native command/renderer/revision/packaging IO is explicitly supplied. These are mechanics boundaries, not security improvements. Source page/enrollment/CSRF values remain deliberate public projections; native provider graphs are not browser data.

## Ten carried owner questions

Each entry is an **open question — current behaviour follows upstream / accepted lower crate**. This documentation selects no fix or new policy. The seven account/API details have exact upstream anchors and Rust proofs in the owning [session source-behaviour page](../../framework-session/docs/source-behavior.md).

| Open question | Current behaviour / evidence |
| --- | --- |
| PGlite | S11-PGLITE: no connector or dummy feature; PostgreSQL dialect reuse alone is not a JS embedded host. [Ledger](../proofs/coverage-map.json). |
| MIT/Apache metadata | SOURCE-LICENSE: upstream MIT package metadata vs Apache LICENSE/README remains unresolved; no upstream source asset copied/distributed here. [Upstream inventory](../../../../.agents/specs/00001-stackpress-rust-foundation/slice-11-design-inventory.md). |
| SQLite purge/TRUNCATE | SQLITE-TRUNCATE: upstream Sqlite.js:404–408 emits rejected TRUNCATE … CASCADE; Rust reuses accepted SQLite DELETE mapping. [Renderer](../../../sql/sql/src/table.rs), [DDL test](../../../sql/sql-sqlite/src/tests/m2/ddl.rs). |
| Account delete keeps Profile | Account removal retains Profile; current behaviour follows upstream. [Account flow tests](../../framework-session/tests/main/account_flows/sql.rs). |
| SQLite ILIKE | Source predicate retained, including accepted backend error; no aggregate rewrite. [Action tests](../../framework-session/tests/main/actions.rs). |
| Auth create-return | Source return branch retained; returned fields need not be a persisted-row acknowledgement. [Action tests](../../framework-session/tests/main/actions.rs). |
| Positive hasnt | Source positive predicate handling retained. [Query implementation](../../framework-session/src/storage/query.rs). |
| Optional phone on update | Source update handling retained; G44 optional signup phone is a separate decided behaviour. [Profile update](../../framework-session/src/account/profile.rs). |
| Email sign-in skips send | Source control flow retained, including branches with no send. [Email page](../../framework-session/src/auth/pages/email.rs), [integration tests](../../framework-email/tests/cases/integration/mod.rs). |
| Moment admin dates | Source token formatting, including literal tokens, retained. [API roles view](../../framework-api/src/views/roles.rs). |

## Named limits and evidence gaps

These describe what the current proof establishes; they do not imply another host or live provider is implemented.

| Name | Boundary |
| --- | --- |
| API-MATRIX | The separate API stdio host has no aggregate injection seam. Its 26 generated-management/consent/native-navigation browser cases are not claimed through the aggregate website. |
| API-BUILD-HOST | Note generation/freshness is exercised; full built-in API regeneration through the aggregate build-host is not replayed. |
| EDGE-REPLAY | Owning source-edge tests remain current, but not every one of the seven account/API questions is reinvoked under aggregate registration. |
| S11-WHATWG | No JS/serverless Request/Response/ReadableStream bridge. |
| NODE-PACKAGING | No npm package.json writer; optional non-awaited app callback retains timing and supervision. Actual bundler/HMR/TTY/listener/process/filesystem integration stays app-owned. |
| LIVE-PROVIDERS | Real finite SQLite and JSON mail do not establish live SMTP/SES/sendmail/Postgres/MySQL, remote acknowledgement or TCP/TLS flush. |
| INSTALLED-TAURI | Native/WASM compilation and Chromium replay do not establish installed desktop/mobile acceptance. |
| JS-MECHANICS | No duplication of flat JS aliases, dynamic classes/imports, Node streams, TS/CJS/ESM/React/Vite/UnoCSS/reset assets. Qualified Rust owners and explicit bindings apply. |

The [verification page](verification.md) links exact current checks and historical accepted runtime scope. The aggregate adds no hardening beyond the source, and configuration absence remains developer-owned source behaviour.
