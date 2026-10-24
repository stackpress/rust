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

### Lane A M1/M2 stop receipt (2026-10-24)

M1 host construction and M2 lifecycle are implemented on `trial/3` from approved M0 `996d31f`. This is the required **M2 stop before M3/M4 and formal rounds 1–4**; it is not whole-Slice-2 closeout or publication. Coordinator Verification notes/status are untouched.

#### Behavior and production boundaries

- **M1:** validated `PluginSpec` selection and complete preflight before callbacks (unique/bounded IDs, missing/self/cycle/order errors, optional-present prerequisites, supplied order retained); immutable plugin descriptors; recursively merged typed config defaults → explicit layers, arrays/scalars/null replacement, unknown roots rejected, duplicate key/type rejection and validation before startup; finite raw/aggregate byte, depth, node and layer budgets; owned JSON passes lib iterative checks before host cloning; layer/default/section/type/plugin/site diagnostics without config document dumps.
- TypeId and named service namespaces retain explicit Arc identity, checked missing/wrong-type errors, Config-only insertion/replacement and original/replacement sites. No registry lock or implicit payload Clone exists. Config → Listen → Route runs through real TaskQueue instances, with stable priorities/ties and mutation gates. Error, Abort, cancellation and panic publish no partial Server; completed external effects are not rolled back.
- All listener/route/method helpers and `RouteSpec` forward synchronously with actual caller metadata. Host route replacement retains capture policy and IDs; builder composition moves listener IDs, owners/sites and pending startup contributions. Named captures atomically override literal aggregate-data keys in listener order; query/post and positional captures remain separate, and failed capture injection leaves aggregate data unchanged.
- **M2:** `handle`, `call`, `handle_route`, `handle_with_options` run one finite lifecycle. `emit`, `resolve`, `resolve_route` and nested Invocation calls remain raw. G12 guard skip, operation Abort, fresh error response, optional one Response pass, second Response error offer, nonrecursive error/setup-failure fallback, typed emptiness/status defaults and authoritative delivery-state handling are covered by decision tables.
- Reports retain original Arc<Error> identity, listener frames, source chains and ordered failures independently of response recovery. A private shared-source wrapper preserves the original allocation in fresh public error responses. Terminal fallback drops partial headers/cookies/body/data/stack and uses generic status 500, falling back to empty output under tiny budgets. Sent/Stopped/Attempted/Failed values never trigger another hook/write; existing private adapter OutputGuard authority survives whole-value Response replacement.
- Checked per-host AtomicU64 correlation is allocated once before input work; raw roots have none, nested work inherits the exact stage/ID/failure/adapter metadata, and independent hosts intentionally have separate ID scopes. `HandleReport::is_success` inspects final public recovery and delivery/terminal state independently of diagnostic outcome; default Display omits cause/payload details and `developer()` opts into causes.
- **G17 clarification:** before any normalized Request exists, input conversion/normalization failure returns sanitized status 400 with an Input failure and no callbacks/fabricated Request. Existing normalized-request load/parse failure gets the full G12 error/Response path. `Server::handle_input` is the crate-private shared loader runner; `Server::input_failure` is the no-request adapter seam. Lane C should use these, without adding another lifecycle. Its unused-method allowance is temporary until that adapter is implemented.
- Production modules are builder/plugin/config/services/context/lifecycle/report/router plus unchanged output/http. M0 proofs are preserved in cfg(test) `core_tests.rs`; no proof-named production module remains. Lib's only production change in this milestone rejects control characters in listener labels/plugin metadata, with an atomic-rejection regression. No lib public signature changed. Root manifests/facade/lock and `tools/cli/**` were not edited; the coordinator's concurrent G17 decision edit was preserved.

#### Verification and exact commands

Fresh effort began with `cargo clean && cargo llvm-cov clean --workspace` (11,175 files / 1.5 GiB removed from this checkout). No dependency, toolchain, listener, commit/push or other trial source was added/read. Compiler is rustc 1.96.0 (ac68faa20, LLVM 22.1.2), native `aarch64-apple-darwin`; cargo-llvm-cov is 0.8.6.

