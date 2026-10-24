# Task 00002: Foundation data, events, and values (M1–M3)

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). Slice 1 milestones M1 (data), M2 (queue/events/status/diagnostics), and M3 (request/response/session/cookie/resolve/facade).

## Task summary

Implement the transport-neutral `stackpress-lib` foundation mapped in the [upstream inventory](../design-upstream-inventory.md) according to the [API contract](../design-api-contract.md) and [data/value/error contract](../design-values-and-errors.md), incorporating the owner/coordinator decisions in [decisions](../decisions.md). Replace the M0 signature fixtures with production types and re-prove them. Stop before refactoring rounds with tests and per-crate coverage.

## Implementation steps

1. M1: Value/Nest/paths/ordered collections/parsers/files, Clone and `Shared` semantics, serialization rejection paths.
2. M2: ItemQueue/TaskQueue, exact/Express/regex/route matching, immutable emitter with hooks, handles, replacement and composition, status catalog, diagnostics; production Send/nested/spawn/compiler/watchdog proofs.
3. M3: request/response/session/cookie/header/body values, one-attempt load/delivery, envelopes, plain-data `resolve`, facade consumer and bootstrap-mapping example.
4. Each milestone's tests pass before the next begins; stop after M3 for coordinator review.

## Verification process

The [risk inventory and verification plan](../design-risks-and-verification.md) sections E1–E4: full test suite including watchdog/compiler fixtures, doctests, fmt, Clippy `-D warnings`, Rustdoc `-D warnings`, the 1.96 check, dependency review, and `cargo llvm-cov` with the 70% per-crate development gate.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on 2026-10-24 by the implementer agent. Coordinator M3 review findings (comment density, whole-tree clone per Nest write, inconsistent Nest naming, long functions, route/method/param DX, DataPath display, Response accessor docs) were carried into [task 00003](00003-review-rounds-and-closeout.md). Committed in `e230359`.

## Verification notes

Coordinator re-verification on 2026-10-24: fmt passed; 33 integration tests and 17 doctests passed; Clippy clean; `cargo llvm-cov` reported `stackpress-lib` at 92.33% (2,708/2,933 lines), matching the receipt. The implementer's complete M1–M3 receipt, moved losslessly from `docs/trial-3/m1-m3-receipt.md` (headings demoted three levels, links rebased), follows.

#### M1–M3 — foundation implementation checkpoint

Status: **implemented and tested; stopped for coordinator review before refactoring rounds**. Date: 2026-10-24. Branch: `trial/3`. This is the requested M3 checkpoint, not the final Slice 1 closeout.

Historical checkpoint: the coordinator approved M3 and authorized rounds 1–4. Those rounds are now complete; the [final Slice 1 receipt](00003-review-rounds-and-closeout.md) supersedes the pending-round status below and records the final API/coverage. This file retains the reviewed M3 baseline.

##### Scope and milestone sequence

The coordinator approved the [M0 receipt](00001-scaffold-and-send-signature-proof.md), independently re-verified its signature proof, and authorized M1 → M2 → M3 without intermediate stops. The [design checkpoint](../index.md) retains the complete upstream inventory, accepted decisions, deliberate compatibility changes and later-slice deferrals.

Only the root `stackpress` facade and `crates/lib` (`stackpress-lib`) exist as Cargo packages. Both use edition 2024, resolver 3, rust-version 1.96, `publish = false`, `unsafe_code = "forbid"`, and `missing_docs = "deny"`. No placeholder package or server adapter was added.

