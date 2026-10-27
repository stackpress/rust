# Task 00011: Slice 6 M0 implementation receipt

Date: 2026-10-27. Branch: `trial/3-s6-m0`. Base and unchanged HEAD: `7029d831f116de607f2a7bb2bfa34cb32a341c7e`.

This receipt covers the four M0 proofs only. Implementation and verification are complete; coordinator re-verification is pending. No M1 work, publication, commit, push, installation, listener or network database was started. This draft requires coordinator approval before being added to the protected KB.

## Scope and public boundaries

- Added `stackpress-framework-schema`, `stackpress-framework-sql`, and the isolated generated workspace tool, native app and pure contracts consumer.
- Added only the granted shared APIs: checked `TransformError::with_source(kind, context, cause)`, read-only `Transaction::dialect()`, and feature-gated SQLite maintenance. Database/TransactionSession traits and upsert are unchanged.
- Root additions are the five members, two optional dependencies and opt-in facade features/exports. Existing default features remain unchanged.
- Cargo.lock adds workspace package entries/dependencies only; it introduces no external package or version. No template engine, direct syn dependency or new procedural macro was added.
- Final scope audit found exactly 66 permitted modified/new files, no other slice edits and no KB changes. Evidence: `/private/tmp/s6-m0/scope-audit-final.json`.

## M0 proof 1: generated contracts, stores and borrowing

The existing Idea composer/finalizer supplies a real checked `Schema`; schema and SQL transforms share one frozen app-owned registry and definition selection. The accepted Profile/Project/Task schema generates actual compiled contracts, input preparation, SQL definitions and create/get stores. A separate pure package imports the identical generated contract source.

The bootstrap script moved the existing output aside, asserted its absence, compiled the independent tool, generated output, compiled both consumers, checked freshness, regenerated and compared every SHA-256. All 15 files (40783 bytes, including ownership metadata) matched exactly. The tool dependency tree has no generated-app dependency.

Native generated-store tests use `Profiles`, `Projects` and `Tasks` on the exact `DataSession::transaction(&mut Transaction)` across an await. All uncommitted writes are visible inside, invisible through the outside pool, and absent after rollback. Explicit commit persists. A store future is also boxed as `Future + Send`. No store captures an Engine and no ambient transaction exists.

The named app-owned `profile_name` handler compiles and registers with the current typed service API, checks an explicit application policy and returns a name-only projection. Six compiler rejection fixtures prove session aliasing, Clone, pooled escape, transaction escape, service registration of the non-Sync session and a non-Send handler are unavailable.

Contract tests run against native and pure imports: omission versus null, required/default fields, enum variants and invalid values, nested fieldsets, bounded lists, unknown fields, validation, roundtrip and strict storage codecs. Regression tests preserve `@active false`, exclude inactive rows from ordinary get, reject unsupported top-level default providers, preserve provider-looking literal list elements and reject scalar relation metadata.

The default schema graph contains Serde/JSON only. An actual installed `wasm32-unknown-unknown` consumer check passed. This proves contract compilation; it is not a Slice 8 UI/browser application acceptance result.

## M0 proof 2: dedicated SQLite maintenance

`SqliteMaintenance::open` owns a dedicated connection. Consuming `begin_rebuild` checks FK enforcement on, sets it off, reads it back off and only then issues BEGIN EXCLUSIVE. `MaintenanceSession` is Send, non-Clone and non-Sync, exposes checked queries/catalog/FK results and consumes commit/rollback. No raw connection or arbitrary PRAGMA setter is exposed.

Production adapter tests execute on independent files under `/private/tmp`. The rebuild preserves an inbound CASCADE child while replacing its parent, updates the ledger/head in the same transaction, reopens and verifies data/head/FK state, then demonstrates later normal FK enforcement and cascade behavior. A deliberately failed required-column copy retires the uncertain connection and leaves exact prior catalog, rows and ledger after rollback.

The adapter now enforces final `foreign_key_check` inside commit itself. A violated reference plus partial ledger change cannot commit and both roll back. Mutation and native-COMMIT cancellation use barriers; uncertain connections retire, explicit cleanup is awaited and subsequent opens start with FK enforcement enabled. A competing writer has a finite Busy outcome and later recovery.

Every hang-class scenario runs in a child under a parent monotonic deadline with bounded captured output and kill/reap on failure, timeout or unwind. Four native maintenance scenarios use 10 seconds; a deliberately parked child must be killed/reaped by the 150ms negative-control deadline. Build/compiler children have separately bounded deadlines (up to 180 seconds). A timeout in a positive case fails the test.

One narrow SQL rendering seam is explicit: `CopyPlan` accepts checked portable identifiers and typed column/literal sources, builds the SELECT with core renderers, joins the fixed INSERT-SELECT grammar and revalidates it through the existing trusted single-statement gate. It does not accept caller SQL or bypass native query validation. Adapter-owned transaction/PRAGMA controls stay private. The ordinary PRAGMA/BEGIN/COMMIT rejection gate was not edited and its rejection assertions pass. No unsafe code was introduced.

