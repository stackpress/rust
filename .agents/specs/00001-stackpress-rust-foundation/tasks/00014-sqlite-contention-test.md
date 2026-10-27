# Task 00014: SQLite contention-sensitive test

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). Opened by the coordinator on 2026-10-27 as a side quest, using the extra agent capacity from G33.

## Task summary

During one full workspace run with default test threads, the existing SQLite adapter test at `crates/sql/sql-sqlite` `cancellation.rs:454` failed with `PoolTimedOut`; isolated and later full runs passed. It was first noted in the [HTTP M1 receipt](00012-receipt-http-m1.md). Find the root cause (a timing-sensitive test or a real ordering or permit-release bug) and fix the right thing without weakening any guarantee.

## Implementation steps

1. Reproduce under controlled load with a parent watchdog; record the failure rate.
2. Fix the test design or the adapter, with a regression that fails before and passes after.
3. Prove repeated passes under load, plus the sql-sqlite and workspace suites.

The write set is `crates/sql/sql-sqlite/**`, excluding the Slice 6 maintenance module.

## Verification process

Coordinator reruns the sql-sqlite suite, repeated loaded runs and the workspace suite.

## Acceptance criteria

Acceptance criteria: none (test reliability).

## Implementation notes

See the [receipt](00014-receipt-sqlite-contention.md). Root cause: test-design races, not adapter bugs. Tests started fresh acquisitions before asynchronous cleanup finished, raced fixed wall-clock budgets (8 s cooperative wrappers, 80 ms acquisition) against loaded schedulers, or started a 200 ms kill deadline before the child wrote its PID file. Fixed with explicit synchronization while keeping each property. Scope grew to the PostgreSQL adapter's shared contract harness (SQLite reference) and the Idea transformer formatter fixture.

## Verification notes

Coordinator review on 2026-10-27 (merge `97b1866`): fmt and Clippy `-D warnings` for sql-sqlite, sql-postgres and idea-transformer; **12 consecutive full test runs of those three packages passed** while other agents' builds kept the machine heavily loaded (before the fix, 5 of 15 sql-sqlite runs failed under similar load). Implementer: 30/30 runs under ten CPU burners. The only manifest change is tokio `test-util` in sql-sqlite dev-dependencies; no lock change.

## Acceptance notes

Pending.