| Milestone | Implemented behavior | Passing evidence before advancing |
| --- | --- | --- |
| M1 | Rich Value/Nest data; dot/literal/bracket paths; deterministic numeric arrays/sparse objects; ordered maps/sets; visitors; args/query/JSON/multipart parsers; owned file buffers; finite budgets; explicit Shared native state and JSON rejection paths. | 11 data contract tests, including falsy/presence cases, clone isolation, malformed/binary input, numeric ordering and atomic failures. |
| M2 | Stable priority ItemQueue/TaskQueue; exact/Express/Rust-regex/neutral-route matching; immutable emitter; hooks, handles, replacement, composition; status catalog; owned diagnostic/error metadata and source chains. | 6 event/queue contracts plus production Send/nested/spawn/compiler/watchdog proofs. The M0 toy registry was removed before these proofs passed. |
| M3 | Independent request construction/load state; response helpers/envelopes/delivery state; headers, cookies and scoped session revisions; resolve from plain data/owned request; facade imports and bootstrap mapping. | 11 request/response contracts and 1 facade consumer, alongside all M1/M2 regressions. |

The final workspace run has 33 passing parent integration tests. Two additional child entrypoints are marked ignored for ordinary harness selection and are explicitly executed by the watchdog parents; they are not omitted behavioral proofs.

##### Public API and ownership choices for review

Named async functions register directly through the sealed `Handler` contract. `on_priority(event, priority, function)` and `listen(ListenerSpec::exact(event).priority(...).label(...), function)` are one-expression registration paths. The secondary `handler` helper supplies the boxed borrowing-closure expectation; its Rustdoc explains `Box::pin`, `async move`, and explicit per-call `Arc::clone` of captured configuration. TaskQueue uses the analogous sealed `TaskHandler<C>` bound, with the lifetime-indexed future family hidden inside its module.

Dispatch is all-match, descending priority, registration-order ties. Abort, error and no-listener remain distinct; no response status or general lifecycle event is inferred. Capture metadata belongs to Invocation. Builder composition moves typed matchers/listeners; receiver hooks apply, source hooks do not import implicitly. Handles retain originating identity and original/replacement sites.

Invocation ancestry is an explicitly named immutable Arc-linked parent chain. Siblings/nested calls share parent nodes without copying every ancestor string. `Invocation::path()` and error annotation materialize owned root-first diagnostic snapshots on demand. There is no runtime registration/handler mutex, global current-event slot, registry Clone, or detached task.

Value/Nest clone ordinary data deeply; only `Value::Shared(SharedNative)` retains shared native identity. Request/Response are non-Clone. Tests prove independent query→data copies, native identity through resolve, and compile rejection of stateful Clone and concurrent mutable input aliases. A separate compiler fixture rejects an escaping Invocation lifetime.

Two concrete implementation refinements are reflected in the design document:

- One-shot loaders/senders receive owned isolated `RequestView`/`ResponseView` data snapshots. Their `FnOnce` callbacks and futures must be Send, without a static requirement. Named async functions and closures borrowing a caller-owned native reader/writer work. Snapshot mutation cannot partially mutate the original; snapshots contain no native/load/delivery attempt state or owned private error causes. Only the original invocation records Attempted/Loaded/Sent/Failed/Stopped. Successful repeated calls are no-ops; failed/interrupted calls never retry automatically.
- `to_envelope()` clones rich values infallibly and `into_envelope()` moves them. Explicit `try_to_json()` is fallible and names the unsupported nested path. `errors: Option<Value>` distinguishes absence from an explicit empty object. Original Error causes remain owned by Response and consuming `into_error`; only deliberately public fields enter the envelope.

The chosen data/path recursion ceiling is 64; callers may lower it. Default payload/node/header budgets remain explicit. Loaded data and outgoing mutable stores are rechecked against the invocation's selected budgets. MIME/disposition parsing preserves quoted spaces/escaped quotes/semicolons instead of trimming a filename's quoted content.

##### DX and behavior evidence

