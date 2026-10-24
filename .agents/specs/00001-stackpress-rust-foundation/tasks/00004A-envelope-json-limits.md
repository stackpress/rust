# Task 00004A: Envelope JSON conversion respects custom limits

Side quest of [task 00004](00004-lib-usage-and-api-docs.md) in [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). Discovered while documenting the response API, and approved by the owner on 2026-10-24, scoped to this defect only.

## Task summary

`ResponseEnvelope::try_to_json` converts its `errors` and `results` values with `Value::to_json()`, which applies `ParseLimits::default()`. A `Response` created with larger custom limits (`Response::with_limits`) can therefore accept values that its own envelope later rejects during JSON conversion. Round 4 of [task 00003](00003-review-rounds-and-closeout.md) fixed this class of issue for Nest and Response JSON text but missed the envelope path. Make envelope JSON conversion honor the originating response's selected limits, and keep an explicit way to choose a wire budget.

Out of scope, per the owner: the other DX observations recorded in task 00004 (`sent()`/`result()` semantics, setters retaining earlier fields, status-only envelope code 0, optional path reads hiding malformed syntax). Those stay documented current behavior.

## Implementation steps

1. Write a failing regression test first. It builds a response under larger custom limits with a result above the default budget, then shows that the envelope JSON conversion succeeds. It also shows the default budget still rejects the same value, and that a hand-built envelope keeps defined behavior.
2. Fix with a non-breaking API shape by preference. For example, add an explicit-limits conversion (`try_to_json_with(&ParseLimits)`) and make the response-derived paths (`to_envelope`/`into_envelope`/`Resolution` usage) able to convert with the originating limits. Do not add required fields that break struct-literal construction of `ResponseEnvelope` unless no sound non-breaking option exists. In that case, stop and report it as an owner decision.
3. Keep the existing error-path context (`results.profile.native`) and the field-presence semantics.
4. Update the affected Rustdoc and `crates/lib/docs` pages (at least `api/response.md` and any guide or concept text that mentions envelope JSON limits) so the docs match the new behavior.
5. Run scoped review rounds over the changed code: logic, responsibility, consistency/docs/comments, and safety.

## Verification process

`cargo fmt --all -- --check`; `cargo test --workspace --all-targets --locked`; `cargo test --workspace --doc --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked`; per-crate `cargo llvm-cov` at or above 70%; and a link check of `crates/lib/docs`. The coordinator re-runs the checks and reviews the diff.

## Acceptance criteria

Acceptance criteria: none

## Implementation notes

Implemented on `trial/3`, scoped to envelope JSON budgets. No commit, push, listener, context promotion, dependency/manifest change, or inspection of another trial branch was performed. Coordinator verification remains separate below.

### Design and compatibility

- Added `ResponseEnvelope::try_to_json_with(&ParseLimits)`. Both present errors/results values use the selected policy independently through `Value::to_json_with`; the existing path prefixing and field-presence logic are shared unchanged.
- Existing `ResponseEnvelope::try_to_json()` delegates with default limits. This remains the defined behavior for hand-built and detached envelopes. No field was added to ResponseEnvelope or Resolution, and their public struct-literal construction remains intact.
- Added `Response::try_to_json()` for automatic use of the owning response's retained limits, and `Response::limits()` to copy that policy before `to_envelope`/`into_envelope` when later conversion needs it. A detached envelope cannot infer its origin without changing its layout or introducing unsound/hidden global policy. Callers of `Resolution.response` select their wire policy explicitly with the new envelope method; a regression exercises this through a real named async operation and resolve.
- The old `response.to_envelope().try_to_json()` chain deliberately keeps its default-budget meaning. Callers requiring automatic originating limits use `response.try_to_json()`; callers holding a detached envelope use `try_to_json_with`. No origin tracking or wrapper type was introduced.
- Metadata conversion, public errors, delivery state, unsupported rich-value policy, and the hard value-depth ceiling remain unchanged. The selected budget applies to each errors/results Value, not to serialized envelope metadata or aggregate JSON text length. Generic Value serialization's independent default policy is outside this fix.

### Test-first evidence and review rounds

The initial regression used an existing larger-limit response with a nested text result one byte above the default byte budget. Running `cargo test -p stackpress-lib --test request_response_contracts response_envelope_json_respects_selected_limits --locked -- --exact` failed with exit 101, kind Limit, message `value byte limit exceeded`, and path `results.profile.bio`. The test already proved the default response and hand-built/default envelope rejected that same value. Its final success assertion uses the new response-owned conversion path, with additional explicit cloned/moved envelope checks.

Five regression tests now cover automatic originating limits for both errors/results, cloned and consumed envelopes, unchanged default rejection, standalone envelope policy, false/null/zero/empty/absent fields, exact and stricter byte/depth/entry limits, retained limits after replacing a mutable store, explicit limits through Resolution, and complete nested native-value error paths on both direct and detached conversions. Delivery remains Open after successful and failed reads.

