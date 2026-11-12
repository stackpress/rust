# Task 00057: hardening integration

Status: local implementation and requested gates passed; independent coordinator verification remains separate.
Branch `hardening/integration`, baseline `80263b2`. Authority: G62 and receipts 00053-00056.
Acceptance criteria: none.

## Implementation notes

- Default and exact compatible workspace/all-target checks passed before regeneration. No handwritten production Rust integration fix was needed.
- Added only the coordinator-approved `(MIT OR Apache-2.0) AND NCSA` snapshot expression, with its `libfuzzer-sys 0.4.10` / unpublished `fuzz/` provenance. Licence check: 707 packages, 23 locks, 39 expressions, three existing dated metadata fallbacks, zero failures.
- Added `source_admin_imports_windows_1252_semicolon_upload_into_storage`: original raw bytes enter the registered API admin import handler, shared tolerant parser and real SQL batch; a separate detail request verifies the stored ID, relation, decoded curly quotes/euro/quoted semicolon and secret.
- Official regeneration refreshed the stale composite-admin source model/catalog code and manifest from merged templates. The legacy SQL session fixture's source bytes/descriptors are unchanged; its manifest now names the canonical model producer `source-models` instead of `native`. No managed source was hand-edited.
- Twenty official generation selections and their checks cover all 27 tracked managed roots: workspace ordinary/matrix/names/contracts, admin ordinary/format, API checkpoint, session/email/API/aggregate families, composite UI/models, account/plugin/SQL-session fixtures, profile/quickstart examples and aggregate build-host/source consumers. The build-host also verifies its existing Typed checkpoint. The blog gate regenerated ignored `.build` and the copied-out nickname products. [Exact generation commands and attempts](../../../resources/hardening-integration-00057/generation.json), [remaining managed selections](../../../resources/hardening-integration-00057/generation-extra.json).

## Review rounds

1. Logic/complexity: reviewed raw-byte admission, delimiter/quote handling and persistence assertions. The actual CSV handler regression passed (1/1); no production refactor was justified.
2. Responsibility/abstraction: reviewed admin route → parser → batch → store ownership and composite keys. Eight actual admin/composite cases passed; existing boundaries remained cohesive.
3. Consistency/Rustdoc/comments: reviewed fixture producer inputs, source descriptor parity, naming and explanatory comments. Three targeted freshness/representation/CSV tests passed; final formatting, Clippy, doctests and Rustdoc passed in both workspace matrices.
4. Safety/soundness: all 23 roots retain all six denies; the 72-file frozen inventory and registry identity remain intact. Live ownership/drop receipts, actual importer storage read-back and the three fuzz smokes passed. No new `allow`, exception, dependency or public behavior was introduced.

## Combined gate

`python3 tools/release/gate.py --allow-dirty`: **27/27 phases passed** in one combined run, using the specified cached `BLOG_PLAYWRIGHT` path.
Default: **1,100 ordinary + 1 serial TCP test passed**, 56 genuinely ignored cases (the ordinary lane also filters TCP).
Compatible: **1,350 ordinary + 1 serial TCP test passed**, 60 genuinely ignored cases.
Both include live PostgreSQL/MySQL; no unchanged serial retry was needed for either ordinary gate.
The facade matrix passed **81 builds and 87 advertised paths**. Default/compatible warning-denied Clippy/Rustdoc and doctests, native/WASM graphs, all eight aggregate and four session consumers, generator checks, README controls, 23 package archives/strict payload checks and five publication dry runs passed. Nothing was uploaded.
[Combined phases and exact commands](../../../resources/hardening-integration-00057/gate-results.json), [Archive checks and dry runs](../../../resources/hardening-integration-00057/packaging-results.json), [Facade compilation and imports](../../../resources/hardening-integration-00057/facade.json).

