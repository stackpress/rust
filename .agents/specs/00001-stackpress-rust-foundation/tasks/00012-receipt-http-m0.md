# Task 00012 HTTP M0 implementation receipt (approval draft)

Prepared in `/private/tmp/s7-http-m0/00012-receipt-http-m0.md`. This draft is not
an approved `.agents` write, milestone acceptance or authorization for M1.

## Identity and boundaries

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s7-http-m0`.
- Branch: `trial/3-s7-http-m0`; starting/current HEAD `7029d831f116de607f2a7bb2bfa34cb32a341c7e`.
- Initial worktree clean. Only coordinator-granted HTTP/server/root facade and lock files changed.
- No Lib, CLI, Idea, SQL, examples, other trials, commit, push, publication, installation or listener.
- Cargo fetched the approved dependency pins after sandbox DNS failure and explicit fetch approval.
- One worktree-local target, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`.
- All logs, scripts, dependency and coverage reports: `/private/tmp/s7-http-m0/`.

## Implemented M0 boundary

`stackpress-server/http-stream` adds a typed Response extension carrying an owned
pinned `Stream<Item = Result<Vec<u8>>> + Send + 'static`. A private
`SharedNative` identity contains `Mutex<Option<OwnedStream>>`; no unsafe code or
public take/clone handle. The lock transfers ownership only, never polls user
code. Stream byte/chunk policy is enforced on each admitted chunk.

`HttpAdapter::handle_with_options` passes trusted metadata. Private shared
normalization/loading/headers/cookies/buffered framing live in `http/common.rs`.
The buffered adapter keeps its existing representation bytes and error rules.
`StreamingHttpAdapter::handle_body` normalizes before invoking exactly one
Send byte loader inside Input, rechecks aggregate head+body budgets, and runs
one lifecycle. Normalization failures retain G17 Input reports without callbacks;
loader failures follow G12. Lifecycle internals remain private. `HandleReport`
and foundation APIs are unchanged.

Preflight validates the final response before taking any stream or publishing.
HEAD and 204/205/304 take/drop producers without polling. Conversion failures
produce one empty 500 before authority is consumed. Replacement and fresh error
responses discard old body/header/cookie associations. Existing buffered output
rejects the typed native stream instead of collecting it. Unrelated native
values remain unsupported. Connection-nominated and fixed hop headers are
removed for the new streaming/native path; caller Transfer-Encoding is rejected.

`stackpress-server-http` owns Hyper HTTP/1 over supplied IO, tracked body drop,
trusted peer/scheme/origin/cancellation metadata, graceful stop and final reports.
No background producer/completion task or completion channel exists.
The connection/service/body/IO are dropped before final report extraction.
Framework Sent records publication; Completed records local EOF handoff to
Hyper, never remote acknowledgement. Native IO failures can downgrade handoff
to Failed. Original producer causes remain typed and win over secondary native
failure; no hook retry or JSON append follows partial output. Default Debug
formatting redacts original native messages/causes; the explicit failure field
still retains the typed original error. The regression checks both properties.

`TlsConfig` and `serve_tls_io` are opt-in with explicit ring, TLS 1.2/1.3,
HTTP/1.1 ALPN and a fixed five-second handshake timeout. Bad handshakes never
fall back to plaintext or fabricate host callbacks. Public test-only DER fixtures
were generated with installed LibreSSL 3.3.6 and SHA-256/P-256; no generator
crate/tool install was added. Their recipe and nondeployment scope are checked in.

The facade's pre-existing `server-http` feature remains buffered-only. The new
`server-http-host` and `server-http-host-tls` select the host and TLS separately;
`stackpress::server_http_host` is the opt-in export. No default expansion.

## Proofs and review rounds

- Real multithread Send handler/adapter/host futures.
- Non-Send stream and borrowed Request/ResponseView/Invocation compile-fail doctests.
- Two retained ResponseView clones and a repeated lifecycle cannot take/poll twice.
- Whole-response/body replacement and error after stream installation: no old producer
  polls or success body/header/cookie leakage. Retained views may extend allocation,
  never producer ownership.
- HEAD/204/205/304 drop without producer polls, with correct unknown/zero length.
- Exact chunk/total byte boundaries, fused producer failure, native output drop.
- Actual Hyper duplex buffered/streamed completion, suppression, foreign output,
  parser rejection versus normalized G17 failure, original typed post-prefix failure.
