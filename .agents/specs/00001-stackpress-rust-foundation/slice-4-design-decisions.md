# Slice 4 decisions and recommendations

Design checkpoint for [task 00007](tasks/00007-slice-4-sql.md), prepared 2026-10-25 on `trial/3`. These are proposals for coordinator review and owner decisions, not accepted behavior or implementation receipts. [G01–G23 and Q06](decisions.md), the [source structure](../../context/source-structure.md), and [quality policy](../../context/testing-and-quality.md) govern this checkpoint.

## Read by concern

- [Core inventory](slice-4-design-inventory-core.md): load for every Inquire export/type, builder, dialect, helper and deliberate Rust replacement.
- [Adapters and framework inventory](slice-4-design-inventory-framework.md): load for native connectors, all framework SQL types/operations, schema ownership, and named deferrals.
- [Core test inventory](slice-4-design-tests-core.md): load for the individual dispositions of all 160 upstream core tests.
- [Adapter and framework test inventory](slice-4-design-tests-integration.md): load for the individual dispositions of all 66 adapter/framework tests and generated test templates.
- [Driver assessment](slice-4-design-assessment.md): load for exact candidate versions, licenses/MSRV, disposable probe commands/results and their limitations.
- [API and examples](slice-4-design-api.md): load for standalone construction/execution, existing ServerBuilder integration and the future Profile store sketch.
- [Execution and SQL contracts](slice-4-design-execution.md): load for ownership, pooling, transactions, cancellation, values, SQL safety and dialect capabilities.
- [Risks and milestones](slice-4-design-verification.md): load for the open-ended risk register, prevention/proving tests, two-lane plan, shared-crate requests, coverage and stop gates.

## Structured choices requiring decisions

| ID / authority | Question | Recommended choice | Other choice and consequence |
| --- | --- | --- | --- |
| S4-D01 / owner, Q06 | Which first backend and execution host? | **Native SQLite**, SQLx 0.9.0 with bundled SQLite and Tokio pool support; no external service needed for tests. | PostgreSQL or MySQL first requires an explicitly provided test service and a new driver proof. PGlite/serverless remains a separate host assessment, not implied by a PostgreSQL dialect. |
| S4-D02 / owner, task scope | How much framework SQL belongs in this slice? | **Broad independent SQL + SQLite adapter + app-owned host proof now; defer a public framework-sql crate and general generated stores** to named follow-up S4-F. The present Idea model has syntax metadata, not the normalized schema/store lenses. Avoid a tiny wrapper crate that falsely promises framework semantics. | Include full framework-schema semantics, generated stores/actions/events/revisions/migrations now. This requires a new semantic design checkpoint and additional milestones before implementation; it is not a two-field extension of the demo. A narrow reusable host wrapper alone adds no missing server capability. |
| S4-D03 / owner, compatibility | What execution/result/aliasing semantics should replace JS thenables and shared lastId? | **Detached, owned builders; explicit render/fetch/execute; non-Clone Engine/transaction; explicit Arc sharing; statement-local rows/affected/insert-ID results.** Transaction ownership is exclusive across await. | Preserve attached builders/callback convenience as a later ergonomic layer after ownership proofs. Do not preserve pooled shared lastId or implicit transaction context; those have cross-call races. |
| S4-D04 / owner, safety and breadth | Which raw SQL and hook surface? | **Typed bound expressions plus explicit dialect-tagged trusted raw SQL; immutable observations only.** No registered arbitrary query capability, mutating before hook or result-short-circuit hook now. | Trusted middleware with rewriting/short-circuit needs a separate ordered, bounded, transaction-aware contract and proof. SQLx is not a sanitizer for that extension. |
| S4-D05 / owner, dialect behavior | How broad are builders/dialects and SQLite alteration? | **All six builder families, explicit schema diff/rename reconciliation, SQLite/PostgreSQL/MySQL renderers now.** SQLite rejects unsupported alterations/cascade/DDL attributes before effects; native add/drop/rename columns and index operations execute. Defer table reconstruction to S4-M. | Add SQLite table-copy reconstruction now, with a dedicated foreign-key/index/trigger/data-preservation milestone. Never emit invalid ALTER COLUMN or silently ignore requested constraints. Network render tests alone do not claim working network adapters. |
| S4-D06 / owner, value compatibility | Retain permissive scalar coercion and default quirks? | **Fallible typed SQL values**, exact i64/finite f64, explicit JSON/date text, separate missing/NULL/default; preserve false/zero/empty defaults. Invalid input is an error. | Explicit legacy coercion module later, with documented lossy behavior; do not silently turn invalid dates/numbers into epoch/zero. |
| S4-D07 / owner, pooling/cancellation | What operational guarantees and defaults? | **Bounded native pool, eager open, file mode by default, finite acquire/busy budgets, explicit close, no automatic retries.** Cancellation is loss of observation, not proof a write/commit did not occur. Ordinary transactions roll back on uncommitted drop. | Lazy pool opening or convenience in-memory persistence can be explicit later APIs. Immediate interruption/exactly-once writes need a separate protocol and cannot be promised by dropping an async future. |
| S4-D08 / coordinator, dependency budget | Accept implementation dependencies and feature budget? | Core: std plus existing serde/serde_json only where used; adapter: SQLx `=0.9.0`, defaults off, `runtime-tokio,sqlite`, existing Tokio; tests: tempfile, trybuild and a stream utility only if needed for bounded row collection. | Added codecs, chrono/decimal/UUID, ORM/query libraries, TLS/network drivers, SQLx macros/migrate/Any and additional toolchains return for review. Resolve the shared lock through the coordinator. |

