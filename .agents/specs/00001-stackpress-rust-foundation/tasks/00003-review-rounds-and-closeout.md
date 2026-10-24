# Task 00003: Review rounds and Slice 1 closeout

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). Post-M3 refactoring rounds 1–4, the Round 3 follow-up, and the Slice 0+1 closeout receipt.

## Task summary

Run the [Rust programming workflow](../../../workflows/rust-programming.md) rounds (logic/complexity, responsibility/abstraction, consistency/Rustdoc/comments) plus a dedicated safety/soundness round, addressing the coordinator's M3 findings, then close out with the E5 receipt. A coordinator-requested Round 3 follow-up formatted Rustdoc examples, wrapped over-width lines, and reviewed remaining long functions.

## Implementation steps

1. Round 1: in-place atomic Nest mutation with budget accounting; split long multi-phase functions.
2. Round 2: responsibility/ownership boundaries, private mutation planner, feature-local named phases.
3. Round 3: uniform Nest naming, DataPath Display, Method constants and `Invocation::param`, comment density, compiling examples.
4. Round 4: safety/soundness review of the mutation change, snapshot boundaries, destructor panics, budgets, and path bounds.
5. Round 3 follow-up: conventional doc-example formatting, 100-column wrapping, long-function justifications.

## Verification process

After every round, rerun affected tests; finally fmt, full tests including watchdog/compiler fixtures, doctests, Clippy, Rustdoc, the 1.96 check, and per-crate coverage, recorded with the [00037 receipt template](../../../references/00037-refactoring-round-checklists.md).

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on 2026-10-24 by the implementer agent; coordinator reviewed the final receipt and the follow-up. One residual source line in `crates/lib/src/request.rs:655` measures 101 bytes (likely a multi-byte character); it was judged immaterial. Committed in `e230359`.

## Verification notes

Coordinator re-verification on 2026-10-24 after the follow-up: fmt passed; 37 integration tests and 263 doctests passed; Clippy and Rustdoc warning-free; no doc-example lines with missing comma spacing remain; `cargo llvm-cov` reported `stackpress-lib` at 92.95% (3,320 executable lines, 234 missed) after comment reflow, consistent with the receipt's 92.94% (3,080/3,314) measured before it; facade N/A (re-exports only). The implementer's complete closeout receipt, moved losslessly from `docs/trial-3/slice-1-receipt.md` (headings demoted three levels, links rebased), follows.

#### Slice 0 + Slice 1 — implementation closeout

Date: 2026-10-24. Branch: `trial/3`. Scope: root `stackpress` facade and `crates/lib` (`stackpress-lib`), native `aarch64-apple-darwin`, no selected feature flags. All four requested post-M3 rounds are complete. No owner decision or soundness blocker remains open.

This is code verification for the agreed slices. It does not constitute owner acceptance, publication, deployment, or permission to commit/push.

##### Intended behavior and approved decisions

The [design checkpoint](../index.md) is the authoritative inventory: all public exports from upstream `../lib/src/index.ts` and behaviors in its 26 test files are mapped to implementation, a named later slice, or an intentional Rust equivalent/omission. The [M0](00001-scaffold-and-send-signature-proof.md) and [M1–M3](00002-foundation-data-events-and-values.md) receipts retain their independently reviewed historical baselines.

The implemented foundation contains:

- Rich Value/Nest data, checked literal and nested paths, ordered collections, sequential visitors, finite boundary parsers, buffered files and explicitly Shared native values. Ordinary data clones deeply; Request/Response and invocation/runtime owners are non-Clone.
- Stable item/task scheduling, all-match exact/Express/Rust-regex/neutral-route matching, before/action/after flow, stable listener handles, replacement/removal/composition at builder time, immutable runtime tables and invocation-local diagnostics.
- Independent request/response/session/cookie/header/body/status/error values; typed Serialize helpers; explicit one-attempt load/send state; neutral plain-data/owned-request resolve; facade imports and bootstrap-to-immutable-table example.