1. **Logic/complexity:** baseline implementation passed all 14 request/response tests. Review added both-field origin checks, a replaced-store/tighter-origin-policy case, and a resolved-envelope case. Exact versus tighter byte/node/depth boundaries and unchanged presence/path branches were checked. The expanded request/response suite passed all 16 tests; no production branch rewrite was justified.
2. **Responsibility/abstraction:** reviewed policy ownership and every conversion entrypoint. The default envelope method and response-owned helper delegate to one explicit-policy encoder; extracting another wrapper/helper would add indirection without separating a new responsibility. No-change round; 15 data plus 16 request/response tests passed.
3. **Consistency/Rustdoc/comments:** documented all three additions, default-versus-explicit behavior, policy omission on clone/move, per-value budgets, unused multipart header limits, and preserved diagnostic paths. Added local explanation at the policy-forwarding call and consistent ErrorKind imports in tests. Updated the response API, concepts, and three affected guides; standalone Markdown examples remain unwired to doctests. The focused 16 tests and 265 library doctests passed.
4. **Safety/soundness:** traced borrowed policy/value lifetimes, owned JSON construction, error propagation, SharedNative handling, the existing depth ceiling, and absence of mutation/delivery/retry. Added direct-response native-path and unchanged-state assertions alongside explicit-envelope checks. No production safety change was needed: no unsafe, global origin registry, new sharing, locks, await, or native payload serialization was introduced. All 42 workspace integration tests passed, including watchdog/compiler parents and five regressions. No material in-scope finding or soundness blocker remains.

### Final self-checks

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `cargo test --workspace --all-targets --locked --quiet` | Passed: 42 integration tests; two child entrypoints intentionally ignored in the parent listing and invoked by watchdog parents. |
| `cargo test --workspace --doc --locked --quiet` | Passed: 266 doctests, 265 foundation plus one facade. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed without warnings. |
| `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked` | Passed without warnings. |
| Read-only docs link/anchor check | Passed: 196 relative links/anchors across all 22 pages. |
| Disposable Markdown consumer | Passed: 47 response reference blocks and three updated guide walkthroughs, outside the repository. |
| `git diff --check` | Passed. |

### Fresh coverage

Toolchain: rustc/Cargo 1.96.0, LLVM 22.1.2, cargo-llvm-cov 0.8.6; native `aarch64-apple-darwin`, no feature flags. Each collection started with `cargo llvm-cov clean --workspace` so earlier profile data was not reused.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov --workspace --all-targets --locked \
  --ignore-filename-regex '(/tests/|/examples/)' \
  --json --output-path target/coverage/task-00004a-workspace.json --fail-under-lines 70
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-lib --all-targets --locked \
  --ignore-filename-regex '(/tests/|/examples/)' \
  --json --output-path target/coverage/task-00004a-lib.json --fail-under-lines 70
```

| Measured scope | Covered / executable lines | Coverage / floor |
| --- | --- | --- |
| `stackpress-lib` independently | 3,090 / 3,329 | 92.82%; passed ≥70%. |
| Workspace integration, all executable production lines in `stackpress-lib` | 3,101 / 3,329 | 93.15%; passed ≥70%. |
| `stackpress` facade | No executable lines; re-exports only. | N/A, not 100%; facade integration test and doctest passed. |

Both JSON reports contain only the 16 first-party foundation source files. No production file or production branch was excluded; the receipt's tests/examples exclusion was retained. Stable doctest coverage and a branch percentage are not claimed. Remaining unmeasured lines are existing alternative/debug/exhaustion paths across the crate; the targeted policy, presence, and error branches are covered by behavioral assertions. Other platforms, host adapters, and publication readiness are not inferred from this native fix.

No owner decision is outstanding. Verification notes and task status are left for the coordinator.

## Verification notes

Coordinator review on 2026-10-24. Result: **verified**.

- **Diff scope:**
  - Production change is limited to `crates/lib/src/response.rs`, with three additive public methods: `ResponseEnvelope::try_to_json_with(&ParseLimits)`, `Response::limits()`, and `Response::try_to_json()`.
  - Existing `ResponseEnvelope::try_to_json()` now delegates with default limits, unchanged in meaning.
  - No field was added to `ResponseEnvelope` or `Resolution`, so struct-literal construction is preserved.
  - No out-of-scope API (`sent`, `result`, setters, `status_code`, `redirect`) was modified.
  - Manifests and `Cargo.lock` are unchanged.
- **Defect closure:** `resolve` builds its response with `Response::new()` (default limits), so its envelopes stay consistent under the default conversion. Responses created with custom limits now convert through `Response::try_to_json()` using their own policy. Detached envelopes select a policy explicitly with `try_to_json_with`. The remaining `response.to_envelope().try_to_json()` default-budget behavior is deliberate for non-breaking compatibility, and it is documented in `api/response.md` and the requests/responses guide.
- **Checks rerun independently:**
  - fmt passed; 42 integration tests passed, 0 failed (5 new regressions); 266 doctests passed; Clippy and Rustdoc warning-free.
  - `cargo llvm-cov` after `cargo llvm-cov clean --workspace` reported workspace coverage of `stackpress-lib` at 93.15% (3,329 executable lines, 228 missed), matching the receipt.
  - All relative links in `crates/lib/docs` resolve.
- **Not rerun by the coordinator:** the implementer's disposable out-of-repository Markdown consumer.

## Acceptance notes

No visual acceptance artifact applies; this task finishes at `verified` after coordinator review.