Retirement on Drop initiates cleanup; a live runtime and awaited explicit completion/rollback/close remain required for acknowledged cleanup. Applications must quiesce and close ordinary pools before maintenance.

## M0 proof 3: approved native plan and reconciliation

Owned immutable `RevisionSnapshot`, `RenameMap`, opaque `PlanHash`, `MigrationPlan`, planner and runner types implement the bounded native subset. Hashes derive from canonical owned structure rather than Serde JSON map iteration order. The runner locks the dedicated writer before authoritative schema/ledger checks, applies effects and ledger/head together and verifies the resulting schema/FKs.

The real file-backed test installs a baseline, preserves a populated `name` column through explicit `name -> heading`, adds nullable `summary`, verifies rows/schema/ledger, and reverses both when summary remains NULL. A populated summary prevents reverse and preserves data/head.

Missing/ambiguous mappings, duplicate rename mappings, a required addition with no approved backfill, and an incorrect approval hash are rejected. M0 exposes no arbitrary backfill execution API; unsupported required/backfill changes are refused rather than guessed. Unknown/unowned catalog objects and unsupported automatic integer identity are also refused by this subset.

Two independent processes open their maintenance owners before a shared start barrier and race the identical baseline plan. One returns Applied and one AlreadyApplied; the resulting ledger contains one application. Losing the application reply after a real commit is simulated by discarding that acknowledgement and applying the same plan again: the ledger produces AlreadyApplied without replay or duplicate rows. The adapter separately cancels an actual in-flight native COMMIT and proves reopening observes its possibly committed result. These are distinct controls; no claim is made of a complete driver/network commit fault matrix.

## M0 proof 4: final names, features and evidence

The complete M0 suite was rerun using the final proposed names and production implementations. Separate checks pass for schema default/codegen; SQL default/server/codegen/migrations/sqlite-maintenance; tool database-tools; opt-in root facade; and the real WASM contracts consumer. Dependency trees confirm pure contracts contain no server, CLI, Idea, SQLx or Tauri dependency and the tool has no generated-app back-edge. These proposed APIs are not the M1 public contract freeze.

All evidence below is local and retained in `/private/tmp/s6-m0/`. `final-results.json`, `proof-final-results.json` and `matrix-results.json` contain exact argv, exits and durations. Reproduction scripts are `verify-final.py`, `bootstrap.py`, `finish-proof.py` and `export-coverage.py` in that directory. The scripts are evidence helpers, not added product code.

The main commands below ran from this worktree with one target directory, `CARGO_INCREMENTAL=0` and `CARGO_PROFILE_DEV_DEBUG=0`. `TMPDIR` was set under that target for the final verification. Both opt-in network database URL variables were absent and explicitly removed. Network PostgreSQL/MySQL execution is skipped, not passed.

```sh
export CARGO_TARGET_DIR="$PWD/target"
export CARGO_LLVM_COV_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
export TMPDIR="$PWD/target/tmp"
export LLVM_PROFILE_FILE="$PWD/target/verify-%p-%m.profraw"
unset STACKPRESS_TEST_POSTGRES_URL STACKPRESS_TEST_MYSQL_URL
set -- -p stackpress-framework-schema -p stackpress-framework-sql \
  -p generated-workspace-tool -p generated-workspace-app \
  -p generated-workspace-contracts -p stackpress-sql-sqlite \
  -p stackpress-idea-transformer -p stackpress-sql
FEATURES=stackpress-framework-schema/codegen,stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance
cargo fmt --all -- --check
cargo +1.96.0 check "$@" --features "$FEATURES" --locked
cargo clippy "$@" --all-targets --features "$FEATURES" --locked -- -D warnings
cargo test --workspace --all-targets --locked --features stackpress-framework-sql/sqlite-maintenance
cargo test --workspace --doc --locked --features stackpress-framework-sql/sqlite-maintenance
RUSTDOCFLAGS='-D warnings' cargo doc "$@" --no-deps --features "$FEATURES" --locked
cargo run -p generated-workspace-tool --locked -- emit generate
cargo run -p generated-workspace-tool --locked -- emit generate --check true
cargo check -p generated-workspace-contracts --target wasm32-unknown-unknown --locked
```

All listed Cargo commands exited zero. The final workspace log reports 491 passes, zero failures and 59 existing ignored helper entries; supervising parents explicitly launch the relevant child scenarios. None of the new M0 critical cases is ignored. Workspace doctests report 868 passes, zero failures/ignored. Exact logs: `fmt-check.log`, `msrv-final.log`, `clippy-final.log`, `workspace-final-tests.log`, `workspace-doctests.log`, `rustdoc.log`, `bootstrap-*.log`, feature-matrix logs, `contracts-tree.log`, `tool-tree.log` and `facade-tree.log`.

### Coverage measurement

Host/compiler: rustc 1.96.0 (ac68faa20), Cargo 1.96.0, `aarch64-apple-darwin`, LLVM 22.1.2. The preinstalled `cargo-llvm-cov 0.8.6` and LLVM tools were used; no tool was installed. An isolated workspace coverage clean preceded the final instrumented suite to avoid mixing uninstrumented artifacts.

