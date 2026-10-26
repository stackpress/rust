# Slice 7 risks, implementation lanes and verification

Load with [decisions](slice-7-design-decisions.md) before releasing M0. This is a
plan, not completed code verification. [Quality policy](../../context/testing-and-quality.md)
and [coordinated workflow](../../workflows/coordinated-crate-development.md) govern.

## Risk register: prevention and proving test

| Risk | Prevention | Required proving test |
| --- | --- | --- |
| Slowloris headers/body trickle | Absolute and progress deadlines, finite parser/body caps. | Duplex one-byte trickles cannot extend total deadline; idle head closes; external process watchdog. |
| Huge declared or chunked request | Early known length check plus counted actual frames and aggregate budget. | At/equal/over caps, dishonest length, chunked stream and many tiny headers; reject before excessive allocation. |
| Smuggling/ambiguous framing | Hyper parser plus reject conflicting lengths, unsupported TE, trailers and hop-by-hop ambiguity. | Raw duplex CL+TE, duplicate CL, bad chunks, folded/control headers, invalid UTF-8; no route invocation. |
| Excess accepted sockets/TLS handshakes | Acquire finite admission before spawn/handshake. | Excess duplex inputs close promptly, active task/permit counts bounded, accept failure drains. |
| HTTP keep-alive/pipelining races | Sequential same-connection dispatch and finite reuse. | Two raw pipelined requests, blocked first stream, order preserved, no concurrent second lifecycle; request count expiry. |
| Buffered origin/path regression | Share current normalize/convert routines; exact native pathname. | Existing HTTP suites plus dot/backslash/absolute URI, repeated slash, HEAD and authority allowlist tests. |
| Trusted header spoof | Ignore by default; explicit immediate-peer/hops/authority policy. | Untrusted/spoofed first/last-hop chains, mixed families, duplicates, IPv4/IPv6 boundary, mismatched host rejected/ignored as specified. |
| One-shot stream cloned/response replaced | Private synchronized take-once body; guard retained outside Response. | Two ResponseView clones, response replacement, success stream followed by error, repeated dispatch; no old bytes or second poll. |
| Error after stream publication | Final delivery report; terminate output, no lifecycle retry. | Chunk then failure/Drop -> partial body only, no JSON append, original typed cause once, one final observation. |
| Backpressure memory growth | Explicit Hyper buffer, bounded chunk and optional bridge permits. | Stalled small duplex stabilizes poll/allocation counts; oversize chunk fails; second exchange stays independent. |
| Stream Send/lifetime escape | Owned Send static stream; local renderer bridge explicit. | Compile-fail Rc across await, borrowed request/view/Invocation capture and non-Send stream; real multithread handler passes. |
| HEAD/bodyless stream executes side effects | Drop without polling; native framing explicit. | HEAD, 204/205/304 zero producer polls; correct absent/zero lengths; no GET substitution. |
| Disconnect leaves work/resources | Body/connection owner cancellation/drop, no detached producer. | Drop peer during load/handler/body, SQL future cleanup, bridge task joins; bounded deadline where immediate EOF unavailable. |
| Shutdown/admission race | Stop admissions first, graceful each connection, bounded drain then abort/reap. | Stop before accept, during accept/handler/write and repeated stop; exact last-admission boundary, no task escapes. |
| Shutdown closes DB too early | Drain HTTP tasks then await DB close while runtime alive. | Open transaction during stop, deadline cancellation, cleanup consumes permit, close re-awaitable. |
| Native/TLS error disclosure | Distinct context wrapper with original source, sanitized public output. | Source downcast survives; formatter prints each cause once; no key/password/cookie/native SQL in public response. |
| Invalid certificate/downgrade | Verify identities and CA, explicit plaintext, bounded handshakes. | Duplex Rustls handshake valid/mismatch/expired/untrusted cert, missing key, slow handshake; no insecure fallback. |
| Static traversal/symlink/file mutation | Immutable manifest + operator-owned tree; preload regular bounded verified bytes. | Traversal/percent/Unicode names, symlink/FIFO before open, wrong hash/size, absent asset, bundle aggregate; no listener needed. |
| Static HTTP/cache behavior | Exact routes/allowlisted MIME/ETag; no runtime path open. | GET/HEAD/304, independent Set-Cookie, unknown resource 404, oversized manifest/file. |
| Observability causes data leak/deadlock | Bounded metadata, synchronous callback outside locks. | Redaction assertions; observer reentry sees no registry/body mutex; document blocking callbacks cannot be preempted. |
| Pool exhaustion accumulates requests | Immediate admission and finite acquisition timeout. | max=1 held TX, admission overflow prompt, timeout vs Closed correct, cancelled waiter returns permit. |
| Uncertain native state recycled | Lease + permit retirement, close_on_drop, cleanup deadline. | Cancel before/after prepare/execute/row/BEGIN/COMMIT; fresh lease has no state/rows; cleanup-error paths retain primary. |
| TX reused after failed/cancelled run | Poison before suspension, clear only on success. | Every cancellation seam leaves RollbackOnly; run/commit reject, rollback still allowed. |
| Commit outcome falsely called rollback | Loss-of-observation contract; no retry. | Controlled completion/cancel race permits either native outcome but no duplicate write or certainty claim. |
| Transaction alias/deadlock | Exclusive &mut borrow across await, no connection mutex held. | compile-fail parallel run, Clone/consumed reuse, nested DB operation max=1 times out instead of hanging test runner. |
| Bind typing/arity drift | Same-lease native preparation, checked widths and typed NULL. | Null in each supported native type, int2/int4 boundaries, missing/extra binds, cast parameters, literal question marks. |
| Lossy codecs/native cells | Strict type matrix, output budget before owned copy. | Blob/text/JSON null, i64/u64 bounds, bool 0/1/2, NaN/Inf, duplicate columns, unsupported numeric/date/UUID types. |
| Driver packet memory exceeds our cap | Explicit limitation plus finite time/admission; no fetch_all. | Large cell/result workload with process memory observation; reject retained output without claiming native hard cap. |
| Sticky ID/row outcome | Native statement completion packet only. | Insert→update→select/DDL and concurrent pool leases, PostgreSQL RETURNING, MySQL ID overflow. |
| MySQL implicit commit/nontransactional tables | Reject TX DDL, require InnoDB generated schema, preserve capability check. | DML then attempted DDL rejected before effect; Engine plan Unsupported; external engine limitation documented, not guessed. |
| Fixture drops unrelated database | Private generated identifier, CREATE without adoption, armed cleanup. | Invalid env URL never becomes DROP target; error/unwind cleanup drops only armed fixture; cleanup failure fails test process. |
| Feature/toolchain leakage | Narrow default edges, no network in browser/core. | cargo tree normal edges; no-default/server-http/each adapter builds, native 1.96; unsupported platforms explicitly reported. |
| Panic/runtime shutdown strands cleanup | RAII retirement and explicit runtime-lifetime contract. | Worker/observer panic, outside-runtime Drop and interrupted pool close under child watchdog; no false acknowledgement. |