Separate strict boundary/copy/licence checks and **44/44** release Python tests passed. All **23/23** candidates passed standalone all-target Clippy with `-D warnings`.
[Tooling results](../../../resources/hardening-integration-00057/auxiliary.json), [standalone lint results](../../../resources/hardening-integration-00057/standalone-clippy.json).

The blog passed **92/92** ordinary and instrumented native tests, both browser modes/public matrix, copied-out README/nickname generation/freshness/browser and database-preserving upgrade, native 70% floors and lock controls. `blog-web`: **2,206 / 3,121 = 70.68%**; its thin browser entry retains G56's separate browser proof exemption. LLVM emitted **92 functions with mismatched data**; the threshold-passing export is not warning-free.
[Blog phases](../../../resources/hardening-integration-00057/blog-results.json), [blog coverage](../../../resources/hardening-integration-00057/blog-coverage.json).

## Live-inclusive release coverage

Fresh `cargo llvm-cov nextest --workspace --all-targets --features <exact compatible set> --locked --offline --no-fail-fast --test-threads 2 --json --summary-only --output-path <file>` with both approved live URLs.
Tool: cargo-llvm-cov 0.8.6, rustc 1.96.0 / LLVM 22.1.2, native `aarch64-apple-darwin`. **1,351/1,351 tests passed**, 60 ignored, with no serial retry needed. The workspace export emitted no warnings. No added exclusions or custom export. Figures sum ordinary LLVM `src/` file rows for each candidate, including in-source tests; separate tests/examples/dependencies are not implementation rows. This is line coverage, not branch or WASM-runtime coverage.
[Exact command and environment](../../../resources/hardening-integration-00057/coverage-command.json), ordinary JSON report (219 KB, kept locally under the ignored `target/evidence-00057/coverage.json`, not committed), [per-file attribution and numerators](../../../resources/hardening-integration-00057/coverage-summary.json).

| Candidate | Covered / executable `src/` lines | Line coverage |
| --- | ---: | ---: |
| `stackpress-leptos` | 7,967 / 8,779 | 90.75% |
| `stackpress-framework-language` | 54 / 58 | 93.10% |
| `stackpress-sql` | 3,292 / 3,394 | 96.99% |
| `stackpress-lib` | 4,214 / 4,432 | 95.08% |
| `stackpress-idea-parser` | 2,261 / 2,391 | 94.56% |
| `stackpress-sql-sqlite` | 740 / 808 | 91.58% |
| `stackpress-sql-postgres` | 842 / 889 | 94.71% |
| `stackpress-sql-mysql` | 986 / 1,049 | 93.99% |
| `stackpress-server` | 2,350 / 2,539 | 92.56% |
| `stackpress-idea-transformer` | 2,132 / 2,282 | 93.43% |
| `stackpress-server-http` | 1,732 / 1,839 | 94.18% |
| `stackpress-cli` | 1,306 / 1,404 | 93.02% |
| `stackpress-framework-view` | 1,799 / 1,852 | 97.14% |
| `stackpress-framework-csrf` | 67 / 69 | 97.10% |
| `stackpress-framework-email` | 2,486 / 2,756 | 90.20% |
| `stackpress-idea` | 41 / 41 | 100.00% |
| `stackpress-framework-schema` | 6,311 / 6,860 | 92.00% |
| `stackpress-framework-sql` | 7,844 / 8,667 | 90.50% |
| `stackpress-framework-admin` | 4,198 / 4,644 | 90.40% |
| `stackpress-framework-session` | 3,704 / 3,968 | 93.35% |
| `stackpress-framework-api` | 2,263 / 2,496 | 90.67% |
| `stackpress-framework` | 1,176 / 1,232 | 95.45% |
| `stackpress` | 0 / 0 | G61 exempt (0/0) |

Every non-facade candidate meets the 90% floor. G61 replaces the facade's 0/0 line metric with its build/import/default-tree matrix.

## Database audit and fuzz

Only fixture-reserved unique `stackpress_*` databases were created and exactly dropped. Captured CREATE/DROP confirmations:

