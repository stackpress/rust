# Design: risk inventory and verification plan

Spec file of [00001 Stackpress Rust foundation](index.md). Load for the Slice 1 risk inventory with required proofs and the milestone/watchdog/review-round/coverage/receipt plan (design sections C and E). Section D (decisions applied) moved to [decisions](decisions.md). Moved losslessly from `docs/trial-3/slice-1-design.md` lines 580–616 and 639–719 at commit `e230359`; headings are demoted one level and links rebased.

### C. Risk inventory and proofs

These are design risks I would review even without the prior-attempt failure list. The engine can exclude internal lock/reentrancy bugs; it cannot guarantee arbitrary user callbacks terminate or are transactionally safe.

| Risk | Prevention / explicit limit | Required proof |
| --- | --- | --- |
| Registry lock held across await; reentrant deadlock | Immutable built table; no registry/handler mutex, no executor blocking | Parent→child→grandchild and same-event bounded recursion under watchdog; inspect lock-free execution path. |
| Concurrent nested dispatch corrupts global current event | Immutable per-dispatch/per-listener paths/captures; no task/thread-local mutable current slot | Barrier-controlled concurrent roots and concurrent siblings with different req/res; parent metadata correct after success/error/abort. |
| Framework lock poisoning after panic | No poisonable production registry/data locks | Panic/unwind in child process followed by a new independent dispatch; audit all lock/unwrap sites; do not claim user-lock recovery. |
| Clone silently aliases mutable state | Value/Nest clone ordinary data deeply; only Value::Shared(SharedNative) aliases. Request/Response/engine/queues/error are not Clone. | Mutate cloned nested objects/arrays/bytes/files independently; Shared retains typed identity; no mutable payload accessor; compile-fail Clone on stateful owners. |
| Unsound borrowed futures/reentrancy | Safe lifetime-bound erasure; shared callable contract, exclusive req/res borrows | Named and borrowing async closures compile; same req/res concurrent emit and escaping Invocation compile-fail; forbid unsafe in first-party crates. |
| Mistaking Send handler storage for Send futures | Require each lifetime-indexed returned future itself to be Send, then erase to a Send boxed future | Named/helper nested dispatch, positive future Send assertion and multithreaded tokio::spawn execution; non-Send future rejected by the same registration bound. |
| Handler mutable capture races | Shared Fn callbacks with lifetime-indexed Send futures; no shared FnMut wrapper | Immutable captures and explicitly synchronized payloads; FnMut-only and non-Send callbacks rejected; concurrent invocations share no ordinary framework mutation. |
| Tie nondeterminism or arithmetic overflow | Signed priority compare then registration sequence; checked counters/front/back extremes | Cross-kind equal priorities, negative/min/max priorities, duplicate callback, compose/remove/readd order and counter exhaustion unit path. |
| Composition or customization corrupts identity/order | Move typed matcher/metadata; explicit handle replacement; receiver sequence separate from ID | Preserve exact/pattern/regex/custom-separator matches across extend, original/replacement sites, foreign/stale handle rejection, replacement keeps position, no stale matches after clear. |
| Literal regex punctuation accidentally broadens a pattern | Typed exact vs Express vs regex; escape all literal Express fragments | `. + ( ) [ ] $ ^ \\` literals, malformed token forms, separator metacharacters, duplicate params. |
| Capture precedence or leakage between listeners | Captures only in per-listener context; no automatic req.data mutation | Conflicting parameter names/values across all-match listeners and nested dispatch, equivalent regex with different param labels, duplicate registrations. |
| Stateful/global regex repeats inconsistently | Immutable regex first captures, no global cursor | Repeated emits, optional/named groups, empty matches, case sensitivity and rejected unsupported JS regex. |
| Deep recursion, oversized input or huge sparse allocation | Dispatch/data/parser depth, bytes/entry/header/regex-size limits; checked indices and sparse objects | Boundary and limit+1 cases, recursive event cycle, huge index, deeply nested JSON/brackets, oversized multipart headers/file. |
| Inconsistent before/action/after stop/error behavior | One documented phase sequence; after not finally | Decision table for Continue/Abort/Err in each phase, zero/multiple listeners, child abort handled/propagated. |
| Cancellation replays an external side effect | No implicit task/load/delivery retry; visible attempt state; partial mutations documented | Drop futures while suspended before/inside/after handler and during load/delivery; unstarted phases never run; second attempt rejected. |
| Double send or mutating already emitted output | Exclusive Response borrow and delivery state guards | Successful repeated dispatch no-op, failure/cancel no retry, stop without output, post-finalization writes rejected. |
| Partial load or parse mutation after errors | Stage results before applying; isolated owned loader/dispatcher snapshots without attempt state; validate chosen budgets at commit/output | Fault after valid prefix, invalid post result, loader error/cancel; snapshot edits do not mutate original data/body; no retry after interruption. |
| Silent missing/null/false/0 loss | Option tracks presence; Value variants and explicit envelope option fields | Query/post/data/load precedence with null/false/zero/empty, envelope round trips including explicit null and total=0. |
| Dates/bytes/shared native collapse into JSON; sharing becomes implicit | Rich Value variants, explicitly named SharedNative and explicit wire conversion | Distinguish null/string/date/bytes/file/Shared, typed downcast success/mismatch, clones alias only Shared, native survives resolve, serialization errors name the path, payload drops after the final Shared owner. |
| Resolve loses outcomes or starts general lifecycle recursively | Neutral plain-data/owned-request input adapter, fresh response and inherited child path, explicit Resolution wrapper | Named/closure operations via resolve, code/status/results/errors/total/stack field presence, NoListener/Aborted/Err distinct, concurrent nested resolve with isolated values, no automatic request/response/error event. |
| Numeric keys normalized in lexicographic order; deletes shift identities | Canonical numeric ordering distinct from explicit sorted-key helper; sparse objects | 0..12 array, insertion order variations, 01/-1/large keys, append after gaps and deletion without shifting. |
| Args/query delimiter injection or double decode | Parse argv directly; decode query once | Values containing `&`, `=`, `+`, `%`, Unicode, quoted empty values, malformed escapes and percent-encoded percent signs. |
| Multipart truncation/binary corruption | Byte parser, explicit boundary, delimiter-position/termination checks and limits | Binary bytes including NUL/CRLF, boundary-like file content, zero-byte file, quoted boundary, missing final delimiter, malformed headers/dispositions. |
| Header/cookie injection and wrong cookie deletion scope | Typed/validated values/options and multi-value headers; scope in tombstone | CR/LF names/values/options, custom codec output, repeated Set-Cookie, case variants, nondefault-path/domain delete/clear. |
| Session mistaken for identity/authorization | Pure cookie state, no implicit trust decision | Docs and API review; no automatic authorized state from a cookie or event name. Access-surface enforcement belongs to Slice 2+. |
| Lost source/cause after nested await; leaking diagnostics | Owned source chain, immutable path, separate public and developer views | Named/anonymous async failure, nested propagation and simultaneous failures; public JSON excludes file/site/secret/cause/backtrace. |
| Panic/failed callback assumed rollback-safe | No recovery promise or automatic replay; invocation values remain caller-owned | Unwind/cancellation receipt shows partial state honestly; fresh invocation still works; no unwinding catch in engine. |
| Request URL/method used as transport authority | Value parsing only, explicit Any matcher, no forwarded-header trust | Extension/invalid verbs, literal URL helper cases and route method separation. Adapter security remains unproven. |
| Neutral foundation acquires native/runtime dependencies | Two real packages only, dependency graph review | cargo tree/metadata review; direct-lib/facade consumers compile without Tokio/Tauri/UI/network/filesystem production dependencies. |
| False MSRV/target/coverage claim | Actual toolchain/target checks and per-crate reports; no stale artifacts | Clean build before testing, installed 1.96 baseline verification, separate coverage per changed crate and explicit untested targets. |

