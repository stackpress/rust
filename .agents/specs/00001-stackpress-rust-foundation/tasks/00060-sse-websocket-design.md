# Task 00060: SSE and WebSocket design checkpoint

Design draft, 2026-11-17, branch `design/sse-websocket`. Coordinator review and owner decisions are pending. This checkpoint authorizes no implementation or release. Acceptance criteria: none; this is a library design review.

## Task summary

Add server-sent events for notifications and WebSockets for two-way connections to the existing Hyper host. Keep Stackpress routes, ordered handlers and configured security checks. Applications still decide who may connect and what each message may do.

Recommend explicit, bounded sessions: rotate connections after at most one hour, cap their number, and disconnect slow clients. On stop, finish the current SSE event and close promptly; send WebSockets a going-away close frame and wait only within the existing shutdown grace. Browser helpers belong in the independent Leptos library.

This can be an additive `0.2.x` release if existing public types and defaults remain compatible. Seven exact dependency tuples need new owner approval. The blog-only G58 exception cannot authorize production WebSockets.

- [API and execution design](00060-sse-websocket-api.md): load for SSE encoding, handshake, one-shot publication, connection ownership, security and Leptos signatures. All new signatures are proposals.
- [Dependency inventory and proof plan](00060-sse-websocket-proof.md): load for exact versions/checksums/licences, upstream dispositions, tests, risk proofs and evidence limits.

## Authority and current behavior

Read the [runtime contract](../../../context/runtime-and-portability.md), [project direction](../../../context/project-direction.md), [Rust style](../../../context/rust-code-style.md), [quality policy](../../../context/testing-and-quality.md), [coordinated stage gates](../../../workflows/coordinated-crate-development.md) and [decision register](../decisions.md), specifically G28, G42/G43, G57/G58 and G62. These govern this proposal.

The [existing streaming implementation](../../../../crates/server/server/src/http/stream.rs) takes one owned `Send + 'static` producer after the lifecycle. `StreamLimits` defaults to 64 KiB/chunk, 64 MiB total, 60 seconds total and 10 seconds idle. In-process callers enforce time themselves. Buffered adapters never collect the private native descriptor.

The [native limits](../../../../crates/server/http/src/options.rs) allow 256 connections, 32 TLS handshakes, 300-second connection lifetime, 60-second response lifetime and 30-second shutdown grace. [Output monitoring](../../../../crates/server/http/src/monitor.rs) currently takes the minimum of stream and host response limits; increasing only a handler's limit cannot create SSE. Quiet SSE also conflicts with the existing 10-second write-idle budget.

[Head validation](../../../../crates/server/http/src/input.rs) rejects every Upgrade header, CONNECT and trailers before routing. [Connection ownership](../../../../crates/server/http/src/connection.rs) drives Hyper HTTP/1 without upgrade hand-off; its cancellation guard and [body drop](../../../../crates/server/http/src/body.rs) currently end the request's ownership. The ordinary streaming converter rejects 101 and strips hop headers. WebSockets therefore need a deliberate new output path, beyond changing head validation.

G28 retains Hyper as the HTTP parser, HTTP/1.1, explicit streaming, finite deadlines and opt-in Rustls. This proposal amends only its deferred long-lived-stream and WebSocket policies. G42 preserves all-matching priority/FIFO dispatch. G43 adds no global Origin/auth/CSRF guard. G57's HTTP Date projection and redirect-body behavior stay under regression tests. G62 governs fallible input/configuration, bounded work and licence review.

## Proposed host policy and limits

Add separate, validated `SseHostLimits` and `WebSocketHostLimits`, selected through new `HttpServer::with_sse_limits` / `with_websocket_limits` methods. Compile-time features alone do not admit long-lived traffic: absent host policy rejects an intent before publication with a sanitized 503. Host defaults continue to reject upgrades when WebSockets are not enabled.

Do not add fields to exhaustive `NetworkLimits`, `StreamLimits`, `HttpRequestContext`, reports or existing public enums. Give new policy types private fields with fallible builders. New counters/report getters and optional observers expose active SSE/WS, rejected admission, bytes, close outcome and forced shutdown without changing old report shapes or logging session/header/message contents.

