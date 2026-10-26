# Slice 4 lane A M1 receipt

Task: [00007 Slice 4 SQL](tasks/00007-slice-4-sql.md). Authority: G24, the coordinator's M1 release, and the [accepted milestone table](slice-4-design-verification.md). This receipt covers SQL core data/rendering only and stops at M1 for coordinator review. It does not mark task 00007 verified or release M2/refactor rounds.

## Baseline and write set

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-lane-a-core`.
- Branch: `trial/3-s4-lane-a-core`; clean starting HEAD `8985fe03300237023adcedc147b0e90d4ea1af26`.
- Owned implementation: `crates/sql/sql/**`. This receipt is the only requested `.agents` write, subject to separate approval.
- No root manifest/lock/facade, SQLite adapter, lib, server, CLI, Idea, task/status or Slice 5 changes. No new dependencies; the core still has zero normal or dev dependencies.
- `Database` and `TransactionSession` declarations are byte-identical to M0. Existing Engine/Transaction execution signatures and SqlValue variants are unchanged. Additions are detached builder conveniences, value/row APIs and pure rendering data.
- No commit, push, listener, publishing, deployment, KB promotion or messages through Paseo.

## Implemented M1 behavior

The [core crate](../../../crates/sql/sql/src/lib.rs) exports all six detached builder families: Select, Insert, Update, Delete, Create and Alter. Owned clones remain independent from an executor and each other.

- DML retains M0's `Render -> Query` boundary. Create/Alter have inherent `render(DialectId) -> Result<QueryPlan>` methods. Plans expose ordered, individually validated statements and a whole-plan budget/dialect validator; they do not execute effects. No arbitrary batch-text constructor is exposed.
- SELECT supports explicit selectors/wildcards, aliases, qualified identifier segments, all five join kinds with checked ON semantics, grouped predicates, ordered sorts, distinct, limit and offset. Some(0) remains zero rows; absent limit differs from zero. Offset-only SQL uses SQLite LIMIT -1. Pagination outside i64 is rejected.
- The existing SELECT `where_eq` replacement behavior is preserved, including cloned builders. `and_where(Expr::eq(...))` appends additional equalities. New Update/Delete filters append in order.
- INSERT supports explicit stable column order, one/bulk rows, exact arity, RETURNING columns/all, SQL NULL and explicit default states. SQLite omits wholly defaulted columns and supports an explicit single DEFAULT VALUES row. Mixed per-cell DEFAULT and bulk DEFAULT VALUES return Unsupported rather than substituting NULL or producing invalid SQL.
- UPDATE validates unique nonempty assignments; DELETE/UPDATE require a predicate or explicit `all_rows()`. An empty/invalid expression remains an error even with all_rows. Both support RETURNING and shared JSON filters.
- Expr covers checked column/value operands, typed comparison operators, grouped AND/OR, NOT, NULL tests, IN, escaped LIKE, JSON extraction/equality/membership and explicit dialect-tagged audited fragments. Nested comparison operands retain parentheses. Empty groups/IN/native fragments error. No raw comma/function selector parsing is inferred from ordinary strings.
- JSON uses typed key/index paths, checked per-path convenience options, standard JSON string encoding, and bound paths/scalars. Numeric-looking key `01` remains a key. JSON equality checks type as well as value; JSON null differs from missing/SQL NULL. Membership is limited to direct scalar elements of an array, with separate bool/integer/real/text/null types. Invalid JSON execution is a native adapter failure, not manufactured empty results.
- Explicit JsonValue encoding is bounded and preserves ordered objects/arrays and scalar types; duplicate keys/nonfinite numbers error. `SqlValue::json` explicitly encodes JSON as native Text. Dates remain explicit canonical UTC text with calendar/time validation and no ambient-timezone or epoch fallback. Native returned JSON/date cells remain text until caller-selected decoding; there is no automatic JSON decoder.
- SqlValue adds lossless primitive inputs, checked u64 conversion and strict nullable/range/finite-real/blob decoding. No variant was added, preserving lane B's exhaustive native binding match. Row preserves cells/order/duplicate names, ordinal reads, optional driver declaration metadata and checked retained-byte accounting. Metadata names borrow existing cells rather than duplicating retained name strings. Adapters attaching declaration text must include `Row::retained_bytes()` in their accounting.
- CREATE owns typed fields/defaults/lengths, ordered primary/compound foreign keys and named indexes. Explicit false/zero/empty/NULL defaults survive. DDL strings are escaped literals, never inferred function calls. Unknown types/expressions require TrustedSql plus a matching dialect tag. SQLite rejects unsigned enforcement, column comments, exact-decimal storage and incompatible auto-increment instead of discarding intent. VARCHAR length is explicitly documented affinity metadata, not enforced length.
- Same-name unique/index normalization preserves the stronger unique definition only for identical ordered fields; incompatible definitions error in either insertion order. This intentionally follows the accepted Rust contract rather than the upstream conflicting-field precedence accident.
- ALTER retains typed add/change/remove/rename/index/primary/FK operations. Explicit original field metadata is required for rename source validation. Matching remove/add halves normalize into a rename; changed field semantics remain a separate ChangeField. Equal names, occupied/duplicate targets, duplicate sources and missing halves/source metadata error. SQLite supports native add/drop/rename/index plans and rejects reconstruction-requiring changes before returning any plan.
- Detached drop/rename/truncate commands validate names; SQLite truncate renders DELETE, leaves sequence behavior unchanged and rejects cascade. Engine's builder/command conveniences perform no IO and capture no engine.

## M1 inventory reconciliation

Source row numbers refer to [the core test inventory](slice-4-design-tests-core.md). Multiple assertions in a Rust test protect related input/branch combinations; these are behavior tests rather than a one-to-one translation of JS mocks.

| Source rows in M1 | Passing core evidence |
| --- | --- |
| Alter 1-3 | `alter_native_plan_and_inspection`, `create_and_alter_unique_precedence_and_conflicts`, `sqlite_alter_rejects_unsupported_operations_as_a_whole`; typed unsupported operations remain inspectable. |
| Alter 4-9 | `alter_direct_rename_and_reconciliation`, `alter_rejects_ambiguous_or_incomplete_renames`; changed-definition rename is retained and rejected by SQLite before a plan is returned. |
| Create 1-3 | `create_full_definition_and_ordered_plan`, `create_and_alter_unique_precedence_and_conflicts`, field/default/foreign-key tests in `tests/ddl.rs`. |
| Delete 1/7, Update 1/7 | `update_and_delete_build_and_returning`, `json_membership_and_dml_share_one_expression_contract`; scope/empty-clause replacements in `mutation_scope_never_silently_widens`. |
| Engine 6/18-23/29 | `engine_conveniences_never_attach_or_execute`, `table_drop_rename_truncate_and_invalid_names`; actual query policy/error/result forwarding is covered in `tests/engine.rs` and retained M0 adapter tests. |
| Helpers 1-2/4 replacements | `hostile_identifiers_and_values_stay_data`, `bounded_explicit_json_encoding`, `json_paths_are_bound_structural_data`, retained source/disclosure tests. Helpers 3 is field equality/schema-diff work at M2. |
| Insert 1/6-8 | `insert_bulk_returning_and_inspection`, `insert_defaults_are_explicit_and_never_inferred`, `insert_rejects_empty_arity_duplicate_and_conflicting_input`. Explicit columns plus one `values` call supply a single row; repeated calls supply bulk rows. |
| Select 1/3-5/10 | `select_build_and_complete_sql`, `select_defaults_and_selector_boundaries`, `select_sort_and_pagination`, path/filter inspection and binding tests. |
| SQLite 1-10/12-16/18-19/28-30: pure render/validation portions | All CRUD/DDL/JSON tests in `tests/ddl.rs`, `tests/dml.rs`, `tests/json_values.rs`; native execution of new DDL/JSON plans is M2 lane B. |
| SQLite 11/17/20-27 replacements | `typed_defaults_preserve_false_zero_empty_and_quotes`, `default_and_attribute_errors_are_typed`, `explicit_trusted_ddl_syntax`, `fields_validate_types_lengths_and_auto_increment`, invalid/empty DML and table-name tests. |
| Attached engine/thenable/implicit dialect replacements across all six families | Independent-clone tests, no-IO Engine convenience tests and two compile-fail doctests: awaiting a builder or rendering without an explicit dialect. Existing transaction/Send/Clone compiler proofs also pass in the workspace suite. |

The following Now rows are explicitly outside this lane's M1 gate, not reported as completed by snapshots:

- PostgreSQL/MySQL source suites: M2 renderers and G26 opt-in native validation; no network server was contacted here.
- Engine 1-5/7-17 and Helpers 3: TableDefinition/SchemaDiff and full structural equality/rename diff at M2.
- Create 7 and Alter 13 atomic multi-statement execution/outcomes: M2 execute_plan/adapter work. Pure ordered plan validation is present now.
- Engine 24-26 and JS future replacement rows: M0/native execution contracts remain covered by the unchanged adapter suite; lane B owns extended real execution, prepared arity and native outcome scenarios. The core's immediate-future driver tests prove policy forwarding/error preservation only, not SQLite IO, cancellation or affinity.

## Implementation evidence and milestone stop

- Core: **45 tests passed**, 0 failed/ignored. Six core doctests passed (four working examples, two intended compiler failures).
- A failing nested-comparison regression reproduced `a = b = ?` where the typed tree required `a = (b = ?)`. Operand rendering now preserves nesting and the regression passes.
- Resource checks now reserve aggregate bind bytes before retaining/cloning each next value. Wide selector/group/sort/filter and CREATE definition joins enforce a SQL byte budget incrementally instead of joining every fragment first. Query-level validation remains defense in depth.
- Hostile-input coverage includes embedded dialect quotes/dots, literal SQL-looking identifier/value/default/path text, controls/NUL, oversized identifiers/values/defaults/JSON/paths, expression depth/nodes, bind count, exact integers, invalid UTC dates, duplicate columns and nonfinite values.
- No dedicated slice refactor rounds were run. [Coordinated workflow section 5.3](../../workflows/coordinated-crate-development.md) and this milestone release require stopping before those rounds. The coordinator releases M2 and later joint rounds separately.

## Final checks

Compiler/target: rustc/Cargo **1.96.0**, LLVM 22.1.2, `aarch64-apple-darwin`; cargo-llvm-cov **0.8.6**. No toolchain/platform/dependency was installed.

An initial default-debug workspace build failed with host ENOSPC while compiling unchanged workspace crates. Only this worktree's disposable Cargo target was cleaned, after preserving logs. The successful workspace checks below used `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0` to reduce disk use; no source or Cargo manifest was changed for this adjustment.

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked --offline
cargo test --workspace --doc --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
cargo +1.96.0 check --workspace --all-targets --all-features --locked --offline
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql --no-deps --locked --offline
cargo tree -p stackpress-sql --edges normal --locked --offline
git diff --check
```

All passed. Workspace results: **315 passed**, 0 failed, 27 ignored entries; **436 doctests passed**, 0 failed/ignored. Existing ignored watchdog child entrypoints remain driven by their parent tests, including the SQLite ownership/runtime proofs; they are not counted as directly passed ignored entries. The normal-edge SQL tree contains only stackpress-sql itself. All tracked/untracked implementation changes stay under `crates/sql/sql/`.

## Fresh core-only coverage

The coverage target is private to this lane: `CARGO_TARGET_DIR=target/s4-m1-coverage`; `CARGO_INCREMENTAL=0`. The measurement ran **stackpress-sql alone**, without any SQLite adapter tests or other packages, after a clean in that target:

```sh
CARGO_TARGET_DIR=target/s4-m1-coverage cargo llvm-cov clean --workspace
CARGO_TARGET_DIR=target/s4-m1-coverage CARGO_INCREMENTAL=0 \
  cargo llvm-cov -p stackpress-sql --locked --offline --fail-under-lines 70 \
  --ignore-filename-regex '/tests/' --json \
  --output-path /private/tmp/stackpress-s4-lane-a-m1-evidence/core-coverage.json
```

**2,028 / 2,063 executable first-party library lines = 98.30%**, above the 70% development floor. Only test-source paths are excluded; no production code is excluded. Stable doctests ran separately. File coverage: ddl 303/324; dml 419/424; engine 94/94; error 45/45; expr 276/277; identifier 59/62; json 142/144; query 184/184; schema 257/258; table 46/46; value 203/205. Remaining lines include convenience/defensive input paths and accounting-overflow guards; this is line coverage, not branch or native-execution certification.

Exact logs, full per-file JSON and the preserved initial ENOSPC log are in `/private/tmp/stackpress-s4-lane-a-m1-evidence/`. These are local disposable evidence; the commands/counts above are retained in this receipt.

## Limits and next gate

- M1 supplies a complete data/rendering foundation for the six builders; network renderers/capability matrix, TableDefinition/SchemaDiff, execute_plan and native new DDL/JSON execution remain M2. G26 is sequenced at M2 by this release.
- Only the installed macOS ARM64 target was checked. No Linux/Windows/WASM/mobile/network-filesystem/network-adapter guarantee follows from these results.
- Trusted SQL remains trusted code, not a sanitizer or sandbox. Builder constructors retain caller-provided data before rendering; limits constrain rendering/execution input rather than arbitrary caller allocations. SQLite native allocation/time limits remain the existing adapter caveat.
- No automatic schema introspection, table reconstruction, migration ledger, general generated stores, retries, immediate cancellation/exactly-once outcome or full-platform claim is introduced.
- Baseline Agent Workspace validation has **146 existing errors and 20 warnings**, chiefly relocated sibling-repository links. No existing KB record is edited. The post-write comparison is reported in the handoff; code checks do not claim green KB validation.

Stop for coordinator re-verification of M1. Task/status ownership, shared integration and release of M2/refactor rounds remain with the coordinator.
