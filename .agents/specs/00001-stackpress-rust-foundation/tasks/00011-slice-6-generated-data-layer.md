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
- **Notes:** the current CLI requires `--check true` (or `-c`), not a bare `--check`; recorded, no CLI change. PostgreSQL/MySQL execution is not part of M0.

## Acceptance notes

Pending.