Accepted differences remain deliberate: deterministic registration-order priority ties; Flow::Abort separate from status; Rust regex rather than ECMAScript parity; raw captures stay in Invocation; mutation/registration at builder time; no implicit lifecycle or native output; original error causes remain owned and private unless explicitly disclosed. Edition 2024, resolver 3 and rust-version 1.96 are the approved toolchain contract.

Slice 2 retains server/CLI/native adapters and top-level lifecycle; Slice 3 retains Idea assessment/generation/filesystem/templates; Slice 4 retains SQL; Slice 5 retains Leptos/Tauri. Terminal/control/input, FileLoader/NodeFS, Reflection and JS callable syntax retain the exact disposition/reason in the inventory. No placeholder crates or speculative adapter implementations were added.

##### Baseline verification

The coordinator independently approved M0 and then re-ran M3 formatting, 33 integration tests, 17 doctests, Clippy and LLVM line coverage: 2,708 / 2,933, 92.33%. The authorized work after that review was rounds 1–4 incorporating the coordinator's material findings. Required removal of stale target/coverage output occurred before the first M0 build; no old trial output is evidence.

##### Round 1 — logic, complexity and budget accounting

Finding: each Nest write cloned and revalidated the complete tree. This preserved returned-error atomicity but copied unrelated payloads and made repeated construction quadratic.

Changes:

- Cache node/payload-byte totals in Nest. Value measurement validates and counts in one traversal.
- Build a read-only path plan, including scalar conflicts, missing intermediates, append overflow, final container form and budget delta. Validate incoming subtree depth at its final path depth. Commit only after every Result-producing check succeeds.
- Move affected values in place. Dense indexed writes/append retain vectors; sparse conversion/deletion moves only the affected container. Numeric normalization uses swap removal by numeric identity instead of repeatedly shifting an ordered map.
- Validate an overlay's source and replaced subtrees, then move fields while preserving top-level incoming-wins order. Clear resets both representation and totals.
- Split multipart framing, header parsing, boundary search and field decoding; split quoted parameter framing from token/quoted-value decoding.

New evidence: allocation-identity regression checks keep a 200,000-byte unrelated payload and a 16,000-byte nested sibling at the same addresses through 128 repeated writes, remove and merge. A deterministic 160-operation history compares incremental decisions with a fresh full-tree budget traversal and verifies unchanged data after failure. Sparse deletion that increases key-byte cost is rejected atomically.

Tests after the round: 13 data contracts passed, then all 35 workspace integration tests passed, including the external-watchdog/compiler suite.

Complexity boundary: edits follow the path plus incoming/replaced subtrees; representation conversion and ordered deletion can still scan/move the affected container. There is no claim that all writes are constant-time or that a benchmark was run. Unrelated payloads are neither cloned nor revalidated per write.

##### Round 2 — responsibility, abstraction and ownership

Changes:

- Keep preparation/delta/commit machinery private in `data/mutation.rs`; Nest owns the public object-root contract and cache. No generic graph-access layer or new external dependency was introduced.
- Preparation owns its path-local state instead of forwarding a long list of mutable bookkeeping arguments through recursion.
- Extract request URL policy and initial-session selection, cookie attribute emission, envelope metadata validation, dynamic token scanning, and Express/regex capture interpretation into named feature-local phases.
- Retain compiled matcher identity, private lifetime-family adaptation and sealed Handler/TaskHandler bounds. Registration remains ordinary named async functions; runtime sharing remains explicit Arc/reference ownership.

The long-function review covered every named coordinator finding:

| Original function | Result |
| --- | --- |
| `parse_multipart` | Framing loop delegates bounded headers, valid-boundary search and named form/file decoding. |
| `parameters` | Separates quote-aware splitting from parameter identity and value decoding. |
| `RequestBuilder::build` | Delegates URL authority and session initialization; source precedence remains together in build. |
| `pattern::fragment` | Keeps the cohesive literal/dynamic scan; named dynamic-token decoding owns parameter/star grammar. |
| `Nest::edit` | Replaced by read-only preparation, explicit budget deltas and non-fallible commit. |
| `cookie::serialize_with` | Encoding/injection checks precede a separate validated-attribute emission phase. |
| `Response::apply_envelope` | Fallible preparation/metadata validation precedes optional-field commit; field-presence sequencing stays together. |
| `EventPattern::captures` | Vocabulary selection delegates Express and Rust-regex capture interpretation. |