### E. Test and verification plan

#### E1. Approved milestone order and review stops

Each milestone must have passing relevant tests before the next begins:

1. **M0 — scaffold/clean/signature proof.** Create only the two real packages and root facade. Before the first build run `cargo clean` and `cargo llvm-cov clean --workspace`, retaining receipts. Prove the exact named/helper three-argument handler signature, returned-future Send bound, nested emit and tokio::spawn with private request/response proof fixtures if necessary; keep fixture limitations explicit rather than presenting a partial value API as finished. **Stop and report after M0.**
2. **M1 — data.** After the coordinator's reply: Value/Nest/paths/ordered collections/parsers/files and clone/Shared/serialization proofs.
3. **M2 — queue/events/status/diagnostics.** Integrate the proved handler mechanism into the real emitter and exercise priority/ties, matching, hooks, handles/composition, recursion, concurrency and useful failures.
4. **M3 — request/response/session/cookie/resolve/facade.** Prove the real value, capability-call, typed Serialize, cookie, attempt-state and direct/facade consumer contracts. **Stop and report after M3, before refactoring rounds, with tests and current coverage.** Continue M1–M3 without intermediate approval stops after M0 is released.
5. After the M3 review, run all four E3 rounds and close with E5. The process stops do not waive the final development coverage gate.

