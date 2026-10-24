# Slice 2 design: risks, milestones and verification

Implementer's risk inventory and staged plan for [task 00005](tasks/00005-slice-2-server-and-cli.md). Load before approving M0 or assigning independent implementation lanes. Risks extend beyond the named Slice 1 failures. This is a plan, not a test/coverage receipt.

## Risk inventory and proving tests

Every row requires prevention in code/review and an observable proof. Passing line coverage does not waive these tests.

| ID / risk | Prevention | Proving test or review evidence |
| --- | --- | --- |
| R01 reentrant lock deadlock | Immutable runtime/registries, no callback or service-registry lock across await. | Watchdog parent→child→grandchild, same-event bounded recursion and error listener nested calls. Inspect production synchronization paths. |
| R02 concurrent nested metadata corruption | Borrowed per-call context and immutable ancestry; no current-event/thread-local slot. | Barrier-controlled roots and sibling resolves with different req/res; verify stage, ID, host, captures and parent path after awaits/errors/aborts. |
| R03 panic poisoning | No framework poisonable runtime lock; no panic catcher claiming recovery. | Watchdog unwind through startup/handler/validator, then fresh independent invocation/prompt; audit lock/unwrap sites. User-owned locks explicitly excluded. |
| R04 Clone-as-alias | Stateful builders/Server/Request/Response/Invocation/native driver non-Clone; share only named Arc services/context. | Compile-fail stateful Clone/escaping context; deep data clone remains independent; Arc ptr identity explicit, no mutable downcast. |
| R05 lifetime/Send regression | Reuse existing lifetime-indexed handler erasure; borrowed Any context is Send+Sync; safe code only. | Named fn, helper closure, nested emit/resolve and spawned owned Server future compile/run; non-Send host/callback and escape/shared mutable pair compile-fail. All Slice 1 signature tests pass. |
| R06 host ownership cycle | HostContext has no Server/emitter/handler; no implicit service container back-reference. | Weak/drop counters prove server/host/service release after calls; source review exposes app-chosen Arc cycles as app responsibility. |
| R07 wrong registry type/hidden sharing | TypeId + optional checked name, Arc<T> insertion, checked typed lookup and explicit replacement. | Missing/wrong name/type, duplicate type/name, absent replacement, explicit newtype roles, concurrent reads and expected service drop counts. |
| R08 startup partial publication/replay | Consume builder, preflight selected graph, finish phases before building table. | Failure/Abort/cancel at each phase never yields Server; prior tasks not replayed; external side effect counter documents no rollback. |
| R09 dependency nondeterminism | Validate IDs, cycles, required/optional order; preserve explicit selection order. | Duplicate/self/cycle/missing/out-of-order, optional absent/present, helper composition, stable phase ties and plugin/site in errors. |
| R10 phase mutation escapes | Seal callback collection; services freeze after Config; checked operation/route phase methods. | Late plugin/phase insertion, service write during Listen/Route, callbacks trying to mutate frozen config all fail without partial state. |
| R11 conflict accidentally replaces listener | Existing handles and all-matching ordering; duplicate matchers allowed, duplicate singleton services/config rejected. | Duplicate exact/Any/parameterized routes, cross-plugin ties; handle replace/remove sites and wrapper retention; no implicit label replacement. |
| R12 public Error consumes original diagnostic | Report retains explicit Arc<Error> before response mutation; source wrapper preserves chain. | Original error/cause/frame identity survives sanitization, successful recovery, error-handler failure and invalid public-error setup. |
| R13 recursive fallback/error amplification | Error stage failure terminal; per-stage offer bound; Response never re-entered. | Request/operation/response each fails; error listener fails/aborts/absent; assert exact finite call trace and both failures, no duplicate hooks. |
| R14 guard failure still runs operation | Failed Request/Input skips Operation under proposed policy. | Guard writes then errors; operation side-effect counter stays zero for all error-offer outcomes. |
| R15 abort/finalization ambiguity | Explicit stage table, DeliveryState checks, output separate from Abort. | Every stage abort with empty/body/status/suppression; error abort; Sent/Stopped/Attempted/Failed at entry and after callbacks; no retry or Response-finally. |
| R16 stale success data leaked on error | Fresh error response; sanitize fallback; source response `apply_error` left unchanged. | Set secret body/headers/cookie/stack then fail; none reaches wire; error handler sees retained original via context; recovery must deliberately rebuild. |
| R17 failed/sent output retry | Preflight conversion, one dispatch_with attempt, no error-hook recursion on Output failure. | Spy writer fails before/after partial bytes, body conversion fails, handler calls dispatch/stop; assert native call count ≤1 and correct report/suppression. |
| R18 truthiness/HTTP status confusion | Body variants and Status distinct from DispatchOutcome; adapter validates HTTP range/body rules. | Empty text/bytes/null/false/0; matched no-op vs missing; status-only redirect; custom 0/700, 1xx, 204/205/304 and HEAD; CLI exit values independent. |
| R19 capture collision/injection | Host route-only wrapper; literal keys; per-listener incoming-wins and atomic bounded merge. | Query/post/explicit id collision, punctuation name, two matching captures, nested raw capability, positional args, limit failure unchanged data; raw lib captures unchanged. |
| R20 URI normalization changes authority | Compute route event from native checked method/path; no caller event selector or trusted forwarded headers. | `%2F`, repeated/trailing slash, dot segments, encoded punctuation, absolute/origin targets, extension verb, query event attempts and invalid targets; never select a different route silently. |
| R21 public surface accidentally emits arbitrary operation | HTTP derives only METHOD/path, no generic emit endpoint. | Arbitrary event `profile-search` exists; `/emit?event=profile-search` is 404 without a registered route; allowed explicit route invokes it once. |
| R22 malformed/excess input or config blowup | Finite bytes/depth/entries/layers/header/body/merged-output limits; staged merge/decoding. | Limit and limit+1, deep JSON, many small layers, arrays/null/scalar replace, unknown sections/fields, invalid typed config; previous accepted data remains unchanged. |
| R23 header/cookie/framing corruption | Multi-value append, checked values, adapter framing, scoped tombstones. | Two cookies/multiheaders, replacement/removal domain/path, injection CRLF, invalid binary header, duplicate content framing and response byte count. |
| R24 CLI token/global-state ambiguity | Direct argv parse, pre-emit operator boundary, supplied sinks/config path. | Embedded spaces/&, grouped short flags, `--`, false/0/empty aliases, non-UTF-8, missing event/value, event flags named force/bootstrap; no global argv reparsing. |
| R25 machine output contaminated/secret dump | One envelope stdout, operator text stderr, safe metadata default diagnostics. | Byte-for-byte sink tests for success/unknown/error/recovery/verbose/color/brand; private cause/session/captures/config/native values absent by default. |
| R26 prompt deadlock/leaked raw state | Exclusive driver lease, bounded event source, RAII restoration, stop signal and joined bounded worker. | External watchdog cancellation before/during validation/read/send, EOF/error/panic/output failure/drop; prior raw state restored and all workers reaped. |
| R27 prompt state races/Unicode corrupt | Serialized state transitions and validation IDs; cancellation invalidates pending results; char-safe edits. | Delayed validators, repeated Enter, stale result after Cancel, Unicode backspace, required/default/pattern/Keep/Clear, Tab/Editable and final transformer. |
| R28 non-TTY read hangs runtime exit | No hidden blocking stdin read; explicit/default/buffered inputs or NonInteractive. | Non-TTY/no-data returns promptly; default and supplied line fixtures; non-TTY output contains no escapes; document native pipe limitation. |
| R29 correlation collision/wrap/log injection | Checked per-host atomic IDs, bounded printable optional prefix, local-only trust contract. | Concurrent uniqueness, nested inheritance, forced exhaustion via unit seam before callbacks, multiple host scope distinction; reject control-character labels/prefix. |
| R30 dependency/feature contamination | Default host free of runtime/network; adapters optional, facade acyclic. | Default/direct/facade/http/no-default CLI dependency trees; compile selected/disabled experimental plugin and prove disabled code is absent from the graph. |
| R31 stale evidence or untested platforms | Fresh commands and per-crate scopes; no other trial access. | Record compiler/target/features/executable lines; real production path and application binary imports. No network/OS-support claim from buffered fixtures. |
| R32 arbitrary callback/Write never returns | Framework creates no liveness promise for user code; output/prompt boundaries explicit. | Watchdog self-test kills/reaps a deliberately blocked child; document synchronous sink behavior and no cancellation guarantee for arbitrary caller-supplied blocking code. |
| R33 whole Response replacement resets its attempt state | Keep native writer and independent one-attempt token private in adapter envelope; value-instance delivery state is not unforgeable native-output history. | Handler replaces Response before/after low-level dispatch/stop; adapter output count remains ≤1. Document inability to observe arbitrary caller-owned external writers and preserve existing lib API. |