No-change decisions: item/task scheduling stays in one module because priority selection and consumed-task semantics are one flow. Request/Response/session files remain their semantic owners; splitting each getter/type into a separate file would scatter the contract. Exhaustive enum conversion/attribute matches remain explicit instead of being replaced with generic registries.

Tests after the round: all 35 integration tests passed. Later review identified a repeated runtime listener search; the follow-up replaces it with immutable table positions collected during matching. Existing matching/order/handle/nested/watchdog tests passed after that change as part of the final 37-test suite.

##### Round 3 — consistency, Rustdoc and comments

Changes:

- Literal-key names are `get/has/set/remove`. Path names are `path/path_as/has_path/set_path/remove_path`; all accept dot strings or reusable DataPath through sealed IntoPath. Redundant `get_path/get_as/set_dot/remove_dot` variants were removed and all consumers/examples updated. The scheme is documented once in Nest's module docs.
- DataPath Display renders readable dot/index paths and quotes literal punctuation/numeric keys. Typed conversion preserves a nested error suffix. Tests pin both parsed and string forms and readable failure locations.
- Add infallible standard Method/MethodPattern constants alongside Any and checked extension verbs, plus `Invocation::param(name)` for borrowed named capture access. The facade bootstrap consumer uses both conveniences.
- Explain Response mutable accessors' finalization guard, native attempt cancellation/panic semantics, explicit sharing, counter/shape invariants, async sequencing, serialization and finite-work boundaries next to the code.
- Add purposeful execution comments throughout production functions and scenario/setup/action/assertion comments in tests. Document important private state; use explicit internal imports and Rust `// ` spacing.
- Add focused compiling examples at public functions/methods and the input-adapter traits. Named async functions remain the primary handler style; the closure helper's secondary example explains boxing/move and per-call Arc cloning. New examples use production APIs and no ignored speculative examples.

Checks after the round: 35 integration tests, 262 doctests, Clippy `-D warnings`, warning-clean Rustdoc and formatting passed. Round 4 added one public budget-conversion example, bringing the final total to 263 doctests. Async examples prove compilation; actual await/spawn/cancellation behavior is exercised by runtime contracts rather than inferred from example compilation.

Per-file normal inline-comment counts appear below, refreshed after the Round 3 follow-up. Wrapping
changes physical line counts without adding new explanations. They exclude `///`/`//!` Rustdoc and are visibility data, not a target or substitute for review. Zero-count production files contain only declarations/re-exports, without function execution bodies.

| File | Normal inline comment lines |
| --- | ---: |
| `crates/lib/src/body.rs` | 4 |
| `crates/lib/src/data/mod.rs` | 2 |
| `crates/lib/src/data/mutation.rs` | 43 |
| `crates/lib/src/data/nest.rs` | 61 |
| `crates/lib/src/data/parsers.rs` | 85 |
| `crates/lib/src/data/path.rs` | 19 |
| `crates/lib/src/data/value.rs` | 71 |
| `crates/lib/src/error.rs` | 40 |
| `crates/lib/src/event.rs` | 102 |
| `crates/lib/src/flow.rs` | 0 |
| `crates/lib/src/future.rs` | 0 |
| `crates/lib/src/headers.rs` | 25 |
| `crates/lib/src/lib.rs` | 0 |
| `crates/lib/src/pattern.rs` | 44 |
| `crates/lib/src/queue.rs` | 41 |
| `crates/lib/src/request.rs` | 63 |
| `crates/lib/src/response.rs` | 108 |
| `crates/lib/src/session.rs` | 76 |
| `crates/lib/src/status.rs` | 6 |
| `crates/lib/tests/data_contracts.rs` | 61 |
| `crates/lib/tests/event_contracts.rs` | 30 |
| `crates/lib/tests/handler_signature.rs` | 57 |
| `crates/lib/tests/request_response_contracts.rs` | 69 |
| `crates/lib/tests/support/signature.rs` | 0 |
| `crates/lib/tests/support/watchdog.rs` | 7 |
| `crates/lib/tests/ui/concurrent_mutable_inputs.rs` | 3 |
| `crates/lib/tests/ui/escaping_invocation.rs` | 3 |
| `crates/lib/tests/ui/non_send_handler.rs` | 4 |
| `crates/lib/tests/ui/stateful_clone.rs` | 2 |
| `src/lib.rs` | 0 |
| `tests/foundation.rs` | 15 |

