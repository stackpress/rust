# Task 00012 — HTTP M1 implementation receipt

Implementation evidence for coordinator review, not acceptance or M2 release.
Scope: [Slice 7 verification](../slice-7-design-verification.md), HTTP M1 only.

## Authority and baseline

- Coordinator released M1 and directly authorized this receipt. Began by fast-forwarding this worktree to `trial/3` at `43a8c213ad09f34c6894bff60b9cd7a035acf0b1`; HEAD remains there.
- Branch/worktree: `trial/3-s7-http-m0`, `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s7-http-m0`.
- Changes are confined to `crates/server-http/**` plus this receipt. No changes to server/lib/SQL, manifest, lock, facade, or other KB files.
- No listeners, TCP bind, devmetrics, installs, commit, push or publication. Every new HTTP/TLS proof uses Tokio duplex IO. M2 has not started.

## Delivered boundary

- `NetworkLimits` validates positive, finite, representable count/byte/time policy. Defaults: 256 connections, 32 handshakes, 64 raw header occurrences, 32KiB Hyper buffer, 16KiB normalized retained header bytes, 8KiB target, 1MiB decoded body, 2MiB method/target/header/body aggregate, 100 requests per IO.
- Head count/bytes/target and head aggregate overflow: 431 before host callbacks. Known/actual body or aggregate overflow: 413 through the once-only Input error offer. Body timeout: 408 with original Elapsed cause. G17 normalization remains sanitized 400 with a host Input report and no callbacks. Native parser failures retain their original typed Hyper cause.
- Absolute/progress head and body budgets cannot be extended by a byte/chunk trickle. Hyper-owned head timers also cover partial successors already in its private read-ahead buffer. Body timers live in the loader to preserve G12.
- Lifecycle has a separate absolute budget. Response total and actual-write progress budgets enforce both network policy and tighter `StreamLimits` controls. No progress resets an absolute clock. Quiet keep-alive expires; connection age includes TLS handshaking and begins bounded drain. Handshake budget starts at admission.
- Lifecycle/head expiry can write one empty 408 only after dropping Hyper and only before native publication; Hyper's own head timeout may close instead. Published output is cancelled/closed without retry, JSON append, or another error lifecycle.
- Keep-alive/pipelining is sequential through producer completion **and native flush**. Request count is finite. An earlier settled exchange cannot be failed by a later parse failure. `Completed` means local producer handoff plus native flush, never remote acknowledgement; forced stalled buffered output reports `Cancelled`.
- Generic `Acceptor`, owned `AcceptedIo`, sticky `Shutdown`, and bounded `DrainReport` supervise supplied IO without binding. Immediate connection/TLS permits precede spawn and handshake; excess IO drops promptly, with 503 in direct rejection reports rather than invented unparsed/TLS wire bytes.
- Stop and synchronous spawn share one gate: after stop returns no further admission is possible. Stop wins over ready accept; source end/error closes this run's admissions and drains. One common absolute grace covers all tasks, followed by abort and reaping of every handle. Repeated stop is harmless.
- Existing phase deadlines remain active during graceful drain. Connection reports are bounded by request count; returned recent connection history is bounded by `max_connections`, with full aggregate/discard counters. Cancellation guards and RAII release handlers, producers and permits on disconnect/drop/panic/timeout. Original accept failure survives cleanup; report Debug remains redacted.

## Accepted framing deviation under G28

The coordinator rejected a fork/patch of Hyper and a first-party framing validator, then accepted Hyper's normalized interpretation as authoritative. There is **no direct httparse dependency, preflight, fork, or second framing parser**.

