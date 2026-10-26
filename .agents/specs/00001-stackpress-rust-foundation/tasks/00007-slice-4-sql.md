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

## Acceptance notes

Library work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
