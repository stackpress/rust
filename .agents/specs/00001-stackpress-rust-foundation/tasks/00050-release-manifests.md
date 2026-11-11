# Task 00050 — release manifests and packaging

Status: implementation complete; 26/26 gate phases green in coordinator-authorized pending-test mode. Integrated strict test/coverage verification remains with the coordinator.
Acceptance criteria: none. Publication, commit and push are not authorized.

## Implementation

The 23 G59/G61 candidates explicitly publish at lockstep `0.1.0`; the workspace default
and 29 other root packages remain unpublished. All 86 internal normal/optional/target
declarations and 31 earlier-order candidate dev declarations have registry versions.
Eight later-order dev edges and two private example edges stay path-only; normalized
archives prove they are stripped. Feature/default-feature options are unchanged.

Owner correction relayed on 2026-11-11: facade plus ten framework crates use the root
Stackpress License through `license-file = "LICENSE"` (eleven total). Twelve other
crates use Apache-2.0 with byte-identical parser licence copies. All sixteen original
licence/notice hashes are unchanged. No facade `LICENSE-APACHE` exists. G60 is updated
by the coordinator on main. Repository/keywords/Apache metadata is shared; README paths
stay crate-local because workspace README inheritance resolves from the root.

Twelve missing READMEs now pass twelve actual doctests. The root README has a crate and
licence table. All eight category slugs have [cached registry provenance](../../../resources/release-manifests-00050/category-sources.json); no unconfirmed slug is used.
Package allowlists retain embedded sources, assets/templates, schemas and notices.
[Tooling instructions](../../../../tools/release/PACKAGING.md) provide runnable proofs.
No crate source, test, fixture or existing lock was edited by this lane.

## First-release dev edges and test boundaries

The coordinator corrected the original instruction to version every candidate dev edge.
These eight later-order edges now stay path-only:

| From | Stripped dev-dependencies |
| --- | --- |
| sql-sqlite | server |
| sql-postgres | framework-schema, framework-sql |
| sql-mysql | framework-schema, framework-sql |
| framework-email | framework-sql, framework-schema, framework-session |

The checker rejects versioned later-order dev edges and actual Rust references to any
stripped dev dependency in packaged code. Comments/generated-source strings are opaque.
The provisional allowlists omit email's integration/generation roots except its two
embedded TLS PEM assets, and SQLite's trybuild tree and host example. The [exact 51 excluded paths and eight edges](../../../resources/release-manifests-00050/dev-cycle-policy.json) are listed for merge reconciliation.

The coordinator assigned remaining source-test moves to the tests/coverage workers.
`--pending-tests` lists only the [original 52 file-reference omissions](../../../resources/release-manifests-00050/pending-test-references.json) plus [17 stripped-dependency references](../../../resources/release-manifests-00050/pending-stripped-test-references.json): SQLite `src/tests/host.rs:7`, and PostgreSQL `src/tests/support/diagnostics.rs` plus `stores/{fixture,mod,preflight,reads,writes}.rs`.
The latter include the already-private generated-workspace-app dependency. MySQL's
transitive helper sharing is covered by its original path-reference pending list.
All 69 exact entries are reported; every other check remains strict. The integrated
checker must pass without this flag after merging the other lanes.

`--exclude-lockfile` was investigated and rejected: it can mask local packaging cycles
while leaving first-release registry dependency validation unresolved. Final package
proofs use normal archive lockfiles. Initial offline leaf dry-runs needed HTTP; the
coordinator authorized anonymous registry reads, and all five final dry-runs passed.
No token, login or upload was used; Cargo explicitly aborted each upload for dry-run.

## Verification and review rounds

The [gate ledger](../../../resources/release-manifests-00050/gate-results.json) and retained phase logs record all 26 effective passes:
fmt; default/compatible Clippy and Rustdoc with warnings denied; both nextest sets,
doctests; aggregate/session graphs and all consumers/build-host checks; generated
workspace/admin/API and three model checkpoints; blog gate; checker controls; READMEs;
packages/dry-runs; facade. Default nextest passed 1,061 tests plus the serial watchdog;
compatible passed 1,226 plus the watchdog. Respectively 88 and 96 ignored tests remain.

Blog passed all 27 phases, including both browser renderers, nickname migration and
coverage floors. LLVM export retained a warning about 92 mismatched functions; this
is a pass with that warning, not a warning-free coverage export. Release coverage of
the five assigned crates is the other worker's deliverable, not a claim of this lane.

G61 is proved by **81 builds**: default, 79 individual declared features, and compatible
full SSR; explicit imports cover all **87** advertised paths. The default normal tree
contains only lib among Stackpress dependencies. This is not a `2^N` powerset. Hydrate,
CSR and SSR selections are separate; no renderer `--all-features` was used.
[Facade evidence](../../../resources/release-manifests-00050/facade.json) records the matrix and tree.

