# Task 00005: Slice 2, server capability host and CLI adapter

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-24 ("please start slice 2"). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Add `crates/server` (`stackpress-server`), the transport-neutral capability host that took over Ingest's responsibilities. Add a CLI adapter in `tools/` that invokes the same capabilities from a terminal, such as `stackpress emit profile-search --id abc123`. The host owns configuration, plugin registration and composition, services, the ordered startup phases (`config` → `listen` → `route`), and the top-level `request` → selected operation → `response` lifecycle with `error` handling. Nested `emit`/`resolve` stays raw dispatch. Transport adapters (CLI here, plus an HTTP adapter if the design checkpoint includes it) map their native edges onto that host.

This slice resolves the carried Gaps Q04 (plugin dependencies and registration conflicts), Q05 (top-level lifecycle, abort, finalization, sent state, error-handler failure), Q10 (configuration authoring, loading, and validation), and Q14 (tracing and correlation) in [decisions](../decisions.md). It may propose an answer to Q01 (first vertical capability) for owner decision.

## Implementation steps

1. Design checkpoint, with no code. Write it as spec design files for Slice 2 (each Agent File within the 500-line cap), linked from the spec index. It covers:
   - the upstream inventory: Ingest `Server`/`Router`/`Route`/`Loader`/handler facets, Stackpress server `Terminal`/`events`/`scripts`, plugin and bootstrap contracts, and the lib items deferred to Slice 2 (`Terminal`, `control`, `input`, `RouteMap`, `RouterContext`, `RouterArgs`, `RouterMap`, host context, capture injection, native envelopes);
   - the API sketch;
   - the implementer's risk inventory;
   - open decisions with recommendations;
   - the verification plan.
2. Coordinator review and owner decisions, then a milestone plan.
3. M0: the riskiest proof.
4. Milestones, with a stop before refactor rounds.
5. Rounds 1–4 and a closeout receipt.
6. Crate docs in `crates/server/docs` (and for the CLI if warranted) under the chrisai-docing standard, as a follow-up task.

## Verification process

As defined in the coordinated workflow: coordinator re-verification at every stop, using fmt, all-target tests including watchdog-bounded concurrency, doctests, Clippy and Rustdoc with `-D warnings`, the 1.96 check, a dependency-tree review, and per-crate `cargo llvm-cov` at or above 70%. No listener is started outside devmetrics; transport tests run in-process.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Design checkpoint written on 2026-10-24, with no implementation code: [host inventory](../slice-2-design-upstream-host.md), [adapter/terminal inventory](../slice-2-design-upstream-adapters.md), [host API](../slice-2-design-api.md), [lifecycle/diagnostics](../slice-2-design-lifecycle.md), [CLI/HTTP/dependencies](../slice-2-design-adapters.md), [33 risks and milestone/verification plan](../slice-2-design-verification.md), and [open decisions](../slice-2-design-decisions.md), all routed from the spec index and individually below 200 lines. Source review found that upstream Route.process discards ordinary operation abort results; the design proposes explicit corrected abort behavior, failed-request operation skip, retained dual failures and a nonrecursive fallback. It includes optional buffered HTTP, typed Arc services/config, compile-time CLI composition, all terminal deferrals, M0 proofs, rounds 1–4 and exclusive shared-file ownership. All recommendations remain unaccepted pending coordinator/owner review; the decision ledger and Verification notes are unchanged. `python3 .agents/scripts/validate-agent-workspace.py` passed with warnings only (11 existing advisory warnings; none reference the new design files). Checkpoint coverage A–E is cross-linked in the decisions file; no code tests/build/coverage, dependency fetch, listener, commit/push, publication or context promotion was performed. Stopped before M0 as requested.

### M0 implementer receipt (2026-10-24)

G12–G16 and the coordinator's service/config shortcuts, explicit delegation helper and app-only CLI packaging are applied. **M0 only is complete for coordinator re-verification; stopped before M1 and before rounds.** The original design pointer above is historical and remains intact. The approved design files now reflect G14 (no stock binary), direct Invocation shortcuts, delegation and the M0 evidence below. Verification notes/status remain coordinator-owned.

#### Scope and production boundaries

