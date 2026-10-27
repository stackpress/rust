# Task 00012 HTTP lane closeout receipt

Four whole-crate review rounds completed on 2026-10-27. Worktree:
`trial3-s7-http-m0`, branch `trial/3-s7-http-m0`. Requested merge fast-forwarded
`ae889ef` → current `trial/3` at `537cb4d` before review. All changes remain
uncommitted. No new dependency, manifest, lockfile or facade change.

## Scope and result

Reviewed every production module of stackpress-server-http, including optional
TLS, and server's http.rs/http/common.rs/http/stream.rs. Reviewed the M0/M1/M2
integration fixtures, existing server HTTP contracts, docs, and ownership seams.
**No real safety bug or required public API change was found.** No public method,
trait, type, field, variant or default policy changed; a declaration-header audit
covers all 19 HTTP production source files, with the actual diff checked as well.

Behavior-preserving production changes: flatten TLS timeout/native results into
one Result before serving (remove impossible success arm); expand the stop-future
selection without changing biased priority; add explanatory comments and public
Rustdoc usage examples. Test infrastructure shares process ownership rather than
copying three watchdog loops. New TCP coverage exercises existing behavior only.

## Four distinct rounds

1. **Logic and complexity.** Traced Input/G17/G12, output preflight, one-shot
   publication, body EOF versus flush, admission, parser head timer versus loader
   timer, absolute/progress/lifetime clocks, and common drain deadline. Simplified
   TLS's nested result match without changing contexts/status/cause identity.
   Expanded serve_acceptor_until's selection so stop → gate → awaited drain is
   visible. Baseline server/http-stream and host default/TLS passed; round1-tests
   reran both crates/all-targets/TLS successfully.
2. **Duplication, responsibility and test harness.** Existing private HTTP common
   module already owns normalization/loading/headers; retained separate buffered
   and streaming publication because their body capabilities differ. Left distinct
   milestone probes/scenarios local instead of creating a multipurpose harness.
   Extracted tests/support/watchdog.rs, preserving 20-second duplex and 10-second
   per-scenario deadlines, selectors, output, kill/reap and failure semantics.
   Its OwnedChild guard also reaps if parent status inspection/assertion unwinds.
   round2-tests reran both crates/all-targets/TLS successfully.
3. **Consistency, #11, Rustdoc and comments.** Audited every native wrapping site
   against task 00010's accepted #11 policy: distinct safe operation context,
   typed External category, retained original cause, no interpolated source
   Display; existing framework errors propagate unchanged. No wrapper-policy
   behavior change was needed. Added positive owned-stream and handle_body
   examples, trusted buffered metadata, preparation, observer, CIDR, limits,
   cancellation, shutdown and TLS-file examples; linked body/reply inspection.
   Added missing private-state explanations and explicit G31 comments. Corrected
   docs that incorrectly advertised Debug for TlsFiles/TlsFileFormat, without
   adding that trait/API. A first new example incorrectly named HandleReport's
   field; compilation caught it, it was corrected, and final doctests passed.
   round3-tests passed; final doctests: server 76 + 5 compile-fail, host TLS 13.
4. **Safety, soundness and shutdown.** Rechecked lock order, user polls/callbacks
   outside locks, native IO exclusivity, retained response views, discard/no-second-
   poll, original cause/redaction, file-tree ownership, TLS admission, exact last
   admission/stop, source-before-drain, abort/reap and supervisor Drop. No unsafe
   or escaping library producer/sender task was introduced. Added one isolated
   ephemeral-loopback TCP test because bind/local_addr/native accept/listener
   closure cannot be proved by duplex IO alone. It verifies a completed response,
   one admitted/reaped task, no fatal failure, then connection refusal after
   awaited stop, before dropping application services. round4-tests and the final
   host default suite passed outside the sandbox; all policy proofs remain duplex.

## G31 and boundary review

Hyper remains the sole framing parser. Equal duplicate CL is normalized to one
length; differing/invalid CL is rejected. Visible CL+TE rejects 400 before routes.
When Hyper dropped CL after TE, TE wins; every TE request responds then closes,
so no pipelined successor runs. Chunked body bytes remain opaque. Existing raw
occurrence-order tests cover both outcomes, chunk errors, folded/control fields,
non-final chunked, HTTP/1.0 TE, unsupported trailers/codings and ordinary CL reuse.
No parser fork, direct httparse, raw preflight or weaker rejection was added.

Absolute deadlines never move with trickle progress; write idle uses native IO
progress. Body timeout remains owned by Input, preserving the once-only error
offer. Publication failure cannot retry or append JSON. The prior flush must
settle before the next lifecycle. Existing deadline-during-drain, stop boundary,
ready-acceptor fairness, bounded history, drop/disconnect and TLS admission proofs
passed again, including the instrumented replay.

Awaited stop drops the source/listener/backlog before group drain, then aborts
and reaps every owned task. Dropping a serving future aborts tasks but does not
promise synchronous reaping; await stop before closing process services. Async
cancellation cannot preempt blocking/CPU-bound callbacks or startup filesystem
reads. The operator-owned stable-file contract still excludes hostile concurrent
replacement. These are existing documented boundaries, not new safety findings.

## Final verification and evidence

Compiler: Rust 1.96.0 (ac68faa20), aarch64-apple-darwin. Logs and machine-readable
argv/results: `/private/tmp/s7-http-closeout/` (individual .log/.json files and
checks-summary.json). Each build/test/docs command had an independent parent
watchdog (900 s, kill process group and reap) and sampled target-size guard (8 GiB).
Native scenario children also retain their independent 10/20 s parent watchdogs.
No watchdog expired or disk guard fired; sampled peak was **793.61 MiB**.

