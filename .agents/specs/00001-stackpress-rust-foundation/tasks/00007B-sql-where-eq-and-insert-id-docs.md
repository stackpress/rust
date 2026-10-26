# Task 00007B: Consistent where_eq and current insert_id Rustdoc

Side quest of [task 00007A](00007A-sql-docs.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner approved it on 2026-10-26 ("please do side tasks"), on the coordinator's recommendation that `where_eq` append everywhere.

## Task summary

The SQL docs review found two defects in `stackpress-sql`:

1. **`Select::where_eq` replaces its equality slot**, while `Update::where_eq` and `Delete::where_eq` append a condition. A second `where_eq` on a Select silently drops the first filter. Make Select append, like Update and Delete. Multiple equalities combine with AND, in call order. This is a deliberate, owner-approved public behavior change.
2. **`QueryResult::insert_id` Rustdoc is stale.** It says the adapter never exposes IDs, but the SQLite adapter now returns `Some(id)` for observed rowid transitions, while some successful inserts (for example reused explicit IDs or WITHOUT ROWID tables) still return `None`. Update the Rustdoc to describe the current contract and point callers who need every generated key to RETURNING.

## Implementation steps

1. Write the failing regression first:
   - two `where_eq` calls on a Select render both predicates joined with AND, with binds in call order;
   - Select, Update and Delete behave the same.
2. Change `Select::where_eq` to append. Audit every caller, example, doctest and doc page that relied on replacement, including `crates/sql/sql/docs` (which documents the current difference), `examples/sql-profile`, and the SQLite tests.
3. Update the `QueryResult::insert_id` Rustdoc, and any doc page that repeats the stale claim.
4. Run the full checks.

## Verification process

The coordinator:

- reruns the full workspace checks;
- confirms the regression failed before the fix;
- confirms both doc sets no longer describe the old behavior;
- confirms the sql-profile parity is still byte-identical.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

- Date: 2026-10-26. Branch `trial/3-sql-where-eq`, starting/closing HEAD `a094418`;
  clean before work. Native `aarch64-apple-darwin`, rustc/Cargo 1.96.0.
  Coordinator review, Verification notes and task status remain coordinator-owned.
- Scope: only `crates/sql/**`, plus this specifically requested Implementation notes update.
  Nine SQL source/test/doc files changed. No implementation outside SQL, manifest/lockfile
  change, commit, push, publication, deployment, indexing, listener or agent messaging.

### Regression and implementation

- Added both core regressions before editing implementation. Command:
  `cargo test -p stackpress-sql --test dml where_eq --offline --locked`.
  **Observed failure: 0 passed, 2 failed, exit 101**. Both tests observed only
  `WHERE "active" = ?` instead of `WHERE "team_id" = ? AND "active" = ?`.
  Full compiler/test output is retained in `regression-before.log` in the evidence directory.
- `Select::where_eq` now delegates to `and_where(Expr::eq(...))`, matching Update/Delete.
  Removed its private replacement index and branch. The regression checks exact SQL and
  ordered binds; the consistency test checks all three builders on SQLite/PostgreSQL/MySQL,
  including PostgreSQL numbering after UPDATE's assignment bind.
- Updated the former replacement test to assert interleaved `where_eq`/`and_where`, repeated
  columns, SQL/bind call order, and clone independence. Added real SQLite execution coverage:
  the combined equality lookup returns no rows, while dropping either predicate returns rows.
- Corrected Select type/method Rustdoc and its executable example, plus core Select/Update
  reference pages and the query-building guide. The guide now uses consecutive `where_eq`
  calls. Updated `QueryResult::insert_id` Rustdoc, the rows reference and write guide:
  SQLite reports observed rowid transitions as `Some(id)`; reused explicit IDs and WITHOUT
  ROWID inserts can succeed with `None`; callers requiring every generated key use RETURNING.
- Audited all 115 Rust/Markdown files under `crates/sql/**` and `examples/sql-profile/**`,
  including 144 `where_eq`/`insert_id` occurrences, callers, tests, doctests and examples.
  No remaining replacement-slot wording or stale non-exposure claim in either doc set.
  SQLite outcome docs already match the current contract. Other callers/examples need no
  migration; sql-profile uses its existing typed `and_where` prefix predicate.
- Initial working baseline after the fix: core 87 tests plus 417 doctests passed;
  SQLite library 14 passed / 9 watchdog-only child or negative-control entry points ignored.

### Three test-verified review rounds

1. **Logic/complexity:** reviewed empty/single/repeated filters, mixed predicates, duplicate
   columns, clone isolation, rendering/validation order and bound parameter numbering.
   The ordinary append path removes the replacement branch; no further refactor justified.
   Re-ran core `--test dml --test dialects`: **29 passed**, no failures/ignores.
2. **Responsibility/abstraction:** reviewed SelectData ownership, the three builder entry
   points, shared ordered filter rendering and the adapter boundary. No new helper/trait or
   cross-crate state is needed; removed replacement state has no remaining use.
   Re-ran SQLite `tests::m2::native_execution_under_watchdog`: **passed**, all 18 child scenarios.
3. **Consistency/Rustdoc/comments:** aligned append/call-order documentation, removed obsolete
   replacement explanations, clarified that SELECT may omit filters, checked the insert-ID
   statement against native outcome code/tests, and made the Select doc example assert exact
   binds. Extracted every Rust fence from all five edited doc pages verbatim into an isolated
   consumer: **5 examples passed**. Its initial locked run required re-projecting the isolated
   lockfile offline; the final run was locked/offline, and the repository lockfile is unchanged.
   Final workspace tests/doctests, formatting, lint and warning-clean SQL Rustdoc passed below.
   No material in-scope finding remains; no ceremonial follow-up changes.

### Final checks and coverage

All Cargo verification used offline/locked dependencies, default workspace feature selection,
incremental compilation disabled and dev/test debug info disabled. `RUSTDOCFLAGS=-D warnings`
was set for the SQL documentation command.

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS. |
| `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | PASS. |
| `cargo test --workspace --all-targets --offline --locked` | PASS: 436 passed, 0 failed, 52 expected ignores. |
| `cargo test --workspace --doc --offline --locked` | PASS: 855 passed, 0 failed/ignored. |
| `cargo doc -p stackpress-sql -p stackpress-sql-sqlite --no-deps --offline --locked` | PASS with `RUSTDOCFLAGS=-D warnings`. |
| `cargo +1.96.0 check --workspace --all-targets --offline --locked` | PASS. |
| `cargo llvm-cov -p stackpress-sql --tests --offline --locked --fail-under-lines 70 --json --summary-only --output-path /private/tmp/stackpress-00007b-sql-where-eq/coverage.json` | PASS: 87 passed, 16 optional network fixtures ignored. |
| `git diff --check` and scope audit | PASS; only the authorized SQL patch and this receipt. |

- Core-only coverage, cargo-llvm-cov 0.8.6 / rustc 1.96.0 / native aarch64-apple-darwin:
  **2,932 / 3,073 executable implementation lines, 95.41%**, above the 70% development floor.
  Report contains only the 16 `crates/sql/sql/src/*.rs` files with executable lines; no custom
  exclusion regex. Dependencies, tests, docs and adapters are outside this core measurement.
  Doctests ran separately. Branch coverage is not claimed. The retained export is
  `core-coverage.json` (re-exported from the same profiles after a command-receipt path collision).
- Remaining coverage gaps are visible in that report; `network_ddl.rs` is the lowest file at
  318/403 lines (78.91%). PostgreSQL/MySQL native fixtures were not configured (16 skips);
  renderer consistency passed for both. No new platform/backend qualification is claimed.
- sql-profile's actual direct/CLI/buffered-HTTP reference-byte parity and real setup/CLI
  executable comparisons passed in the workspace suite. The example itself is unchanged.
- Evidence/scripts/consumer sources/logs: `/private/tmp/stackpress-00007b-sql-where-eq/`.
  All directly invoked builds, the documentation consumer and LLVM coverage used one configured
  target there via `CARGO_TARGET_DIR` and `CARGO_LLVM_COV_TARGET_DIR`. It was deleted after all
  code gates. Existing workspace tests also use self-cleaning temporary consumer targets.
- KB validator baseline in this isolated worktree: **exit 1, 146 errors / 21 warnings**,
  with missing relative links into absent sibling checkouts and existing KB warnings.
  These are outside this task; baseline output is retained for comparison after this update.
  This implementation receipt is evidence for coordinator review, not task verification.

## Verification notes

Coordinator review on 2026-10-26. Result: **verified**; merged as `4faf7ae`.

- **Scope:** only `crates/sql/**` and this task file changed. No stale "replaces"/"does not expose IDs" wording remains in either SQL doc set.
- **Regression-first evidence:** recorded by the implementer. Before the fix the two core regressions failed (0 passed / 2 failed), rendering only `WHERE "active" = ?`.
- **Checks rerun by the coordinator:**
  - fmt passed; Clippy and Rustdoc `-D warnings` clean; `cargo +1.96.0 check` passed;
  - 436 workspace tests and 855 doctests passed;
  - sql-profile against capability-host parity: 12/12 byte-identical.
- **Coverage:** core-only 95.41% (implementer figure).

## Acceptance notes

Library work has no visual acceptance artifact; this task finishes at `verified` after coordinator review.