## M0: riskiest proof and first stop

After design approval only, scaffold the real server/CLI packages and root optional facade wiring, then prove **borrowed host context through the existing sealed Send handler machinery and route/startup wrappers**, before implementing broad host APIs. Use current foundation types, not replacement proof models.

Proof signals:

1. Named async operation borrows a typed Arc-backed service across await, calls nested emit and concurrent isolated resolves, and sees stable parent stage/correlation/host before/after them.
2. Existing root lib calls still have no host; its current behavior/tests remain unchanged. No escaping Invocation/context, non-Send service or aliased mutable pair compiles.
3. A host route wrapper calls an existing named Handler after bounded capture injection, retaining source registration/plugin/label metadata and a Send future. Test whether sealed supertrait invocation already suffices; add only justified forwarding methods if compiler evidence requires them.
4. A plugin named function schedules a named async StartupContext callback through TaskQueue; wrapper supplies diagnostic identity; an aborted/error/cancelled startup cannot publish a Server.
5. Minimal `main_with` consumer proves application compile-time composition and operator config timing in a disposable consumer; no runtime code loading or double argv parse.
6. Early native prompt guard probe verifies nonblocking readiness/worker stop, raw-state restoration and cancellation/drop ownership. If crossterm cannot satisfy the claimed bound, stop with evidence before implementing a more complex native input mechanism or adding dependencies.
7. Whole-value Response replacement cannot obtain/reset the adapter's private output authority; low-level delivery state and native adapter output history remain distinct. The current mutable Handler signature cannot enforce a stronger arbitrary-external-I/O claim.