| Command | Result |
| --- | --- |
| `cargo test --workspace --all-targets --locked` | **77 passed**, 0 failed; six ignored child entrypoints are explicitly launched/reaped by watchdog/PTY parents. |
| `cargo test --workspace --doc --locked` | **270 passed**, 0 failed. |
| `cargo test -p stackpress-server --features http --all-targets --locked` | **31 passed**, 0 failed; two helper entrypoints run by parent watchdogs. HTTP remains the existing value-type seam, not an implemented adapter. |
| `cargo clippy --workspace --all-targets --features server-http --locked -- -D warnings` | Passed warning-free. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --features server-http --no-deps --locked` | Passed warning-free. |
| `cargo +1.96.0 check --workspace --all-targets --features server-http --locked` | Passed. |
| `cargo fmt --all -- --check`; `git diff --check` | Passed. Rust source/comment lines in the server are at most 100 columns. |
| `cargo run --offline --manifest-path /private/tmp/stackpress-slice2-m0.bnnJng/Cargo.toml --target-dir /Users/cblanquera/server/projects/stackpress/rust/target/slice2-m0-consumer --bin stackpress -- -b /private/tmp/stackpress-slice2-m0.bnnJng/config.json emit greet --bootstrap target-data` | Preserved M0 application consumer rebuilt and exited 0, printing the expected code-200 envelope with `bootstrap: target-data`. This remains raw composition evidence, not full CLI lifecycle evidence. |

Runtime evidence uses real lib values: request/operation/response failure crossed with missing/completing/aborting/failing/recovering/cosmetic-only error listeners; repeated Response failure has exactly two bounded offers; each abort and each delivery state at entry and after ordinary/error stages; malformed public-error setup and tiny-budget fallback; no-op/missing/status-only/null/false/zero/empty text/bytes; capture collisions/limits; pre/post-normalization Input behavior. Original source allocation survives the fresh response chain and recovery cannot erase report failures.

The externally watched concurrency child has a two-second async deadline and ten-second parent kill/reap deadline. It proves two concurrent roots with four rendezvousing parent→child→grandchild leaves, 64 additional concurrent unique IDs, bounded same-event recursion, nested work from Error, cancellation in each lifecycle and startup phase, release counters/Weak ownership, and independent success after handler/startup/validator panic. Four new external ten-second rustc rejection fixtures enforce non-Send services/startup futures, escaping host borrows and non-Clone Server/ServerBuilder/OutputGuard; existing lib fixtures and the deliberately blocked-child watchdog self-test remain green.

Full logs and machine-readable coverage reports are retained under ignored `target/slice2-receipts/`.

#### Coverage and dependency evidence

Each final measurement was independently preceded by `cargo llvm-cov clean --workspace`; no ignore/exclusion pattern was added. Exact measurement commands:

```sh
cargo llvm-cov -p stackpress-lib --all-targets --locked --json --output-path target/slice2-receipts/coverage-lib.json --fail-under-lines 70
cargo llvm-cov -p stackpress-server --all-targets --locked --json --output-path target/slice2-receipts/coverage-server.json --fail-under-lines 70
cargo llvm-cov -p stackpress-cli --all-targets --locked --json --output-path target/slice2-receipts/coverage-cli.json --fail-under-lines 70
```

| Crate / feature scope | Executed / executable lines | Line coverage |
| --- | --- | --- |
| stackpress-lib, defaults | 3,154 / 3,393 | **92.96%** |
| stackpress-server, defaults | 1,509 / 1,639 | **92.07%** |
| stackpress-cli, default tty baseline, unchanged by lane A | 53 / 55 | **96.36%** |

M1's earlier independently cleaned server measurement was **91.36%** (899/984 lines); its construction tests passed before M2 began. The final per-crate measurements all exceed the 70% development floor. These are the tool's ordinary line metrics, not a branch or publication guarantee. The facade has no executable implementation and is not claimed as 100% covered. Remaining uncovered paths include defensive setter failures impossible under the checked Open/standard-status invariant, arithmetic/formatting failures, some config composition conflicts, and adapter output-report convenience methods pending M3. Critical lifecycle decision combinations are exercised separately from the line percentage.

```sh
cargo tree -p stackpress-server --no-default-features --edges normal --locked
cargo tree -p stackpress-server --features http --edges normal --locked
```

Trees are retained as `target/slice2-receipts/server-default.tree` and `server-http.tree`. Default direct normal edges are stackpress-lib + serde 1.0.229 + serde_json 1.0.154. The http feature adds http 1.5.0 → bytes 1.12.1 + itoa 1.0.18. Neither server normal tree contains Tokio, Hyper, crossterm, mio, parking_lot or signal-hook. No dependency-budget expansion occurred.

**Deviations/limits:** G17 is the coordinator-approved clarification recorded in decisions and the lifecycle design, not an unapproved behavior change. No public signature deviation was needed. No real HTTP adapter/network/streaming/serverless/UI/platform support is claimed; lane C and lane D consume the API after coordinator release. No formal review round was started ahead of its gate. Next action is independent coordinator M2 re-verification and release of the adapter/proof lanes.

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

**M1+M2 re-verification (coordinator, 2026-10-24): passed; M1 and M2 accepted.**

- Checks rerun independently: fmt passed; 77 workspace tests passed, 0 failed; 31 `stackpress-server --features http` tests passed, 0 failed; 270 doctests passed; Clippy (`--features server-http`, `-D warnings`) and Rustdoc warning-free.
- Dependency trees: `cargo tree -p stackpress-server` normal edges contain no tokio, hyper, crossterm, or mio, for both the default and `--features http` builds. The url/icu crates arrive through lib's existing `url` dependency.
- Coverage, each measurement preceded by `cargo llvm-cov clean --workspace`: `stackpress-lib` 92.96% (3,393 lines); `stackpress-server` 92.07% (1,639 lines).
- Lib delta: `event.rs` +14 lines, all additions (control-character rejection in registration metadata per R29). No public signature removed.
- Layout: the module set matches the design. `m0.rs` was removed, and its proofs now live in `core_tests.rs`. The crate-level hello-world is about 12 lines with a named `async fn`.
- Carried into lane C: `crates/server/src/lifecycle.rs:130` carries a temporary `#[allow(dead_code)]` on the adapter input seam (`handle_input`/`input_failure`). Lane C must consume it and remove the allowance.
- Next: commit M1+M2; lane C (HTTP adapter) and lane D (proof application) in isolated worktrees from that commit; lane B finishes its `tools/cli` rounds and is then merged and extended with `run`/`main_with`.