- The client reads `first` before releasing a producer error: exactly one response,
  no private cause or error-envelope append; report retains original IO cause.
- Graceful stop drains a released stream; grace expiry drops pending stream/handler.
  Serving-future abort, peer disconnect and already-signalled stop cancel/drop work.
- Real duplex Rustls valid localhost handshake and hostname mismatch, ALPN check,
  no lifecycle for invalid certificate, empty certificate-chain config rejection.
- All hang-class host scenarios run in an ignored child explicitly launched by a
  parent 20-second watchdog; timeout kills and reaps and fails. The ignored child
  is not a skipped proof: parent executes it. No host timeout occurred.

Baseline: existing server tests plus real new stream/duplex/TLS proofs passed.
A meaningful initial failure found that Hyper can finish Content-Length output
without polling EOF; `HttpBody::is_finished` now records known buffered completion
and empty/bodyless output rather than a false cancellation.

Round 1 (logic/complexity): terminal-state and limit/metadata review; added exact
byte-limit and trusted-options regressions. `round1.log` passed.
Round 2 (responsibility/ownership): made native body state exhaustive, removing
parallel invalid optional fields/unwraps and a large-enum warning; moved final
report assembly to the report module. `round2-final.log` passed.
Round 3 (consistency/Rustdoc/comments): documented all additive APIs, lifetime,
publication/cancellation and M0 limitations; added independent borrowed-capture
compile-fail docs and updated server HTTP reference. `round3.log` passed.
Round 4 (soundness): reviewed mutex lifetimes, producer Drop, output-authority,
Hyper final-frame behavior and cancellation; strengthened observed-partial-output
failure proof and watchdog reap-on-kill-race. `round4-soundness.log` passed.
Follow-up final review: replaced derived report Debug with redacted formatting,
with original typed-cause and disclosure regressions. A repeated shutdown test
found a fixture race: producer completion could win before stop consumption.
The fixture now awaits an explicit stop-consumed signal before releasing the
producer; no host behavior was changed for that race. The TLS watchdog suite
then passed 20/20 independent replays (`duplex-replay-01.log` to `20.log`).
An additional real 204 response proves completion even when Hyper omits all
body polling. `final-redaction-test.log`, `final-redaction-clippy.log`, and
`final-host-default.log` pass after those changes.

## Verification

Compiler: Rust 1.96.0 `ac68faa20`, LLVM 22.1.2, aarch64-apple-darwin.
Exact commands/outcomes are recorded in `verification.json`,
`coverage-commands.json` and their named `.log` files. All Cargo gates use
`--locked` after approved resolution. Rustdoc uses `RUSTDOCFLAGS=-D warnings`.

- `cargo fmt --all --check` and `git diff --check`: passed.
- Server `cargo test` with no default features and with `http-stream`: passed,
  including existing buffered suites and doctests.
- Host `cargo test` without TLS and with TLS: passed, including watchdog child.
- Clippy `--all-targets -- -D warnings`: passed for server default, buffered-only
  `http`, streaming via host, host default and host TLS.
- `cargo +1.96.0 check` affected crates/all targets/TLS and facade
  `server-http-host-tls`: passed.
- `cargo doc --no-deps`, default and TLS, with `RUSTDOCFLAGS=-D warnings`: passed.
- `cargo test --workspace --locked`: passed, then passed again after the final
  report-formatting change (`workspace-final.log`, exit 0).
- PostgreSQL/MySQL environment gates were both unset. Existing ignored native
  tests are not counted as backend evidence. No network database proof is claimed.

Final production executable-line coverage: server **2018/2174 = 92.82%**;
server-http with TLS **306/321 = 95.33%**. Both exceed the 70% development floor.
Tool: cargo-llvm-cov 0.8.6 / LLVM 22.1.2 on Rust 1.96.0, native aarch64-apple-darwin.
The scoped tests include server `http-stream` and host `tls`; watchdog children
inherit the profile path. Only production `src` lines are counted. Integration
fixtures, `src/core_tests.rs`, and the trailing `#[cfg(test)]` lifecycle module
are excluded as test source; no production file/branch is excluded. The exact
LCOV and per-file denominators are in `coverage.lcov`, `coverage-summary.json`
and the explicit `coverage-summary.py` filter.