- Real `stackpress-server` and `stackpress-cli` workspace packages; root optional `server`/`server-http` facade wiring; one reviewed Cargo.lock. No example app or empty tools binary was scaffolded. Cargo metadata confirms CLI targets are a library and its test only (`autobins = false`).
- Real immutable host/service context, consuming TaskQueue startup, named function plugins, route wrapper and raw host emit/resolve. `inv.service/config/shared_service/host` are available directly. `inv.delegate` maps Completed→Continue, Aborted→Abort and NoListener→typed DelegationError with code 404; it runs no lifecycle.
- Real bounded ConfigLayers input storage, independent OutputGuard and exclusive crossterm TtySession. These are proof seams, not the full later adapters.
- **Not implemented at M0:** config recursive merge/section decoding/unknown-section policy, full plugin dependency/order preflight, full route catalog/customization, top-level handle/error lifecycle/correlation allocation, HTTP request/response conversion, actual library main_with/CLI run/parser, prompt state/render/validation driver, or the full profile-search CLI+HTTP application. Typed config and correlation values in the host proof are explicitly seeded fixtures carried through the real registry/CallContext. The disposable main_with callback prototype uses raw resolve and is not CLI lifecycle evidence.

#### Lib diff and justification

Only production `crates/lib/src/event.rs` changed in lib: **133 added / 12 removed lines**, including Rustdoc/examples. The additive surface is `EventEmitter::{emit_with_context,resolve_with_context}` and `Invocation::context<T>()`. An internal DispatchContext carries unchanged ancestry plus `Option<&(dyn Any + Send + Sync)>`; phase construction and nested emit/resolve inherit that exact borrow. Existing root emit/resolve supplies None. No server type/dependency, lock, global context, unsafe code, signature rewrite or blanket Clone was added.

Compiler/runtime tests in the external `context_contracts` integration crate prove existing public Handler/TaskHandler bounds can call the sealed supertrait methods directly. **No Handler/TaskHandler forwarding method or queue source change was necessary.** Stored wrappers require the existing `Handler + 'static` registration bound; the returned borrowing futures still are not static. One early fixture incorrectly retained a raw pointer across await and was rejected as non-Send; the corrected fixture retains the actual shared reference, which is the intended contract.

Lib tests add the context/watchdog proof, an escaping-context compile-fail fixture and an Rc context compile-fail fixture. Existing signature tests now exercise those additional fixtures. All existing Slice 1 regression suites passed after the additive change.

#### Proof signals 1–7

| Signal | Evidence/result |
| --- | --- |
| 1 borrowed host context / Send | Real typed service/config references survive awaits; parent→child emit and concurrent isolated resolves preserve host pointer, explicit root stage/correlation and ancestry. Two spawned owned roots share the immutable Server through explicit Arc. Service drop counters and Weak host references prove no framework emitter/context ownership cycle. |
| 2 unchanged root lib behavior | A contextual call followed by raw emit observes None; wrong typed context returns None. Existing matching/ties/hooks/abort/error/load/delivery/Clone/serialization tests pass. Raw Server roots retain services with Dispatch stage and no fabricated top-level correlation. |
| 3 wrappers / DX | Real route wrapper overlays captured id over explicit aggregate id without changing query storage. Listener plugin and caller file provenance survives forwarding. Startup wrappers call existing sealed TaskHandler methods. Direct lookup shortcuts and typed delegation completion/abort/missing behavior pass. |
| 4 failed startup | Config error/Abort returns no Server; accumulated services are dropped exactly once. A build dropped at its first pending Config await releases the partial service. Errors retain phase, plugin and actual registration site. Duplicate service insertion does not replace the original; a service write during Listen fails. |
| 5 main_with composition | Disposable app binary named stackpress compiles/runs with a named function plugin and `FnOnce(ConfigLayers) -> Send Future<BuildResult<Server>>`. Explicit file data reaches the app before build; post-event `--bootstrap target-data` remains capability data. Optional server-http facade values compile without sockets. Prototype stays outside the library until the CLI milestone. |
| 6 native prompt guard | Actual macOS PTY proof passes: exclusive lease, actual idle poll, bounded worker stop/join, pending-future cancellation, unwind, prior raw-mode preservation and Ctrl-C event. Parent compares real termios attributes before session-leader exit. Non-TTY input immediately returns Unsupported; no pipe reader is spawned. |
| 7 output authority | Real OutputGuard remains outside handler Response reach. Replacing Response cannot repeat a successful, partial-failed, cancelled, suppressed or externally-finalized adapter output opportunity. Spy writes count actual writer entry/partial bytes; no second call occurs. Arbitrary callback-owned external I/O is explicitly outside this authority. |

Nine compiler-rejection fixtures pass with relevant diagnostics: the six lib fixtures (non-Send future, stateful Clone, aliased mutable pair, escaping Invocation, escaping context, non-Sync context) plus disposable consumer fixtures rejecting Rc services, escaping host borrows and Server/OutputGuard Clone. Each compiler subprocess has a 10-second deadline. Context/signature/host scenarios also have async deadlines and an external kill/reap watchdog; the existing 250-ms intentionally blocked-child self-test still passes.

