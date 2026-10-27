# Task 00011: Slice 6 M4 integrated DX receipt

2026-10-27. Worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s6-m0`, branch `trial/3-s6-m0`, HEAD `a734d4509e2c3c398a3d10bbfc160df81b1635c2`. The requested `git merge trial/3` fast-forwarded cleanly after the sandboxed attempt required the usual parent-worktree metadata permission. M3's verified merge/record is the starting contract. M4 is implemented and ready for coordinator re-verification; task status and verification notes remain coordinator-owned.

## Delivered example and source boundary

- [Getting-started walkthrough](../../../../examples/data-quickstart/README.md): exact existing `emit` commands, all managed plan SQL/binds, the actual executed SELECTs/binds, and a compiled Rust query function. It uses cached dependencies offline, embedded SQLite, a unique disposable database/history directory and explicit lifecycle steps.
- [Verbatim proof](../../../../examples/data-quickstart/prove.py): builds the independent tool while all three new generated roots are absent, executes all six README shell blocks in one shell, validates rows/relations/search/paging/head, compares every SQL/Rust snippet and compiles the exact Rust snippet in a disposable consumer. All 50 generated files, including ownership manifests, reproduce byte-for-byte; every original root is restored even after failure.
- The new app queries Profile, Project and Task in one explicit read transaction: literal `Ship` search, base-first offset/limit and exact total, then owner/tasks includes. Expected result: Ada, p2/Ship UI, two included tasks, project total 2 and task count 3. The native query app awaits transaction and pool cleanup; it starts no listener.
- Revision 002 adds nullable Project.summary and explicitly maps `project.title=heading`. An unmapped plan reports candidates without execution authority. Upgrade preserves values; the separately hashed reverse succeeds while summaries remain NULL. The original app reads the same selected rows and SQL afterward.
- Revision 003 adds Link and its owning/inverse Profile relation; upgrade succeeds and the expanded app counts zero links. Its automatic reverse is unavailable under the existing recovery policy. The safe 002 round trip deliberately precedes this separate model step; no automatic model-data recovery is claimed.
- Revision 004 removes Task.done. Inspection reports the destructive `drop_column` finding, but unapproved application returns a failure. The complete closed database file is byte-identical before/after; selected rows, counts and ledger stay at revision 003, applied sequence 6.

One new workspace member/package is registered in root Cargo.toml/Cargo.lock, with existing dependency versions only. The existing app-owned tool is reused with explicit paths; its only source change approves quickstart heading/summary/inverse-link fields in the fixture registry. No parser or stock maintenance binary was added. The new app's default path is pure contracts/metadata; native/evolved/expanded explicitly select runtime and schema source. The SQL observation adapter is private demo code and never rewrites SQL.

No shared runtime crate, facade, original generated contract/metadata file or revision-2 shape changed. Original workspace and admin generation checks stay fresh. No MySQL width policy was implemented. PostgreSQL/MySQL migrations remain render-only; their application stays refused.

## Inventory and network evidence

The [schema inventory](../slice-6-design-inventory-schema.md) and [SQL inventory](../slice-6-design-inventory-sql.md) now have dated status/evidence columns on all 52/62 data rows. Every pre-existing cell, section, prose line and decision is mechanically unchanged. Evidence distinguishes implemented Rust replacements, metadata versus rendering, explicit refusals, partial surfaces and named deferrals. In particular, there is structured JSON loss reporting rather than a claimed standalone human formatter, crypto/custom-type refusals remain visible, and automatic rename acceptance stays refused.

Network store evidence is referenced from [task 00012's generated-store receipt](00012-receipt-sql-store-integration.md), not duplicated or rerun. It proves supported PostgreSQL reads/writes and MySQL reads/capability refusals, with clearly harness-owned VARCHAR(191) DDL for MySQL. It does not supply a generated MySQL width default, successful generated MySQL writes, network q/includes/totals, or migration application. The open width options in [SQL closeout](00012-receipt-sql-closeout.md) are unchanged.

## Verification and four rounds

Evidence lives at `/private/tmp/s6-m4/`: each command's actual argv/exit/duration, final-commands.json, feature-commands.json, coverage-variant-commands.json, source hashes, scope audit, complete plan artifacts, public CLI/read reports and compiled-example logs. All Cargo runs use one worktree target, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0 and unset PostgreSQL/MySQL URL gates. The runner uses two build jobs and an external 600-second process-group watchdog with kill/reap; native Rust parents have independent 25-second limits. Positive native tests did not time out. The deliberately stuck proof-control child raises failure and its exact PID is reaped.

| Gate | Result |
| --- | --- |
| final-scoped | Eight packages: 164 runtime tests + 432 doctests passed (596 total), 0 failed; 16 network-gated and 9 supervised/helper entries ignored. No new skips. |
| sqlite-default | Unchanged adapter default suite: 14 runtime + 3 doctests passed; 9 child/helper entries ignored. Maintenance is included in the scoped suite. |
| round2-expanded / round2-variants | New generated callable/authorized-operation lifecycles: 3 passes per native schema version plus 1 pure-default contract pass; 0 failures/skips. |
| final-fmt / final-clippy / final-rustdoc / final-msrv | Formatting, all-target Clippy -D warnings, warning-clean Rustdoc and explicit +1.96.0 check passed. Baseline/evolved Clippy variants also pass. |
| generation-original / generation-admin | Both existing generated roots fresh; original contract/metadata bytes unchanged. New quickstart roots regenerate/check within the README proof. |
| feature-commands / feature-summary | 13 normal dependency trees, 9 native feature checks and 3 actual WASM contracts checks pass. Details below. |
| round4-proof / final-instrumented-proof | Every README command, SQL snippet, exact Rust snippet, absent-output bootstrap, data/page/include result, safe reverse and complete-file loss refusal pass. |
| round4-controls | Four Python proof-safety controls pass: finite failure, watchdog kill/reap, bootstrap failure restoration and partial-move restoration. |
| fresh coverage | 180 instrumented runtime passes; 0 failed, 25 reported gated/helper entries ignored. README's actual CLI/query binaries also run instrumented. Doctests pass separately. |
| KB / scope | Validator exits 1 with the same 146 inherited errors and 24 warnings; no finding added. Source SHA-256 matches, only authorized paths changed, original inventory text retained, frozen/shared bytes unchanged, staged set empty and whitespace clean. |

1. Complexity/branch review: prevalidate all roots before mutation and restore only acknowledged moves in reverse order. A disposable second-move failure initially reproduced cleanup disturbing an untouched root (`round1-partial-move-red`); corrected cleanup passes all controls. Unmapped-plan authority and explicit candidate assertions were tightened. The full real proof passes afterward.
2. Responsibility review: extracted the shared checked-table fixture, host registration and single-model operation lifecycle. Domain query, SQL observation, operator/generator and proof/documentation responsibilities remain separate. No generic framework layer or app→tool bootstrap cycle was introduced. Tests reran across all three native revisions and pure default.
3. Consistency/Rustdoc/comments: documented module/ownership/transaction/cleanup contracts and observation limits; formatted handwritten/generated code; clarified actual native ALTER/index SQL and existing network limits. The proof compiles the README's literal Rust with real dependencies in the same target. Clippy/Rustdoc/MSRV and generation checks pass.
4. Safety/soundness: reviewed source/root ownership, partial publication/restoration, exact hash authority, no raw connection/PRAGMA escape, lock lifetime, awaited rollback/close, typed causes and no automatic retries. Added complete-file comparison across the refused step so an unchanged page cannot conceal other row changes. The positive proof and all failure/watchdog controls pass. No further production edit was justified.

## Feature and dependency matrix

All trees use `cargo tree -p <package> -e normal --offline --locked` with separately selected features. Nine separate feature checks supplement the trees; all native schema variants compile in the test/Clippy sets. WASM checks compile real pure consumers for wasm32-unknown-unknown.

| Package / selected feature | Observed normal edges |
| --- | --- |
| framework-schema default / codegen | Default pure schema; codegen adds existing Idea parser/transformer build-host dependencies. |
| framework-sql default / codegen | Default SQL/schema runtime without drivers; codegen adds the existing schema/Idea generator. |
| framework-sql server | Explicit Server/Lib edge; no SQLx driver. |
| framework-sql migrations | Pure planner/history plus hash dependency; no SQLx driver. |
| framework-sql sqlite-maintenance | Existing SQLx SQLite/Tokio adapter only; no PostgreSQL/MySQL driver edge. |
| tool default / database-tools | Default tool has no SQLx; database-tools adds SQLite maintenance. Neither depends on the query app. |
| original pure contracts; quickstart default / expanded without native | No SQLx, Server, CLI, Idea or Tokio normal edges. All three actual WASM checks pass. |
| quickstart native | Explicit SQLite/SQL/Server/Lib/Tokio runtime. No PostgreSQL/MySQL driver. |

## Fresh per-crate coverage

Rust/Cargo 1.96.0, aarch64-apple-darwin, LLVM 22.1.2 and installed cargo-llvm-cov 0.8.6. Final measurement starts with cargo clean and llvm-cov clean, then all eight packages and explicit baseline/evolved/pure/matrix/names runs. An exploratory mixed-cache measurement is retained in coverage-initial and is not the gate. No files or branches were removed to raise coverage.

The verbose export is repeated after removing only the tool's default ignore-filename-regex, so all generated/example source is visible. Every implementation src file in each declared crate/feature scope is included. Separate test-only modules, dependencies and disposable tooling are not production; the unfiltered first-party totals also include compiled tests/helpers. Source-include origins attribute the pure consumer's exact shipped modules to that consumer, with canonical physical paths used to distinguish test helpers. Raw exports, argv and per-file counts are retained. Doctests are not instrumented; branch/platform coverage is not claimed.

| Crate / complete selected scope | Production covered / executable | Percent | All compiled first-party covered / executable | Percent |
| --- | ---: | ---: | ---: | ---: |
| stackpress-framework-schema (codegen) | 3218/3546 | 90.75% | 4087/4415 | 92.57% |
| stackpress-framework-sql (codegen/server/migrations/SQLite) | 4722/5295 | 89.18% | 7036/7639 | 92.11% |
| generated-workspace-app (default/matrix/names) | 10376/13589 | 76.36% | 13686/16930 | 80.84% |
| generated-workspace-contracts (original/matrix pure) | 1463/2083 | 70.24% | 1999/2621 | 76.27% |
| generated-workspace-tool (database-tools) | 513/642 | 79.91% | 1136/1267 | 89.66% |
| data-quickstart (default/native/evolved/expanded) | 8891/11861 | 74.96% | 9292/12265 | 75.76% |
| stackpress-sql (shared) | 3016/3154 | 95.62% | 6056/6899 | 87.78% |
| stackpress-sql-sqlite (maintenance; shared) | 736/808 | 91.09% | 4982/5098 | 97.72% |

All six Slice 6 crates independently clear 70%; SQL core and SQLite are also reported separately. This is the development gate, not publication's 90% gate or complete branch/platform certification. Remaining defensive observation-lock/close-failure and unsupported backend branches are not certified merely by a percentage.

Core reproduction commands (all native/build groups run under the recorded parent watchdog; full exact feature/tree/argv lists are in the JSON files):

```sh
export CARGO_TARGET_DIR="$PWD/target" CARGO_LLVM_COV_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_BUILD_JOBS=2
export TMPDIR="$PWD/target/tmp" LLVM_PROFILE_FILE="$PWD/target/trial3-s6-m0-%p-%m.profraw"
unset STACKPRESS_TEST_POSTGRES_URL STACKPRESS_TEST_MYSQL_URL
mkdir -p "$TMPDIR"
python3 examples/data-quickstart/prove.py
python3 -m unittest discover -s examples/data-quickstart -p test_proof.py -v
s6_packages=(-p stackpress-sql -p stackpress-framework-schema -p stackpress-framework-sql -p stackpress-sql-sqlite -p generated-workspace-app -p generated-workspace-tool -p generated-workspace-contracts -p data-quickstart)
s6_features=stackpress-framework-sql/codegen,stackpress-framework-sql/server,stackpress-framework-sql/sqlite-maintenance,generated-workspace-tool/database-tools,data-quickstart/native,data-quickstart/expanded
cargo test "${s6_packages[@]}" --features "$s6_features" --offline --locked
cargo clippy "${s6_packages[@]}" --all-targets --features "$s6_features" --offline --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc "${s6_packages[@]}" --no-deps --features "$s6_features" --offline --locked
cargo clean
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report "${s6_packages[@]}" --features "$s6_features" --offline --locked
cargo llvm-cov --no-report -p data-quickstart --features native --offline --locked
cargo llvm-cov --no-report -p data-quickstart --features native,evolved --offline --locked
cargo llvm-cov --no-report -p data-quickstart --offline --locked
cargo llvm-cov --no-report -p generated-workspace-app --features matrix --test matrix --test matrix_contracts --offline --locked
cargo llvm-cov --no-report -p generated-workspace-app --features names --test names --test names_contracts --offline --locked
(
  eval "$(cargo llvm-cov show-env --sh)"
  python3 examples/data-quickstart/prove.py
)
cargo llvm-cov report --json --output-path /private/tmp/s6-m4/coverage-default.json -vv
python3 /private/tmp/s6-m4/export-coverage.py
python3 .agents/scripts/validate-agent-workspace.py
```

## Resources, limits and stop

The largest sampled/clean-reported target size was about 2.7 GiB, below 8 GiB. The final target was deleted after exporting all evidence; no owned build/test process remains. One target root was used, including nested trybuild and disposable documentation-consumer output. Three exploratory profiles created outside it by cached instrumented binaries were moved inside and removed; all later command environments direct profiles inside the target. The 136 older ignored profiles discovered in the worktree were preserved. Final owned-profile and source audits are retained.

No listener-capable workspace HTTP/server suite, network database, UI/browser, Tauri/mobile, deployment or publication was run or certified. SQLite is the only migration execution target. Source history requires cooperative writers and safe reverse preconditions; new-model/data recovery is not automatic. No installs, commits, pushes, GitHub writes, coordinator messaging, other-trial edits or sibling Idea access occurred. Stop for coordinator review of M4; no next milestone or publication work began.
