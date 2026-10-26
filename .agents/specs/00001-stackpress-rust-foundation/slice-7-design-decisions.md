# Slice 7 design decisions and recommendations

Design checkpoint for [task 00012](tasks/00012-slice-7-production-servers.md),
2026-10-26, `trial/3` at `61457c9`. This is a proposal, not M0 approval,
implementation evidence, a Frozen spec, or production readiness. Inherited
[G06, G09, G12–G17 and G23–G26](decisions.md) remain binding.

## Read by concern

- [HTTP inventory](slice-7-design-inventory-http.md): load for upstream exports,
  adapter/helper tests and operational recipes, including deliberate deviations.
- [SQL inventory](slice-7-design-inventory-sql.md): load for connector exports,
  individual upstream test dispositions and existing Rust contract obligations.
- [Library assessment](slice-7-design-assessment.md): load for versions, licenses,
  MSRV, candidate comparisons, disposable probe receipts and evidence limits.
- [HTTP contracts](slice-7-design-http.md): load for streaming ownership,
  resource limits, shutdown, static assets, TLS, proxy trust and observability.
- [SQL contracts](slice-7-design-sql.md): load for leases, cancellation, codecs,
  backend differences, TLS and configuration.
- [Public API and DX](slice-7-design-api.md): load for the proposed production
  `main()`, streaming page interface and config-only database selection.
- [Risks and delivery](slice-7-design-verification.md): load for prevention and
  proving tests, two-lane ownership, G23 requests and stop gates.

## Choices for the owner

Every row is unresolved until the owner/coordinator records its answer. Questions
are written without requiring the owner to know the internal identifiers.

| ID / authority | Plain-language choice | Recommendation | Alternative and consequence |
| --- | --- | --- | --- |
| S7-D01 / owner | Should the production web host use Hyper directly, or adopt Axum as another application framework? | Hyper behind a small Stackpress host. Keep Stackpress's ordered, all-matching listeners and lifecycle as the only application dispatcher. | Axum as one fallback service can work, but adds an unused router/extractor layer. Axum route registration itself cannot replace all-matching dispatch. Writing our own HTTP parser is vetoed: substantially larger protocol/security burden. |
| S7-D02 / owner | Should the first host speak HTTP/1.1 only, with HTTP/2 handled by a reverse proxy, or also implement HTTP/2 now? | HTTP/1.1, bounded keep-alive and sequential pipelined requests; no upgrades, CONNECT tunnel or WebSocket API. Add HTTP/2 as named follow-up S7-H2. | HTTP/2 now needs independent stream/window/concurrency, shutdown and TLS/ALPN tests before release. It is feasible but expands the two-lane plan. |
| S7-D03 / owner | Should streaming be an explicit HTTP response capability while existing buffered responses keep their current contract? | Yes. A one-shot stream owned by the final Response, published only after lifecycle completion. Record network completion separately from buffered publication. | General streaming in foundation Request/Response would change non-HTTP consumers and G07 cloning contracts; requires a separate foundation design. Never hide streams in a request-global side table. |
| S7-D04 / owner | Should this slice include direct HTTPS as well as deployment behind a TLS proxy? | Both: opt-in Rustls HTTPS with certificate/key files, and explicit proxy deployment. Plain HTTP is an intentional listener choice; forwarded headers are ignored by default. | Proxy-only first reduces scope, but direct TLS is then explicitly deferred rather than implied by a TLS dependency. Automatic certificate issuance/reload stays S7-Ops. |
| S7-D05 / owner | Should static files be an explicit, startup-verified asset bundle or a runtime directory server? | Immutable manifest allowlist, finite total size, GET/HEAD, ETag/304 and cache policy. Preserve the example's checked asset identities; no request-controlled filesystem lookup. | Runtime files/ranges/precompressed negotiation need separate race-safe file ownership and bounded file-stream design; defer as S7-Files. |
| S7-D06 / owner | Which network database drivers should implement the existing SQL contract? | SQLx `=0.9.0` for both PostgreSQL and MySQL, matching SQLite's lease/admission model. | tokio-postgres and mysql_async are viable native clients but require different pooling/driver-task and cleanup integrations. They have no executed probe here because they are not cached and installs are prohibited. |
| S7-D07 / owner | Should database results retain the current small, strict value set, or add automatic date/decimal/UUID conversions? | Retain strict values; support current generated SQL types, JSON as validated text, DateTime as the current VARCHAR UTC text. Reject unsupported native types clearly. | Advanced native codecs belong to S4-X. Converting native DECIMAL to float or an unsigned value outside i64 is not acceptable. This means some existing external schemas need explicit casts. |
| S7-D08 / owner | Should production defaults use finite per-request and stream deadlines even when a client remains connected? | Yes; configurable finite defaults below. Long-lived SSE is a separately chosen future policy, not an accidental unlimited stream. | An explicit long-lived stream mode needs its own heartbeat, retained-resource and shutdown policy. Never use zero to mean unlimited. |
| S7-D09 / owner | Should transport rejections have HTTP-specific statuses while normalized framework input keeps its accepted lifecycle? | Yes: 408 deadline, 413 body bytes, 431 headers and 503 admission where a response can safely be sent. Malformed normalization remains G17's sanitized 400 with no callbacks. | Keep every transport rejection at 400 for compatibility with the buffered adapter. The recommended statuses apply to the new network boundary only; existing buffered behavior is unchanged. |
| S7-D10 / coordinator | Accept the crate and dependency boundary? | New optional `stackpress-server-http` at `crates/server-http`, two SQL adapter crates, optional server `http-stream` feature and coordinator-owned facade/lock changes. | Keeping socket/runtime code in `stackpress-server` is possible behind a feature, but would couple the host crate more tightly to Tokio/Hyper. No dependency lands at this checkpoint. |

## Cross-slice dependencies and approval boundary

- Slice 6 stores consume `Arc<Engine>` and existing `Database` /
  `TransactionSession`; they must not depend on SQLx or acquire pool connections.
  Its migration planner must distinguish MySQL implicit DDL commit from ordinary
  transactional DML. This slice does not change migration semantics silently.
- Slice 8 streamed SSR consumes the proposed `HttpResponseExt::stream` interface:
  owned `Send + 'static` chunks after the Send loader/lifecycle, cancellation by
  dropping the stream, backpressure by demand. A local Leptos renderer needs a
  bounded bridge and supervised local executor; Send handler bounds stay intact.
- G23 requires coordinator sequencing for every server/http edit. Slice 8 must
  request this interface through the coordinator; no concurrent shared edits.
- PGlite/serverless remain S4-H. Auth, sessions, CSRF, application authorization,
  migrations and generated stores remain their owning slices. A public listener
  never makes every event remotely callable.
- No source/manifests/lockfile, other slice files, task statuses, accepted ledger,
  or Context Files are part of this design write set. Coordinator should add an
  inbound link from the task/index when accepting the checkpoint.

## Next gate

Coordinator reviews the eight documents and disposable evidence, brings these
choices to the owner, then releases M0 only. The first M0 stop must establish the
real streaming/report boundary and both real adapter lease transitions. No
commit, listener, install, context promotion or later milestone is authorized.