The PTY harness initially exposed two test-harness issues: EOF draining needed to stop on an empty read, and macOS revokes its slave after session-leader exit. The fixed harness checks actual attributes while the child waits for a bounded parent acknowledgement, then reaps it. Process inspection after the first watchdog timeout found no leftover proof processes. No crossterm bound/soundness blocker or extra dependency was found. Bounded polling assumes the exclusive framework reader; competing application crossterm calls are outside that contract. Explicit close reports OS restoration errors; Drop attempts cleanup and retains the terminal lease on persistent restoration failure rather than reporting success.

#### Commands and results

Before the first build: `cargo clean && cargo llvm-cov clean --workspace` passed; clean removed 25,314 files / 1.6 GiB from this checkout. `cargo fetch` initially hit sandbox DNS restrictions, then the approved network rerun passed. It added only the budgeted dependency graph (http 1.5.0, crossterm 0.29.0 with events and their transitives); no further dependency/toolchain was added.

| Exact check | Result |
| --- | --- |
| `cargo test -p stackpress-lib --test context_contracts --locked` | Passed external sealed-wrapper/context proof and its watchdog child. |
| `cargo test --workspace --all-targets --locked` | **52 passed**, zero failed; five ignored helper entrypoints are explicitly run/reaped by their parent proofs. |
| `cargo test --workspace --doc --locked` | **270 passed**: 268 lib, one facade, one server. |
| `cargo test -p stackpress-server --features http --all-targets --locked` | Seven ordinary tests passed; its child is run by the parent watchdog. Native HTTP value seam compiles. |
| `cargo test -p stackpress-cli --no-default-features --all-targets --locked` | Passed compilation with tty absent; zero executable test cases in that disabled-feature scope, not claimed as behavior coverage. |
| `cargo test -p stackpress-cli --test tty_contracts --locked -- --nocapture` | Two parent tests passed; PTY child completed/reaped with native termios restored. |
| `cargo clippy --workspace --all-targets --features server-http --locked -- -D warnings` | Passed without warnings. Default workspace Clippy also passed. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --features server-http --no-deps --locked` | Passed without warnings. |
| `cargo +1.96.0 check --workspace --all-targets --features server-http --locked` | Passed on the installed declared toolchain. |
| `cargo fmt --all -- --check`; `git diff --check` | Passed. |

Disposable consumer retained at `/private/tmp/stackpress-slice2-m0.bnnJng`; source/config/manifest/lock and the three rejection sources/logs remain available for independent re-verification. It uses this checkout's path dependencies and a separate ignored target directory. The exact positive command was:

```sh
cargo run --offline \
  --manifest-path /private/tmp/stackpress-slice2-m0.bnnJng/Cargo.toml \
  --target-dir /Users/cblanquera/server/projects/stackpress/rust/target/slice2-m0-consumer \
  --bin stackpress -- \
  -b /private/tmp/stackpress-slice2-m0.bnnJng/config.json \
  emit greet --bootstrap target-data
```

It exited zero and printed `{"code":200,"status":"OK","errors":{},"results":{"bootstrap":"target-data"},"total":1}`. Warm `cargo check --offline --locked` against the same manifest/target/bin rejected each negative fixture within ten seconds with the expected diagnostic strings, then restored the positive source. This proves compiled app composition and the callback/argument seam, not the unimplemented full CLI lifecycle.

#### Dependency review

```sh
cargo tree -p stackpress-server --no-default-features --edges normal --locked
cargo tree -p stackpress-server --features http --edges normal --locked
cargo tree -p stackpress-cli --edges normal --locked
```

Full receipts: ignored `target/m0/slice2-server-default.tree`, `slice2-server-http.tree` and `slice2-cli.tree`. Default server direct normal edges are lib + serde 1.0.229 + serde_json 1.0.154. The HTTP build adds http 1.5.0 → bytes 1.12.1 + itoa 1.0.18. Neither server normal tree contains Tokio, hyper, crossterm, mio, parking_lot or signal-hook. Server Tokio is dev-only. CLI owns the optional crossterm events edge (including native mio/parking_lot/signal dependencies) and limited Tokio rt/macros/sync/time; no net/process/signal/full feature was requested on Tokio. Direct/facade/tty-disabled consumer builds remain separate from runtime plugin registration.

#### Coverage and remaining gate

Instrumented workspace tests passed using cargo-llvm-cov 0.8.6 on Rust 1.96 / native aarch64 Apple Darwin. The first report write failed because clean had removed target/coverage; creating that directory and exporting the retained profiles recovered the report without repeating passing tests. A report without explicit package selection selected only the re-export facade and had no files; it was rejected as unmeasured. The final report explicitly selected all four packages and its 21 first-party production file summaries were checked for nonempty per-crate counts and ≥70% in **each** changed executable crate.

```sh
mkdir -p target/coverage
cargo llvm-cov --workspace --all-targets --features server-http --locked \
  --ignore-filename-regex '(/tests/|/m0.rs$)' \
  --json --output-path target/coverage/slice2-m0.json --fail-under-lines 70