## Failing fast and database policy

Use Tokio paused-time for deterministic timer tests only when IO/driver clocks
permit it; use a real-time deadline for native IO. Every hang/deadlock-class
test runs in a child process with a parent watchdog: typically 10s unit/duplex,
30s backend scenario, 120s total backend fixture. Parent terminates and reaps a
stuck child. Cleanup runs separately under its own 10s deadline. A timeout is a
failure, never a skip. CPU loops and synchronous user callbacks require the
parent watchdog; an in-runtime timeout is insufficient.

Opt-in backend harness keeps the exact existing environment gate names:
`STACKPRESS_TEST_POSTGRES_URL`, `STACKPRESS_TEST_MYSQL_URL`. Unset means reported
ignored/skipped cases, never early-return passing tests. CI backend jobs must
require variables so accidental skips fail the job. Plain cargo test remains
self-contained with offline/duplex tests. No credentials in logs or receipts.

Each backend run creates one unique bounded ASCII database using CREATE without
IF NOT EXISTS, arms cleanup only after success and changes only that database.
Never list/read/alter other databases/roles/global settings, terminate unrelated
sessions or adopt/drop an existing fixture by prefix. Close all own connections
before DROP. Failure and unwind cleanup preserve failure status. SIGKILL/power
loss can leave the exact named database behind; report it for operator review,
never sweep prefixes. The design probe's two owned databases were dropped.

No listener is needed for codec/lease tests against already-running databases or
HTTP/TLS duplex tests. A later real bind/readiness/disconnect smoke test requires
explicit coordinator release and devmetrics; it is not authorized here.

## About two implementation lanes, bounded stage gates