##### Round 4 — dedicated safety/soundness review

Re-examined the in-place mutation change: the immutable plan is prepared against the same exclusively borrowed tree it later edits; paths are nonempty/bounded; no user callback or payload Clone runs during commit; shape changes account for materialized/removed numeric key bytes; cache publication follows the exact tree change. Object-root and private-value borrowing prevent external cache invalidation.

Material findings and fixes:

- A final Shared native destructor can execute user code and panic. Set/merge/clear now retain removed values until both tree and cache are published, then drop them. Merge reserves its old-value holders before mutation. A caught-unwind regression uses a replacement with a different node cost and proves set/merge/clear/remove leave correct post-panic shape, limits and subsequent edit behavior. The framework itself does not catch application panics or claim rollback.
- Selected Nest/response JSON conversion previously reapplied Value's default budget. Nest serialization/typed reads now retain the store's policy, Response JSON text retains its policy, and explicit `Value::to_json_with` exposes a deliberate wire budget. Tests cover valid data above the default 1 MiB under a chosen finite limit. Rich unsupported variants still fail with their nested context.
- Dot/bracket path parsers now bound source spelling and component count before collecting every key. The 16 KiB source ceiling exceeds the largest valid spelling under the existing 8192 key-byte/64-component contract; it does not exclude valid accepted paths. Tests cover oversized/deep input.

Extended the independent budget history to include merge and clear; added actual usize-maximum append rejection, unchanged failed sparse deletion, and nested unsupported-value context. No unchecked lifetime conversion, unsafe code or manual Send/Sync implementation was introduced.

Re-examined loader/sender snapshots: views own ordinary copied data, with sharing only through the named Shared variant. They carry no unique read/send attempt state or owned private error cause. FnOnce callbacks and returned futures are Send without a static native reader/writer requirement. The original records Attempted before callback entry; cancellation and reader/writer panics keep that state and reject replay. The external-watchdog scenario now verifies immediate loader/writer unwind as well as cancellation, real nested/concurrent/spawn dispatch, bounded recursion, post-panic table reuse and no-listener behavior.

Compile rejection still uses production types for non-Send returned futures, Request/Response Clone, concurrent mutable input aliases and escaping Invocation. No production registry/handler lock exists to poison or hold across await. Opaque application payloads may choose their own synchronization; the engine does not acquire it.

Tests after fixes: all 37 workspace integration tests passed, including the watchdog and compiler fixtures. No soundness veto or unresolved behavior decision remains.

##### Follow-up rounds

Round 2 follow-up removed per-match listener re-search while retaining precomputed all-match order and original stable IDs. Round 3 follow-ups documented the safety findings and added the explicit-budget conversion example. Each was followed by affected/full tests; the final verification below is after the last code edit. No further material in-scope finding emerged from the final review.

##### Final verification and coverage

Compiler: `rustc 1.96.0 (ac68faa20 2026-05-25)`, Cargo 1.96.0; host/target `aarch64-apple-darwin`; LLVM 22.1.2; cargo-llvm-cov 0.8.6. No feature flags are defined or selected and no other toolchain was installed.

| Check | Final result |
| --- | --- |
| `cargo test --workspace --all-targets --locked --quiet` | Passed: **37 integration tests** (15 data, 6 events, 4 watchdog/compiler parents, 11 request/response, 1 facade). Two ignored child entrypoints are explicitly run/reaped by their parent tests. |
| `cargo test --workspace --doc --locked --quiet` | Passed: **263 doctests**, 262 foundation + 1 facade. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed without warnings. |
| `cargo fmt --all -- --check` | Passed; negative fixtures also pass separate edition-2024 rustfmt checking. |
| `cargo +1.96.0 check --workspace --lib --locked` | Passed on the declared installed baseline. |
| Normal-edge dependency review | Only the facade → foundation plus locked serde/serde_json/indexmap/regex/url/httpdate dependencies and their transitive dependencies; Tokio remains dev-only. |
| Coverage command below | Passed all tests and the 70% executable-line gate. |

