# Task 00010 lane 1: lib owner-review issue receipt

Implementation evidence for [task 00010](00010-owner-review-issues.md), issues [#11](https://github.com/stackpress/rust/issues/11), [#1](https://github.com/stackpress/rust/issues/1), [#2](https://github.com/stackpress/rust/issues/2), [#5](https://github.com/stackpress/rust/issues/5), and lib portions of [#4](https://github.com/stackpress/rust/issues/4) and [#7](https://github.com/stackpress/rust/issues/7). Coordinator review and task status remain separate.

## Scope and baseline

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-issues-lib`.
- Branch: `trial/3-issues-lib`; starting HEAD `9c0929ddd89bf42986e32b81197499cf3438c3f8`.
- Starting worktree was clean. Read all six authoritative GitHub issues with `gh issue view <n> --repo stackpress/rust`; their old merge guard is superseded by task 00010.
- Implementation, tests and crate documentation are confined to `crates/lib/**`; this receipt is the separately approved `.agents` write. No caller compile fixes were required.
- Compiler/host: Rust/Cargo 1.96.0, `aarch64-apple-darwin`; workspace default feature selection. No publication, commit, push, issue comments/closure, listeners, or agent messages.
- Ran `cargo clean` (no previous target output) and `cargo llvm-cov clean --workspace` before building. Debug info and incremental compilation were disabled to limit disk use.

## #11 error-wrapping policy for lane 2

At each wrapping boundary, choose developer Display text describing that operation's distinct context, retain the concrete original cause with `with_cause`, and leave its underlying explanation to `source()`. Do not interpolate source Display into the wrapper or remove causes to suppress duplication. Prefer propagating an existing framework error with `?` when no new context is needed; this retains its origin, invocation frames, path and disclosure choice. Generic `Error::from_source` now displays `external operation failed`. Concrete `From` conversions display `JSON operation failed`, `I/O operation failed`, or `integer parsing failed`. Known operations should use `Error::new(context).with_kind(ErrorKind::External).with_cause(cause)` and select status/public disclosure explicitly. Developer formatters print the wrapper and each source once; public output continues through `public_view`, excluding private causes, origins and frames.

This is the explicitly approved Display/message change: previously `from_source` copied the cause's Display. Rustdoc and `docs/api/error.md` document the compatibility change. Concrete causes, downcasting, native capture, tracked origins, inner invocation frames and default public `Internal Server Error` are retained.

Lib caller audit:

- `data/parsers.rs`: malformed JSON adds `JSON input could not be parsed`, retaining External, code 400 and the typed serde_json cause.
- `pattern.rs`: compilation adds `event pattern could not be compiled`, retaining Registration classification and regex cause.
- `request.rs`: both URL parsing branches use `request URL could not be parsed`, retaining the typed URL error.
- `error.rs`: all three concrete conversions retain their concrete cause and tracked caller; generic conversion remains available.
- `data/json.rs`, `data/nest.rs`, `request.rs` deferred serialization, and `response.rs` JSON helpers use the contextual JSON conversion without copying cause text. Existing response `with_cause` boundaries already add separate context; retained errors are not reconstructed.
- Lib tests' JoinError/source conversions remain generic; they do not assert the old Display spelling.

Out-of-lane follow-up inventory (not edited):

- Lane 2: `crates/idea/idea-transformer/src/lib.rs` IdeaError conversion still copies its source Display; change it to phase context. `tools/cli/src/diagnostics.rs::developer` already prints each source once and must retain bounded traversal. CLI `entry.rs`, `execution.rs` and diagnostic write/flush paths, server `context.rs` and `http.rs` use generic `from_source`; they can choose operation context under this policy. Server `config.rs` source-retention work belongs to #10. Prompt cancellation tests must retain concrete PromptError downcasting.
- Other generic lib-error callers: `crates/framework/framework-view/src/server.rs`, SQL SQLite examples/tests, `examples/sql-profile/src/{database,profiles}.rs`, `examples/profile-ui/{web/src/lib.rs,gateway/src/assets.rs}`, generated-profile codegen, and capability-host tests. These now receive the approved generic message while their causes/public defaults remain intact. Separate `ViewError::from_source` APIs are not the lib constructor.

## #1 priority queue

`ItemQueue` now owns a sorted `VecDeque`, uses a binary upper bound to insert after equal priorities, and makes storage contiguous after insertion. `entries() -> &[Item<T>]` remains in execution order without copying. `pop_front` advances the head and cannot fragment an already contiguous remainder.

Complexity: O(log n) priority comparisons and O(n) worst-case insertion movement/contiguity work; O(1) pop; O(n) complete drain; O(1) slice inspection. This favors registration/startup queue inspection and draining without adding a second ordered index or changing the slice API. It deliberately does not claim heap-like insertion complexity.

Tests cover descending priority, FIFO ties, mixed insertion/removal with 400 operations and slice comparisons, growth/head movement, front/back insertion after a full drain, retained extrema, checked priority arithmetic, and synthetic sequence exhaustion without state changes. Existing sequential/cooperative task tests remain green.

## #2 borrowed Serde adapters

`data/json.rs` owns one private checked JSON view shared by Value serialization, Nest serialization, and typed `path_as`. Whole-tree budget validation runs first with the original traversal/error precedence, then compatibility preflight runs in wire order. No serializer/target visitor is called until both succeed. Literal empty diagnostic components retain the original root-to-leaf joining rules. Serializers borrow values/keys and never build the duplicate JSON tree. Their own later IO/custom failures can still produce partial output, as before.

Typed reads borrow ordinary maps/arrays and create owned scalar outputs only when requested. Unknown/ignored fields are still prevalidated. Missing stays None, while present null is passed to the target. Numeric/boolean map-key coercions delegate through small single-key scratch maps to preserve serde_json's private parsing and exact errors; ordinary string keys avoid these maps. Enum and newtype decoding deliberately retain an owned-subtree fallback to preserve every variant and dependency-specific newtype protocol. This is not a claim that every target deserialization is allocation-free.

Compatibility tests compare against the original owned-JSON decoder across scalars, signed/unsigned/128-bit numbers, floats, text/characters, null/unit, arrays/tuples/structs, byte targets, optional/newtype/enum keys and invalid numeric key spellings. They also cover insertion-order wire bytes, unsupported dates/bytes/files/native values, nested paths, budgets before unsupported values, ignored fields before target invocation, selected limits, and empty literal keys.

### Focused allocation evidence

Standalone single-thread counting-System-allocator probe, using the final uninstrumented lib build and serde_json 1.0.154. Input construction is outside the measured interval. Each input has 500 strings of 1,024 bytes. Alloc/realloc calls and cumulative requested bytes include output allocation; they are not peak-live-memory figures or latency claims. The old path is explicitly `to_json` followed by Serde. Probe source is `/private/tmp/stackpress-task10-allocation.rs`; it is disposable measurement code outside production and uses unsafe only to forward GlobalAlloc to System. Production code adds no unsafe or dependency.

| Operation/input | Old calls | New calls | Old allocated bytes | New allocated bytes |
| --- | ---: | ---: | ---: | ---: |
| Serialize array | 2,528 | 1,021 | 1,681,566 | 1,093,410 |
| Serialize object | 2,038 | 521 | 1,825,340 | 1,102,204 |
| Typed array selection | 2,518 | 1,514 | 643,840 | 567,882 |
| Typed object selection | 2,029 | 1,515 | 804,654 | 597,106 |

An earlier first-pass adapter used scratch maps for every object key (2,514 calls for typed object selection); round two removed that overhead for ordinary keys. A separate unoptimized 100-round timing sample showed 1.184 s owned versus 1.249 s direct serialization and 27.1 ms versus 19.6 ms typed array selection. That noisy sample does not establish a serialization speedup; allocation reduction is the selected evidence. `json_adapter_performance_receipt` is an explicitly ignored, assertion-free timing probe, not a flaky latency gate.

## #5, #4 and #7

- Private `ContainerBefore::{Missing,Object,Array}` and `MembershipChange::{Unchanged,Added,Removed}` replace contradictory boolean bundles in mutation accounting. Independent operation/final-shape flags remain booleans. A shared key-membership adjustment preserves the original object/array transition phase order. Existing full-validation budget-oracle histories, sparse deletion, append overflow, native-drop unwind and atomic-rejection tests remain in place.
- `#[must_use]` covers RequestBuilder and ListenerSpec types, consuming Error enrichment methods, and the consuming emitter build result. Mutable registration/setter APIs remain discardable when their side effect is useful. Chaining remains unchanged.
- Public fallible lib APIs were reviewed with opt-in `missing_errors_doc` and direct source review. Added meaningful error precedence, retained-state/atomicity, consumed-builder retry, loader/writer cancellation/partial side effects, sparse-removal budget growth, selected-value preflight, import failure and cookie-codec retry contracts. No runtime changes were made for #7.

## Regression evidence and review rounds

1. #11: first ran `cargo test -p stackpress-lib --test error_contracts --locked` before changing error code. Both tests failed: generic Display was `disk unavailable`, and the three-layer parser chain repeated its explanation. Both passed after contextual wrapping; later tests also verify lib wrapper causes and invocation metadata/disclosure.
2. #4: added a `compile_fail` doctest with `#![deny(unused_must_use)]` and discarded `Request::builder().method("POST")`. It failed before annotation because the snippet compiled successfully; it passes after annotation. Ordinary builder examples still compile.
3. #1/#2/#5 are performance/private refactors: the initial prevalidation, wire-output and typed-selection compatibility tests passed on the original paths before adapter changes; their observable semantics are preserved. Existing mutation budget-oracle tests and task execution tests were rerun throughout.
4. Round 1, complexity/branch risk: reviewed queue contiguity/arithmetic, mutation transitions, budget-versus-variant precedence and diagnostic ordering. An added empty-literal-key test failed (`.x.leaf` versus the required `x.leaf`); corrected preflight to retain borrowed path components and join only on error. Unit/data/event/serialization suites passed afterward.
5. Round 2, responsibility/abstraction: centralized checked Serde ownership, isolated JSON key rules without duplicating serde_json's parser, specialized ordinary key decoding after measurement, and extracted shared membership charging without changing checked-arithmetic order. Unit/data/serialization suites passed afterward.
6. Round 3, consistency/Rustdoc/comments: audited consuming APIs and public errors, documented Display compatibility and queue complexity, refreshed affected crate references/guides, and reviewed inline state/serialization explanations. Final checks below verify this result.
7. Safety/soundness: reviewed finite preflight before bounded recursion, owned scalar versus borrowed-container lifetimes, key/value cursor sequencing, enum/newtype fallback bounds, deque slice invariants, cause/downcast ownership, atomic mutation plans, and one-attempt native boundaries. No unsafe production code, new dependency, public signature change, or additional adapter capability was introduced. Existing concurrency/IO/process tests use their existing watchdog parents.

During verification, two new test-harness compile/lint issues (EventPattern has no Debug for unwrap_err, and a byte visitor needed visit_str alongside visit_string) were corrected; final checks were restarted afterward. These were test-code issues, not unresolved production failures.

## Final verification

All commands passed after the final source/test changes, using `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`:

- `cargo fmt --all -- --check`.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- `cargo clippy -p stackpress-lib --all-targets --locked -- -D warnings -D clippy::missing_errors_doc -D clippy::return_self_not_must_use`.
- `cargo test --workspace --all-targets --locked`: 454 passed, zero failures, 53 ignored fixture/probe entries across 98 suites. Watchdog children are explicitly invoked by their parent tests; the timing probe is intentionally opt-in.
- `cargo test --workspace --doc --locked`: 856 passed, zero failures, across 24 result groups, including the discarded-builder compile-fail example.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p stackpress-lib --no-deps --locked`.
- `cargo +1.96.0 check --workspace --all-targets --locked`.
- `git diff --check`: passed. Crate documentation relative-link scan: zero broken links.

Command logs are `/private/tmp/stackpress-task10-{workspace-clippy,lib-opt-in-clippy,workspace-tests,workspace-doctests,lib-rustdoc,msrv-check}.log`. The first workspace verification attempt stopped on the test-code issues noted above; these paths hold the successful rerun. No out-of-lane compiler/test fixes were needed.

After writing this receipt, `python3 .agents/scripts/validate-agent-workspace.py` exited 1 with 146 missing sibling-source link errors and 21 warnings in unchanged KB files. The isolated worktree lacks the sibling TypeScript repositories those historical links target (for example `context/source-map.md` resolves to `../lib/src/index.ts`). There are no findings naming this receipt; it is within the line cap and its task link resolves. Log: `/private/tmp/stackpress-task10-kb-validation.log`. Repairing unrelated historical links is outside this lane; this KB validation limitation does not change the passing Cargo verification above.

## Coverage and limits

`cargo-llvm-cov 0.8.6`, rustc `1.96.0 (ac68faa20 2026-05-25)`, native `aarch64-apple-darwin`, default lib features. Command: `CARGO_LLVM_COV_TARGET_DIR=target CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 cargo llvm-cov -p stackpress-lib --all-targets --locked --fail-under-lines 70 --json --summary-only --output-path /private/tmp/stackpress-task10-lib-coverage.json`. Passed: 54 lib tests, zero failures; five opt-in fixture/probe entries, with watchdog children executed by their parents.

Tool line summary: **3,457 / 3,712 = 93.13%**, above the 70% development floor. Reports contain only the 18 lib implementation source files, excluding separate integration tests, dependencies and other workspace crates. This standard source-file summary includes inline unit-test code in error.rs and queue.rs.

A separate production-only audit of LCOV physical `DA` source-line records excludes exactly the two inline test blocks: `error.rs:598-632` and `queue.rs:17-34`. It reports **3,298 / 3,522 = 93.64%**. For transparency, the raw LCOV physical-line records are 3,335 / 3,559; their denominator differs from LLVM summary `LF`/`LH` counters. The full original summary and LCOV files are preserved, without any production-code exclusion. The audit script is `/private/tmp/stackpress-task10-production-coverage.py`; it asserts the 70% floor.

New adapter source-file coverage: json.rs 213/243 (87.65%), json/key.rs 37/41 (90.24%). Mutation 227/233 (97.42%), queue 145/151 (96.03% tool summary). Branch coverage and doctest inclusion were not enabled; doctests were verified separately. This is development evidence, not permission or sufficient evidence to publish.

Default-feature native tests do not establish WASM/browser, packaged Tauri/mobile, every optional feature combination or release readiness. Enum/newtype owned-subtree and coerced-key scratch-map fallbacks remain intentional. Latency and production workload scaling were not established by the allocation proof.

## Cleanup and handoff

Deleted the owned worktree `target/` directory (about 1.8 GiB) after preserving verification reports/logs under `/private/tmp`; verified `target/` is absent. Cargo commands and coverage used this one target root (`CARGO_LLVM_COV_TARGET_DIR=target` for coverage), rather than a second coverage target. Existing consumer tests manage their own disposable fixture targets and cleanup. Staging remains empty and HEAD is unchanged. All implementation changes are still uncommitted for coordinator review.

No commit or acceptance is claimed. The coordinator should review this lane, adopt the #11 policy in lane 2, integrate the disjoint changes, and rerun the task-wide gates.