**Lane B re-verification and merge (coordinator, 2026-10-24): passed; merged.**

- Checks rerun in the lane worktree after rounds 1–4: fmt passed; `stackpress-cli` tests 14 passed with default `tty` and 11 with `--no-default-features`, 0 failures; 48 doctests; Clippy and Rustdoc warning-free; zero lines over 100 columns.
- Coverage after `cargo llvm-cov clean --workspace`: 96.02% (1,054 executable lines).
- Inline-comment density rose from about 3 per 100 code lines to 13–23 across production files.
- Scope: changes are confined to `tools/cli/**` plus the coordinator-approved `unicode-width` lockfile entry.
- Committed as `cb09e90` on `trial/3-lane-b-cli` and merged into `trial/3` as `55ca02b` with no conflicts. The receipt moved losslessly to [slice-2-receipt-lane-b-cli.md](../slice-2-receipt-lane-b-cli.md).
- Remaining for lane B: CLI `run`/`main_with`/exit mapping against `Server::handle`, under an extended brief.


**Lane D re-verification and merge (coordinator, 2026-10-24): passed; merged.**

- Checks rerun in the lane worktree: fmt passed; 12 example tests passed in both the default and `--features experimental` modes; Clippy clean.
- Scope: changes are confined to `examples/capability-host/**`, the root workspace `members` entry, and the lockfile package entry.
- DX review: `profiles.rs` matches the design's example. It is one plain `register` function with named `async fn` config, listen, and route steps, `inv.service::<Profiles>()`, and a one-line `inv.delegate` route, with no traits or macros.
- Committed as `340750d` on `trial/3-lane-d-app` and merged into `trial/3` as `253734a`. `Cargo.lock` auto-merged, and `cargo metadata --locked` confirms consistency.
- Merged workspace: 101 tests passed, 0 failed; 320 doctests; Clippy clean with `server-http`.
- Coverage, reproduced with the receipt's corrected command (the tool excludes `examples/` by default): 100% (108 lines) by default and 99.22% (129 lines) with `experimental`.
- The receipt moved losslessly to [slice-2-receipt-lane-d-app.md](../slice-2-receipt-lane-d-app.md).
- Remaining for lane D after lanes B and C merge: the application `stackpress` binary via `stackpress_cli::main_with`, and the HTTP-path parity proof.


**Lane C re-verification and merge (coordinator, 2026-10-24): passed; merged.**

- Checks rerun in the lane worktree: fmt passed; 53 `stackpress-server --features http` tests and 77 default workspace tests passed, 0 failed; Clippy warning-free with default features and with `server-http`; Rustdoc warning-free; zero lines over 100 columns.
- Coverage after `cargo llvm-cov clean --workspace`: `stackpress-server --features http` 93.72% (1,911 lines); `http.rs` 98.16%.
- Shared edit: the only change outside lane C's files is the coordinator-authorized replacement of the `input_failure` dead-code allowance with `#[cfg(any(feature = "http", test))]` plus a comment.
- Coordinator decisions during the lane: list-valued session cookies keep the explicit-codec requirement, with a named diagnostic in the report and a sanitized 500 on the wire; a future `HttpOptions` codec is noted, not built. The fixed empty-500 fallback starts from a fresh `Response`.
- Committed as `96dd37c` on `trial/3-lane-c-http` and merged into `trial/3` as `9d96fd7`; the lockfile is consistent.
- Merged workspace with `server-http`: 123 tests passed, 0 failed; 322 doctests; Clippy clean in both feature sets.
- Receipt moved losslessly to [slice-2-receipt-lane-c-http.md](../slice-2-receipt-lane-c-http.md).
- Round finding to carry: `http.rs` inline-comment density is about 9 per 100 code lines, below the 13–23 reached elsewhere.