Recheck branch/worktree and preserve unrelated edits throughout. No other trial branches, listener, commit/push or publication. Crates.io fetching for listed dependencies is authorized; sandbox network approval is requested normally if required. Test observable behavior and the A3/C failure classes rather than private implementation shape. The relevant 00006 checklist proof items 2–4 and 15 remain explicit; host/UI/generation proofs are not inferred from a foundation milestone.

#### E2. Deadlock-class tests must terminate

Use both async deadlines and an **external process watchdog**. An executor timer alone cannot interrupt a callback that blocks its executor thread on a synchronous lock or never yields.

- Ordinary async ordering/concurrency tests use barrier/oneshot coordination rather than timing guesses, with a 2-second per-scenario timeout on a test executor.
- Reentrant/concurrent nested, same-event recursion, unwind/no-poison and cancellation probes run in a dedicated test subprocess. A parent test uses `try_wait` with a short bounded polling interval and a generous fixed deadline (proposed 10 seconds), kills **and reaps** a stuck child, and reports the scenario as failure with captured bounded output. Never join a possibly deadlocked worker thread as the only timeout strategy.
- Include a deliberately nonterminating helper in a test-only watchdog self-test so the kill/failure mechanism is itself demonstrated. Child selection uses an exact marker/argument and cannot recursively launch its parent.
- Run no network/listener and leave no child/executor/background work alive after success or failure. For legitimate slower instrumented runs, set an explicit larger bounded watchdog deadline and record it; do not remove deadlines.

#### E3. Required review rounds

After the baseline meaningful suite passes, record distinct rounds with actual findings and relevant tests rerun after each:

1. **Logic/complexity:** phase/abort/error/cancel precedence, matching, input normalization, mutation atomicity, branch-risk tables and uncovered meaningful paths.
2. **Responsibility/abstraction:** separate matcher compilation, registry construction, invocation execution, values/parsing, delivery state and diagnostics; review borrow/ownership/coupling and remove unnecessary wrappers.
3. **Consistency/Rustdoc/comments:** consistent Result/Flow/state vocabulary, idiomatic names/imports, explicit public API, real standalone compiling examples for every public entry point, `//!` module docs, `///` on every function/method including meaningful private helpers, field/invariant docs, `# Errors`/`# Panics` where applicable, purposeful explanations per logical block and detailed branching/state/async comments. Use conventional `// ` spacing. Apply [00036](../../references/00036-rustdoc-commenting-examples.md)/[00037](../../references/00037-refactoring-round-checklists.md); JS no-space comments, JSDoc/tag restrictions, class-per-file and JS import grouping are not Rust rules.
4. **Dedicated safety/soundness:** independently trace callback lifetimes/erasure, mutable alias prevention, concurrency/recursion, cancellation/drop, panic behavior, source-chain ownership, integer/length bounds, dependency features and absence of unsafe/lock-over-await. Rerun the concurrency/watchdog/compile-fail/error suites plus affected tests.