Compile-fail checks and deadlock-class runtime proofs use the watchdog rules below. Run all Slice 1 regressions after lib changes. **Stop with exact commands/results and the lib diff justification for coordinator re-verification.** A failed proof is a blocker for the affected design, not permission to bypass Send, use unsafe, hide locks, detach input workers or broaden dependencies.

Before the first implementation build, follow the coordinated workflow's `cargo clean` and `cargo llvm-cov clean --workspace` rule on this checkout only, recording the action; existing output from this trial is not proof by itself. No cleaning/building is needed during this design-only checkpoint.

## Implementation milestones after M0 release

| Milestone | Deliverable and required evidence |
| --- | --- |
| M1 host construction | Plugin specs/preflight, config merge/typed validation, services, startup phases and route/listener registration. R07–R11/R22 tests; deterministic failure reports; no partial host. |
| M2 lifecycle | Handle/call/raw distinction, context extension, correlation, complete stage/error/finalized table and sanitized fallback. R01–R06/R12–R19/R29 tests with real lib types. |
| M3 adapters | Buffered optional HTTP and CLI parse/run/control/exit/output; compare same capability/response and lifecycle in-process. R17/R20–R25/R30 tests. |
| M4 prompts and application proof | All input/control behaviors, native driver limitation/cleanup tests, explicit compiled profile app, customization and disabled-plugin builds. R26–R28, actual app binary stdout/exit proof. |
| Baseline stop | Full relevant workspace tests, per-changed-crate ≥70% line coverage, feature/dependency matrix, inventory disposition audit. **Stop before rounds** with coordinator receipt. |
| Rounds 1–4 | Incorporate coordinator findings, then dedicated reviews below, each test-verified. Follow-up rounds for remaining material findings only. |
| Closeout | Final checks, per-crate coverage, exact feature/target/limitations and receipt in task Implementation notes. Coordinator owns Verification notes/verified status. |
| Documentation follow-up | Separate task for server/CLI docs with README/getting-started/concepts/guides/api under chrisai-docing; plain Markdown examples compile-checked in disposable consumers. No silent implementation change during docs. |

No real listener, commit, push, publishing, KB MCP/indexing or context promotion. No generated model, DB-backed profile implementation or UI proof is inferred. The sample profile search is an in-memory application capability only.

## Potential parallel implementation ownership

Do not start lanes until coordinator assigns separate task briefs and approves shared contracts after M0. This checkpoint spawns no agents. Up to four implementers may work on disjoint files; the coordinator directs/re-verifies and does not write implementation code.

| Lane | Exclusive write set |
| --- | --- |
| A host/integration owner | All `crates/lib` changes/tests; server core files except http; **root Cargo.toml, Cargo.lock, src/lib.rs**, server Cargo.toml/src/lib.rs and all shared manifest/export changes; spec integration receipt. Sole owner of shared files throughout integration. |
| B CLI/prompt owner | `tools/cli/**` including its local manifest/tests/docs drafts. Proposes root member/lock changes to A; never edits shared files. |
| C HTTP owner | `crates/server/src/http.rs`, `crates/server/tests/http_*.rs` only. Proposes dependency/export/feature changes to A. No lifecycle duplicate or shared test helper edits. |
| D proof application owner | `examples/capability-host/**` and application fixtures only. A adds its workspace member/lock changes. No other lane edits these paths. |

M1/M2 remain sequential under A. B can implement pure args/control/prompt state after signatures freeze; C/D consume the agreed API only after M2. Later changes to shared signatures are coordinated with A before consumer edits. Use isolated Paseo worktrees if write sets need to overlap; integrate only reviewed scoped results, rerun consumer checks after shared changes. Never read another trial branch as an integration shortcut.

