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

Pending.

## Verification notes

Pending.

## Acceptance notes

Pending.