| Policy | Proposed default and boundary |
| --- | --- |
| Admission | At most 64 SSE and 64 WS sessions per host, including pending publication/handoff; both also hold the existing global connection permit until final cleanup. No waiting admission queue. |
| Configuration | Each cap must fit the global cap; combined caps must leave a nonzero ordinary-HTTP allowance. Validate the complete chosen policy before serving, independent of builder call order. |
| Session age | 3,600 seconds from realtime publication, capped by host policy; handlers may request less. Always finite, never zero-as-unlimited. |
| SSE event / session | 64 KiB encoded event including fields and terminator; 64 MiB session bytes including heartbeats. EOF rotates the connection; applications own replay. |
| SSE heartbeat / idle | Comment heartbeat after 15 seconds without an event; actual native-write idle expires at 30 seconds. Host rejects heartbeat >= idle. |
| WS message / frame | 64 KiB reassembled message and 64 KiB incoming frame payload; outgoing messages also <=64 KiB. Enforce fragmented-message aggregate, text UTF-8 and control-frame rules. |
| WS buffers | 16 KiB read and write target; 256 KiB maximum tungstenite write buffer. Explicit inbound/outbound queues each <=16 messages and <=1 MiB, charged before enqueue. |
| WS timing | Ping every 20 seconds when no ping is outstanding; matching Pong within 10 seconds; 60-second inbound-idle ceiling; send/queue wait <=10 seconds. Pre-upgrade application work retains the 30-second lifecycle deadline. |
| Close | Normal/application close handshake <=5 seconds. Host stop uses the lesser of this budget and remaining host grace, default grace 30 seconds. |

Reserve protocol slots only after bounded validation and before publishing headers; a full slot returns 503 with no callback or producer polling. Release on replacement, denial, failed conversion/101, failed upgrade, EOF, disconnect, timeout and stop. Slots must not be retained by a discarded `ResponseView` clone.

At explicit realtime commitment, replace the ordinary output and remaining connection-lifetime clocks with the selected finite session clock; otherwise the old 60/300-second limits would still win. Ordinary request parsing, loading, lifecycle, response and keep-alive clocks retain their current meanings. Before commitment the existing clocks apply. Stop/grace always wins over session age. This is an explicit G28 amendment, not a very large duration workaround.

SSE remains an HTTP response occupying its connection; do not dispatch pipelined successors until it ends and flushes. WS ends HTTP reuse permanently. Track upgraded IO separately so a 101 body's EOF/flush cannot release its permit or cancel its connection. A successful local write/close is not remote delivery acknowledgement.

## Stop and cancellation

Close admission first and use one absolute drain deadline across HTTP, SSE, upgrade handoffs and WS tasks. Stop wins a race with pending publication; no new realtime callback starts after stop is observed. All owned tasks are aborted and reaped if grace expires; dropping the serving future also drops/aborts its children.

For SSE, stop polling new application events immediately, finish at most the current complete encoded event/heartbeat if writable, send HTTP EOF and close the connection. An optional application shutdown event is not guaranteed. Do not wait the whole grace for another producer item. If write progress stalls, grace/idle forces a drop. The client may reconnect elsewhere; replay/duplicates remain application concerns.

For WS, stop accepting new application sends, prioritize one Close(1001, "server stopping"), continue protocol reads/flush long enough for the peer's Close, then drop IO. Never wait for an uncooperative callback to finish before sending Close. Cancel and reap it at the same absolute deadline. No detached upgrade or heartbeat task survives return from `serve_until`/`serve_io`.

Session age also rotates SSE with EOF and WS with 1001. Malformed traffic, overload and transport failure may end immediately when a close frame cannot be safely sent. Async deadlines cannot preempt blocking user code; preserve the existing limitation and use process watchdogs in tests.

## Semver and release impact

Follow [release versioning](../../../workflows/release-versioning.md). Recommend an additive lockstep `0.2.x` patch: new optional features, extension traits, methods and types, keeping existing signatures, trait requirements, exhaustive layouts, default features and MSRV 1.96. Do not add a required method to `HttpResponseExt` or a variant to `StreamingHttpOutput`/`HttpDelivery`.

Accepting a valid Upgrade on an enabled endpoint changes observable behavior from 400 to 101. Record that in changelog and migration guidance, together with explicit activation, Origin/session responsibilities, proxy support, reconnect/replay and finite limits. The release policy says observable behavior change alone is not a semver break; feature-off behavior remains the existing rejection.

If M0 cannot retain these API constraints, stop and propose `0.3.0` before implementing a breaking change. Owner review chooses the version after the final public-API diff across all 23 packages. Actual release work later updates lockstep versions/internal requirements, generated version output, changelog, migration guide and READMEs, then runs the release gate; publication remains owner-only.

## Implementation steps and milestone stops

Each milestone is released separately by the coordinator after owner choices. Coordinator owns shared manifest/lock/facade edits; this checkpoint edits only its three documents.

