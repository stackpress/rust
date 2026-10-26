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

Pending.

## Verification notes

Pending.

## Acceptance notes

Pending.
