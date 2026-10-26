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

Pending.

## Verification notes

Pending.

## Acceptance notes

Documentation has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