| Run log | Created | Confirmed DROP |
| --- | ---: | ---: |
| `nextest-compatible.log` | 46 | 46 |
| `nextest-default.log` | 34 | 34 |
| `coverage.log` | 46 | 46 |

All captured created names pair exactly. Initial and final PostgreSQL/MySQL `stackpress_*` listings are both empty. No prefix sweep or database listener was started.
[Exact owned names and pairing](../../../resources/hardening-integration-00057/db-pairs.json), [final catalog listing](../../../resources/hardening-integration-00057/db-after.json).

Merged nightly `cargo fuzz build` and 60-second runs passed without crashes:

| Target | Executions |
| --- | ---: |
| `idea_parser` | 1,743,693 |
| `admin_csv` | 370,210 |
| `lib_query_form` | 721,240 |

Nightly 1.102.0 / cargo-fuzz 0.13.2 retain the inherited `AtomicU64::fetch_update` deprecation warning. Mutations stayed in ignored discovered-corpus paths. Finite smokes do not establish exhaustive or other-platform correctness. [Fuzz invocation results](../../../resources/hardening-integration-00057/fuzz.json), [execution counts](../../../resources/hardening-integration-00057/fuzz-summary.json).

## Invariants and recorder

Root lock bytes and all **699 registry tuples** are unchanged. All **72** tracked frozen lock/golden/oracle/`.stderr` files are byte-identical; there is no such diff to show. No manifests/dependencies, new no-panic exceptions, installs, development listeners, commits or pushes. [Invariant audit](../../../resources/hardening-integration-00057/invariants.json).

All Cargo/proof/clean commands use the required shared recorder, sccache, jobs=2 (fuzz build jobs=1), incremental=0 and dev debug=0. Separate owned targets prevent consumer/facade/coverage collisions. Native sampled peak: **12.027 GiB**, briefly about 28 MiB above the workflow's default 12 GiB budget at the native→graph transition; completed proof/core/consumer targets were retired, with the real peak retained. The sampler's cleanup-scan race was recovered by the independent monitor. No test or deadline changed.

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`.
[Live task ledger snapshot and counters](../../../resources/hardening-integration-00057/benchmark-summary.json), [Recorded attempts and timings](../../../resources/hardening-integration-00057/benchmark-records.json). Failed invocation/audit attempts remain recorded. `command_seconds` is end-to-end; `cargo_build_seconds` is only observed Cargo stages. Concurrent monitoring/proofs overlap, and captured child output can hide stages; sums are neither calendar elapsed time nor complete compilation time.

KB validation retains the **146 inherited errors and 44 warnings**; final comparison is recorded with the evidence. This is not a clean KB validation claim.

## Verification and acceptance notes

Local results above are implementation evidence. Coordinator independent verification remains pending; acceptance criteria are none. No publication, deployment, commit or push was performed.


## Coordinator note

The raw coverage report and log (`coverage.json`, `coverage.log`, about 760 KB) are kept locally under the ignored `target/evidence-00057/`, not in git; `coverage-summary.json` holds the per-crate numbers.

## Coordinator verification

The four hardening branches (00053–00056) were merged into `hardening/integration`. A coordinator integration commit swapped in `Language::default()`/`PublicLocation::session_asset()` and added the template exceptions to `.agents/context/no-panic-exceptions.md`. Then this task. Main was fast-forwarded to `d9a8561`.

Independent sanity check on a detached clean checkout of `d9a8561`, 2026-11-12:
- 699 registry tuples identical;
- strict test-boundary, test-copy and dependency-licence checks; release tooling unit tests;
- fmt; compatible Clippy `-D warnings`;
- compatible nextest: 1,304 passed; `tcp_watchdog` alone passed;
- aggregate consumers;
- `package_workspace.py`: all 23 archives verified, strict `check_packages.py --archives`, five leaf `cargo publish --dry-run` runs with the upload aborted.

The full gate and live coverage are the integration worker's run above.
