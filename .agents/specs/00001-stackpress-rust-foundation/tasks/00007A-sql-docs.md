# Task 00007A: SQL usage and API docs

Follow-up of [task 00007](00007-slice-4-sql.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner requested it on 2026-10-26, after the Slice 4 closeout.

## Task summary

Write developer documentation for the two SQL crates, covering how to use them plus an API reference:

- `stackpress-sql` in `crates/sql/sql/docs/`;
- `stackpress-sql-sqlite` in `crates/sql/sql-sqlite/docs/`.

Follow the installed `chrisai-docing` skill and the conventions of [task 00005A](00005A-server-and-cli-docs.md) and [task 00006A](00006A-idea-docs.md):

- guided-learning pages first, then lookup-oriented reference pages;
- examples as plain Markdown, compile-checked in a disposable consumer outside the repository;
- no source changes;
- API rough edges reported as DX observations.

The crates' Rustdoc and runnable examples already exist. The docs link to them rather than duplicating them.

## Implementation steps

1. Core docs:
   - `README.md` with reading paths;
   - `getting-started.md`: build, render and inspect a query, with no database;
   - `concepts.md`: owned builders, dialects and the capability matrix, binds versus identifiers, strict values, rows, the Engine and transaction ownership, `execute_plan`, schema diff, TrustedSql, and limits;
   - `guides/` by task intent;
   - `api/`: an index plus one page per public surface.
2. SQLite docs, with the same structure scaled to the crate:
   - options and validation;
   - eager open and pooling;
   - leases and admission;
   - transactions and cancellation, including an unknown commit outcome and no retries;
   - closing and shutdown;
   - error disclosure;
   - registering as a server service.

   A guide walks through `examples/sql-profile`; link to its README rather than copying it.
3. Verify every claim against the source, tests, the Slice 4 receipts, and decisions G24 and G26.
4. Apply copy-editing and formatting, and run the skill's review gates.

## Verification process

The coordinator:

- reviews the pages against the `chrisai-docing` review gates;
- spot-checks accuracy against the source, especially transaction ownership and cancellation, the dialect capability matrix, the strict value rules, and the TrustedSql boundary;
- runs a mechanical format and link scan;
- confirms no source, tests or manifests changed.

Workspace checks must still pass.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on 2026-10-26 by two implementers working in parallel in disjoint directories. Their full receipts:

- [core receipt](00007A-receipt-core.md): `crates/sql/sql/docs`, 32 pages.
- [SQLite receipt](00007A-receipt-sqlite.md): `crates/sql/sql-sqlite/docs`, 8 pages, with 13 Rust examples and quick-start/CLI walkthroughs compile-checked and run in disposable consumers.

**DX observations, no code changes:**

1. **Candidate fix:** `Select::where_eq` *replaces* its equality slot, while `Update` and `Delete` *append*. A second `where_eq` on a Select silently drops the first filter.
2. **Candidate fix:** the `QueryResult::insert_id` Rustdoc is stale. It says the adapter exposes no IDs, but SQLite returns `Some(id)` for observed rowid transitions (still `None` for some inserts).
3. QueryPlan has no public arbitrary-batch constructor; app statements compose through a Transaction.
4. Engine has no limits getter, and Transaction has no dialect getter.
5. Every transaction connection is retired, and statements refresh schema metadata. These are correctness-first costs that haven't been benchmarked.
6. Awaited shutdown is application-owned; there is no host async-finalization hook.
7. SqliteOptions uses public fields, so invalid settings persist until `validate`/`open`.

## Verification notes

Coordinator review on 2026-10-26. Result: **verified**.

- **Scope:** 40 pages (32 core, 8 SQLite) plus two receipts. `git status` shows no source, test, manifest or lockfile change.
- **Mechanical scan** of all 40 pages and 342 relative links/anchors: 0 real issues. The single flag is a scanner false positive on an anchor in `examples/sql-profile/README.md`, outside the scanned set; the heading exists. The four cross-crate anchors into core concepts resolve.
- **Accuracy against source:**
  - **Capability table:** every row matches `DialectCapabilities` in `dialect.rs` (RETURNING, transactional DDL, alteration, cascade, JSON number kinds, text NUL).
  - **Strict value rules:** checked against the value module.
  - **TrustedSql:** stated as an authorship assertion, not sanitization.
  - **SQLite pages:** transaction retirement, rollback-only after native failure, unknown cancelled-commit outcome, path validation (no `:memory:`/`file:`), and eager-open behavior all match the code and the Slice 4 receipts.
- **Example evidence** comes from the implementers and was not rerun by the coordinator.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
