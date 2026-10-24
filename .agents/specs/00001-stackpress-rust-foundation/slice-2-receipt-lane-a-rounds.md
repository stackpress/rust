# Slice 2 lane A: server core refactor rounds receipt

Implementer evidence, 2026-10-24. Load this with task [00005](tasks/00005-slice-2-server-and-cli.md)
when reviewing the released server-core rounds. This is a stop for coordinator re-verification, not
acceptance or publication.

## Scope and baseline

- Confirmed clean `trial/3` at `3ab0a80` before edits. The coordinator merged lane C/D work during
  this effort; final checks ran at integrated HEAD `5e3deba` plus this uncommitted patch.
- Owned server source/tests except `http.rs` and `http_*` tests. Added private `registration`,
  `server` and `startup` modules. No HTTP, CLI, example, lib, manifest, lockfile or root facade
  edits; no dependency changes.
- Public root exports and all externally public function/trait signatures remain unchanged.
  Comparing the initial signature inventory finds only the former private `StartupContext::failure`
  moved to crate visibility for sibling modules; it is not external API. Ownership rejection
  fixtures and real CLI/example consumers compile in the workspace matrix.
- Started with `cargo clean && cargo llvm-cov clean --workspace`: 33,968 files / 3.1 GiB removed.
  Native rustc 1.96.0 (`ac68faa20`, LLVM 22.1.2), `aarch64-apple-darwin`; cargo-llvm-cov 0.8.6.
- Logs/JSON remain in ignored `target/lane-a-rounds/`. No commit/push, listener,
  other-trial/worktree access, unsafe block, new dependency or context promotion.

## Round 1: logic and complexity

- Split lifecycle Request gating from selected-operation decisions. `run_stages` reads Request →
  stop/error decision → operation eligibility → one Response pass. `operation_stage` retains raw
  outcome/Abort/error precedence; `prepare_operation_output` names absent body/status and
  success-default rules.
- Split `ServerBuilder::build` into selected-plugin preflight/registration, typed config decode,
  queued startup, and immutable publication. Plugin preflight now names dependency presence/edges,
  cycle elimination and supplied-order validation, preserving missing → cycle → order precedence and
  never sorting callbacks.
- Split configuration layering/provenance from consuming section decoding. All unknown roots and
  aggregate limits still fail before validators or resource startup.
- Current code-line counts, excluding blank/comment lines: `run_stages` 35, `operation_stage` 29,
  `build` 14, `register_plugins` 15, `run_phases` 39, `decode` 24, `merge_layers` 37, `preflight`
  13, `dependency_edges` 29, `reject_cycles` 15.
- Cohesion wins for `offer` (53 code lines): setup, original-failure index, single Error dispatch
  and exhaustive terminal/continuation decision stay visible together. Splitting this match would
  conceal G12's nonrecursive dual-failure boundary. `Sections::register` (48) likewise keeps
  duplicate/type/default validation and the consuming decoder's ownership in one atomic contribution
  boundary.
- `cargo test -p stackpress-server --all-targets --features http --locked`: baseline **53 passed**,
  round 1 **53 passed**, zero failures; two child entrypoints run by watchdog parents
  (`baseline.log`, `round1.log`).

## Round 2: responsibility and abstraction

- `builder.rs` shrank from 629 to 149 code lines. It owns composition, plugin selection,
  configuration inputs and publication. `startup.rs` owns phases/tasks, build diagnostics and typed
  startup context. `server.rs` owns immutable runtime state/raw entry points. `registration.rs` owns
  listener/route mutation and thin builder forwarding.
- The shared registrar is `StartupContext` itself: phase gates, contributor sites, handles,
  replacement/capture wrappers and method helpers have one implementation. Builder forwarding
  delegates synchronously with existing `track_caller` behavior. No new trait, macro or generic
  registrar layer.
- Construction state is crate-visible only where sibling modules need exclusive ownership; published
  host/service/config values stay immutable. Server initialization has one checked `from_parts`
  boundary. The legacy outside-Config error for `StartupContext::register` is preserved rather than
  changing its public diagnostic by unifying a dissimilar gate.
