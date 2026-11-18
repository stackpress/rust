# Testing and quality policy

Authority: owner coverage policy, G56/G61/G62 and the implemented verification tools. Normal development requires at least 70% coverage; publication preparation requires at least 90%. This supersedes legacy TypeScript contributor references requiring >90% on every new change.

## Meaning of the thresholds

Use executable line coverage for the initial operational definition, measured on first-party Rust implementation in the declared crate/feature scope. Record tool, compiler/target, selected features, numerator/denominator or report, and exclusions. The owner specified percentages; this line-coverage convention makes them usable without inventing a branch-score requirement.

- Development closeout: at least 70% in each changed crate's measured scope, plus the relevant workspace/integration result. Do not hide an under-tested changed crate behind unrelated well-covered code.
- Publication preparation: at least 90% for every non-facade release crate and the applicable release integration scope. Publication itself requires explicit authorization.
- Preserve stronger existing regression coverage where practical; a lower allowed floor is not permission to delete valuable tests.
- An empty or unmeasured implementation is not 100% covered. Source/doc/KB presence is not coverage evidence. Apply only the named exemptions below.
- Dependencies, target build output, and tooling fixtures are not first-party production coverage. Generated code that is shipped/executed still needs a stated verification strategy; do not exclude it merely to raise the percentage.
- Do not add broad ignore patterns, unreachable branches, trivial assertions, or tautological mocks to game the gate. Document justified exclusions, especially target-specific code.

Line coverage does not prove branch, permission, lifecycle, concurrency, or error behavior. Even at 70%, tests must protect the meaningful critical paths affected by the change. At 90%, the remaining gaps must be visible and assessed.

## Named coverage exemptions

- **Facade (G61):** `stackpress` has no executable implementation lines. Replace line coverage with [check_facade.py](../../tools/release/check_facade.py): default and every declared feature individually, the full compatible native SSR set, independent consumer imports for every advertised path and the default normal dependency tree containing only `stackpress-lib` among Stackpress dependencies. This is the implemented compatible matrix, not a claim that every power-set combination builds; renderer modes are separate.
- **Thin browser entry crates (G56):** startup/mount-only crates may be exempt when real browser tests exercise them and the receipt names the exemption. [Blog client](../../examples/blog/client/Cargo.toml) is the named example; its decisions/logic are extracted to natively measured crates. WASM profiler_builtins limitations do not excuse business logic or other native crates. `blog-xtask` remains measured against the 70% development floor.

## Meaningful tests

- Unit tests prove clear contracts, boundaries, invalid input, false/empty/unknown states, mutation, and error propagation.
- Event tests prove matching, priority/ties, abort, nested calls, independent concurrent invocation metadata, and lifecycle behavior.
- Plugin tests prove selected/disabled builds, simple function registration, composition, customization, and declared dependency behavior.
- Generator tests prove parsing/normalization, clean and repeat generation, removed/renamed output policy, generated compilation/import, and runtime registration.
- Database tests prove actual adapter/dialect behavior and transaction affinity; SQL string snapshots alone do not prove execution.
- Access tests cover allowed and denied callers across each exposed surface; UI hiding is not authorization.
- Leptos/Tauri tests distinguish SSR/hydration/CSR, packaged rendering, operation adapters, target-specific integrations and public snapshots. Desktop deferral and MCP limitations in G52 remain explicit; browser execution is not installed-app acceptance.
- Tests assert observable behavior rather than reproduce implementation decisions. Use representative decision tables where combinations matter.

## Required implementation/refactoring sequence

Write the intended behavior naturally and obtain passing meaningful tests. Then run at least three distinct rounds:

1. Logic/complexity and branch-risk review; refactor while preserving behavior and rerun relevant tests.
2. Responsibility boundaries, abstraction, ownership, coupling, and duplication review; refactor and rerun relevant tests.
3. Consistency, idiomatic Rust, public API, Rustdoc, and detailed inline-comment review; refactor/document and rerun relevant tests/doctests.

Repeat targeted rounds while material findings remain. A round with no justified edits is valid if it records its review scope and conclusion. Do not perform pointless rewrites or an endless loop. After the last behavioral edit, run the meaningful affected checks again; broaden only for actual cross-crate/target concerns.

Use [the programming workflow](../workflows/rust-programming.md) for exact execution and [round checklists](../references/00037-refactoring-round-checklists.md) when reviewing. The standing user request authorizes routine behavior-preserving rounds inside the task, including Rust adaptation of installed chrisai-coding guidance.

## Build and test tooling

Follow [fast build and test](../workflows/fast-build-and-test.md) and [build benchmarking](../workflows/build-benchmarks.md): one warm target per effort, sccache, bounded jobs, narrow affected tests in edit loops and full checks at milestone gates. Record Cargo/proof commands with `.agents/scripts/build-benchmark.py`, preserving scope, cache state, compiler/target/features and failures. Doctests remain a separate `cargo test --doc` lane because nextest does not run them.