| Stage | HTTP lane | SQL lane | Coordinator / stop |
| --- | --- | --- | --- |
| Design | These files and disposable probe only. | Same checkpoint. | Stop for owner choices. |
| M0 | Real stream extension, shared HTTP conversion and final exchange report; Send/clone/replacement/drop/duplex graceful/TLS proof. | Both real database owners, one-row typed/null bind, exclusive TX, cancelled lease cleanup and pool close. | Coordinator owns shared server/manifest/facade sequencing; stop and replay riskiest proofs before breadth. |
| M1 | Head/body/time/admission limits, sequential reuse, cancellation and bounded draining. | Complete strict codecs, native arity, statement outcomes, transaction/error/cleanup cases. | Inspect branches/race semantics and actual native results; no silent API widening. |
| M2 | TLS/proxy policy, immutable assets, observations and app main integration. | Full backend contract harness, pool stress and generated-store integration with Slice 6. | All milestone tests and >=70% per changed crate before rounds; stop. |
| Rounds 1–4 | Complexity; responsibility/abstraction; consistency/Rustdoc/comments; safety/soundness. | Same required rounds, targeted reruns. | No ceremonial edit if nothing material found; repeat targeted rounds while findings remain. |
| Closeout | Receipt, docs follow-up and separately approved managed smoke test. | Driver docs/options/errors and exact backend version/skip receipts. | Coordinator independently re-verifies; owner decides product/visual acceptance and any commit. |

HTTP lane owns `crates/server-http/**`. Shared `crates/server/**` changes land
first through the coordinator; no other lane edits them simultaneously. SQL
lane owns only `crates/sql/sql-postgres/**` and `sql-mysql/**`. Do not refactor
SQLite/core merely for code reuse during adapter implementation. Slice 6/8 source
and files remain separately owned; integration patches are sequenced explicitly.

## G23 requests and dependency budget

| Shared request | Why required / boundary |
| --- | --- |
| Server HTTP common normalize/header/framing extraction | Buffered and streaming paths must preserve identical G12/G17 and cookie authority; private refactor with existing tests. |
| Server `HttpAdapter::handle_with_options`, streaming `handle_body` | Existing HttpAdapter drops trusted adapter metadata and accepts only prebuffered input; new host needs normalized Input loader errors and cancellation context. Do not make private lifecycle internals generally public. |
| Server optional `http-stream` module and typed Response extension | Existing conversion intentionally rejects all opaque Shared values. New explicit stream recognition and one-shot conversion, with existing buffered rejection unchanged. |
| Server report integration if needed | New delivery report distinguishes body/connection completion from native publication. Prefer network crate-owned report; any HandleReport method change must return to coordinator before editing. |
| Root Cargo members/dependencies/lock and facade exports/features | Add `server-http`, `sql-postgres`, `sql-mysql`; preserve empty default and no browser network features. Coordinator is sole writer. |
| Lib / CLI / Idea / SQL core | None requested. Existing SharedNative and Database contracts suffice, subject to M0 proof. Any contrary compiler/cancellation finding is a new shared request, not license to edit. |

Network budget: pinned assessed Hyper/hyper-util, bytes, http-body-util,
futures-core, existing Tokio (net/io-util/sync/time/signal only in app), and
optional Rustls `=0.23.46` / tokio-rustls `=0.26.6` (defaults off,
`ring,tls12`). Assessed MSRV/licenses are in the library assessment. TLS
transitive resolution and certificate-fixture tooling need coordinator
confirmation in M0; tokio-rustls was not installed/probed here. Test-only
trybuild/tempfile and current JSON/testing utilities; no Axum/Tower production
edge. Database budget: SQLx 0.9.0 runtime-tokio + per-backend + rustls-ring +
JSON, existing Tokio and serde/serde_json. No macros/migrate/Any/chrono/decimal.

## Required release evidence

After real implementation, clean fresh build/coverage targets, then run affected
fmt/tests/doctests/Clippy/Rustdoc and scoped feature builds. Workspace gates use
`--locked`; document all feature selections and enabled backend variables without
their values. Review normal dependency edges and public API changes. Measure
>=70% production executable lines for each new/changed crate separately; do not
inflate SQL coverage with skipped native tests or blend server and driver totals.
Publication remains >=90% plus supported platform/feature evidence, not granted
by this development checkpoint.

Rerun the Agent Workspace validator for this spec write. Its link/structure
checks do not prove Cargo behavior. Stop after design with unresolved owner
choices and exact probe limitations; no task status/verification promotion.