| Gate | Result |
| --- | --- |
| Baseline server/http-stream, host default/TLS all-targets | Passed. |
| Four round-specific test replays | Passed after the documented example/sandbox corrections. |
| Final server default, http, http-stream tests | Passed. |
| Final host default and tls all-targets (including ephemeral TCP) | Passed. |
| Combined default-host/TLS Clippy all-targets, -D warnings | Passed. Host dependency enables server/http-stream in both combined graphs. |
| Server/http-stream and host default/TLS doctests | Passed, no ignored examples. Combined host-default also unifies server/http-stream. |
| fmt --all -- --check; git diff --check | Passed. |
| Rustdoc, both crates/stream/TLS, RUSTDOCFLAGS=-D warnings | Passed. |
| Documentation scan, 23 affected/routed pages | One H1, tagged fences, headings, spacing, relative links/anchors clean. |
| Public declaration audit | Unchanged in 19 HTTP source files. |
| Default and old buffered server-http normal cargo trees | Zero Hyper/hyper-util/Tokio/Rustls/tokio-rustls edges. |

The first TCP attempt failed at bind with sandbox PermissionDenied; the test
properly failed rather than skipping. Approved outside-sandbox replays passed
with no source workaround. No listener ran outside the test process; no devmetrics,
managed smoke, install, commit, push or publish was performed. The full unrelated
workspace suite was not rerun; this change's two crates/features and consumer
example are covered by their all-target gates.

## Per-crate coverage and publication work

Installed cargo-llvm-cov 0.8.6; fresh clean TLS-enabled test build in the same
worktree target (CARGO_LLVM_COV_TARGET_DIR points to target, not another tree).
Watchdog children inherit instrumentation. Exported coverage.lcov and
coverage-all.json; coverage-summary.json aggregates canonical physical DA lines
by crate and lists every uncovered line. Counts are executable production lines:
exclude only server/core_tests.rs and lifecycle.rs's trailing cfg(test) module
(starting at line 726), separate tests, dependencies and examples. Every production
source is inventoried; declaration-only lib.rs has no executable mapping, not
an invented 100% result. No production path/branch/file was excluded. Doctests
were run separately from instrumented line coverage.

| Crate/scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| stackpress-server, entire production crate with http-stream | 2068 / 2219 | **93.20%** |
| stackpress-server-http, entire production crate with tls | 1838 / 1902 | **96.64%** |
| Supplemental server HTTP-only subset | 601 / 665 | 90.38% |

Both crates exceed 70% and already cross the numeric 90% publication floor in
this declared native scope; no extra lines are needed just to cross 90 here.
Publication preparation would still need the intended release feature/target
and integration matrix, plus assessment or targeted proofs for the remaining:

- Server HTTP conversion failures that valid checked status/header/MIME/JSON
  values cannot readily produce; stream ownership/poison and late output-failure
  defenses; non-HTTP startup/registration/config/report/delegation branches.
- Host native accept/local-address failure, empty-body fallback, timeout-write
  WriteZero, reuse-exhaustion guard, shutdown-grace setter, observer-ID exhaustion,
  alternate CIDR/proxy syntax failures, startup post-metadata file changes, DER
  diagnostic Display, relative timer fallback and aggregate report Debug.
- TLS over the native TCP adapter and supported OS/runtime/kernel edge behavior;
  TLS handshakes/files/invalid certificates are currently proven through duplex.
  The new native TCP proof is plaintext, not an application/process-signal smoke.
- Actual publication/package/dependency-license/MSRV review and declared platform
  support. Line coverage is not branch, security, release or all-platform proof.
  HTTP/2, mTLS, certificate reload, asset ranges/compression remain unsupported.

## Reproduce and handoff

From the worktree root, use CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0,
CARGO_TARGET_DIR="$PWD/target". Protect all commands with a parent process
watchdog; tests/tcp.rs requires permission for one ephemeral loopback listener.
Final command shapes (no managed server):

```sh
cargo test --locked --offline -p stackpress-server --all-targets
cargo test --locked --offline -p stackpress-server --all-targets --features http
cargo test --locked --offline -p stackpress-server -p stackpress-server-http --all-targets --features stackpress-server/http-stream,stackpress-server-http/tls
cargo test --locked --offline -p stackpress-server-http --all-targets
cargo test --locked --offline --doc -p stackpress-server -p stackpress-server-http --features stackpress-server/http-stream,stackpress-server-http/tls
cargo clippy --locked --offline -p stackpress-server -p stackpress-server-http --all-targets --features stackpress-server/http-stream,stackpress-server-http/tls -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --offline --no-deps -p stackpress-server -p stackpress-server-http --features stackpress-server/http-stream,stackpress-server-http/tls
# Wait for every Cargo process before cleaning; reuse this same target for instrumentation.
cargo clean --target-dir target
CARGO_LLVM_COV_TARGET_DIR="$PWD/target" cargo llvm-cov clean --workspace
CARGO_LLVM_COV_TARGET_DIR="$PWD/target" cargo llvm-cov --no-report --tests --locked --offline -p stackpress-server -p stackpress-server-http --features stackpress-server/http-stream,stackpress-server-http/tls
CARGO_LLVM_COV_TARGET_DIR="$PWD/target" cargo llvm-cov report --lcov --output-path /private/tmp/http-closeout.lcov --locked --offline -p stackpress-server -p stackpress-server-http
```

KB baseline/post-receipt: same **146 existing missing-source link errors and 24
warnings**, no new findings. Target removed after export; no lane-owned build
remains. Source/fixtures stay uncommitted in the granted write set. Next step:
coordinator review and integration; no further public decision or API request.