Each row is a choice, not acceptance by silence. S4-D01 only answers Q06's first-backend part. First serverless adapter and PGlite remain unresolved with owner authority; recommendation is defer both to **S4-H Hosted/serverless SQL adapters**. No browser/local Tauri database support follows from the native proof; coordinate those choices with Slice 5/Q02/Q15.

## Named follow-ups used by the inventories

| Label | Future scope / reason |
| --- | --- |
| S4-F Framework schema, generated stores and operations | Design normalized schema storage metadata, generated validations/serialization/relations, stores/actions/events and explicit host integration together. Then create real framework-schema/framework-sql crates. |
| S4-M Schema revisions and migration execution | Revision signatures, clear/ambiguous rename plans, destructive policy, applied-state ledger, SQLite reconstruction, install/upgrade/purge/populate tooling. Schema history is never treated as applied state. |
| S4-N Network SQL adapters | PostgreSQL/MySQL native pools, connection affinity, codecs/TLS/service fixtures and real integration tests. SQL renderers now do not establish support. |
| S4-H Hosted/serverless SQL adapters | PGlite, serverless lifecycle/concurrency/persistence and target-specific adapters, only after a deployment is chosen. |
| S4-X Advanced SQL extensions | Query middleware/short-circuit, savepoints, callback transaction sugar, streaming cursors, native-resource escape hatches and advanced codecs. Each needs its own concrete contract. |

These labels name deferred slices of work; they are not new approved tasks or fabricated completion. S4-D02/S4-D04/S4-D05 are disclosed breadth changes for owner review.

## Current finding and next gate

Disposable native probes support the SQLx recommendation and show that the existing host API can hold the database service with Send handlers. They do not prove the proposed production abstraction, every cancellation transition, cross-platform support or development coverage. Native SQLite permits an unbound placeholder to become NULL; the adapter must explicitly validate prepared parameter arity. SQLx 0.9 requires an explicit audit marker for generated dynamic SQL; its safe wrapper does not perform sanitization.

No change is requested in lib/server/CLI/Idea for the recommended scope. Coordinator-owned manifests/facade wiring and a new SQL proof example are the only integration edits proposed after approval. Existing examples, Idea documentation, Slice 5 files, and task status/verification fields remain outside this design write set.

**Stop after this checkpoint.** The [coordinated workflow §5.1](../../workflows/coordinated-crate-development.md) ends design with coordinator review and owner choices before M0. The next release should authorize M0 only, using actual production types and a process watchdog.
