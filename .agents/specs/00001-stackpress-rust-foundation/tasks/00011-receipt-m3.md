# Task 00011: Slice 6 M3 implementation receipt

2026-10-27. Worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s6-m0`, branch `trial/3-s6-m0`, HEAD `eb7f7fcdba0a05a5503e10b0ad52a9ad0611b48a`. Implementation is ready for coordinator re-verification. G35 approved the two adapter additions and was merged from `trial/3`. M4 has not started. No commit, push, installation, listener, network database or GitHub write occurred.

## Delivered scope

- Immutable format-2 framework storage DTO snapshots and explicit parent/source/generator/definition identities; cooperative create-new publication, retained interruption state, canonical byte/hash checks, complete parent-chain checks and NoChanges. History stays separate from disposable generated output. Re-rendered plan recipes and SQL companions include exact statements/binds, mappings, findings, reverse authority and physical order separately from logical/UI order.
- Explicit field/table mappings; same-shaped candidates are shown and never accepted automatically. Deliberate removal and typed literal backfills are explicit. A pure aggregate review reports every loss finding even when a missing backfill prevents executable planning; blocked inspection creates no plan authority. Unsupported conversions remain blocked regardless of loss approval.
- Explicit checked reverse or recovery refusal; populated nullable additions refuse reversal until all values are NULL, including additions with defaults. Dropped data, required additions, backfills and seed effects have no automatic data-preserving reverse. Downgrade requires the separately published reverse hash.
- SQLite native/reconstruction execution under one dedicated exclusive transaction, with locked source/ledger/catalog checks, row counts, FK checks, index/constraint preservation and one ledger/head commit. Reserved format-2 `_stackpress_migrations` / `_stackpress_schema_state` track the applied chain. Drift, unknown formats, collisions, unmanaged schemas and unknown dependencies refuse. Unrelated independent tables survive. Unowned views/triggers are refused conservatively; single-quoted references are recognized.
- G35 only: `MaintenanceSession::sequence_high_water` and `preserve_sequence_high_water`, private helpers and supervised cases in exactly the two granted SQLite files. AUTOINCREMENT high-water marks survive empty/nonempty forward/reverse rebuilds and deleted maximum IDs. Plain INTEGER PRIMARY KEY reuse is explicitly reported and documented. No other shared crate, root manifest, lockfile, facade or frozen contract/metadata file changed.
- App-owned `emit` events: revision, history-check, migration-plan, db-status, install/upgrade/downgrade/purge/uninstall/populate. DB events require database-tools; normal startup opens nothing. Strict options, target/hash-bound loss approval, global-force refusal, bounded named typed seeds, create/upsert semantics and whole-seed rollback are proved. Capability refusal occurs before the tool opens maintenance. No new CLI parser or binary was added.

The [migration API and operator reference](../../../../crates/framework/framework-sql/docs/api/migrations.md) documents the actual APIs, commands, allocation behavior and limits. Existing G34 contract revision 2 and original contracts/metadata bytes remain unchanged; framework-sql's runtime compatibility constant remains 1. Migration/ledger format 2 is a separate private-native format.

## Verification and review rounds

All logs, exact argv/result JSON, scripts and raw coverage are retained in `/private/tmp/s6-m3/`. Compiler: rustc/Cargo 1.96.0 (ac68faa20), aarch64-apple-darwin, LLVM 22.1.2; existing cargo-llvm-cov 0.8.6. Every Cargo run used one owned target root, CARGO_INCREMENTAL=0 and CARGO_PROFILE_DEV_DEBUG=0; PostgreSQL/MySQL URL variables were removed. Parent watchdogs kill/reap on timeout or unwind; no positive native test timed out.

| Gate / evidence | Result |
| --- | --- |
| final-scoped; final-matrix; final-names | Seven-package scope: 158 runtime tests and 432 doctests passed, 25 gated/helper entries ignored; alternate fixtures add 9 passes. Later focused tests cover all final migration/tool changes, aggregate review and seed/index atomicity. |
| sqlite-default; sqlite-maintenance | Existing default suite: 14 runtime + 3 doctests passed, 9 helpers ignored. Maintenance: 16 runtime + 3 doctests passed, 9 helpers ignored. Existing cases remain unchanged; new sequence cases are supervised. |
| aggregate-closeout-tests; aggregate-clippy; aggregate-feature-*; aggregate-rustdoc | Final changed-package tests, default/pure-migrations/maintenance feature variants, all-target Clippy -D warnings and warning-clean Rustdoc passed. |
| final-msrv; final-wasm; tree-* | Explicit 1.96 check and actual wasm32-unknown-unknown pure contracts check passed. Contracts have no native/server/CLI/Idea/Tokio edge; default tool has no SQLx; maintenance tool has no generated-app back-edge. |
| generation-2 through generation-5; bootstrap-* | All four owned output checks fresh. With the real generated root absent, the tool builds alone, regenerates all 18 files byte-stably, and both consumers compile. Exact original root restored. |
| fixed tests/fixtures/m3 SQL snapshots | PostgreSQL/MySQL baseline/up/down rendered bytes pass; neither backend is applied. MySQL indexed TEXT/targeted-conflict restrictions remain visible renderer errors. |
| documented-example | Lookup Rust example compiled offline in a disposable library under the owned target root. |
| coverage-*; coverage-summary.json | 170 runtime tests passed, 0 failed, 25 gated/helper entries ignored; no added skips. Doctests pass separately and are not instrumented. Per-crate results below. |
| KB and final scope checks | 146 inherited errors and 22 warnings before/after; scope, staged set, frozen bytes and whitespace checks passed. |

1. Complexity/branch review: parent/app checks precede publication; literal/seed/backfill budgets precede retained clones; mappings cannot be stale/unused; reverse drop conditions, ledger/head agreement and status refusals were tightened. `round1-final` passed.
2. Responsibility review: separated storage DTOs, history, pure SQL compilation, policy and SQLite orchestration; source composition has its own tool module. Revision recording and emitted stores now share one checked DDL definition function. `round2` passed; generated bytes stayed unchanged.
3. Consistency/Rustdoc/comments: documented API/state/ownership boundaries, preserved typed causes under distinct context, used safe context independently of cause Display, aligned feature imports and documented operator/native limits. `round3-tests`, Clippy and Rustdoc passed.
4. Safety and follow-ups: actual native failures/cancellation at every enumerated phase through both ledger writes preserve catalog/data/head; actual commit followed by lost response reconciles once. Native COMMIT cancellation is separately proved by the unchanged adapter suite. Extra tests prove second-model seed failure and late unique-index failure roll back the whole operation even with loss approval. UTF-8 catalog slicing and single-quoted dependency regressions failed before their fixes, then passed. FIFO, unknown trigger/view/index, default-bearing reverse, exact reverse hash, wrong target/hash, global force, no-DB capability refusal and logical/physical-order status cases pass. Final aggregate review covers loss findings even when rendering refuses.

## Coverage without production exclusions

The exporter repeats llvm-cov export from the verbose report after removing only its default ignore-filename-regex. The raw export includes every compiled first-party source, including generated/example/test source. No production file, low-scoring function, branch or generated code was excluded. Production totals use every implementation src file in each changed crate; cfg(test)-only modules/integration fixtures are also reported separately in the unfiltered first-party column.

| Changed crate / complete selected scope | Production covered / executable | Percent | All compiled first-party covered / executable | Percent |
| --- | ---: | ---: | ---: | ---: |
| framework-sql: codegen/server/migrations/SQLite | 4672/5245 | 89.08% | 6911/7509 | 92.04% |
| shared sql-sqlite: maintenance | 736/808 | 91.09% | 4943/5058 | 97.73% |
| generated-workspace-tool: database-tools | 510/639 | 79.81% | 1095/1224 | 89.46% |

All changed crates clear the 70% development floor independently. This is not the 90% publication gate or complete branch/platform coverage. Raw file summaries, exact export argv and the source SHA-256 manifest are retained; the initial measurement is preserved in coverage-initial before the final aggregate-review follow-up.

Core reproduction commands from this worktree (full argv, input/output paths and exits are in final-commands.json, final-followup-commands.json, coverage-commands.json and each result JSON):

```sh
export CARGO_TARGET_DIR="$PWD/target" CARGO_LLVM_COV_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 TMPDIR="$PWD/target/tmp"
unset STACKPRESS_TEST_POSTGRES_URL STACKPRESS_TEST_MYSQL_URL
s6_packages=(-p stackpress-sql -p stackpress-framework-schema -p stackpress-framework-sql -p stackpress-sql-sqlite -p generated-workspace-app -p generated-workspace-tool -p generated-workspace-contracts)
s6_features=stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance,generated-workspace-tool/database-tools
cargo fmt --all -- --check
cargo test "${s6_packages[@]}" --features "$s6_features" --locked
cargo test -p stackpress-sql-sqlite --locked
cargo test -p stackpress-sql-sqlite --features maintenance --locked
cargo clippy "${s6_packages[@]}" --all-targets --features "$s6_features" --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc "${s6_packages[@]}" --no-deps --features "$s6_features" --locked
cargo clean
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report "${s6_packages[@]}" --features "$s6_features" --locked
cargo llvm-cov --no-report -p generated-workspace-app --features matrix --test matrix --test matrix_contracts --locked
cargo llvm-cov --no-report -p generated-workspace-app --features names --test names --test names_contracts --locked
cargo llvm-cov report --json --output-path /private/tmp/s6-m3/coverage-default.json -vv
python3 /private/tmp/s6-m3/export-coverage.py
python3 .agents/scripts/validate-agent-workspace.py
```

## Limits and stop

SQLite execution is proved on the recorded native target only. PostgreSQL/MySQL are rendering evidence; application stays refused. Literal backfills are supported; arbitrary conversions/SQL expressions and automatic recovery of lost data are not. History requires cooperative writers and portable regular files; synced directory publication is not a portable crash-atomic transaction or malicious-edit authentication. Unknown catalog features/dependencies and old/unknown ledger formats refuse; adoption/repair and reinstall after uninstall are refused. The applied ledger is bounded to 1024 entries; plans to 128 statements/1 MiB aggregate retained data; schema/seed/mapping/catalog bounds are explicit refusals, not truncation.

The scoped integration suites were used; no listener-capable workspace-wide network/HTTP server suite was launched. No browser UI, Tauri/mobile, network migration, production rollout or publication claim follows. The only target root peaked at about 2.3 GiB observed and was deleted after verification. No owned build/test process remains. Staged set is empty. Stop for coordinator review; do not begin M4.

## Historical preflight retained

The following preflight records the earlier G23 stop. Its incomplete/unmeasured statements apply only to that earlier state; G35 and the completed evidence above supersede them.

**Historical preflight: shared-API request**

Date: 2026-10-27. Branch: `trial/3-s6-m0`. Integrated HEAD: `44aa10fc6490ce37d7ba72ee1d4e976a79173e1a`.

**M3 is incomplete.** This is a preflight receipt and a G23 request, not an implementation or verification claim. The implementer stopped at the brief's explicit shared-crate gate. No M4 work began. The coordinator owns the task status and verification notes; neither was edited.

### Start and contract review

The initially clean worktree fast-forwarded from `25346a2` using `git merge trial/3`. The first sandboxed attempt could not write `ORIG_HEAD.lock` in the parent repository's worktree metadata; the authorized escalated rerun succeeded. Final HEAD is recorded above.

Read AGENTS, the context index, coordinated-crate development, Rust style, testing/quality and Rust programming workflows. Reviewed the task and its verification notes, Slice 6 decisions, migrations, verification and DX contracts, accepted G27/G30/G34, and the M0/M1/M2 receipts. Inspected the existing framework migration planner/runner, SQLite maintenance owner/session and its native tests, and the app-owned tool.

Contract revision 2 and generated contracts/metadata remain unchanged. No shared crate, source, manifest, lockfile, generated output or task-status file was changed. This receipt is the only repository edit.

### Shared capability needed for the high-water preservation case

The M3 verification matrix explicitly requires sequence high-water marks to survive reconstruction forward and reverse. The migration contract places typed sequence inspection in the SQLite adapter. It also permits refusing identity rebuilds until that capability is proved. M0 currently takes that refusal: `RevisionSnapshot::new` rejects every `auto_increment` field.

The current maintenance session exposes checked execution, bounded catalog inspection, FK checks, consuming commit and rollback. It has no typed sequence inspection/preservation capability. `CopyPlan` preserves surviving IDs but cannot preserve the historical sequence when the highest issued rows have been deleted. Framework-side direct writes to SQLite's internal sequence table would move that backend responsibility outside the contracted adapter seam.

Keeping M0's identity refusal is safe. It would leave the requested M3 high-water preservation case unimplemented. Recommended resolution: approve the narrow additive adapter capability below, then release the implementer to finish M3. Alternatively, the coordinator must explicitly confirm that an identity-rebuild refusal satisfies this M3 matrix case; that is not assumed here.

### Concrete G23 proposal for coordinator approval

Requested write scope, only if granted:

- `crates/sql/sql-sqlite/src/maintenance.rs`: add the two maintenance-only methods below, private helpers if needed, and Rustdoc explaining caller ownership and sequence semantics.
- `crates/sql/sql-sqlite/src/maintenance/tests.rs`: add supervised real-file sequence preservation, refusal and rollback cases to the existing parent-watchdog harness.

Proposed additive signatures, using the adapter's existing SQL `Result`:

```rust
impl MaintenanceSession {
    pub async fn sequence_high_water(&mut self, table: &str) -> Result<Option<i64>>;
    pub async fn preserve_sequence_high_water(
        &mut self,
        table: &str,
        minimum: i64,
    ) -> Result<()>;
}
```

Required behavior:

- Both methods use this exact exclusive transaction and its existing cancellation/retirement machinery. They never acquire another connection or commit independently.
- Names use the adapter's conservative checked identifier policy; query values remain bound. No raw connection, arbitrary PRAGMA or general internal-table setter is exposed.
- Inspection is bounded to the selected table, distinguishes no sequence row from an integer high-water value, and refuses duplicate, malformed or negative sequence state. A database with no sequence table reports no sequence rather than creating one.
- Preservation accepts a nonnegative historical minimum, requires the rebuilt target to exist, preserves the greater of its current value and the historical minimum, handles an empty rebuilt table's absent sequence row, and verifies the result. It never lowers a sequence or resets unrelated tables.
- The framework must first verify the managed table/identity definition against its checked snapshot and catalog under the same writer lock. The methods do not adopt unowned tables or infer migration authority.
- Copy/rebuild uses native SQLite behavior to account for surviving IDs; the monotonic restoration then retains deleted-ID history. Failure or cancellation follows existing retirement/rollback behavior; no retry is added.
- Tests cover a deleted high-water row, an empty table, a larger rebuilt value, absent/malformed/duplicate state, invalid names/minimum, explicit rollback and foreign-key policy on reopen. All hang-class cases use the existing parent watchdog with kill/reap; timeout fails.

No SQL-core builder, adapter trait, ordinary raw-query gate, default feature, driver dependency, Server/Lib/CLI/Idea API or frozen Slice 8 contract changes are proposed. These methods remain behind the existing `maintenance` feature. No installation is needed.

### Counterexample and bounded mechanism probe

A disposable Python standard-library SQLite probe was run outside the repository at `/private/tmp/s6-m3/sequence-probe.py`, under a parent process with a 10-second deadline and process-group kill/reap on timeout or unwind. It uses in-memory SQLite only, with no listener or network database. The source and exact result are retained alongside `sequence-probe.log` and `sequence-probe-result.json`.

The probe creates an AUTOINCREMENT table, issues IDs 1 through 100, deletes IDs 2 through 100, reconstructs it by copying surviving explicit IDs, drops the original, and renames the replacement. A second independent run restores the previously inspected sequence minimum before commit.

| Observation | Copy surviving IDs only | Explicit minimum restoration |
| --- | ---: | ---: |
| Historical high-water mark | 100 | 100 |
| High-water mark after copy/drop/rename | 1 | 1 |
| Next generated ID | 2, a previously issued ID | 101 |

Python-linked SQLite version: 3.51.0. Both expected-outcome assertions passed; process exit 0. The preservation run proves a SQLite mechanism only. It does not prove the proposed Rust API, SQLx-linked SQLite version, cancellation, rollback, portability, production safety or M3 completion.

Reproduction commands from the worktree root:

```sh
git merge trial/3
python3 /private/tmp/s6-m3/verify-preflight.py
git diff --check
git diff --no-index --check /dev/null .agents/specs/00001-stackpress-rust-foundation/tasks/00011-receipt-m3.md
```

`verify-preflight.py` retains the supervising parent's timeout and kill/reap behavior, runs the probe, runs `python3 .agents/scripts/validate-agent-workspace.py` with a 45-second deadline, checks the tracked diff, and compares the KB findings to the baseline. The recorded result JSON contains the actual Python executable/argv. All positive process deadlines completed without timeout. The final no-index command also checks the untracked receipt's whitespace.

### Verification, coverage and remaining work

No Cargo build or test ran, and no target directory was created. Per-crate coverage is **unmeasured** for M3; no 70% gate or passing review round is claimed. The four test-verified rounds have not started because production implementation stopped at G23. Earlier milestone percentages were reviewed as history and are not reused as M3 evidence.

The KB validator exited 1 before and after this receipt: **146 inherited errors and 22 warnings**, with identical finding lines (`kb-comparison.json`). No receipt finding was added. `git diff --check` exited 0. The untracked receipt's no-index check emitted no whitespace diagnostics and exited 1 because the new file differs from `/dev/null`. The staged set is empty. This is document validation only, not Rust verification.

All M3 implementation gates remain open: durable history/artifacts/NoChanges, complete rename/loss/reverse planning, expanded ledger/drift checks, native and reconstruction matrices, lock/cancel/fault/unknown-commit tests, app-tool operator commands, PostgreSQL/MySQL rendered snapshots, final quality checks, per-crate coverage and the four review rounds. The existing M0 subset is unchanged. No PostgreSQL or MySQL application is claimed.

No installs, listeners, network database connections, commits, pushes, GitHub writes, coordinator messaging, other-trial edits or sibling Idea access occurred. No owned build target needs cleanup. Stop for coordinator review of the exact shared-API proposal; do not start M4.