- `cargo test --workspace --all-targets --features server-http --locked`: **130 passed**, zero
  failures, 13 child markers (`round2.log`).

## Round 3: consistency, Rustdoc and comments

- Added **65 compiling/running `# Examples` blocks** across the owned core:
  builder/plugin/configuration/registration/method helpers, startup service/config access,
  Invocation shortcuts/delegation, raw/handled Server entry points, RouteSpec and report accessors.
  All requested starting APIs have examples. Async handler examples use named `async fn`; hidden
  current-thread test mains execute the async examples using the existing dev dependency.
- Current server doctests total **70**: 65 new core examples, one existing crate example and four
  merged lane C examples. No ignored example or dependency was introduced.
- Added local explanations for G12 branches, state/phase changes, offers, terminal fallback, budget
  checks, cancellation, Arc ownership, validation/publication order and async sequencing. Core proof
  tests now explain scenario setup/action/assertions. A final follow-up filled remaining multi-block
  test-driver comments; only comments changed in that follow-up.
- `cargo test -p stackpress-server --all-targets --features http --locked`: **56 passed**, zero
  failures, two child markers; `cargo test -p stackpress-server --doc --features http --locked`:
  **70 passed**. Server Clippy and Rustdoc with `-D warnings` also passed (`round3-*.log`).

Counts below distinguish inline `//` lines from Rustdoc (`///`, `//!`). Code means nonblank lines
excluding all `//` lines; braces count. Wrapped explanation lines count separately. These are
navigation/review signals, not a substitute for block/branch review. `lib.rs` has
declarations/re-exports only and intentionally has no filler inline comment.

| File under `crates/server/` | Before inline/code | Final inline/code | Inline / 100 code | Examples |
| --- | ---: | ---: | ---: | ---: |
| `src/builder.rs` | 74/629 | 27/149 | 18.1 | 6 |
| `src/config.rs` | 24/271 | 58/269 | 21.6 | 5 |
| `src/context.rs` | 17/131 | 17/131 | 13.0 | 3 |
| `src/core_tests.rs` | 18/430 | 72/430 | 16.7 | 0 |
| `src/lib.rs` | 0/26 | 0/29 | 0.0 | 0 |
| `src/lifecycle.rs` | 22/524 | 116/532 | 21.8 | 3 |
| `src/output.rs` | 6/49 | 9/49 | 18.4 | 0 |
| `src/plugin.rs` | 11/133 | 23/141 | 16.3 | 4 |
| `src/registration.rs` | — | 37/227 | 16.3 | 30 |
| `src/report.rs` | 10/212 | 37/212 | 17.5 | 5 |
| `src/router.rs` | 8/63 | 11/63 | 17.5 | 4 |
| `src/server.rs` | — | 9/76 | 11.8 | 2 |
| `src/services.rs` | 9/94 | 19/94 | 20.2 | 0 |
| `src/startup.rs` | — | 35/221 | 15.8 | 3 |
| `tests/concurrency.rs` | — | 49/389 | 12.6 | 0 |
| `tests/construction.rs` | — | 32/408 | 7.8 | 0 |
| `tests/lifecycle.rs` | — | 70/730 | 9.6 | 0 |
| `tests/ownership.rs` | — | 8/81 | 9.9 | 0 |

## Round 4: safety, ownership and resources

- Audited actual call lifetimes/Send constraints, Any/TypeId downcasts, Arc ownership/no implicit
  payload Clone, emitter/host references, checked sequences, bounded merge recursion, fallible paths
  and panic/cancellation behavior. No framework lock crosses await, no detached work or emitter/host
  reference cycle is introduced, and no unsafe code is needed.
- Rechecked G12/G17 finite error offers, original Arc/source retention, fresh sanitized fallback,
  delivery gates and private OutputGuard authority independent from whole-value Response
  replacement. The existing real-type decision tables, compile rejection fixtures and external
  watchdogs remain intact. No soundness defect required a public contract change.
- Added composed-startup cancellation to the existing externally watched production-path child: an
  imported Config task registers a real Arc resource, imported Listen work suspends after entry, and
  a successor is queued. Polling reaches actual suspension without sleeps; dropping the build
  releases the sole resource allocation exactly once and never starts the successor. The existing
  two-second async and ten-second external kill/reap bounds apply.