## Watchdogs and branch-risk review

Reuse the existing test watchdog pattern: ordinary async scenario deadline 2 seconds; external subprocess deadline 10 seconds. Compiler fixtures get an external 10-second deadline. Instrumented runs may declare a larger finite deadline with a reason. Each timeout kills **and waits/reaps** the exact child; output reads are bounded. Include the deliberately blocked-child self-test, with a short 250-ms deadline. No sockets/listeners even in tests.

Barriers/oneshots establish actual interleavings, not sleep guesses. Child entry uses an exact test marker so it cannot recursively spawn its own parent. Never join a possibly deadlocked test thread as the sole timeout. Driver-drop proof needs the external deadline even when production driver claims bounded cleanup. No successful run leaves framework workers or children alive.

Before each round/closeout, compare the stage decision tables and meaningful uncovered branches: input failure, guard error, no-listener/no-op, each abort, original+secondary errors, finalized error handling, malformed public fields, preflight failure and partial write. Review config null/arrays/mixed-type collisions, dependency graph/order combinations, capture collisions and prompt cancellation while validating. Record untested critical combinations explicitly; a percentage is not proof of those branches.

## Review rounds

1. **Logic/complexity:** stage/offer/output transitions, bounded fallbacks, phase mutation gates, graph/order/merge policies, capture/wire/exit precedence, prompt transitions and remaining branch-risk combinations. Refactor real findings, rerun affected tests.
2. **Responsibility/abstraction:** host versus lib versus adapter ownership; config/service authority, wrappers and error-source sharing; no hidden runtime/global/lock; avoid separate duplicate lifecycles or unnecessary traits/utilities. Rerun affected consumers and dependency checks.
3. **Consistency/Rustdoc/comments:** named async examples first; every function/private helper documented; purposeful comments per logical block and line-level branching/state/async explanations; uniform Result/Flow/status/delivery language; 100-column readable examples. Rerun tests/doctests/fmt/Clippy/Rustdoc.
4. **Safety/soundness:** lifetimes and erasure, immutable host sharing, Arc cycles, mutable borrows, recursion, cancellation/drop/panic/destructor paths, checked bounds/counters, raw terminal ownership, worker shutdown, one-attempt I/O and source-chain ownership. Rerun compile-fail/watchdog/concurrency/error/output suites.

No-change rounds state reviewed scope and why no edit is warranted, with passing relevant tests. Repeat targeted rounds while material issues remain. Standing authorization covers behavior-preserving rounds after the coordinator releases that stage; new public behavior/dependencies/scope returns for owner decision.

## Verification commands and evidence scope

Use actual introduced package/feature names after approval. Planned native baseline is installed Rust 1.96 on `aarch64-apple-darwin`; no other toolchain installation or OS/platform claim. Preserve current regression suites.

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo +1.96.0 check --workspace --all-targets --locked
cargo test -p stackpress-server --features http --all-targets --locked
cargo test -p stackpress-cli --no-default-features --all-targets --locked
cargo tree -p stackpress-server --no-default-features --edges normal
cargo tree -p stackpress-server --features http --edges normal
```

Also run doctests/Clippy/Rustdoc with HTTP and valid CLI tty/no-tty features; compile facade default/server/server-http consumers and selected/disabled example plugin configurations. Inspect the direct CLI and facade normal-edge graphs, not only workspace feature-unified output. No all-features assumption if future target features conflict.

Coverage collection is clean per measured feature set and checked per crate, including server-http and CLI prompt production code:

```sh
cargo llvm-cov -p stackpress-lib --all-targets --locked \
  --ignore-filename-regex '/tests/' --fail-under-lines 70
cargo llvm-cov -p stackpress-server --features http --all-targets --locked \
  --ignore-filename-regex '/tests/' --fail-under-lines 70
cargo llvm-cov -p stackpress-cli --all-targets --locked \
  --ignore-filename-regex '/tests/' --fail-under-lines 70
```

Retain JSON/LCOV artifacts with file summaries, covered/executable lines, tool/compiler/target/features and justified exclusions. Instrumented watchdog children inherit profile settings and participate in merged reports. Tests/dependencies/build output are outside production coverage; shipped source is never broadly excluded. Example app executable code also needs its ≥70% scope if changed. Re-export-only facade is N/A, never 100%. Doctests run separately; no stable branch/doctest coverage claim. Publication requires ≥90% plus separate authorization, not this development gate.

The final receipt lists inventory now/deferred differences, approvals, M0/baseline checks, each round's findings/tests, final per-crate coverage, exact supported feature/target scope, native prompt limitation and S2-T deferrals. KB validation is recorded separately from code verification. Coordinator independently re-runs evidence before marking task verified.
