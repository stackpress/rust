# Task 00010 lane 2 receipt: server, CLI and transformer

Evidence for [owner review task 00010](00010-owner-review-issues.md), implemented on 2026-10-26. Coordinator review and the final cross-lane #11 policy consistency pass remain separate from this receipt.

## Scope and baseline

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-issues-host`.
- Branch: `trial/3-issues-host`; base/HEAD: `9c0929ddd89bf42986e32b81197499cf3438c3f8`.
- Initial worktree was clean. Changes are confined to `crates/server/`, `tools/cli/`, `crates/idea/idea-transformer/`, and this explicitly approved receipt.
- Read each assigned GitHub issue in full with `gh issue view <n> --repo stackpress/rust`. Task 00010 explicitly authorizes bypassing the issues' original merge-timing guard.
- No dependency/lockfile change, commit, push, GitHub issue comment/closure, listener, or KB publishing operation.

## Implemented behavior

| Issue | Implementation and compatibility |
| --- | --- |
| [#10](https://github.com/stackpress/rust/issues/10) | Configuration JSON parsing and default serialization keep their original serde causes through existing `with_cause`. Display and public snapshots retain sanitized context; rejected layers are unchanged. Analogous host/CLI/transformer wrapping sites were audited. Manifest serialization/parse failures and lock contention now retain their causes too. |
| [#3](https://github.com/stackpress/rust/issues/3) | Additive public `TransformErrorKind` is Copy/Eq and non-exhaustive, exposed by `kind()`. Categories cover Other, Io, Parse, LockExists, RecoveryRequired, Ownership, Selection, Limit and Callback. Existing constructors/signatures stay available. Parser and JSON visitor budgets, acquisition limits, and nested install/import wrappers retain classification without matching message text. Contextual callbacks wrap their original error; legacy callbacks preserve existing Display/source behavior and acquire Callback kind. |
| [#8](https://github.com/stackpress/rust/issues/8) | Bootstrap acquisition/validation runs on Tokio's blocking pool before the FnOnce builder. Pre-open regular-file and post-open handle checks reject special files; source symlinks remain followed. Existing bytes/UTF-8/JSON checks and exit/no-builder-on-invalid policy are preserved. Join failures retain their source. |
| [#9](https://github.com/stackpress/rust/issues/9) | `read_source` and ProjectLoader share a native regular-file opener with pre/post checks. Finite detection-byte reads, source symlinks and UTF-8 behavior remain supported; IO and encoding causes are retained. |
| [#13](https://github.com/stackpress/rust/issues/13) | Supervised formatter drains stdout/stderr concurrently with stdin writing. Each stream is capped before retaining a chunk; limit, timeout, IO and unwind paths kill/reap the child before scoped workers join. Invalid UTF-8/nonzero exit remain failures, and formatting still precedes installation/locking. |
| [#12](https://github.com/stackpress/rust/issues/12) | A prestarted cleanup owner receives and joins the exact TTY reader. Drop signals stop without waiting for native work. The reader retains the terminal lease through restoration. Explicit close awaits the join result, preserves its receiver across cancellation, reports failures and remains idempotent. |
| [#4](https://github.com/stackpress/rust/issues/4) | Type-level must_use covers PluginSpec, RouteSpec, StartupSpec, PromptOptions, TransformSpec, TransformSelection and Composer. Mutating registration/writer APIs remain valid when their return is discarded. A compile-fail doctest diagnoses a discarded PluginSpec chain. |
| [#6](https://github.com/stackpress/rust/issues/6) | Private installation planning/test helpers borrow OutputPlan. Formatter pipe work borrows source text. Public installation intentionally consumes completed plans for compatibility, with a local documented lint expectation. |
| [#7](https://github.com/stackpress/rust/issues/7) | Added/reviewed public failure contracts for builder configuration/composition, registration/route forwarding, host/invocation service access, lifecycle entrypoints, startup and PromptInput. Contracts explain rejection state, phase/error precedence and cancellation where applicable. Updated seven existing Markdown pages. |

### Separate resource policies

- Final generated artifacts: unchanged aggregate budget of 1 MiB, checked after formatting and before installation.
- Formatter acquisition: at most 1 MiB stdout and 64 KiB stderr per invocation, enforced before retaining each read chunk; input and both drains run concurrently.
- Formatter runtime: ten seconds per invocation, including pipe operations and exit supervision. Tests lower that policy for controlled fixtures.
- The trusted installed formatter must not delegate inherited pipes to independently surviving descendants. Arbitrary OS spawn/termination stalls are outside the native deadline guarantee.
- Source/bootstrap reads: regular-file policy, caller/default finite byte ceilings, UTF-8 validation and cooperative path ownership. Metadata/open/post checks are portable guards, not race-free acquisition against concurrent special-file replacement. Arbitrary regular-file OS stalls remain possible. Moving bootstrap to a blocking worker does not make the operation cancellable; runtime shutdown may wait for such work.

### TTY cleanup ownership and fallbacks

- Cleanup is prestarted before the reader. Failure to start cleanup creates no reader; reader-spawn failure drops the provisional guard and closes the transfer channel.
- The cleanup owner never reads stdin. It owns the reader's join handle even after the async owner/runtime is dropped. A competing acquisition fails while the worker still holds the lease.
- Transfer failure retains the handle locally, signals stop/unparks and joins synchronously during acquisition as an exceptional fallback. It cannot promise a bound for arbitrary OS stalls. Normal Drop does not take that join path.
- Restoration failure retains the lease/restore flags; guard Drop retries. Explicit close reports the worker's restoration result. Drop cannot report errors.
- Native input errors remain delivered through PromptInput::next; close reports worker panic/restoration failure. Panic payloads are deliberately replaced with a fixed message because arbitrary payloads need not be Errors and may contain private data.
- Formatter stderr remains bounded and discarded, preserving the pre-existing disclosure policy. Atomic correlation exhaustion maps a counter value, not an underlying Error, so that deliberate map does not discard a source.

## Failing regressions observed before fixes

| Contract | Recorded red evidence |
| --- | --- |
| #10 JSON/default causes | Both server error_causes tests failed by finding source() absent. The serialization failure was independently rerun against the original config module before restoring the fix. |
| #3 additive classification | External error_kinds consumer failed to compile because TransformErrorKind/kind() did not exist. Further runtime regressions exposed duplicate-selection classification and JSON visitor Limit being mislabeled Parse; both were fixed and rerun. |
| #8 executor progress | Current-thread entry watchdog child failed with `bootstrap blocked executor progress` before its sibling future could run. |
| #9 device/FIFO | The original reader accepted /dev/null. An independent FIFO run blocked without a writer; its two-second watchdog killed and reaped the child. |
| #13 streams/runtime | Original collector accepted excessive stdout and stderr and accepted a child after its two-second stall; all three regressions failed. |
| #12 Drop | Controlled stalled worker made original Drop wait about 404 ms, failing the <100-ms assertion. |
| #4 diagnostics | The new compile-fail discarded-chain doctest initially failed because the chain compiled successfully; after annotation it passes. |

Raw development logs are in `/tmp/host-serialization-red.log`, `/tmp/host-fifo-red.log`, `/tmp/host-must-use-red.log` and the tool session transcript. These are local evidence, not durable tracked artifacts.

New native/process/concurrency probes run as exact ignored children under independent kill/reap watchdog parents. Controlled formatter fixtures also assert completion in under one second and, on Unix, verify their PID no longer exists with kill -0, covering reaping as well as termination. Existing real PTY tests cover idle/backpressured workers, cancellation/unwind, prior raw mode, repeated close and actual termios restoration. Controlled tests additionally cover cancelled close, worker panic and restoration failure/lease retention.

## Review rounds

1. Logic/complexity: reviewed combined selection/ownership/budget guards and callback precedence. Separated kind decisions without parsing messages; preserved legacy callback behavior. All three assigned crate suites passed (`/tmp/host-round1.log`).
2. Responsibility/abstraction: separated formatter child ownership, pipe startup, supervision and capped reading; moved its controlled process tests to a dedicated module. Reused the native source opener. Private install helpers borrow plans; the public consuming contract remains intact. Added cancellation-safe TTY completion ownership and independent watchdog parents. All three assigned crate suites passed (`/tmp/host-round2.log`).
3. Consistency/Rustdoc/comments: reviewed analogous wrappers, builder attributes and public failure contracts, documented source/process/TTY boundaries, and updated API reference pages using chrisai-docing. Three-crate HTTP-feature tests/doctests passed (`/tmp/host-round3.log`); optional must_use/ownership/errors-doc lint audit passed.
4. Safety/soundness follow-up: traced child RAII-before-scope-join ordering, TTY lease ownership through Drop/restoration, transfer/spawn failure paths, and regular-file race limitations. Strengthened formatter PID/latency checks and fixed parser/serde budget classification with a failing regression. No unsafe code, lock-across-await, detached competing stdin reader, or public ownership/signature change was introduced. Transformer tests/doctests passed (`/tmp/host-safety.log`); final workspace checks follow below.

## Final verification

| Check | Result | Command |
| --- | --- | --- |
| fmt | passed | `cargo fmt --all -- --check` |
| audit | passed | `cargo clippy -p stackpress-server -p stackpress-cli -p stackpress-idea-transformer --no-deps --all-targets --features stackpress-server/http --locked -- -D warnings -W clippy::return_self_not_must_use -W clippy::needless_pass_by_value -W clippy::missing_errors_doc` |
| clippy | passed | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| workspace-tests | passed | `cargo test --workspace --all-targets --locked` |
| workspace-doctests | passed | `cargo test --workspace --doc --locked` |
| rustdoc | passed | `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-server -p stackpress-cli -p stackpress-idea-transformer --features stackpress-server/http --no-deps --locked` |
| msrv | passed | `cargo +1.96.0 check --workspace --locked` |
| cli-without-tty | passed | `cargo test -p stackpress-cli --no-default-features --locked` |

Commands use Rust 1.96.0 / LLVM 22.1.2 on aarch64-apple-darwin. The persistent build directory was this worktree's absolute `target/`; dev/test debug info and incremental compilation were disabled, with four build jobs. Existing consumer tests own temporary nested Cargo fixtures and remove them. No all-features claim is made; the valid workspace default selection and explicit server HTTP/CLI feature cases were checked.

Workspace test summary: 451 passed, zero failed, 57 ignored watchdog/feature children across 99 reported suites. Workspace doctests: 856 passed, zero failed/ignored. CLI without default TTY: 62 passed, zero failed, six ignored child fixtures. Ignored native/watchdog children are invoked by their active parent tests; platform/feature-gated cases retain their declared limits. Changed Markdown QA checked seven pages: one H1 and all relative file links resolve. Final fmt/diff whitespace checks passed.

### Per-crate coverage

Tool: cargo-llvm-cov 0.8.6, stable executable line coverage. The same persistent target directory is reused via CARGO_TARGET_DIR and CARGO_LLVM_COV_TARGET_DIR. External tests and standalone tests/core_tests modules are excluded with `(^|/)(tests|core_tests)\.rs$|/tests/`; production files are not excluded. Ordinary doctests run separately, without claiming nightly doctest coverage or branch coverage.

| Crate / features | Covered lines | Total lines | Coverage | Development gate |
| --- | --- | --- | --- | --- |
| stackpress-server / http | 1871 | 1982 | 94.40% | passed >=70% |
| stackpress-cli / default | 1258 | 1313 | 95.81% | passed >=70% |
| stackpress-idea-transformer / default | 1989 | 2133 | 93.25% | passed >=70% |

Exact coverage commands (with the environment above):

```sh
cargo llvm-cov -p stackpress-server --locked --lcov --output-path /tmp/host-stackpress-server.lcov --fail-under-lines 70 --ignore-filename-regex '(^|/)(tests|core_tests)\.rs$|/tests/' --features http
cargo llvm-cov -p stackpress-cli --locked --lcov --output-path /tmp/host-stackpress-cli.lcov --fail-under-lines 70 --ignore-filename-regex '(^|/)(tests|core_tests)\.rs$|/tests/'
cargo llvm-cov -p stackpress-idea-transformer --locked --lcov --output-path /tmp/host-stackpress-idea-transformer.lcov --fail-under-lines 70 --ignore-filename-regex '(^|/)(tests|core_tests)\.rs$|/tests/'
```

Remaining measured gaps include native input-error/key branches, the actual OS restoration-failure Drop fallback, exceptional cleanup-owner transfer failure, defensive formatter worker-disconnection paths, and existing host/ownership validation branches. Controlled restoration-failure and cancelled-close tests cover state/lease intent; they do not simulate an actual failing OS termios restoration. Line coverage is not branch or cross-platform coverage.

LCOV reports and full command logs are `/tmp/host-stackpress-server.lcov`, `/tmp/host-stackpress-cli.lcov`, `/tmp/host-stackpress-idea-transformer.lcov`, `/tmp/host-coverage-results.json`, and `/tmp/host-final-results.json`. These paths are local evidence and may be removed by host cleanup.

Cleanup completed after all checks: removed the task-created persistent `target/` and the early regression helper's small `crates/target/` fixture/log tree. No task build tree remains. Reports kept under /tmp are outside the deleted build tree.

## Remaining scope and review

- This lane does not modify crates/lib, SQL, framework, accepted task status, or other lanes' work. Coordinator review is required before task 00010 is marked verified.
- The promised lane-1 #11 final policy paragraph/merge has not arrived in this implementer session. Existing with_cause is used, and owned wrappers add distinct phase context. A final comparison with lane 1's accepted policy is still due; this receipt does not claim it has occurred.
- Native evidence is macOS aarch64 only. Windows/mobile/WASM, uncooperative path replacement, arbitrary OS stalls and independently surviving formatter descendants are not established by these tests.
- No acceptance, release, publication or deployment claim follows from these code checks.