- `cargo test -p stackpress-server --all-targets --features http --locked`: **56 passed**, zero
  failures, two child markers (`round4-tests.log`).
- Targeted responsibility/resource follow-up found duplicate whole-accumulator staging in
  `ConfigLayers::combined` → `append`. `combined` now prepares one candidate; `append` atomically
  commits it. Ordered inputs, original sites and budgets remain intact. Added a public-builder
  regression: a rejected cumulative input append retains the previously accepted typed configuration
  and defaults. The same server test command then passed **57 tests**, zero failures, two child
  markers (`followup-tests.log`).

## Final verification

The final matrix below was rerun after the last comment follow-up, against the integrated HEAD and
complete lane A patch. All commands exited 0. The 15 ignored workspace entries are deliberate child
markers explicitly invoked by their watchdog/PTY parents, including blocked/noisy watchdog
self-tests.

| Exact command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `git diff --check` | Passed. |
| `cargo test --workspace --all-targets --locked` | **143 passed**, zero failed; 15 child markers. |
| `cargo test --workspace --all-targets --features server-http --locked` | **143 passed**, zero failed; 15 child markers. |
| `cargo test --workspace --doc --locked` | **394 passed**, zero failed. |
| `cargo test --workspace --doc --features server-http --locked` | **394 passed**, zero failed. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed. |
| `cargo clippy --workspace --all-targets --features server-http --locked -- -D warnings` | Passed. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features server-http --locked` | Passed. |
| Python width scan of every `crates/server/**/*.rs` line | **0 lines over 100 columns**, including merged HTTP files. |

The workspace's example enables HTTP through its dependency, so package-scoped coverage below
separately proves the actual no-HTTP selection. Native target/toolchain only; cross-platform,
branch/MCDC, stable doctest coverage, network listeners and publication readiness are not claimed.

## Independently cleaned server coverage

Each measurement immediately followed its own `cargo llvm-cov clean --workspace`. No
ignore/exclusion pattern was added; these are the tool's ordinary package line metrics, with
doctests verified separately. Exact commands:

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-server --no-default-features --all-targets --locked --json --output-path target/lane-a-rounds/coverage-default.json --fail-under-lines 70
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-server --features http --all-targets --locked --json --output-path target/lane-a-rounds/coverage-http.json --fail-under-lines 70
```

| Package / selection | Executed / executable lines | Coverage |
| --- | ---: | ---: |
| `stackpress-server`, without HTTP | 1,556 / 1,680 | **92.62%** |
| `stackpress-server`, with HTTP | 1,862 / 1,976 | **94.23%** |

Both exceed the 70% development floor. Lifecycle is 468/479 (**97.70%**) in each measurement; merged
`http.rs` is 291/296 (**98.31%**) in the HTTP scope. Line percentages do not establish
decision/branch completeness; real-type G12 table tests independently check hook traces, retained
failures, error recovery and finalized delivery combinations.

Remaining uncovered areas include some composition conflict returns, formatter/diagnostic
convenience methods, thin registration forwarding/method options, defensive
serialized-output/checked-arithmetic failures, and standard-status/error-setup failures behind
checked invariants. `output.rs` is 23/33 (**69.70%**) as an individual file; this is not the crate
gate. Its real cancellation/replacement/no-retry scenarios run inside external watchdog children,
and ordinary aggregate coverage does not establish complete per-branch evidence there. No branch was
excluded or synthetic shallow test added to raise the percentage.

## Receipt routing and stop

The separate receipt keeps the existing task near its current length rather than pushing it beyond
about 480 lines. A link was appended only to task 00005's Implementation notes; historical
implementation and coordinator Verification/Acceptance notes were preserved. No public
signature/design deviation or new decision is requested. Stopped after released rounds 1–4 for
independent coordinator re-verification.

Workspace validator result: `python3 .agents/scripts/validate-agent-workspace.py` passed with 16 advisory warnings and no errors. The receipt is 195 lines; prior task sections and all Verification/Acceptance notes remain unchanged.