Measurement diagnostics: first reuse of ordinary artifacts emitted no profiles,
so it was not counted as coverage. This cargo-llvm-cov version rejects `--features`
on its `report` subcommand despite listing it in help; feature selection belongs
to the instrumented test run. The valid final run cleaned selected artifacts and
cleared old raw/merged profiles, rebuilt instrumented code in the same worktree
`target`, ran scoped tests, then exported LCOV with package selection. Exact
replay is `python3 /private/tmp/s7-http-m0/final-coverage.py`. Intermediate default
profile files from ordinary runs of instrumented cached binaries were moved out
of source paths into `/private/tmp/s7-http-m0/discarded-profiles/`.

Remaining uncovered/branch risks are explicit: detailed native request-body
frame/trailer/overrun branches and full deadlines belong to M1; the fixed TLS
handshake timeout/early-stop cases are not the accepted M0 valid/mismatch proof.
Some defensive conversion paths are unreachable behind the existing validated
status/MIME invariants. Coverage does not imply branch/platform completeness.

Final workspace rerun: exit 0. Final rustfmt/diff checks: passed. Source audit:
20 changed/new files, all inside the granted write set; HEAD/branch unchanged,
no `.agents` write. `source-audit.json` records every file's SHA-256.
`cargo clean` completed and the entire worktree-local `target` was removed.
Generated temporary CA/private signing material was removed; checked-in public
test-only fixtures remain. Logs/LCOV/receipt draft remain under the approved
`/private/tmp/s7-http-m0/` location. No build output remains in the worktree.

## Dependencies and feature evidence

Hyper `=1.10.1` (MIT, declared MSRV 1.63), hyper-util `=0.1.20` (MIT, 1.64),
http `=1.5.0` (MIT OR Apache-2.0, 1.57), http-body `=1.1.0` (MIT, 1.61),
http-body-util `=0.1.4` (MIT, 1.61), bytes `=1.12.1` (MIT, 1.57),
futures-core `=0.3.34` (MIT OR Apache-2.0, 1.36); existing Tokio `1.54.1`
(MIT, 1.85). Optional Rustls `=0.23.46` (Apache-2.0 OR ISC OR MIT, 1.71),
tokio-rustls `=0.26.6` (MIT OR Apache-2.0, 1.71), defaults disabled.

`dependencies.md` and `dependencies.json` contain the exact normal/build native
closure (76 third-party packages), licenses and declared MSRVs, plus all 19 new
third-party lock packages, including Windows target edges. Highest declared MSRV
in this native closure is 1.88 (ICU); undeclared MSRVs are explicitly identified.
Rust 1.96 native compilation is verified, not inferred from manifest declarations.
No Windows/cross-platform compile proof is claimed.

Normal-edge `cargo tree` assertions confirm no host/Hyper/Tokio/Rustls/socket2/mio
edge for default facade, the old `server-http`, and browser CSR/WASM graphs.
Evidence: `tree-default.txt`, `tree-buffered.txt`, `tree-browser.txt`,
`tree-host-tls.txt`, `tree-host-tls-build.txt`. No Axum/Tower production edge.

## Limitations, deviations and next gate

M0 has no binding/accept API and serves one exchange per supplied IO. Fixed head
controls and incremental body counting are scaffolding for M1, not the full
limits/status/admission matrix. Stream time controls are retained but **not yet
enforced** by this host. Full body/lifecycle/write/reuse deadlines, bounded
keep-alive/pipelining, admission/drain supervision across connections are M1.
Proxy/authority policy, certificate file/deployment policy, immutable assets,
observers and app main are M2. No production readiness, remote acknowledgement,
HTTP/2, actual TCP/listener/deployment, GUI or browser-host proof is claimed.
A CPU-bound/blocking user poll cannot be preempted by async cancellation.

The additive M0 constructor/serve_io and TLS DER-config types replace proposal
sketches. There is no premature NetworkOptions/bind/app-main surface. The facade
uses separate host feature names as explicitly required by coordinator review.
No unapproved dependency, public core/report change or other shared write request.
Stop at HTTP M0; do not start M1 until coordinator release.

Approval required before copying this draft to the sole allowed KB path:
`.agents/specs/00001-stackpress-rust-foundation/tasks/00012-receipt-http-m0.md`.