Both `cargo package --workspace` passes (with all 29 non-candidates excluded), first
`--no-verify` then verified, passed with normal lockfiles. All five leaf dry-runs passed.
[Command results](../../../resources/release-manifests-00050/packaging-results.json), [complete package checks](../../../resources/release-manifests-00050/packages.json) and [measured archive sizes and file counts](../../../resources/release-manifests-00050/package-sizes.md) record the actual archives: largest payload 1,938,302 bytes, largest compressed archive 279,202 bytes, each below 10 MB.

Three review rounds covered logic/complexity (token-aware reference checks and seven
negative controls), responsibility (separate inventory/scanner/archive/consumer tools),
and consistency/documentation (metadata patterns, guidance and repeated doctests).
The later dev-order correction added four policy controls; all eleven controls pass.
Earlier failed probes and the unused-import consumer fix remain in the recorder.
[Final invariants](../../../resources/release-manifests-00050/manifest-invariants.json) prove 699 exact root registry tuples, byte-identical root lock, unchanged existing nested locks/source/fixtures, and preserved notices.

## Recorder and remaining integration

Recorder store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
[Task-only record metadata](../../../resources/release-manifests-00050/benchmark-records.json) preserves commands and outcomes. Environment: sccache, jobs=2, incremental=0, dev debug=0,
own `target/00050-release` plus a separate blog target. Completed consumer targets were
retired near the cap. Blog's sampled peak was 11,419,160,576 bytes, below 12 GiB.

| Record | Outcome | Command seconds / observed Cargo stages |
| --- | --- | ---: |
| `20261111T022914.830511Z-04c5c5972dc5` | First 13 gate phases passed; intentionally stopped at the consumer boundary to add approved Playwright environment | 3788.826 / 2744.990 |
| `20261111T033251.700536Z-f49797e52dad` | Remaining phases passed except original dev-cycle/offline packaging probes | 3302.866 / 3022.690 |
| `20261111T042821.772496Z-f87a55109173` | Corrected package/controls continuation; effective gate 26/26 | 50.507 / 200.650 |
| `20261111T042849.045880Z-7d456e6330b0` | Final manifest, lock, notice and source ownership invariants passed | Recorded in task metadata |

Command time and observed Cargo stages differ; nested/parallel/repeated Cargo output
makes stage sums incomplete build evidence, not CPU time or a comparable wall clock.
No completed full-gate phase was rerun without an affected change. Coordinator review,
strict integrated package tests and the separate 90% publication coverage proof remain.
The owner preparation/publication runbook is in [Task 00047](00047-publication-plan.md).

Agent Workspace validation retains 146 inherited errors and 44 warnings (including the
existing preferred-length warning for the coherent 451-line plan). No new hard error
or broken link was introduced. `git diff --check` passed. These inherited KB diagnostics
are separate from the green release gate.

## Coordinator note

The raw gate logs (1.5 MB) were kept out of git. They remain locally under the ignored `target/evidence-00050/gate-logs/`; the committed `gate-results.json` records each phase's result.

## Coordinator verification (combined release, 00050–00052)

Merged into main: `release/self-contained-tests` (`758ab67`), `release/coverage` (`e4087f9`), `release/manifests` (`25129c4`).
- Four manifest conflicts were resolved by publish order: API and session test dev-dependencies are versioned, with framework-sql `source`; email keeps only tokio; sql-sqlite's server dev edge is removed.
- Follow-up `235fd54`: sql-sqlite ships `tests/**`, because its in-crate maintenance tests use `tests/support/watchdog.rs`; `test-copies-coverage.json` uses the `{"copies": [...]}` shape.

Independent runs on a detached clean checkout, 2026-11-11:
- **At `25129c4`:**
  - 699 registry tuples identical;
  - strict `check_test_boundaries.py`; release checker unit tests;
  - every candidate `cargo clippy -p <crate> --all-targets -D warnings` alone (23/23);
  - `tools/release/gate.py`: workspace default and compatible fmt/Clippy/nextest/doctests/Rustdoc, aggregate and session consumers/graphs, generator freshness, the blog gate, README doctests and facade checks, all passed;
  - packaging failed only on the sql-sqlite `tests/support` omission, and the copy check failed only on the registry shape. Both fixed in `235fd54`.
- **At `235fd54`:**
  - strict boundaries; copies, 46 in 2 manifests;
  - sql-sqlite Clippy alone;
  - `package_workspace.py`: strict `check_packages.py --archives` with no pending allowance, `cargo package --workspace` verified for all 23, and five leaf `cargo publish --dry-run` runs (read-only index access, uploads aborted);
  - live-inclusive `cargo llvm-cov nextest --workspace --all-targets` with the compatible set and both live URLs: 1,313 passed, 60 skipped. Every non-facade candidate is ≥90%: lowest framework-admin 90.03%, framework-email 90.12%, framework-sql 90.43%, leptos 90.81%; PostgreSQL 94.71%, MySQL 93.99%, sqlite 91.58%. The facade is exempt under G61;
  - no `stackpress_*` databases left on either server.

The release is ready for the owner's runbook: `cargo login`, then `cargo publish -p` in the documented order.