# Report-only recovery from the same collected profiles:
cargo llvm-cov report \
  -p stackpress-lib -p stackpress-server -p stackpress-cli -p stackpress \
  --json --output-path target/coverage/slice2-m0.json \
  --ignore-filename-regex '(/tests/|/m0.rs$)' --fail-under-lines 70
```

| Changed crate / current feature scope | Covered / executable lines | Result |
| --- | --- | --- |
| stackpress-lib | **3,154 / 3,379** | **93.34%** |
| stackpress-server, including http value seam | **342 / 379** | **90.24%** |
| stackpress-cli, tty enabled and real PTY child included | **53 / 55** | **96.36%** |
| stackpress re-export facade | No executable lines | N/A; real consumer/doctest compilation passed. |

Only tests (including the cfg(test)-only server src/m0.rs), dependencies and build output are outside production coverage. No production branch or platform implementation was excluded. Doctests pass separately; stable branch/doctest coverage is not claimed. Percentages describe the current M0 scaffold, not future Slice 2 behavior, release readiness or cross-platform support. Native OS failure injection, real network/streaming/serverless, distributed correlation and other native platforms remain unverified.

The coordinated stage gate is intact: no M1–M4 work or formal rounds 1–4 were started. No listener, other-trial access, commit/push, publishing or context promotion occurred. Unrelated pre-existing spec changes were preserved. Next action is coordinator re-verification of M0 and explicit release of the next milestone/lanes.

## Verification notes

**Design checkpoint review (coordinator, 2026-10-24): approved with changes.**

- **What was reviewed:** all seven Slice 2 design files.
  - The decisions, API, lifecycle, and adapters files were read in full.
  - The verification file was read for M0, milestones, and lane ownership.
  - The inventory files were checked for coverage of the task's deferral list.
- **Findings:** the design is consistent with the binding Slice 1 decisions and the KB. The lifecycle decision table is complete and finite under repeated failures, and HTTP event selection is limited to method plus path.
- **Owner decisions:** recorded as G12–G15 in [decisions](../decisions.md). Coordinator approvals are recorded as G16.
- **DX changes requested for M0 and later:**
  - a `ServerInvocationExt` shortcut for typed service/config lookup;
  - a route-to-capability delegation helper replacing the repeated three-arm `DispatchOutcome` match;
  - no stock binary (G14).
- **Released:** M0 only. Lanes B–D start after M0 re-verification.

**M0 re-verification (coordinator, 2026-10-24): passed; M0 accepted.**

- Checks rerun independently: fmt passed; 52 tests passed, 0 failed (including watchdog and compile-fail fixtures); 270 doctests passed; Clippy and Rustdoc warning-free.
- Dependency trees: `cargo tree -p stackpress-server` normal edges contain no tokio or hyper, for both the default and `--features http` builds.
- Per-crate `cargo llvm-cov`, each preceded by `cargo llvm-cov clean --workspace`, with all crates above the 70% floor:

  | Crate | Coverage | Executable lines |
  | --- | --- | --- |
  | `stackpress-lib` | 92.93% | 3,379 |
  | `stackpress-server` | 92.88% | 800 |
  | `stackpress-cli` | 96.36% | 55 |

- Lib diff is additive. `git diff` shows no removed `pub fn`. The only additions are `EventEmitter::emit_with_context`, `EventEmitter::resolve_with_context`, and `Invocation::context`; the removed lines are private dispatch internals. `handler_signature.rs` has additions only.
- DX changes present: `ServerInvocationExt::{service, config, shared_service, delegate, host}`. No stock binary exists in `tools/cli`.
- Carried into M1: `crates/server/src/m0.rs` is an M0 proof module in production source. M1 replaces it with real modules, keeping its proofs as tests.
- Next: lane A (M1 host construction, then M2 lifecycle) in this checkout, and lane B (`tools/cli` pure parsing/control/prompt/TTY) in an isolated worktree. Lanes C (HTTP) and D (proof app) start after M2.

## Acceptance notes

Library, host, and CLI work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