The implemented [release gate](../../tools/release/gate.py) runs default and compatible-feature formatting, strict Clippy, nextest (including a separate single-threaded tcp_watchdog selection), doctests, strict Rustdoc, consumers/graphs, official generation freshness, blog proof, checker controls, README/package/dry-run and facade checks. It preserves phase logs and appends retries selected by `--only PHASE`. See [release tooling documentation](../../tools/release/PACKAGING.md) and [release versioning](../workflows/release-versioning.md). Do not use `--all-features` for incompatible SSR/hydrate/CSR renderers. No CI is adopted by G62.

Line coverage uses actual LLVM source instrumentation. `cargo llvm-cov` is used in repository coverage work; the [locked workspace driver](../../tools/release/coverage.py) and [blog coverage driver](../../tools/examples/blog/coverage.py) also measure native crates directly with the installed LLVM tools. Workspace release comparisons sum ordinary per-crate `src/` rows, including in-source tests, as in the 00057 receipt; report that scope explicitly. Record instrumented scope, exclusions and report; do not silently require nightly for the native line gate. Doctests run separately. The release gate is not a substitute for standalone per-crate Clippy, live-inclusive 90% coverage or finite fuzzing required by the release workflow.

## Live database tests

Live PostgreSQL/MySQL suites are opt-in using `STACKPRESS_TEST_POSTGRES_URL` / `STACKPRESS_TEST_MYSQL_URL`. Unset cases stay ignored/skipped; do not report them as passed. The workspace coverage driver also invokes the existing MySQL M2 `--g37-ddl` opt-in group when its live URL is set; ordinary nextest discovery does not include that generated-DDL proof. Ordinary tests remain self-contained with temporary SQLite/native fixtures. The [adapter tests](../../crates/sql/sql-postgres/src/tests) and [crate-local framework harness](../../crates/framework/framework-session/tests/fixtures/live_support.rs) document the allowed passwordless local endpoints and finite budgets.

Each live run creates a unique throwaway database, arms cleanup only after CREATE acknowledgement and drops only that owned database, including error paths. Never read, alter or drop other databases, roles or global settings. Report created/dropped names and outstanding cleanup failures without exposing connection secrets. Do not infer network-database execution from renderer snapshots.

## Browser and fuzz proofs

Use cached Playwright/Chromium and separate hydrate/CSR products. [Blog browser driver](../../tools/examples/blog/browser.py) invokes [the browser proof](../../examples/blog/tests/browser.cjs), routing requests in process through a finite stdin/stdout native probe; it starts no HTTP listener and installs nothing. Similar owning-example proofs retain DOM/hydration/input/focus and relevant access/error checks. Any interactive development preview/listener still uses the devmetrics skill; do not start it directly.

[Realtime proofs](../../crates/server/http/tests/realtime.rs) use finite test-owned ephemeral HTTP/HTTPS listeners with watchdog kill/reap, duplex backpressure and simulated time for age/heartbeat/Pong/grace boundaries. [Browser proof](../../crates/server/http/tests/realtime_browser.rs) mounts actual Leptos helpers separately in hydrate/CSR, routes assets in-process and owns its ephemeral protocol listener inside the test. Browser DOM glue is covered by these proofs, not native LLVM instrumentation; no G56 exemption hides portable helper decisions. [Feature graphs](../../tools/realtime/graphs.py) pin disabled transport and browser normal-edge isolation. Test listeners are finite proof resources, not development previews.

[Parser fuzzing](../../fuzz/README.md) uses the separate unpublished `fuzz/` workspace and nightly cargo-fuzz for `idea_parser`, `admin_csv`, `lib_query_form` and the bounded `sse_encoder`. [Release fuzz rules](../../tools/release/FUZZING.md) require finite runs of at least 300 seconds each, recorded tool versions/executions/crashes, unchanged root lock bytes/registry tuples and only the approved fuzz-lock extras. Minimize a crash, add a regression in its owning crate, then fix and rerun. Finite no-crash evidence does not prove correctness exhaustively.

## Self-contained packaged tests

[Release test boundaries](../../tools/release/README.md) require published packages' test inputs to live inside the package and internal dev-dependencies to use only earlier-published release crates; unpublished/later packages cannot supply packaged test code. Repository consumer workspaces remain independent proofs. `python3 tools/release/check_test_boundaries.py` scans source/test/example/bench and fixed support-script inputs conservatively; extracted package compilation supplies additional evidence.

`python3 tools/release/check_test_copies.py` checks all `test-copies*.json` manifests: source hashes always remain pinned, byte copies may have only the recorded provenance line, and intentional divergences require a reason plus a separate copy hash. Refresh source/copies/manifests together after review. Use official regeneration examples for session/API/aggregate test families and their `--check` freshness tests; do not hand-edit generated fixtures.

## Verification receipt

Record task/scope, implementation tests, each round's findings and justified edits/no-change result, tests rerun after edits, formatting/lint/docs, coverage scope/result, outstanding risk/target limitations, and any deferred behavior decision.

Passing a local proof is not full platform support, package acceptance, or release readiness. Spec acceptance is distinct from implementation, and publishing/commit/push/deployment remain separately authorized operations.

[Historical style and quality context](../references/00045-preimplementation-style-quality.md) retains the old command templates and proposed-tooling statements; load only for provenance. Workspace documentation validation does not verify code behavior or coverage.
