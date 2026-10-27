# Task 00012: Slice 7, production servers

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner accepted it on 2026-10-26, in parallel with [task 00011](00011-slice-6-generated-data-layer.md) and [task 00013](00013-slice-8-more-ui.md). It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md).

## Task summary

Replace demo-only hosting with production-grade adapters. This covers the deferred work named S2-Network and S4-N:

- **Production HTTP server (S2-Network):** a real socket server adapter on top of the existing transport-neutral `stackpress-server` lifecycle (G12/G17). It covers:
  - network read limits and timeouts;
  - keep-alive and pipelining policy;
  - disconnect and cancellation propagation;
  - graceful shutdown and draining;
  - streaming responses, which also unblocks streamed SSR (S5-F6);
  - static assets;
  - TLS and proxy-header policy;
  - observability hooks.

  The example-only gateway from task 00008 is the reference to replace, not to grow.
- **PostgreSQL and MySQL production adapters (S4-N):** `stackpress-sql-postgres` and `stackpress-sql-mysql` implementing the existing `Database`/`TransactionSession` contracts. That means:
  - pools and connection affinity;
  - codecs;
  - TLS options;
  - cancellation semantics matching the SQLite adapter;
  - real integration tests against the owner's local servers (G26 policy: throwaway databases only), plus CI-friendly gating.
- Serverless/PGlite hosting (S4-H) stays out of scope unless the design argues otherwise.

## Implementation steps

1. Design checkpoint, with no production code. Write it as `slice-7-design-*.md`, per workflow §5.1. It covers:
   - the upstream inventory (references 00020, 00026, 00030, 00014);
   - an assessment of HTTP server and driver libraries, using disposable probes under `/private/tmp`;
   - the API sketch with DX examples;
   - the risk inventory: slowloris and resource limits, backpressure, cancellation, shutdown races, TLS, header/proxy trust, pool exhaustion, driver cancellation;
   - open decisions written in plain language for the owner;
   - dependencies on tasks 00011 and 00013;
   - the G23 shared-crate requests, especially any change to `stackpress-server`;
   - a milestone plan for about two lanes.
2. Coordinator review and owner decisions, then M0 and the milestones, rounds 1–4, closeout and a docs follow-up.

Any live server runs only through devmetrics. Tests use in-process or ephemeral connections, and network listeners are opened only where a test strictly needs one, with justification.

## Verification process

As in the coordinated workflow:

- resource-limit and slow-client tests under watchdogs;
- graceful-shutdown proofs;
- adapter integration tests on the local PostgreSQL 12.2 and MySQL 8.0.27 servers;
- per-crate coverage of at least 70%.

## Acceptance criteria

Acceptance criteria: none (library work).

## Implementation notes

Design checkpoint delivered on 2026-10-26: start at [Slice 7 design decisions](../slice-7-design-decisions.md). Owner decisions recorded as G28/G30 on 2026-10-26; M0 released on 2026-10-27.

## Verification notes

**M0, coordinator review on 2026-10-27.** Both lanes verified and merged into `trial/3`.

