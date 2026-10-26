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
- **M1 released to both lanes.** SQL M1: the owner-level SQLx fork was declined by the coordinator; MySQL reuse follows a conservative retire-unless-proven-stateless rule. HTTP M1: a pinned direct `httparse =1.10.1` preflight (already locked under Hyper) is approved, reject-only and bounded, so duplicate Content-Length and CL+TE can be rejected before Hyper normalizes them.

## Acceptance notes

Pending.