- [Data contracts](../../../../crates/lib/tests/data_contracts.rs): literal `get`, string `path`, typed `path_as`, string setters, null/missing/scalar distinctions, numeric array normalization, ordered collection behavior, typed dates/bytes/files/native values and explicit JSON errors.
- [Event contracts](../../../../crates/lib/tests/event_contracts.rs): exact/Express/regex all-match priority/ties, custom separators, repeatable optional/named regex captures, route method matching, composition and stable replacement/removal, hook stop/error precedence, registration and phase call sites, queue before/action/after error semantics and remaining-task ownership.
- [Signature/watchdog contracts](../../../../crates/lib/tests/handler_signature.rs): production named/helper nested calls, multithreaded spawn, Send assertions, concurrent root/sibling isolation, a barrier rendezvous detecting hidden dispatch serialization, recursion limits, panic propagation followed by a usable table, cancellation of emit/queue/load/delivery, non-Send future rejection, stateful Clone rejection, mutable alias rejection and escaping-lifetime rejection.
- [Request/response contracts](../../../../crates/lib/tests/request_response_contracts.rs): source precedence, borrowed loaders/writers, atomic failed loading/serialization, typed Serialize structs for results/rows/json, false/zero/null/empty envelope fields, public/private error separation, buffered delivery/finalization guards, header injection, cookie options/codecs/date bounds and scoped deletion revisions, native resolve, plain-data resolve and concurrent nested resolve. Completed/Aborted/NoListener/Err remain distinct from response code.
- [Facade/bootstrap consumer](../../../../tests/foundation.rs): named config/listen/route startup tasks mutate a host-owned builder in order, the listen task calls a minimal plugin registration function, then build freezes registrations. The registered route explicitly calls the capability using `inv.resolve(..., json!({...}))`. This is an illustrative startup mapping with no server lifecycle implementation or listener.

The [signature support file](../../../../crates/lib/tests/support/signature.rs) now only re-exports production types. No test registry or toy Request/Response remains behind the production claims.

##### Verification

Compiler: `rustc 1.96.0 (ac68faa20 2026-05-25)`; Cargo 1.96; native target `aarch64-apple-darwin`; LLVM 22.1.2; cargo-llvm-cov 0.8.6. No feature flags are defined or selected. The required clean-before-first-build/coverage occurred in M0; no stale pre-trial target output is evidence.

| Command / check | Result |
| --- | --- |
| `cargo test --workspace --all-targets --locked` | Passed: 33 parent integration tests; both watchdog-controlled children executed by parents. |
| `cargo test --workspace --doc --locked` | Passed: 17 doctests, 16 foundation + 1 facade. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings, including the final added tests. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed without warnings. |
| `cargo +1.96.0 check --workspace --lib --locked` | Passed on the explicitly selected installed baseline. |
| `cargo fmt --all -- --check` | Passed; negative compiler fixtures also checked separately with `rustfmt --edition 2024 --check`. |
| `cargo tree --workspace --edges normal --locked` | Reviewed; no executor/native transport/database/UI dependency. Tokio is dev-only. |
| Coverage command below | Passed all 33 tests and the 70% line gate; production-only file scope verified in JSON. |

Resolved production dependencies are serde 1.0.229, serde_json 1.0.154, indexmap 2.14.2, regex 1.13.1, url 2.5.8 and httpdate 1.0.3, plus their locked transitive dependencies. Token decoding/encoding uses small explicit foundation helpers; no native filesystem/network access is required. Tokio 1.54.1 is used only by tests. The authorized Cargo dependency download retry used normal network approval; no extra toolchain was installed.

Potentially deadlocking scenarios run in a child with a 2-second async deadline and a 10-second external wall-clock deadline. A deliberately blocked child proves kill + wait/reap at 250 ms. Compiler fixtures each have a 10-second external deadline. Pending/cancelled callbacks are dropped before inspecting their exclusively borrowed state; no detached native side effect is started by the foundation. Logs are bounded on read and live in ignored `target/m0/`.

##### Current per-crate line coverage

```sh
mkdir -p target/coverage
cargo llvm-cov --workspace --all-targets --locked \
  --ignore-filename-regex '(/tests/|/examples/)' \
  --json --output-path target/coverage/m1-m3.json --fail-under-lines 70
```