**Lane D HTTP proof re-verification and merge (coordinator, 2026-10-24): passed; merged.**

- Checks: 16 example tests passed in both the default and `experimental` modes, 0 failed; Clippy clean with `server-http`.
- The four new in-process proofs: `HttpAdapter` and `Server::call` return identical public envelopes for `profile-search`; an unknown route gives a public 404; malformed input gives the G17 400 with no callbacks; the customization plugin is visible through HTTP. Proof item 5 (CLI and HTTP invoke the same capability) is complete on the HTTP side; the CLI side awaits lane B.
- Committed as `1f1a319` and merged as `93654d7`. The receipt was appended to [slice-2-receipt-lane-d-app.md](../slice-2-receipt-lane-d-app.md).


**Lane B CLI execution re-verification and merge (coordinator, 2026-10-24): passed; merged.**

- Checks rerun in the lane worktree: fmt passed; `stackpress-cli` 17 tests with `tty` and 14 without, 0 failed; Clippy and Rustdoc warning-free.
- Coverage: 96.32% (1,278 lines). Scope: `tools/cli/**` only.
- `run` enters `Server::handle_with_options` once with `CliContext` as adapter context. The exit codes follow G14: `run` maps success to 0 and failure to 1 through `HandleReport::is_success`, and `main_with` maps usage/config/startup errors to 2 and prompt cancellation to 130. Tests cover executable exits, exit precedence, usage errors, the `-b` layer and its boundary, and cancellation.
- Committed as `6e820d1` and merged as `fc14274`; the lockfile is consistent.
- Merged workspace with `server-http`: 130 tests passed, 0 failed; 327 doctests; Clippy clean.
- Receipt moved losslessly to [slice-2-receipt-lane-b-run.md](../slice-2-receipt-lane-b-run.md).
- Remaining: lane D adds the application `stackpress` binary for the CLI side of proof item 5.


**Lane D CLI binary re-verification and merge (coordinator, 2026-10-24): passed; merged. Proof item 5 complete.**

- Checks rerun in the lane worktree: 25 example tests passed in both the default and `experimental` modes, 0 failed; Clippy clean with `server-http`.
- The coordinator ran the real compiled binary (`target/debug/stackpress`):
  - `emit profile-search --id abc123` printed one envelope, `{"code":200,"status":"OK",...,"results":[{"id":"abc123","name":"Ada Lovelace"}],"total":1}`, and exited 0;
  - an unknown event printed a public 404 envelope and exited 1;
  - a usage error printed nothing on stdout and exited 2;
  - `-b x.json` after the event was treated as capability data, with no file read.
- The tests prove that envelope equals the HTTP (`GET /profiles/abc123`) and `Server::call` envelopes.
- `main.rs` is a short copyable entry: a named `build_application` plus `stackpress_cli::main_with`.
- Committed as `d00114d` and merged as `6f53513`, alongside lane A's in-progress uncommitted rounds. The files don't overlap.
- Receipt moved losslessly to [slice-2-receipt-lane-d-cli.md](../slice-2-receipt-lane-d-cli.md).


**Lane C rounds re-verification and merge (coordinator, 2026-10-24): passed; merged.**

- Checks rerun in the lane worktree: fmt passed; `stackpress-server` 56 tests with `http` and 31 with `--no-default-features`, 0 failed.
- Clippy warning-free for `-p stackpress-server` alone and for the workspace with `server-http`.
- Coverage: server+http 93.80% (1,935 lines); `http.rs` 98.31%.
- Rounds results: `http.rs` comment density is now 92/366 (25 per 100), with 4 `# Examples`. `finish` went from 67 to 39 lines. The public API is unchanged.
- Committed as `cd5a518` and merged as `94be99b`. The receipt was appended to [slice-2-receipt-lane-c-http.md](../slice-2-receipt-lane-c-http.md).
- Scope note for closeout: the example enables `stackpress-server/http`, so workspace-wide builds feature-unify HTTP. Default-only server behavior must be verified with `-p stackpress-server` alone, not `--workspace`.

## Acceptance notes

Library, host, and CLI work has no visual acceptance artifact under the spec workflow; this task finishes at `verified` after coordinator review.
