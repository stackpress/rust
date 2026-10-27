# Task 00011: Slice 6, generated data layer

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-26, in parallel with [task 00012](00012-slice-7-production-servers.md) and [task 00013](00013-slice-8-more-ui.md). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Make Stackpress a framework rather than a set of libraries: an Idea schema should become a working, queryable data layer. This covers the deferred work named S4-F and S4-M in [Slice 4 decisions](../slice-4-design-decisions.md):

- **framework-schema:** normalized schema semantics from compiled Idea config, including models, fields, types, relations, validation rules, serialization, and the field/filter/list/view roles that later UI generation needs (S5-F3).
- **framework-sql:** generated stores (CRUD, search/filter/sort/pagination, relations), generated server operations and events registered through the existing ServerBuilder, and transactions that span store calls.
- **Revisions and migrations (S4-M):** schema revision snapshots, diff to migration plans (using `stackpress-sql`'s SchemaDiff), an applied-state ledger, explicit install/upgrade/purge/populate tooling, and SQLite table reconstruction.
- **Generation:** through the existing Idea transformer function plugins and owned output (G20), producing inspectable Rust that compiles.
- **Proof:** a realistic multi-model example with relations, generated end to end, running against SQLite. Production network adapters arrive in task 00012.

## Implementation steps

1. Design checkpoint, with no production code. Write it as `slice-6-design-*.md`, per workflow §5.1. It covers:
   - the upstream inventory (references 00013, 00021, 00022, 00029, 00031, 00041);
   - the API sketch with DX examples, showing what a developer writes in `.idea` and what they get;
   - generated-code shape and ownership;
   - the risk inventory: migration safety, destructive-change policy, transactions across await, N+1 queries and relations, validation/disclosure, generated-code drift;
   - open decisions with recommendations, written in plain language for the owner;
   - dependencies on tasks 00012 and 00013;
   - the G23 shared-crate requests;
   - a milestone plan for about two lanes.
2. Coordinator review and owner decisions, then M0 and the milestones, rounds 1–4, closeout and a docs follow-up.

## Verification process

As in the coordinated workflow:

- generated code compiles;
- repeat generation is byte-stable;
- migrations are tested forward and back on SQLite, with opt-in PostgreSQL/MySQL per G26;
- per-crate coverage of at least 70%.

## Acceptance criteria

Acceptance criteria: none (library work), unless the design proposes an owner-visible artifact.

## Implementation notes

Design checkpoint delivered on 2026-10-26: start at [Slice 6 design decisions](../slice-6-design-decisions.md). Owner decisions recorded as G27/G30 on 2026-10-26; M0 released on 2026-10-27.

## Verification notes

**M0, coordinator review on 2026-10-27** ([receipt](00011-receipt-m0.md); merge after `60f3a2b`). Result: **verified**; merged into `trial/3`.

- **Checks rerun by the coordinator:** fmt; Clippy `-D warnings` over framework-schema, framework-sql, the generated workspace tool/app/contracts, sql-sqlite, idea-transformer and sql core with `codegen,server,sqlite-maintenance`; their tests (653 passed, 0 failed); `emit generate --check true` (generated output fresh); contracts `cargo check --target wasm32-unknown-unknown`. Merged workspace: 567 passed, 0 failed, 75 ignored; workspace Clippy clean. Conflicts were only additive root members/facade entries; the lockfile gained only the five new workspace packages.
- **Shared seams (G23/G30):** additive `TransformError::with_source`, read-only `Transaction::dialect()`, feature-gated SQLite maintenance (ordinary PRAGMA/BEGIN/COMMIT gate unchanged). 38 lines across the shared crates.
- **Coverage (implementer):** framework-schema 92.20%, framework-sql 89.75%, generated app 79.41%, contracts 83.33%, tool 70.32%; touched shared crates 89–96%.
- **M1** ([receipt](00011-receipt-m1.md); merge `6c1bb86`), coordinator review on 2026-10-27: reran fmt, Clippy `-D warnings` with `codegen,server,sqlite-maintenance`, the five packages' tests (37 passed), `emit generate --check true` and the WASM contracts check; all passed. The Slice 8 interface (contract revision 2) is frozen as G34. The receipt lists three review rounds; the four required rounds run at closeout after M4.
- **M2** ([receipt](00011-receipt-m2.md); merge `9506eac`), coordinator review on 2026-10-27: generated `<Model>Store`s with keyed create/get/update/upsert/delete, find/count/search with typed filters and deterministic sort, bounded breadth-first relation loading (no N+1), soft remove/restore, whole-batch preflight in one transaction, atomic targeted upsert, conservative unknown-outcome reporting, and selected named operation handlers with authorization before decoding. **Shared SQL-core grants** (exactly as approved): `InsertConflict` with `Insert::on_conflict`/`conflict` (SQLite/PostgreSQL render `ON CONFLICT (...) DO NOTHING/UPDATE`; MySQL refuses before any effect because it cannot restrict the action to the target), and `Engine::limits`/`Transaction::limits`; the coordinator reviewed the renderer diff. Contract revision 2 and the original contracts/metadata bytes are unchanged. **Merge follow-up:** the new store generator made the generated-admin example's committed stores stale; the coordinator regenerated them (commit after `9506eac`'s branch tip; check reports no differences). After merging current `trial/3` the coordinator reran: fmt, workspace Clippy, scoped Clippy with `codegen,server,sqlite-maintenance` plus the `matrix` and `names` fixture features, the scoped suite (567 passed), the matrix and names fixtures, both generation checks (fresh), the WASM contracts check with no server/SQLx/Tokio/Idea edges, the full workspace (675 passed, 0 failed, 78 ignored; 895 doctests), and the generated-admin checks, compile proof, build and Chromium proof (all pass). Coverage (implementer): sql 95.59%, framework-schema 90.75%, framework-sql 86.60%, generated app 76.31%, contracts 70.24%, tool 75.94%. **Limits:** PostgreSQL/MySQL generated-CRUD execution is a later SQL-lane checkpoint (task 00012 store integration); MySQL generated RETURNING writes and targeted conflicts stay gated; no optimistic version, keyset paging or nested relation mutation.
- **M3** ([receipt](00011-receipt-m3.md)), coordinator review on 2026-10-27: immutable revision history (separate from disposable generated output, create-new publication, hash and parent-chain checks, NoChanges); explicit old→new rename mapping (candidates shown, never auto-accepted); loss findings reported in full and approved only against the exact target/plan hash; explicit checked reverse plans or recovery refusal; SQLite native and reconstruction execution in one exclusive transaction with locked source/ledger/catalog checks, row counts, FK checks, index/constraint preservation and a namespaced ledger (`_stackpress_migrations`/`_stackpress_schema_state`); drift, unknown formats, unmanaged schemas and unknown dependencies refuse; app-owned operator events (revision, history-check, migration-plan, db-status, install/upgrade/downgrade/purge/uninstall/populate) through the existing `emit` grammar. **G35 shared grant applied exactly:** `MaintenanceSession::sequence_high_water`/`preserve_sequence_high_water` in sql-sqlite (maintenance feature only); AUTOINCREMENT high-water marks survive forward/reverse rebuilds and deleted maximum IDs; plain INTEGER PRIMARY KEY reuse is reported and documented. PostgreSQL/MySQL plans are rendered snapshots only; application stays refused. Contract revision 2 and the original contracts/metadata bytes are unchanged. After merging current `trial/3` the coordinator reran: fmt, workspace Clippy, scoped Clippy with `codegen,server,sqlite-maintenance,database-tools`, the seven-package suite (593 passed), sql-sqlite with `maintenance`, the matrix and names fixtures, both generation checks (fresh), the WASM contracts check, and the full workspace (693 passed, 0 failed, 80 ignored; 918 doctests). Coverage (implementer): framework-sql 89.08%, sql-sqlite 91.09%, tool 79.81%. **Remaining for task 00011:** M4 integrated DX, the MySQL string-key width decision (options in `00012-receipt-sql-closeout.md`), and closeout rounds.
- **M4 integrated DX** ([receipt](00011-receipt-m4.md)), coordinator review on 2026-10-27: new `examples/data-quickstart` with an owner-readable [walkthrough](../../../../examples/data-quickstart/README.md) from an Idea schema to an installed, seeded SQLite database; it queries three related models (relation includes, literal search, skip/take paging), then adds fields, renames one through an explicit mapping, adds a model, upgrades and reverses without losing rows, and shows a lossy step refused with the ledger unchanged. `prove.py` runs the README commands verbatim (including absent-output bootstrap) under a watchdog. The Slice 6 inventories gained status/evidence columns only; the coordinator checked mechanically that every existing disposition and all prose are unchanged. After merging current `trial/3` the coordinator reran: the quickstart proof and its unit tests, fmt, workspace Clippy, quickstart Clippy, both generation checks (fresh), and the full workspace (701 passed, 0 failed, 81 ignored; 929 doctests). Coverage (implementer): framework-schema 90.75%, framework-sql 89.18%, generated app 76.36%, contracts 70.24%, tool 79.91%, quickstart 74.96%. Network migrations stay render-only; PostgreSQL/MySQL generated CRUD evidence is task 00012's. **Remaining for task 00011:** closeout review rounds and the owner's MySQL string-key width decision.
- **Notes:** the current CLI requires `--check true` (or `-c`), not a bare `--check`; recorded, no CLI change. PostgreSQL/MySQL execution is not part of M0.

## Acceptance notes

Pending.