The executable-line numerator/denominator comes from the final workspace LLVM JSON report. Dependencies, tests, UI/watchdog fixtures, and examples are excluded from first-party production coverage. No production module or shipped generated code is excluded. Normal doctests pass separately; stable doctest coverage and branch coverage are not claimed. Watchdog children inherit instrumentation and produce their own profiles, included in the merged report.

| Changed crate | Covered / executable lines | Coverage | Development gate |
| --- | --- | --- | --- |
| `stackpress-lib` | **2,708 / 2,933** | **92.33%** | Passed ≥70%. |
| `stackpress` | No executable lines: explicit re-exports only. | **N/A**, not 100%. | Facade consumer integration and doctest pass; no implementation percentage is invented. |

The source file breakdown is below. These are not separate per-file gates; they keep weaker areas visible for the later reviews.

| Production file | Covered / executable lines | Coverage |
| --- | --- | --- |
| body.rs | 20 / 31 | 64.52% |
| data/mod.rs | 14 / 14 | 100.00% |
| data/nest.rs | 252 / 284 | 88.73% |
| data/parsers.rs | 342 / 371 | 92.18% |
| data/path.rs | 37 / 43 | 86.05% |
| data/value.rs | 219 / 260 | 84.23% |
| error.rs | 123 / 145 | 84.83% |
| event.rs | 423 / 436 | 97.02% |
| headers.rs | 85 / 87 | 97.70% |
| pattern.rs | 186 / 197 | 94.42% |
| queue.rs | 132 / 139 | 94.96% |
| request.rs | 221 / 224 | 98.66% |
| response.rs | 331 / 367 | 90.19% |
| session.rs | 303 / 315 | 96.19% |
| status.rs | 20 / 20 | 100.00% |

Representative remaining gaps include structured Body type-name presentation arms, Value accessor/conversion alternatives, some malformed-input/budget/metadata branches, debug rendering and unreachable-in-normal-operation counter exhaustion. The line percentage does not establish exhaustive branch, fuzz, performance or platform coverage. Safety-critical nested/concurrent/attempt-state contracts have dedicated behavior/compiler proofs rather than relying on the aggregate percentage.

The JSON report remains an ignored local artifact at `target/coverage/m1-m3.json`; the numbers above are the durable checkpoint receipt. A root-only `cargo llvm-cov report` selects the facade and reports zero executable lines, so reproductions must use the stated workspace command.

##### Pending review rounds and limits

**Rounds 1–4 have not started.** Routine implementation fixes and tests above belong to the milestone baseline. The next work requires the coordinator's M3 review reply as explicitly requested:

1. Logic/complexity, meaningful branch risks and any justified behavior-preserving changes, followed by tests.
2. Responsibility/abstraction, ownership and coupling review, followed by tests.
3. Consistency/idiomatic API/Rustdoc/comment-density review, including the complete per-entrypoint compiling-example standard and explanatory private-helper/block comments, followed by tests/doctests/lint/docs. Current successful `missing_docs`/Rustdoc checks do not by themselves establish that full density standard.
4. Dedicated safety/soundness review, followed by affected tests and final per-crate coverage. The final receipt will use E5/00037 and identify actual findings and no-change results.

No soundness blocker or unresolved owner decision was encountered in M1–M3. Soundness, API/breadth acceptance and documentation completeness remain subject to those reviews. This checkpoint proves the native macOS target only. Server/CLI/lifecycle/native streaming (Slice 2), Idea/filesystem/generation (Slice 3), SQL (Slice 4), and Leptos/Tauri (Slice 5) remain deferred as inventoried; cross-target compilation and real adapter execution have not been claimed.

No other trial branch was read, diffed, logged or checked out. No commit, push, publication, deployment, listener, or KB publishing occurred. Existing unrelated files were preserved; all requested implementation/doc files remain uncommitted for review.
## Acceptance notes

No human-reviewable visual output applies, so this task finishes at `verified`. The owner approved committing the Slice 0+1 work on 2026-10-24.
