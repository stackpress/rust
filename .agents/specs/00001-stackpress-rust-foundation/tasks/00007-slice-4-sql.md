# Task 00007: Slice 4, SQL and the first backend

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-25, to run in parallel with [task 00008](00008-slice-5-ui.md) under [G23](../decisions.md). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Bring the former Inquire family into the monorepo under the [accepted source structure](../../../context/source-structure.md):

- `crates/sql/sql` (`stackpress-sql`): independently usable query builders, database contracts, dialects and execution;
- one first backend adapter (`stackpress-sql-sqlite`, `-postgres` or `-mysql`), selected by the owner through Q06;
- `crates/framework/framework-sql` (`stackpress-framework-sql`): integration of the SQL facilities with Stackpress schema metadata, generated stores, events, configuration and lifecycle, kept distinct from the lower-level `sql` family. Its scope in this slice is settled at the design checkpoint.

Create only crates that hold real code. Backend drivers stay out of `stackpress-lib`, the default facade, and any future browser build.

This slice resolves Q06 (first database/backend; whether PGlite or a serverless adapter applies) in [decisions](../decisions.md). It also settles how SQL connects to the server host, as a service, plugin or capability, within the existing `stackpress-server` API.

## Implementation steps

1. Design checkpoint, with no production code. Write it as spec files `slice-4-design-*.md`. It covers:
   - the upstream inventory: Inquire packages, [SQL API contracts](../../../references/00025-upstream-sql-api-contracts.md), [database adapter contracts](../../../references/00026-upstream-database-adapter-contracts.md), and the Idea/schema relationship to stores and migrations. Map each item to now, a named later slice, or not ported;
   - backend and driver library assessment, using disposable probes outside the repository;
   - the API sketch, with DX examples;
   - the risk inventory: async and blocking driver calls, connection pooling and cancellation, transactions, parameter binding and injection safety, dialect differences, `Send` bounds, and test isolation;
   - open decisions with recommendations;
   - the milestone plan, with lane ownership sized for about two lanes.
2. Coordinator review and owner decisions, then M0, the riskiest proof.
3. Milestones, a stop before rounds, rounds 1–4, and closeout.
4. Docs as a follow-up task.

Any change needed in `stackpress-server`, `stackpress-lib`, the CLI or the Idea crates goes through the coordinator and lands in sequence, per G23.

## Verification process

As defined in the coordinated workflow, with coordinator re-verification at every stop. This includes:

- backend tests that run without external services wherever the chosen backend allows;
- a dependency-tree check that drivers don't leak into lib, the default facade or the server;
- per-crate coverage of at least 70%.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Design checkpoint prepared on 2026-10-25. Start with [Slice 4 decisions and recommendations](../slice-4-design-decisions.md), which routes to the core and framework inventories, the test dispositions for all 226 upstream tests, the driver assessment, the API, the execution contracts, and the risks and milestones. Probes ran only in `/private/tmp/stackpress-s4-probe/`; no production code or manifests changed.

## Verification notes

**Design checkpoint review (coordinator, 2026-10-26): approved; M0 released.**

- **Reviewed:** decisions, API, assessment, and the risks and milestones plan.
- **DX:** a standalone query needs no host or Idea schema. SQL joins the server as an ordinary typed service through the existing `ServerBuilder` API, with no shared-crate changes.
- **Probes rerun by the coordinator** with `--offline --locked`:
  - SQLx 13/13 and tokio-rusqlite 3/3 passed;
  - the expected compile failures reproduced: `aliasing` E0499, `clone` E0599, `non_sync` E0277.
- **Material finding carried into M0:** native SQLite binds a missing parameter as NULL, so the adapter must validate prepared-parameter arity.
- **Owner decisions:** recorded as [G24](../decisions.md). Every recommendation S4-D01 to D07 was accepted, and the D08 dependency budget is approved by the coordinator.
- **Amendment [G26](../decisions.md), 2026-10-26:** from M1, opt-in tests run the rendered PostgreSQL and MySQL SQL against the owner's local development servers. Each run uses a throwaway database of its own; network drivers are dev-only; production adapters stay S4-N.

**M0 re-verification (coordinator, 2026-10-26): passed; merged as `70af7a6`; M1 released.** Details are in the [M0 receipt](../slice-4-receipt-m0.md).

- **Checks rerun by the coordinator in the M0 worktree:**
  - fmt passed;
  - Clippy `-D warnings` clean;
  - `cargo +1.96.0 check --workspace --all-targets` passed;
  - 272 workspace tests passed, 0 failed (the implementer counted 275, including ignored child entries), and 432 doctests passed;
  - the `m0-host` proof app prints `profile-name: Ada`.
- **Isolation:** the core `stackpress-sql` has **no** normal dependencies. `cargo tree` for the default `stackpress` facade, `stackpress-lib` and `stackpress-server` shows no `sqlx`, `libsqlite3` or `stackpress-sql`. No shared-crate edits.
- **Coverage:**
  - the coordinator measured each crate alone after a clean: `stackpress-sql` **81.19%**, `stackpress-sql-sqlite` **95.45%**;
  - the receipt's 99.23% for core comes from a combined run that counts lines exercised by the adapter's tests, so the per-crate 81.19% is the figure of record;
  - both pass the 70% floor.
