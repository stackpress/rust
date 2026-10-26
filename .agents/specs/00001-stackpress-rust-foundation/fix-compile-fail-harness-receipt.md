# Shared-crate compiler-rejection harness fix receipt

Date: 2026-10-26. Status: implemented; coordinator verification remains separate.
Branch: `trial/3-fix-compile-fail-harness`; base: `8b74328c704fea3666492b95750c17fd54b37be3`.
Authority: the coordinator's scoped brief under G23 in [decisions](decisions.md).

## Scope and behavior

- Changed only `crates/lib/tests/handler_signature.rs`, `crates/server/tests/ownership.rs`, their ten new `tests/ui/*.stderr` snapshots, the two crate dev-dependency lists, and Cargo's two lockfile dependency entries, plus this requested receipt.
- No production source or fixture Rust source changed. No commit, push, deployment, publishing, or agent messages.
- Removed both mtime-based rlib selectors and all manual `rustc --extern` assembly. `trybuild = "1"` reuses locked version 1.0.122, already present through SQLite; no dependency version was updated and no `cargo update` was run.
- Trybuild asks Cargo to compile fixtures against one consistent dependency graph for the current crate/features. It uses the current checkout's manifest/path dependencies and Cargo-managed artifacts rather than selecting old rlibs beside the executable. This is a rebuild of the fixture dependency graph, not a claim that trybuild physically reuses the parent binary's identical rlib files.
- Exact normalized diagnostic snapshots replace partial substring checks. An incidental missing `service` method, E0460/E0464, a syntax failure, or an unexpectedly successful fixture cannot satisfy the proof.
- The lib's two compiler tests are consolidated into one suite while retaining all six fixtures. A separate ignored child runs each trybuild suite; its ordinary parent invokes it through the existing external process watchdog. Each whole compiler suite has a 180-second wall-clock bound, matching the existing SQLite trybuild proof and allowing cold Cargo dependency checking (the first lib snapshot build took 27.04 seconds). The existing child kill/reap and bounded file-backed output behavior is preserved.
- The positive lib runtime proof retains its two-second executor timeout and ten-second external deadline. Its deliberately stuck-child watchdog control also remains.

## Reproduction before editing

Commands ran in the same worktree and the same root `target/` tree, without cleaning between feature combinations. Initial `cargo clean` and `cargo llvm-cov clean --workspace` found no pre-existing build artifacts.

All builds used these disk-saving environment settings:

```sh
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=2
```

| Step, in execution order | Before fix | After fix |
| --- | --- | --- |
| `cargo test -p stackpress-server --locked` | Pass, 25.86 s | Pass, 8.85 s |
| `cargo test -p stackpress-server --features http --locked` | Pass, 15.59 s | Pass, 11.44 s |
| `cargo test -p stackpress --features sql-sqlite,framework-ui-ssr --locked` | Pass, 78.37 s | Pass, 1.74 s |
| `cargo test -p stackpress-lib --test handler_signature --locked` | Pass, 1.83 s | Pass, 0.72 s |
| `cargo test -p stackpress-server --test ownership --locked` | **Fail (101), 0.34 s** | **Pass, 0.61 s** |

After-fix commands additionally used `--offline`; their package/feature selections were identical. Snapshot generation used `TRYBUILD=overwrite` once per crate. All subsequent regression, round, and final verification commands explicitly removed that environment variable and compared the saved snapshots.

The original server replay failed at the existing semantic-diagnostic assertion in `ownership.rs:87`, with:

```text
escaping_host.rs: error[E0599]: no method named `service` found for struct `Invocation<'a>` in the current scope
method not found in `Invocation<'_>`
test result: FAILED. 0 passed; 1 failed
```

That fixture must fail for its host-borrow lifetime, not for an unavailable extension method. The original lib harness passed in this particular reproduction; no lib failure is claimed. The requested before/after sequence was completed without deleting the mixed build artifacts between runs.

Raw logs and exact command/exit/timing records: `/private/tmp/compile-fail-harness-evidence/`, especially `results.tsv`, `before-server-replay.log`, and the five `after-*.log` files.

## Diagnostic intent audit

All six lib and four server fixtures have reviewed snapshots; their Rust inputs remain unchanged.

| Crate / fixture | Required diagnostic preserved |
| --- | --- |
| lib / `non_send_handler.rs` | Returned future retains `Rc<()>` across await; `EmitterBuilder::on` requires Send |
| lib / `stateful_clone.rs` | E0599 for both Request and Response Clone attempts |
| lib / `concurrent_mutable_inputs.rs` | E0499 for both request and response mutable aliases |
| lib / `escaping_invocation.rs` | Invocation lifetime cannot become static |
| lib / `escaping_context.rs` | Static lifetime violation and E0515 for borrowed invocation context |
| lib / `non_sync_context.rs` | E0277 for both Send and Sync restrictions on `Rc<()>` |
| server / `non_send_service.rs` | E0277 for both Send and Sync service-registration restrictions |
| server / `escaping_host.rs` | Static lifetime violation and E0515 for the borrowed host service |
| server / `stateful_clone.rs` | E0599 for Server, ServerBuilder, and OutputGuard Clone attempts |
| server / `non_send_startup.rs` | Returned future retains `Rc<()>` across await; `on_config` requires Send |

## Test-verified review rounds

Followed the [Rust programming workflow](../../workflows/rust-programming.md), [quality policy](../../context/testing-and-quality.md), and the installed coding principles through their explicit Rust adaptation.

1. **Logic/complexity.** Reviewed success, diagnostic mismatch, unexpected successful compilation, timeout, and child failure paths; checked every snapshot against the old intended semantic assertions. The initial replacement removes the artifact-selection branches; no additional refactor was justified. Both targeted integration suites passed with snapshot rewriting disabled (`round1-lib.log`, `round1-server.log`).
2. **Responsibility/abstraction.** Cargo owns dependency resolution, trybuild owns normalized diagnostic comparison, and the existing watchdog owns external deadlines. Short crate-local launchers keep manifest/fixture ownership explicit without introducing an out-of-scope shared helper. No further extraction was justified. Lib default and server HTTP targeted suites passed (`round2-lib.log`, `round2-server-http.log`).
3. **Consistency/Rustdoc/comments.** Reviewed matching parent/child names, crate working directories, explanatory comments, current diagnostic intent, and preserved positive runtime evidence. Both replacements use the same pattern as the existing SQLite proof. No further edits were justified. Formatting, workspace tests/doctests, Clippy, and warning-clean Rustdoc results are recorded below.
4. **Safety/soundness.** No unsafe code or runtime ownership changes. Reviewed the existing watchdog's ChildGuard cleanup, timeout reporting, and bounded diagnostics. The positive runtime proof and watchdog negative control execute in the workspace/coverage suites. The watchdog retains its existing direct-child cleanup contract; this patch does not introduce process-group termination.

## Final verification

Toolchain: `rustc 1.96.0 (ac68faa20 2026-05-25)`, host `aarch64-apple-darwin`.

| Check | Command | Result | Seconds |
| --- | --- | --- | --- |
| Formatting | `cargo fmt --all -- --check` | Pass | 0.85 |
| Workspace tests | `cargo test --workspace --all-targets --locked --offline` | 327 passed, 0 failed, 30 ignored | 155.45 |
| Workspace doctests | `cargo test --workspace --doc --locked --offline` | 438 passed, 0 failed | 23.33 |
| Clippy | `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pass | 36.91 |
| Rustdoc | `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --offline` | Pass | 7.88 |
| Rust 1.96 check | `cargo +1.96.0 check --workspace --all-targets --locked --offline` | Pass | 8.52 |

Ignored entries include child cases that ordinary parent tests execute under watchdogs; this does not claim all ignored cases were invoked. Remote database tests remain opt-in, and no remote database validation is claimed.

KB validator baseline: `python3 .agents/scripts/validate-agent-workspace.py` exits 1 with 146 pre-existing missing-file link errors and 20 warnings. The unresolved paths point at upstream sibling checkouts unavailable beside this isolated worktree. They are outside this fix; no green whole-KB validation is claimed. The new receipt stays below 200 lines and its links resolve locally.

## Coverage

Tool: `cargo-llvm-cov 0.8.6`, executable line coverage, native aarch64 macOS, Rust 1.96.0. Same disk-saving profile settings as above; coverage output used the same root `target/` via `CARGO_LLVM_COV_TARGET_DIR="$PWD/target"`.

```sh
cargo llvm-cov --no-rustc-wrapper -p stackpress-lib -p stackpress-server \
  --features stackpress-server/http --tests --locked --offline --json \
  --output-path /private/tmp/compile-fail-harness-evidence/coverage.json \
  --fail-under-lines 70
```

| Production scope | Covered / executable lines | Coverage | Development floor |
| --- | --- | --- | --- |
| stackpress-lib (all 16 reported source files) | 3177 / 3393 | 93.63% | Pass, >=70% |
| stackpress-server (all 13 reported source files) | 1862 / 1976 | 94.23% | Pass, >=70% |

Only first-party production source is reported; the report contains 16 lib source files and 13 server source files, with no custom source exclusions. Dependencies, test harnesses, compile-fail fixtures, doctests, and non-selected platform/features are outside this measurement. No production source changed in this task.

The initial `--no-clean`/default-wrapper attempt returned exit 1 and an invalid zero-count report because ordinary cached library artifacts were reused without instrumentation. That attempt is retained as `coverage-attempt1-invalid.{log,json}` and is not coverage evidence. After preserving watchdog logs, `cargo clean` removed the mixed artifacts. The command above, with explicit rustc instrumentation and no wrapper, rebuilt and passed in 84.63 seconds; both compiler suites and runtime watchdog controls passed under instrumentation. The clean report is `coverage.json`, with per-crate totals in `coverage-summary.json`.


## Limits and cleanup

- This is native aarch64 macOS test-harness evidence, not platform or release acceptance. No WASM, installed Tauri, remote database, or publication support is claimed.
- The exercised feature matrix is server default, server HTTP, the SQL/SSR facade combination above, and compatible workspace defaults; mutually exclusive renderer features were not indiscriminately combined.
- Compiler diagnostics are snapshots for Rust 1.96.0 and may require deliberate review on future compiler upgrades.
- All explicitly managed build output used this worktree's `target/` root; trybuild's standard nested cache remains inside that root. Existing workspace consumer tests may create and automatically drop private temporary targets.
- All local build output was removed with final `cargo clean` after preserving the logs and coverage report: `Removed 1938 files, 617.4MiB total`. No unrelated target directories or other worktrees were deleted.
- Coordinator re-verification and integration remain outstanding. No product/public API decision is open.