- For CL-framed requests, equal duplicate Content-Length values are accepted as one length, as allowed by [RFC 9110 section 8.6](https://www.rfc-editor.org/rfc/rfc9110.html#section-8.6). Hyper rejects differing and invalid lengths.
- CL preceding TE remains visible in the HeaderMap: reject 400, close, invoke no route. TE preceding CL causes Hyper to discard CL: process using TE, as sanctioned by [RFC 9112 section 6.1](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.1).
- **Every request carrying TE closes after its response**, even valid chunked requests. A pipelined successor on that IO is never dispatched. This satisfies the RFC 9112 closure requirement even where Hyper removed CL. Request-looking body bytes remain body bytes.
- Non-final chunked coding, HTTP/1.0 TE, invalid/differing CL, bad chunks and folded/control headers retain Hyper's rejection. Unsupported transfer codings, request trailers, upgrades and CONNECT are rejected by host policy; no operation route runs.
- Source evidence: pinned `hyper-1.10.1/src/proto/h1/role.rs:259–297` (`is_te`, CL skip, equal-CL merge, differing-CL rejection); `conn.rs:435–446,490–512` and `dispatch.rs:205–271` show private read-ahead/body driving; `server/conn/http1.rs:143–185` exposes parts only after driving ends. A read-wrapper every-head preflight would require independent body framing, which was explicitly declined.
- Service-only strict rejection was insufficient because equal duplicate CL is merged and CL following TE is dropped before service entry. The accepted relaxation is exact, tested in both occurrence orders, and limited by unconditional TE connection closure. Normalized byte budgets charge retained fields; Hyper's independent raw occurrence/buffer caps bound parsing before normalization.

## Proofs and review rounds

- M0 stream ownership, Send compile-fails, replacement/clone/error, bodyless behavior, cancellation and valid/name-mismatch Rustls handshake regressions remain intact. Existing buffered conversion tests remain unchanged. M0 duplex fixtures now explicitly request close where their original assertions required EOF.
- M1 watchdog scenarios cover raw framing; at/equal/over body/head/aggregate limits; malformed normalization; absolute and idle head/body/stream/native-write deadlines; blocked stream sequential pipeline; buffered partial next head; request-count/idle/age expiry; independent connections/backpressure; admission overflow; mixed TLS/plaintext caps; stop before/during accept; exact last-admission gate; handler/write drain; source failure; task panic; parent drop; bounded history; disconnect at head/body/handler/output; invalid options; and a continuously ready acceptor running with stop on one local executor.
- Each hang-class scenario runs in a child process, with a parent ten-second kill/reap watchdog; timeout fails. The ignored child entry is invoked by its parent, not treated as skipped coverage. The intentional task panic in logs is an asserted supervision case.
- Round 1 (complexity): immutable deadline precedence, phase wakeup races, report settlement. Tests: `round1.log`.
- Round 2 (responsibility): extracted input policy/loading, expiry handling, synchronous spawn gate and task drain; native Hyper timer covers read-ahead partial heads without parsing. Tests: `round2.log`, `round2-final.log`, `round2-complete.log`.
- Round 3 (consistency/Rustdoc/comments): documented API, policy and limitations; added mixed TLS admission proof. Tests/lint: `round3-final.log`, `round3-clippy.log`.
- Round 4 (safety): found graceful drain could replace a shorter phase deadline; preserved existing deadlines while grace is an additional bound. Tests/lint: `round4.log`, `round4-clippy.log`.
- Round 5 (report truth): failing regression demonstrated `Completed` before a stalled native flush during forced drain (`round5-before.log`). Native flush is now the sole completion boundary; `round5.log` passes. Non-TLS test-vector unused-mut lint was corrected without suppressing warnings.
- Round 6 (deadline ownership): handshake deadline now starts at admission and is capped by connection age; `round6.log` passes. Repetition caught a fixture with grace/write deadlines only 10ms apart; separated them so the forced-drain test selects its intended condition. Final stress: 20/20 full TLS-enabled watchdog-suite runs pass after the last behavior change (`closeout-stress-summary.json`, `closeout-stress-*.log`).
- Round 7 (cooperative scheduling): an always-ready custom acceptor now consumes a cooperative task budget per iteration, so it cannot monopolize one executor thread and starve stop or owned connection tasks. A LocalSet watchdog proof exercises this without a second worker masking starvation. Tests: `round7.log`; final repeated suite passes 20/20.
- Final lock/ownership review: no locks across awaits, no user callback under admission/diagnostic locks, one Hyper reader, original error ownership, cancellation/permit Drop, no detached producer, all awaited supervisor handles reaped. No additional shared API request.

## Verification and coverage

Final command arrays, exit codes and durations: `/private/tmp/s7-http-m1/release-final/verification.json`; corresponding `*.log` files. The preserved default-thread workspace failure and successful serial retry are reconciled in `closeout-results.json`. Build environment: `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`. Rust/target: `rustc 1.96.0 (ac68faa20)`, `aarch64-apple-darwin`.

- `cargo fmt --all --check`; `git diff --check`.
- `cargo test -p stackpress-server --no-default-features --locked`; server `--features http-stream`.
- `cargo test -p stackpress-server-http --no-default-features --locked`; host `--features tls`. Each includes ordinary doctests.
- Isolated server-default Clippy: `cargo clippy -p stackpress-server --all-targets --no-default-features --locked -- -D warnings` (`clippy-server-default-isolated.log`).
- Clippy `-p stackpress-server -p stackpress-server-http --all-targets --locked -- -D warnings`, both no-default and `--features stackpress-server-http/tls`.
- `cargo +1.96.0 check` server/host all-targets with TLS; facade no-default `--features server-http-host-tls`.
- `RUSTDOCFLAGS='-D warnings' cargo doc` server/host `--no-deps --locked`, no-default and host TLS.
- `cargo test --workspace --locked`: earlier default-thread runs passed. The final default-thread run hit an unrelated SQLite M2 contention `PoolTimedOut` at `cancellation.rs:454` (`release-final/workspace.log`); its exact isolated watchdog replay passed (`workspace-sqlite-replay.log`). Final full retry uses `RUST_TEST_THREADS=1 cargo test --workspace --locked`: passes, including doctests (`workspace-serial.log`). No SQL source was changed. PostgreSQL/MySQL backend URL variables were unset; native opt-in backend harnesses were ignored, not reported as native proof.
- Normal dependency graphs (`trees.json`, `tree-*.log`): default facade, old `server-http`, and wasm CSR facade have **zero Hyper/Tokio/Rustls/socket2/mio edges**. New host-TLS graph contains only its opt-in network stack. No manifest/lock package changed; [M0 dependency/license/MSRV assessment](00012-receipt-http-m0.md) remains applicable. Test certificates remain the checked-in test-only fixtures; no generator dependency added.
- Coverage: server-http **1043/1075 = 97.02%**; unchanged server regression scope **2023/2174 = 93.05%**. Each exceeds the 70% development floor. `cargo-llvm-cov 0.8.6`, fresh build in the same worktree target, TLS enabled; instrumented tests include watchdog children. Production executable lines only; exclude test-only source/trailing cfg(test) blocks, dependencies and generated test/build artifacts. Doctests run separately and are not claimed as covered. LCOV, per-file counts and commands are under `/private/tmp/s7-http-m1/`. Exact coverage commands: `cargo llvm-cov --no-report -p stackpress-server -p stackpress-server-http --features stackpress-server-http/tls --tests --locked`, then `cargo llvm-cov report -p stackpress-server -p stackpress-server-http --lcov --output-path /private/tmp/s7-http-m1/coverage.lcov --locked`; `CARGO_LLVM_COV_TARGET_DIR` selects the same worktree target. See `final-coverage.py`, `coverage-run-final.log`, `coverage-export-final.log`, `coverage-summary.json`.
- Target cleanup: `cargo clean` after LCOV export; target removal verified. Existing tracked SQL-lane raw profiles are unchanged. Existing workspace consumer tests internally create/drop temporary private targets; this lane configured only the single worktree target.
- KB validation: validator rerun exits 1 with the same 146 pre-existing missing worktree-relative upstream sibling-source links and 22 warnings as before receipt creation; no new findings. See `kb-validation-before-receipt.log`, `kb-validation-after-receipt.log`, `kb-validation-comparison.json`. These unrelated KB links are outside this lane’s authorized write set and were preserved.

## Limits and next gate

Async deadlines cannot preempt blocking/CPU-bound user polls; process watchdogs protect tests. Disconnect detection during computation is bounded by native IO polling/lifecycle deadlines, without a second reader. Hyper/TLS/kernel/user allocations are not a whole-process memory ceiling; configured parser/body/chunk/admission bounds constrain retained host work. The acceptor owns its external backlog and must be cancellation-safe. Dropping a supervisor aborts owned tasks; awaiting stop additionally guarantees reaping before service closure. Report-only pre-parse admission statuses and native parser-timeout closure are documented boundaries, not wire-response guarantees.

No further shared request or unresolved M1 product decision. Coordinator should independently review/replay this milestone before accepting it. TCP bind/serve-until, deployment TLS/proxy policy, immutable assets, observer/app integration and managed devmetrics smoke remain M2/closeout and require separate release.