| Milestone | Deliverable and stop proof |
| --- | --- |
| M0: streaming policy and output seam | Additive ownership/API probe; compile-fail borrowed/non-Send captures; ordinary response/stream behavior unchanged; simulated clocks prove SSE survives old 60/300-second boundaries but expires at new age; slot replacement/drop, grace races and native output guard proven. Stop before expanding. |
| M1: SSE | Encoder, headers, bounded source, native heartbeat/idle/age/byte policy, resume-header access and finite in-process driver. Unit and finite loopback/TLS slow-client/stop proofs; no buffered collection. Stop with >=90% changed-crate coverage. |
| M2: WS | Handshake, lifecycle-preserving output, read-ahead-safe handoff, owned connection/queues, close/ping/deadlines and admission. Negative/header/framing/fragmentation/stop proofs over duplex and loopback/TLS; disabled-feature graphs. Stop with >=90% changed-crate coverage. |
| M3: browser and docs | Independent Leptos SSE/WS helpers, separate hydrate/CSR browser proofs, native SSR inertness, facade/consumer graphs, runnable documentation examples and release notes. Update approved lock checks and retire only the superseded G58 tuple exception. Stop for coordinator review. |
| Closeout | Three test-verified rounds: complexity/branch risk, responsibility/ownership, consistency/Rustdoc/comments; then a dedicated safety/soundness round. Repeat material findings only; final relevant workspace/docs/feature/coverage/package checks. Owner decides release. |

Use the [fast test workflow](../../../workflows/fast-build-and-test.md) and [benchmark recorder](../../../workflows/build-benchmarks.md) for future proofs. No build, runtime proof or coverage was performed for this checkpoint.

## Exactly the owner decisions needed

| ID | Plain-language choice | Recommendation / alternative |
| --- | --- | --- |
| O1 | May SSE/WS use a separate one-hour session clock instead of today's short HTTP clocks? | Yes, explicitly enabled and capped as above. Alternative: retain 60-second responses/300-second connections and reconnect more often; indefinite sessions are not proposed. |
| O2 | Are the proposed connection, byte, queue and timeout defaults suitable? | Accept the table. Owner may choose concrete different finite values; no automatic load tuning. |
| O3 | How should open SSE streams behave when the server stops? | Finish only the current event, then EOF within grace. Alternative: let the application continue producing throughout grace, causing longer shutdown and requiring additional tests. WS sends Close then aborts at grace in either option. |
| O4 | Should endpoints use normal ordered routes and configured security before upgrade? | Yes: normal request/route/response stages, application-owned Origin/auth/CSRF checks, no implicit per-message guards. Alternative: a new globally protected route model changes G42/G43 and needs a separate design. |
| O5 | Should WS expose an owned send/receive connection or automatically emit each message as a framework event? | Connection API now; applications can explicitly delegate. Automatic event mapping is deferred and would need payload, error and abort semantics. |
| O6 | Approve the seven exact dependency tuples for production and retire the blog-only tuple exception after integration? | Approve the enumerated closure, minimal features and revised lock proofs. Alternative: defer WS; do not hand-write a frame engine or use synchronous tungstenite on async IO. |
| O7 | Should browser wrappers ship in independent `stackpress-leptos` under separate SSE/WS features? | Yes; native SSR opens nothing. SSE uses EventSource reconnect; WS reconnect is explicit and off by default. Alternative: server-only first, client helpers as a named later milestone. |
| O8 | Install/run Autobahn conformance tooling? | Optional later gate; no Python/Docker package installation in this task. Approve a pinned finite harness if desired; core release proofs do not depend on it. |

These are review questions, not requests to execute an installation or release now. O1/O3 amend G28; O4/O5 must be recorded without weakening G42/G43; O6 supersedes only G58's lock-placement constraint. No new decision is inferred as accepted.

## Implementation notes

Read-only exploration used this branch's KB, source/manifests/tests, root/blog locks and installed Cargo registry sources/index/archive. No other branch or worktree, network, install, listener, build, Rust source, manifest or lock edit was used. New APIs/defaults and seven-tuple root closure are design conclusions; an offline resolver/build has not proved them.

The three files form one checkpoint, split by owner decisions, API/execution and dependency/proof retrieval. All requirements in the task brief are represented. No accepted context, decision ledger, task status, sprint or unrelated file is rewritten.

## Verification notes

Implementer: `python3 .agents/scripts/validate-agent-workspace.py` before and after drafting has the identical inherited baseline: **146 errors / 44 warnings**, exit 1, with no added or removed diagnostic. The errors are external sibling paths in imported references. This documentation validation is not code verification. Coordinator verification is pending.

## Acceptance notes

Acceptance criteria: none. Stop here for coordinator review and owner decisions O1–O8; no M0 implementation is authorized by this note. No context promotion, commit, push or publication occurred.