The watchdog uses a 2-second async deadline plus a 10-second external process deadline for concurrency/suspension scenarios. A deliberately blocked child proves kill + wait/reap at 250 ms. Compiler fixtures each have a 10-second external deadline. Output reads are bounded; logs remain in ignored `target/m0/`.

```sh
cargo llvm-cov --workspace --all-targets --locked \
  --ignore-filename-regex '(/tests/|/examples/)' \
  --json --output-path target/coverage/slice-1.json --fail-under-lines 70
```

This uses the reviewed M3 command's workspace/all-targets/filter/gate scope, with a separate final artifact filename. Source-file summaries were checked to contain only first-party production Rust. No production file, panic path or generated shipped implementation was excluded. Dependencies, tests/examples and compiler/watchdog fixtures are outside production coverage. Stable doctest and branch coverage are not claimed; doctests pass separately. Instrumented watchdog children inherit profile settings and are included in the merged report.

| Changed crate | Covered / executable lines | Line coverage | Applicable development floor |
| --- | --- | --- | --- |
| `stackpress-lib` | **3,080 / 3,314** | **92.94%** | Passed ≥70%. |
| `stackpress` | No executable lines; explicit re-exports only. | **N/A**, not 100%. | Real facade consumer and doctest pass; no percentage is invented. |

| Production file | Covered / executable lines | Coverage |
| --- | --- | --- |
| body.rs | 20 / 31 | 64.52% |
| data/mod.rs | 14 / 14 | 100.00% |
| data/mutation.rs | 247 / 254 | 97.24% |
| data/nest.rs | 251 / 276 | 90.94% |
| data/parsers.rs | 379 / 409 | 92.67% |
| data/path.rs | 77 / 84 | 91.67% |
| data/value.rs | 223 / 270 | 82.59% |
| error.rs | 123 / 145 | 84.83% |
| event.rs | 432 / 448 | 96.43% |
| headers.rs | 85 / 87 | 97.70% |
| pattern.rs | 205 / 213 | 96.24% |
| queue.rs | 132 / 139 | 94.96% |
| request.rs | 228 / 231 | 98.70% |
| response.rs | 337 / 374 | 90.11% |
| session.rs | 307 / 319 | 96.24% |
| status.rs | 20 / 20 | 100.00% |

Per-file figures expose weaker areas; the policy gate is per changed crate. Remaining uncovered areas include structured Body type-name arms, Value accessor/conversion alternatives, some metadata/invalid-input branches, debug views and exhaustion paths that require unattainable normal-workload counters. The final artifact is ignored `target/coverage/slice-1.json`; the table preserves its numerator/denominator independently of build output.

##### Remaining limits and unresolved decisions

No unresolved user decision remains. Target support is verified only for native aarch64 Apple Darwin; Linux/Windows/WASM/Tauri/real server adapters are unverified. No fuzz campaign, performance benchmark, branch score or exhaustive proof of arbitrary application callback behavior is claimed. Runtime behavior/concurrency/atomicity tests supplement the line percentage.

The library is async-runtime-agnostic and owns no listeners, filesystems, subprocesses or native transport handles. Later adapters still own lifecycle, authorization/security policy, real streaming, native status legality and wire/native integration. Shared payloads expose immutable typed access; applications explicitly choose any interior synchronization and native cleanup behavior.

All requested files remain uncommitted for review. No other trial branch was read/diffed/logged/checked out, and no commit, push, publication, deployment, listener or KB publishing occurred. Unrelated files were preserved.


##### Round 3 follow-up

The coordinator independently verified the preceding closeout, then requested a focused readability
pass without behavior changes. This follow-up is complete.

###### Examples, prose and source width