- **Soundness highlights:**
  - prepared-parameter arity is validated on the same lease;
  - an admission permit is coupled to native completion, after a defect where cancellation released it early was found and fixed;
  - transactions are non-Clone and exclusive, with rollback on drop, abort and unwind;
  - a cancelled commit has an unknown outcome and is never replayed;
  - SQLite's sticky `changes()` counts are corrected;
  - seven trybuild compile-fail proofs;
  - every concurrency test runs under a process watchdog, with a negative control.

**M1 re-verification (coordinator, 2026-10-26): passed; merged as `bb078d5` (core) and `5a265a8` (SQLite); M2 released.** Details are in the receipts for the [core lane](../slice-4-receipt-lane-a-m1.md) and the [SQLite lane](../slice-4-receipt-lane-b-m1.md).

- **Core lane (each lane checked in its own worktree):** the scope stays within `crates/sql/sql/**`. fmt, Clippy and the 1.96 check passed; 312 workspace tests and 437 doctests passed. Coverage measured alone is **98.30%**, and the core still has zero normal dependencies.
- **SQLite lane:** the scope stays within `crates/sql/sql-sqlite/**`. fmt, Clippy and the 1.96 check passed; 276 workspace tests and 433 doctests passed. Coverage measured alone is **95.15%**.
- **Integrated on `trial/3` after both merges:** fmt passed and Clippy is clean; 328 tests and 438 doctests passed, 0 failed.
- **Operational note:** the disk reached 0 bytes free during verification. The coordinator deleted only finished-work build output, keeping logs and evidence, and lanes now reuse at most two target dirs each.

**M2 re-verification (coordinator, 2026-10-26).** Details are in the receipts for the [core lane](../slice-4-receipt-lane-a-m2.md) and the [SQLite lane](../slice-4-receipt-lane-b-m2.md).

- **SQLite lane:** merged as `3ec57a9`. The scope stays within the adapter. fmt, Clippy and the 1.96 check passed; 329 workspace tests and 438 doctests passed. Adapter-only coverage is **95.35%**. The implementer fixed stale pooled column metadata after ALTER. The public `execute_plan` integration test is assigned post-merge.
- **Core lane:** merged as `90aa05e`. fmt, Clippy and the 1.96 check passed; 352 workspace tests and 440 doctests passed. Core-only coverage is 95.49% (implementer figure), and the core keeps zero normal dependencies. The lockfile adds 23 dev-only packages and changes no existing version. The work adds `Engine::execute_plan`/`Transaction::execute_plan` on the existing traits and fixes a primary-key `SET NULL` validation gap.
- **G26 live renderer validation, rerun by the coordinator:** **16/16 passed** on PostgreSQL 12.2 and MySQL 8.0.27, and both owned throwaway databases were dropped.
  - No `stackpress_test_` database remains on MySQL.
  - A first-run PostgreSQL 12 cleanup-syntax failure (`WITH (FORCE)` needs PG13+) left one owned database. It was dropped by an audited, prefix- and nonce-bounded recovery, and cleanup is now plain `DROP DATABASE IF EXISTS`.
- **Integrated `trial/3`:** Clippy clean; 365 tests and 441 doctests passed.

**M3 re-verification and before-rounds review (coordinator, 2026-10-26): passed; rounds 1–4 released.** Details are in the receipts for the [core lane](../slice-4-receipt-lane-a-m3.md) and the [SQLite and example lane](../slice-4-receipt-lane-b-m3.md).

- **Core docs (merged as `fae98c8`):** a crate-root first-success guide and task table; 852 workspace doctests run, with no ignored doc examples; four standalone examples run; 391 tests passed; Rustdoc `-D warnings` clean.
- **sql-profile example (merged as `9f9f787`):**
  - The coordinator ran the SQLite app and the in-memory capability-host app over 12 inputs (prefixes, case variants, no match, `%`, `_`, quote, backslash, accented text, empty), with **12/12 byte-identical** CLI envelopes.
  - Prefix matching is a byte-substring comparison on bound values, not LIKE.
  - Setup refuses an existing installation, and startup is read-only.
  - 395 tests and 445 doctests passed; Clippy and the 1.96 check are clean.
  - The default `stackpress`, `stackpress-lib` and `stackpress-server` trees have no `sqlx`/`libsqlite3`.
- **Accepted deviations:**
  - MySQL nullable columns without an explicit default omit DEFAULT.
  - No read-only observation hook (deferred with S4-X).
  - Physical reorder, identity conversion and prefix/expression indexes return Unsupported.
  - PostgreSQL primary-key removal assumes the generated `<table>_pkey` name.
- **Disk:** free space fell to 4.4 GB during M3. The coordinator deleted superseded build output only (logs kept) and restored about 30 GB.

## Acceptance notes

Library work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