- **SQL lane** ([receipt](00012-receipt-sql-m0.md); merge `4a1901d`): coordinator reran fmt, Clippy `-D warnings`, offline tests and the opt-in native tests against the local PostgreSQL 12.2 and MySQL 8.0.27 servers; all passed. The coordinator's own runs created and dropped `stackpress_s7_m0_16ad8_18e221e0a9e2f120` and `stackpress_s7_m0_16adc_18e221e0cdea1620`; a MySQL listing limited to `stackpress\_s7\_m0\_%` showed no leftovers. Coverage (implementer): PostgreSQL 78.56%, MySQL 82.93%. M0 retires every lease; safe reuse is an M1 item.
- **HTTP lane** ([receipt](00012-receipt-http-m0.md); merge `43a8c21`): coordinator reran fmt, Clippy for server `http-stream` and host `tls`, and tests for server default/`http-stream` and host default/`tls`; all passed. The pre-existing facade `server-http` feature has zero Hyper/Tokio/Rustls normal edges. Coverage (implementer): server 92.82%, server-http with TLS 95.33%. No listener was opened.
- **Combined `trial/3`:** `cargo test --workspace --all-targets --locked` passed (491 passed, 0 failed, 62 ignored).
- **SQL lane M1** ([receipt](00012-receipt-sql-m1.md); merge `3b01cf5`), coordinator review on 2026-10-27: reran fmt, Clippy, offline and native tests (M0 and four M1 groups on both servers) and the Idea parser suite affected by the reverted `arbitrary_precision` feature; all passed. The coordinator's own runs created and dropped `stackpress_s7_m0_181d8_…`, `stackpress_s7_m1_181db_…`, `stackpress_s7_m0_181e1_…` and `stackpress_s7_m1_181ed_…`; the scoped MySQL listing showed no leftovers. Coverage (implementer): PostgreSQL 90.88%, MySQL 92.11%. Accepted boundaries: PostgreSQL domains decode by their advertised base wire type; MySQL reuse follows a lexical stateless allowlist (no SQLx fork). Two LLVM `.profraw` files committed by mistake with M0 were removed, and `.gitignore` now excludes coverage profiles.
- **HTTP lane M1** ([receipt](00012-receipt-http-m1.md); merge `baf001e`), coordinator review on 2026-10-27: reran fmt, Clippy (server + host with TLS), server `http-stream` tests, host TLS tests three consecutive times and host without TLS; all passed. The pre-existing facade `server-http` graph still has zero Hyper/Tokio/Rustls edges. Coverage (implementer): server-http 97.02%, server 93.05%. The httparse preflight was dropped; framing follows G31. No listener was opened.
- **SQL lane M2** ([receipt](00012-receipt-sql-m2.md); merge `2426647`), coordinator review on 2026-10-27: reran fmt, Clippy, offline tests and native M0/M1/M2 on both servers (6 owned databases created and dropped; no leftovers; no SQLite temp files left). Coverage (implementer): PostgreSQL 92.14%, MySQL 93.26%. Shared contract suite runs over PostgreSQL, MySQL and the SQLite reference; pool stress at sizes 1/2/4. Generated-store integration waits for Slice 6 stores.
- **HTTP lane M2** ([receipt](00012-receipt-http-m2.md); merge `c8f7bca`), coordinator review on 2026-10-27: reran fmt, Clippy, server `http-stream` tests, host TLS tests three times and host without TLS; old facade `server-http` graph still has zero network edges. **Managed smoke test (coordinator, via devmetrics on port 3000, stopped after use):** `/health` 200; asset GET/HEAD with ETag and 304 on `If-None-Match`; chunked `/stream`; spoofed Host 400; unknown asset 404; `..` traversal 400; 70 KB header 431; CL+TE smuggling attempt answered once with 400 and the trailing request never dispatched; slow header trickle closed after about 11 s; managed stop drained 13 connections and exited 0 with no listener left. Coverage (implementer): server-http 94.94%, server 93.20%.
- **Docs follow-ups** (receipts [HTTP docs](00012-receipt-http-docs.md), merge `1c5dc0b`; [SQL docs](00012-receipt-sql-docs.md), merge `80e372e`), coordinator review on 2026-10-27: compiled examples, doctests, Rustdoc and Clippy pass. **Security fix found during SQL docs:** the PostgreSQL ambient-variable guard checked misspelled `PGPg*` names, so SQLx's `PGSSLROOTCERT`/`PGSSLCERT`/`PGSSLKEY` bypassed it; corrected with an isolated regression (failed before, passes after); no other ambient SQLx variables found. Coordinator reran fmt, Clippy and the adapter tests (27 passed); the scoped MySQL listing showed no leftovers.
- **Generated-store integration** ([receipt](00012-receipt-sql-store-integration.md)), coordinator review on 2026-10-27: the generated Slice 6 stores run through the PostgreSQL and MySQL adapters in the shared contract harness alongside the SQLite reference. PostgreSQL uses the generated DDL and passes the CRUD, conflict, batch and soft-delete sequences. MySQL refuses generated TEXT-key DDL before IO (an honest limit; a MySQL string-key width policy is a task 00011 follow-up), so its fixture tables are harness-owned DDL that exercise the generated queries, codecs, relations, batches and refusals. **Two fixes found by this work:** (1) generated `search_atomic`/`batch_atomic` opened a transaction before known backend refusals, and a BEGIN failure then masked `Unsupported` as `Sql`; fixed under a coordinator G23 grant by preflighting the existing pure checks before BEGIN (`require_returning` and `preflight_atomic_search`, never stricter than the real read), with zero-entry tests; (2) the MySQL adapter decoded `utf8mb4_bin` text as bytes because of its BINARY flag; it now uses the wire charset, keeping true BINARY/VARBINARY/BLOB as bytes, with native tests both ways. The adapters gain dev-only path dependencies on the generated example (stripped on publish). After merging current `trial/3` the coordinator reran: fmt, workspace Clippy, matrix/names Clippy and tests, both generation checks (fresh), the full workspace (675 passed, 0 failed, 78 ignored; 895 doctests), the native PostgreSQL/MySQL M0–M2 suites on the local servers (13 passed, 0 failed), and the generated-admin checks and compile proof. No `stackpress_%` databases remain on either server (checked by listing). Coverage (implementer): PostgreSQL 92.61%, MySQL 93.27%, framework-sql 87.00%, generated app 76.68%. **Task 00012 implementation is complete;** closeout review rounds remain.
- **SQL lane closeout** ([receipt](00012-receipt-sql-closeout.md)), coordinator review on 2026-10-27: four whole-crate review rounds over the PostgreSQL and MySQL adapters and the shared contract harness (consolidated test watchdogs and sanitized diagnostics, ten compiled Rustdoc examples, stronger pre-IO checks; no public API change). **Safety fix, approved as a closeout exception:** the MySQL connection-reuse classifier matched allowlisted builtin names inside longer tokens (`éABS(...)`, `ABSé(...)`, `1ABS(...)`, a quoted name followed by letters) and allowed reuse; it now classifies whole tokens and retires on unquoted non-ASCII or adjacent quoted/unquoted fragments, which only makes reuse more conservative, with offline and native regressions. **MySQL string-key width options** for task 00011 are recorded in the receipt (recommended: an explicit optional field storage length selecting VARCHAR(n); no silent default). After merging current `trial/3` the coordinator reran: fmt, workspace Clippy, matrix/names Clippy and tests, the full workspace (681 passed, 0 failed, 80 ignored; 918 doctests), and the native PostgreSQL/MySQL M0–M2 suites on the local servers (13 passed); no `stackpress_%` databases remain on either server. Coverage (implementer): PostgreSQL 92.61%, MySQL 92.89%.
- **HTTP lane closeout** ([receipt](00012-receipt-http-closeout.md)), coordinator review on 2026-10-27: four whole-crate review rounds over the Hyper host (all features including `tls`) and the server `http`/`http-stream` modules; behavior-preserving only, no public API, framing-policy (G31), limit or default change, and no safety bug found. One new test binds an ephemeral `127.0.0.1:0` listener (the thin bind/accept/drop adapter cannot be exercised over duplex IO): it serves one request, then proves awaited stop completes and reaps the admitted task and that the port refuses a fresh connection, under a 10-second parent watchdog. After merging current `trial/3` the coordinator reran: fmt, workspace Clippy, server+host Clippy with `http-stream,tls`, those tests three consecutive times (102 passed each), the host default build tests, the facade `server-http` normal graph (zero Hyper/Tokio/Rustls edges), and the full workspace (693 passed, 0 failed, 81 ignored; 929 doctests). Coverage (implementer): server 93.20%, server-http 96.64%. Publication gaps (platform/release verification, TLS over real TCP, residual native-error paths) are listed in the receipt.
- **Task 00012 is complete** (implementation and closeout for both lanes); owner acceptance is not required (library work).
- **Open follow-up:** the implementer saw one `PoolTimedOut` in the existing SQLite suite (`crates/sql/sql-sqlite` `cancellation.rs:454`) under default-thread workspace contention; its isolated replay and the coordinator's merged-workspace run passed. Treat as a possible contention-sensitive test; investigate before publication.
- **M1 released to both lanes.** SQL M1: the owner-level SQLx fork was declined by the coordinator; MySQL reuse follows a conservative retire-unless-proven-stateless rule. HTTP M1: a pinned direct `httparse =1.10.1` preflight (already locked under Hyper) is approved, reject-only and bounded, so duplicate Content-Length and CL+TE can be rejected before Hyper normalizes them.

## Acceptance notes

Pending.
