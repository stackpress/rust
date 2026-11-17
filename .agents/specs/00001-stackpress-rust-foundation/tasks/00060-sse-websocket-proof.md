# Task 00060: dependencies, inventory and proof plan

Companion to the [design checkpoint](00060-sse-websocket-design.md); load with the [proposed APIs](00060-sse-websocket-api.md) when checking feasibility and completeness. Everything below is a planned proof unless marked as read-only evidence.

## Dependency choices and exact root-lock additions

Read-only evidence: current root has **699** registry tuples. Blog has **448**, including exactly the six extras in [G58's lock checker](../../../../tools/examples/blog/locks.py) and its [receipt](00046-blog-dev-loop.md). Installed Cargo sources contain tungstenite and tokio-tungstenite **0.29.0**; no fetch was used.

Propose native optional `tokio-tungstenite = { version = "=0.29.0", default-features = false, features = ["handshake"] }` and direct optional `tungstenite = { version = "=0.29.0", default-features = false, features = ["handshake"] }` in server-http `websocket`. The direct edge supplies the handshake generator/config/message conversion; it resolves to the same crate version as the async adapter. Use optional `futures-util = "=0.3.34"`, defaults off, `sink` and `std`; reuse existing futures-core 0.3.34, http 1.5.0, Hyper 1.10.1, hyper-util 0.1.20 and Tokio 1.54.1. Strict key decoding may directly reuse root-locked base64 0.22.1, defaults off with `std`.

No tungstenite `connect`, native-tls, rustls connector, URL or compression feature. Hyper's existing Rustls host terminates TLS. Async adaptation avoids synchronous `Read`/`Write` on a Tokio worker; tungstenite alone is insufficient. Existing Leptos and wasm-bindgen 0.2.129 / js-sys 0.3.106 / web-sys 0.3.106 need feature changes only, no new versions. SSE needs no new registry tuple.

The proposed closure adds exactly these **seven** `(name, version, source, checksum)` tuples. Source for every row is `registry+https://github.com/rust-lang/crates.io-index`. Six identities/checksums come from the blog lock; the tokio-tungstenite checksum matches both the cached registry index entry and SHA-256 of its cached `.crate` archive.

| Name | Version | SHA-256 checksum | Cached manifest licence |
| --- | --- | --- | --- |
| data-encoding | 2.11.1 | `4583a4551df46e2792f82ceeac45e850d2e2d5debba0b91f102385cda5b11f06` | MIT |
| rand | 0.9.4 | `44c5af06bb1b7d3216d91932aed5265164bf384dc89cd6ba05cf59a35f5f76ea` | MIT OR Apache-2.0 |
| rand_chacha | 0.9.0 | `d3022b5f1df60f26e1ffddd6c66e8aa15de382ae63b3a0c1bfc0e4d3e3f325cb` | MIT OR Apache-2.0 |
| rand_core | 0.9.3 | `99d9a13982dcf210057a8a78572b2217b667c3beacbf3a0d8b454f6f82837d38` | MIT OR Apache-2.0 |
| sha1 | 0.10.6 | `e3bf829a2d51ab4a5ddf1352d8470c140cadc8301b2ae1789db023f01cedd6ba` | MIT OR Apache-2.0 |
| tungstenite | 0.29.0 | `6c01152af293afb9c7c2a57e4b559c5620b421f6d133261c60dd2d0cdb38e6b8` | MIT OR Apache-2.0 |
| tokio-tungstenite | 0.29.0 | `8f72a05e828585856dacd553fba484c242c46e391fb0e58917c942ee9202915c` | MIT |

Root sha1 is **0.11.0**, so blog sha1 0.10.6 is still a new root tuple. Existing digest 0.10.7, block-buffer 0.10.4, crypto-common 0.1.7, getrandom 0.3.4, ppv-lite86, zerocopy, futures-sink 0.3.34, log 0.4.34, httparse 1.10.1 and thiserror 2.0.21 cover the remaining closure. Preserve all 699 prior tuples; projected total is **706**, subject to an approved one-time offline resolution, not proved by a resolver in this checkpoint. Seed exact blog versions rather than let a cached newer sha1 0.10.7 drift in.

Both licence expressions in the new rows already exist in [the G62 SPDX snapshot](../../../../tools/release/dependency-licenses.toml); read all seven cached manifests and matched their exact strings. No new expression/owner licensing choice or first-party licence change is proposed. Later run the dependency-licence gate and refresh attribution evidence as required; this read-only comparison is not a full package/license audit.

After O6 approval, coordinator updates root lock and every invariant that freezes 699 tuples. The [blog checker](../../../../tools/examples/blog/locks.py) currently asserts six extras and 448 tuples; simply adding root dependencies will make its equality assertion false. Replace that historical exception with an approved root-subset check, preserve exact dev-only handshake dependency/feature assertions, re-seed/recheck the nested lock by the authorized offline procedure, and update controls/baselines/receipts that refer to the old count. Do not edit historical G58 acceptance as if it never existed.

Then G58's **blog-only tuple-placement exception** can retire because all six extras are root members. Its authorization of the dev-only reload implementation remains historical; production use comes from O6. Blog xtask need not migrate to the new production API. A changed root dependency is not permission for unrelated upgrades, new tuples, installs or relaxed lock checking.

## Feature and package graph proofs

Native `server-http/websocket` enables only the generic server `http-upgrade` seam and optional async/protocol dependencies. Server has no dependency on server-http; the facade only forwards optional features. Keep native HTTP host edges under non-wasm target dependencies, and reject/document selecting native host features for browser WASM rather than pulling them into Leptos. Native renderer tests select SSR separately from hydrate/CSR.

Prove default server, SSE-only, WS-only, SSE+WS, each with/without native host TLS, facade forwarding, standalone release packages, and independent Leptos consumers. Browser SSE/WS features activate target-wasm web-sys capabilities without native host edges. Native SSR plus client-helper features remains inert. Use normal-edge cargo trees/consumer builds, not presence of `cfg` annotations alone, as graph evidence.

## Relevant source inventory and dispositions

The existing [HTTP upstream inventory](../slice-7-design-inventory-http.md) maps transport exports and test families; the [source transport reference](../../../references/00020-upstream-server-and-transport-contracts.md) records the imported contracts. These are KB references, not newly inspected sibling checkouts. Current Rust source has no production SSE encoder, EventSource helper or WS endpoint. The only WS implementation found is the blog dev-only reload channel. Do not invent upstream SSE/WS APIs or a parity claim.

| Export/type/behavior | Disposition and reason / proof |
| --- | --- |
| Node/WHATWG `gateway`, `handler`, Adapter, dispatcher, Node/Web stream conversion | Retain the Rust HttpServer/StreamingHttpAdapter counterparts; extend explicit streaming rather than raw Node socket/native Response passthrough. Existing conversion/lifecycle tests remain. |
| `ServerBuilder::get`, `StartupContext::get`, plugin registration, ordered routes | Reuse now for both protocols; duplicate accept/deny/replace/Abort tests pin G42. No route uniqueness registry. |
| Request/RequestView, Session, Invocation, typed config/services | Retain; add explicit bounded WS snapshot/service captures. Existing cloning, cookie/default Path, Date/redirect and strict native-value tests remain. |
| HttpResponseExt::stream/immutable_bytes, StreamLimits, HttpBody, StreamingHttpOutput/Reply | Retain published signatures/layouts. Add separate SSE extension and policy getters; sibling native-output reply types preserve enum exhaustiveness. One-shot, HEAD/bodyless, replacement and error regression suite. |
| NetworkLimits, HttpRequestContext, HttpDelivery, observers/reports | Retain layouts/variants/defaults. Add separate validated policy, connection report/accessors; prove old consumers compile and ordinary limits still hold. |
| SharedIo/Monitor/TrackedBody/ConnectionCancellation; supervisor serve_io/serve_acceptor/serve_until | Extend private ownership for SSE and WS; new native output path must not detach tasks or inherit premature body cancellation. Watchdog/read-ahead/stop proofs. |
| TLS/proxy/static assets; Hyper HTTP/1 parser | Retain existing behavior; SSE/WS traverse existing TLS/trust policy. Assets/ordinary pipelining regressions. No new parser/Axum. |
| Blog tungstenite accept/read/write reload implementation | Retain as dev-only; dependency/lock integration now, migration to the production endpoint is a later optional task. |
| Leptos render/stream and browser consumers | Retain SSR/hydrate/CSR separation; add independent client-sse/client-websocket helpers in M3. SSR HTML streaming is not SSE. |
| New SseEvent/SseResponseExt/SseDriver, WS intent/connection, browser handles | Implement M1/M2/M3 per checkpoint; these are new contracts with the explicit tests below, not source ports. |
| HTTP/2 extended CONNECT, arbitrary upgrades/tunnels, WS compression | Intentionally not implemented in this slice; G28 stays HTTP/1.1 and these need separate bounds/protocol proofs. |
| Automatic message-event routing, rooms/broadcast, durable replay, offline sync | Deferred named follow-up `00060-follow-up-realtime-app-protocol`; requires owner scope and concrete payload/authorization/delivery contracts, no implicit task approval. |

## Unit and adapter tests

SSE: exact wire bytes for empty/multiline data, CRLF/bare CR, trailing blank lines, Unicode, comments, id reset, named/default event, retry zero/max, single-line CR/LF/NUL injection, duplicate Last-Event-ID, field/event byte boundaries and arithmetic overflow. Assert rejected input produces no partial event. Count prefixes and UTF-8 bytes, not characters. Cover finite source EOF/error, post-error fused EOF and body replacement/drop with zero producer polls.

Clock-controlled driver tests: initial comment, quiet heartbeat, ready-event tie, no catch-up burst, byte accounting, no heartbeat after EOF/stop, max-age expiration, 60/300-second ordinary vs realtime boundaries, actual-write vs producer progress. Use injected/manual time, not hour-long sleeps. `text/event-stream` alone must not select realtime policy.

Handshake decision table: HTTP version/method; mixed-case token matching across repeated Connection fields; missing/duplicate/invalid Upgrade/Version/Key; non-base64 or decoded lengths 0/15/16/17; offered/selected/unoffered subprotocol; unsupported extension offers remain unnegotiated; body/TE/CONNECT/other-upgrade/oversized headers. Standard known Key/Accept vector proves response generation. Test unsupported version 426 and otherwise empty sanitized errors with no route callback.

Adapter/lifecycle: normalized request hooks and configured session/CSRF/app-Origin check before acceptance; denial, absent config per G43, missing endpoint, handler priority/FIFO, Abort, two intents, replacement in response hook, error recovery and cookie revisions before 101. Exactly one native output attempt; invalid headers/status/foreign finalization never run the connection callback. Plain buffered adapter rejects intents without collecting/polling/opening anything. Preserve G57 Dates and redirect bodies and all existing HEAD/204/205/304 stream behavior.

WS driver: Text/Binary round-trip; masking/RSV/opcode/control frames; fragmented message and aggregate-limit failure; invalid UTF-8; ping/pong without application reads; unmatched Pong/unsolicited frames do not satisfy an outstanding ping; queue-count and byte boundaries, full inbound 1013 and bounded outbound wait, withheld peer reads, queue-permit return. Validate every outgoing close code/reason boundary, invalid config inequality and duration overflow without panicking.

## Real-host and browser proofs

Follow [tcp_watchdog](../../../../crates/server/http/tests/tcp.rs) and [its kill/reap guard](../../../../crates/server/http/tests/support/watchdog.rs): a parent process starts one ignored child with a <=10–20-second deadline; child binds only loopback `127.0.0.1:0`, owns its endpoint, and drops/reaps it on every exit. This is a finite required test-listener pattern, not a dev server. Use in-memory duplex IO for most branch/slow-writer tests. No listeners run at this checkpoint.

Real SSE tests read an event before source EOF, resume with Last-Event-ID, observe automatic heartbeat and byte/age rotation, disconnect/stop, enforce cap+1 refusal, and prove post-stop port closure. Slow peer must release its permit within idle/grace. Direct TLS tests use existing packaged fixture certs. Assert source Drop, cancellation and active counts reach zero, with normal HTTP health still serviceable under realtime caps.

Real WS tests send handshake and first masked frame in **one write** to prove Hyper read-ahead survives handoff; exchange frames over TCP and TLS, verify 101 cookies/subprotocol and no second lifecycle. Withhold Pong, stop reading, fill queues, exceed message/frame limits and race peer close/host stop/admission/failed 101. Assert close 1001 before cooperative stop, forced close after finite grace, one terminal observation, no permit leak/detached callback and closed listening endpoint. A positive-control stuck child proves watchdog kill/reap itself.

Use cached Playwright/Chromium with separate hydrate and CSR builds. Read-only inspection confirmed Chromium/headless-shell caches under `~/Library/Caches/ms-playwright`; executable/module compatibility remains a future proof. Native EventSource/WebSocket require a real streaming/upgrade fixture: the blog's buffered stdin/stdout interception proof alone is insufficient. Run it as a finite owned loopback child under the same watchdog discipline, or a short devmetrics-managed preview if doing interactive inspection; no bare persistent server. No browser install/download.

Browser cases: native SSR creates zero connections; hydration mounts once; SSE text/named-event/id/retry/reconnect including header resume; WS Text/ArrayBuffer/open/error/close, explicit reconnect without resend and bufferedAmount refusal. Dispose during open/connect/retry, remount and change URL; assert listeners/closures/timers/connections are released and no callbacks occur after disposal. Exercise app Origin allow/deny and cookie session behavior; browser hiding is not server authorization. Use bounded fixture data and per-case deadlines.

Autobahn is optional O8: a pinned client fuzzing suite against the finite host can add RFC 6455 conformance evidence. It requires installing a Python package/environment or using Docker/image tooling; neither is authorized here or assumed available. If approved later, pin the harness, bound run/connection/message budgets and preserve its report; passing it would not prove app security or every platform.

## Implementer risk inventory and proving tests

| Risk | Prevention and decisive proof |
| --- | --- |
| Two consumers/output attempts or callback after rejection | Private take-once descriptor + OutputGuard; cloned views, replacement, error, invalid headers and failed publication execute callback at most once. |
| Upgrade loses initial bytes or cancels on 101 body Drop | Keep Hyper Upgraded read buffer; separate HTTP-body/realtime ownership; handshake+frame single-write and callback survives handshake flush tests. |
| Deadlock/unbounded task on full queues or stop | Driver owns protocol control independently, bounded waits and absolute reap; blocked app send/recv, ignored close and process watchdog tests. |
| Quiet SSE killed by old clocks / fake progress keeps slow peer alive | Explicit selected realtime clock and native-write idle; manual-clock boundary plus stalled duplex/TCP tests. |
| Permit or memory leak on races | Reserve before commit, scoped count/byte permits, bounded queues/snapshots; cap+1, repeated connect/drop/stop and counter-zero assertions. |
| Cross-site session misuse / stale authorization | App-selected Origin/auth and per-message checks; allowed/denied origins and expired-session app-policy tests; no false global-security guarantee. |
| Malformed input reaches dependency assertion/allocation | Validate config and bounded headers/frames before delegation; invalid config, huge lengths/fragmentation, UTF-8 and close reason regressions under strict lints. |
| Accidental API break or native graph in WASM | New sibling types/features, frozen API consumer fixtures and normal-edge per-target graphs; stop for semver decision on incompatibility. |
| Dependency/lock drift or false conformance claim | Enumerated seven-tuple ceiling, old tuple equality, licence gate and recorded target limits; fail the resolver proof on any unexpected tuple. |

## Verification process and evidence limits

For each changed release crate, require **>=90% executable line coverage** in its declared relevant native feature scope, including failure/timeout/close branches. Record numerator/denominator/report, compiler/target/features and visible uncovered risks. Extract retry, send-size/retention and disposal decisions into portable modules for native measurement. Facade uses the G61 feature/consumer matrix; meaningful browser-helper logic is not hidden behind the thin-entry G56 exemption. Browser DOM adapters need real hydrate/CSR proofs and explicit native-measurement limitations, not a claim that WASM was covered by LLVM.

Run narrow nextest proofs through the recorder during development; milestone gates include formatting, strict per-crate/workspace Clippy, nextest (watchdogs separately as required), doctests, warning-clean Rustdoc, graph/consumer/package checks and appropriate release integration. Preserve packaged-test boundaries: fixtures within each published crate, only earlier-published internal dev-dependencies, and updated copy-provenance/freshness checks when applicable.

Implementation rounds and coordinator re-verification remain separate from owner acceptance/release. No runtime tests, Cargo resolution/build, browser/Autobahn run, coverage measurement, installation, network access or listener was performed for this design. Current evidence is source/manifest/cache inspection and identical-before/after KB diagnostics only.