- Formatted all **263 Rustdoc examples** with the installed edition-2024 rustfmt, accounting for the
  enclosing comment prefix so visible source stays within 100 columns. Comma/operator spacing,
  signatures, struct literals, calls, assertions and imports use conventional Rust layout. Existing
  indented examples now use explicit Rust fences; hidden setup remains hidden in rendered docs.
- Spaced JSON macro payloads and added short explanations for the non-obvious steps: explicit Arc
  sharing per call, nested emit reborrowing versus isolated resolve, snapshot isolation, ordered
  visitation, retained cookie scope, and load/output finalization. Named async functions remain the
  primary registration examples; the boxed closure remains the secondary style.
- Wrapped long normal comments and Rustdoc prose. A complete first-party Rust source scan, including
  tests and negative fixtures, reports **zero lines over 100 columns**, down from 406 in the local
  pre-follow-up snapshot. The per-file comment table above now reflects the wrapped source.
- Split five long test fixture strings with `concat!` and expanded two long match assertions. A
  separate exact-content comparison confirmed all five concatenated fixture values are identical
  to their originals, including multipart CRLF, escaped filenames and cookie spacing.
- Repeated the Rustdoc formatter as a read-only check: no formatting differences remained. Inspected
  generated crate, handler and invocation HTML examples to confirm setup markers stay hidden and
  displayed indentation/comments remain readable.

###### Long-function review: retained with justification

| Function | Review result |
| --- | --- |
| `Preparation::inspect` | Retains one recursive path-node sequence: resolve the selected child, reserve its plan slot, account for the leaf or inspect descendants, then determine the ancestor's final shape and key budget while unwinding. Keeping that sequence together makes slot identity and child-before-ancestor ordering visible. Append resolution and key accounting already have named helpers; another split would distribute the coupled immutable-container/plan-position state. |
| `account_keys` | Retains the small representation-transition decision table: object/new-container keys are charged or removed, dense normalization cancels keys, and vector-to-sparse-map conversion materializes index keys. Keeping paired additions/removals together makes their net budget effect auditable. Decimal index sizing remains a separate named helper. |
| `commit` | Retains dense-vector fast path, sparse object fallback, leaf/recursive edit, then planner-proven normalization in one ordered transaction. The owned replacement and returned old value stay visible across those paths, preserving the reviewed destructor/cache boundary in Nest. Further extraction would move exclusive-container and replacement state between helpers without establishing another semantic phase. |
| `parse_args` | Retains one cursor-driven token scan: terminator, lone dash, long option plus optional following value, grouped short flags, then positional/negative numeric data. Cursor advancement, option mode and positional numbering remain together so consumption precedence is explicit; scalar decoding, path parsing and input-budget checks already delegate to named helpers. |

No production declarations or executable code changed. Removing comment-only lines and comparing
rustfmt-canonicalized source against the local pre-follow-up snapshot produced identical hashes for
every production Rust file. This is a documentation/comment pass plus equivalent test formatting;
there is no new behavior or public API decision.

###### Verification after the final edits

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| Separate edition-2024 rustfmt check of all four negative fixtures | Passed. |
| `cargo test --workspace --all-targets --locked --quiet` | Passed: **37 integration tests**, including watchdog/compiler parents; both child entrypoints are invoked/reaped by their parents. |
| `cargo test --workspace --doc --locked --quiet` | Passed: **263 doctests**, 262 foundation + 1 facade. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed without warnings. |
| Source width, repeated example-format and production-code comparison | Passed as described above. |

Coverage was **not rerun**, as requested for a follow-up without production code changes. The latest
independently verified per-crate report remains the preceding **3,080 / 3,314 = 92.94%** for
`stackpress-lib`; the re-export-only facade remains **N/A**. That report predates comment/source-line
relocation and is not presented as a newly generated artifact.

No soundness blocker or owner decision arose. Native target/feature limitations remain those stated
above. No commit, push, publication, listener or other-trial access occurred; unrelated files remain
preserved.
## Acceptance notes

No human-reviewable visual output applies, so this task finishes at `verified`. The owner approved committing the Slice 0+1 work on 2026-10-24.