```sh
unset LLVM_PROFILE_FILE
cargo llvm-cov clean --workspace
cargo llvm-cov "$@" --features "$FEATURES" --locked --no-clean \
  --json --output-path /private/tmp/s6-m0/coverage.json
cargo llvm-cov report --json --output-path /private/tmp/s6-m0/coverage-diagnostic.json -vv
python3 /private/tmp/s6-m0/export-coverage.py
```

The exporter reruns the logged LLVM export without its default example-file exclusion. It counts executable production `src/` lines per changed crate, including generated shipped app code and pure imported contracts, and excludes actual test modules/fixtures. Doctests are verified separately and not instrumented. No workspace average or blanket examples exclusion is used. Every measured scope clears the 70% development floor; this is not the separate 90% publication gate.

| Production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-schema | 792 / 859 | 92.20% |
| framework-sql | 727 / 810 | 89.75% |
| idea-transformer | 2011 / 2149 | 93.58% |
| sql-sqlite | 640 / 718 | 89.14% |
| sql core | 2938 / 3076 | 95.51% |
| generated app (including shipped output) | 401 / 505 | 79.41% |
| pure contracts consumer | 215 / 258 | 83.33% |
| app-owned tool | 109 / 155 | 70.32% |

Raw coverage, exact export argv and aggregation are retained as `coverage-all-sources.json`, `coverage-export-command.json` and `coverage-summary.json`; `coverage.log` records the executed test suite.

## Test-verified review rounds

1. Complexity/branch behavior: canonical hashes avoid map-order feature drift; strict integer-to-real conversion rejects precision loss. Tests remained green (`round1-tests.log`).
2. Responsibility/abstraction: split schema naming, emission and diagnostics; isolate SQLite runner orchestration from pure plan construction. Tests remained green (`round2-tests.log`).
3. Consistency/Rustdoc/comments/errors: document public APIs and execution boundaries; preserve concrete causes under distinct context, reject unsupported providers/relations and preserve active semantics. Final fmt/Clippy/Rustdoc/workspace tests and doctests pass.
4. Soundness/resource follow-ups: commit enforces FK integrity; aggregate metadata preflight runs before cloning; bounded record serialization rejects the byte budget before later codec work. Regressions failed before fixes and passed after, then the complete final suites passed. Evidence: `integrity-tests.log`, `budget-regression-before.log`, `budget-regression-after.log`, `record-budget-before.log`, `record-budget-after.log`, `active-regression-before.log`, `active-regression-after.log`, `providers-tests.log` and `relation-contract-final.log`.

The branch audit explicitly covered cancelled mutation/commit retirement, consumed ownership, writer contention, invalid copy/FK/ledger rollback, reversal data preconditions, stale approval and post-commit reconciliation. Untested broader reconstruction/DDL failure transitions remain later-milestone work, not implied by coverage percentages.

## Limitations and execution deviations

- M0 supports the accepted fixture's String/Text/Integer/Boolean, string enum, bounded list, acyclic fieldset, explicit relation, literal default and nonempty validation subset. Full numeric/date/object wrappers, rule/provider/security matrix, public UI descriptors and semantic diagnostic attachment remain M1. Unsupported selected semantics fail generation.
- Current-thread native SQLite create/get is proved. Remaining CRUD, atomic upsert, relation includes/pages/query counts, generated operations, exposure contracts and direct/CLI/HTTP parity remain M2. The named handler is handwritten application wiring.
- Snapshots/plans are owned in memory. Durable revision directories, complete history validation, general reconstruction planning, sequence/index/view/trigger matrix, destructive/backfill execution and every DDL/ledger fault transition remain M3. Dedicated reconstruction is separately proved by the adapter test.
- `database-tools` selects maintenance dependencies but registers no unfinished DB commands. The full integrated DX/seed/migration CLI is M4. PostgreSQL/MySQL execution and UI/native/mobile packaging are unrun.
- Existing CLI long options require a value, so freshness uses `--check true` or supported `-c`; explicit `--check false` wins. The bare design shorthand is not accepted by the current CLI. No shared CLI edit was needed.
- The existing Idea aggregate emits module declarations without Rustdoc. Only those declarations receive a documented `missing_docs` allowance; generated leaves have Rustdoc. No parser/transformer output semantics changed.
- An early existing workspace fixture created its own disposable target under the system temp directory and cleaned it. Later runs set TMPDIR inside this worktree's sole target. Mixed instrumented execution also created 526 raw profile byproducts outside target; those owned artifacts were moved to the private evidence directory, with `moved-profile-artifacts.json`. Final audit found none left in source paths.
- KB validation before any KB write exited 1 with 146 inherited missing-link errors and 21 warnings (isolated-worktree upstream paths and existing KB findings). See `kb-baseline.log`. It is not reported as passed. Approved receipt validation must compare against this baseline.

The worktree target was deleted after all verification; `target-cleanup.json` records successful removal. Source is uncommitted, HEAD is unchanged and no other target, branch or trial output was used or removed. No additional shared change is requested. Stop here for coordinator re-verification and approval of this receipt; do not begin M1.