Repeat targeted rounds for material findings. A justified no-change round records scope, conclusion and rerun results; no ceremonial edits or skipped verification. The standing instruction authorizes these routine rounds once implementation is approved.

#### E4. Final commands, coverage and target limits

Planned commands once code/manifests exist (adapt only to actual compatible features):

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets
cargo test --workspace --doc
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
cargo llvm-cov --workspace --lcov --output-path target/coverage/workspace.lcov
cargo llvm-cov -p stackpress-lib --fail-under-lines 70
cargo llvm-cov -p stackpress --fail-under-lines 70  # only if the facade has executable lines
```

Use isolated/clean profile collection per coverage command as needed so previous measurements cannot contaminate results. Preserve text/LCOV reports and assess each changed crate separately, plus workspace integration. Record compiler, target, features, tool version, covered/total executable lines and justified exclusions. Doctests run separately; do not silently rely on unstable/nightly doctest coverage. Dependencies/tests/fixtures/build output are not production coverage. No broad source exclusions or artificial executable facade code to manufacture a denominator.

The facade may be entirely re-exports and have **no executable lines**. Report that as N/A/unmeasurable, not 100%; its proof is real import/example/doctest/consumer compilation. If executable facade code exists, its measured scope must independently reach 70%. `stackpress-lib` must reach at least **70% executable line coverage** for development, and critical risk tests remain mandatory at any percentage. Publication preparation would require 90% per published crate and a separately authorized release scope.

Verify `cargo +1.96.0 check --workspace --lib --locked` and relevant tests/doctests using the installed 1.96 compiler. Install no other toolchain and claim no 1.85 support. Inspect production dependency trees for the facade and direct-lib paths. Initial verification target is native aarch64 Apple Darwin. Other OS/architectures, WASM, Tauri and real server-adapter integration remain unverified; a Send foundation future alone does not prove those integrations.

#### E5. Closeout receipt

The implementation receipt will list implemented/deferred mappings, deliberate deviations approved in D, baseline tests, each review round and tests rerun, final format/lint/doc results, per-crate coverage evidence, MSRV/target/feature scope, and unresolved limitations. Passing those checks is code evidence, not owner acceptance, publication, deployment, commit or push.

Use [00037's receipt template](../../references/00037-refactoring-round-checklists.md), extended with the dedicated safety round:

```text
Task / crate / feature / target:
Intended behavior and approved design decisions:
Baseline tests and result:
Round 1 logic/complexity — findings, changes/no-change reason, tests/result:
Round 2 responsibility/abstraction — findings, changes/no-change reason, tests/result:
Round 3 consistency/Rustdoc/comments — findings, changes/no-change reason, checks/result:
Round 4 safety/soundness — findings, changes/no-change reason, watchdog/compile tests/result:
Follow-up rounds:
Coverage tool / command / crate / features / target:
Covered lines / total lines / result / applicable 70% floor:
Excluded or unmeasured code and rationale:
MSRV result and exact toolchain:
Remaining branch/adapter/platform risks:
Unresolved user decisions:
```

**Current status:** M0 and M3 were independently reviewed and approved. All four post-M3 rounds are complete; see the [final Slice 1 receipt](tasks/00003-review-rounds-and-closeout.md) for findings, current checks/coverage and limitations. Earlier [M0](tasks/00001-scaffold-and-send-signature-proof.md) and [M1–M3](tasks/00002-foundation-data-events-and-values.md) receipts retain their historical checkpoint scope.

The final naming scheme uses literal `get/has/set/remove` and path `path/path_as/has_path/set_path/remove_path`, accepting either dot strings or reusable DataPath through sealed IntoPath. Standard Method/MethodPattern constants and Invocation::param cover common route/capture calls. DataPath Display quotes literal punctuation/numeric keys. Selected Nest JSON conversion and Response JSON text conversion honor their chosen budgets; Value's default wire conversion remains finite, with to_json_with for an explicit policy.