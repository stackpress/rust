# Task 00021: Slice 10, API

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner asked on 2026-10-30 to continue with the pending framework crates and to run independent work in parallel. Its design runs alongside [task 00020](00020-slice-9-session-csrf-email.md); its implementation waits for that task's identity contract. It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md) and the [fast build and test workflow](../../../workflows/fast-build-and-test.md).

## Task summary

Design, then build, `framework-api` from the [accepted source structure](../../../context/source-structure.md): configured method/path-to-event endpoint mappings for public, app and session callers, fixed request data, priorities, per-surface scopes, CORS, OAuth routes and outgoing webhooks.

Depends on task 00020 for caller identity and the permission check. The aggregate `framework` plugin comes after both.

## Implementation steps

1. Design checkpoint, with no production code, written as `slice-10-design-*.md` per workflow §5.1. It covers:
   - the upstream inventory for `stackpress-api` (references 00041, 00023, 00032, 00024 and the current sibling sources), each item mapped to now, a named later slice, or not ported, with a reason;
   - an overlap map against existing Rust code: the server host and HTTP adapter, the generated admin API routes and operations, and the #14 plugin conventions;
   - the assumed identity and permission interface from task 00020, stated in one place so the coordinator can reconcile the two designs;
   - a library assessment with disposable probes outside the repository (OAuth/token handling, CORS, webhook delivery and signing), with new external dependencies listed as an explicit owner request;
   - the API sketch with DX examples, including plugin registration and direct use;
   - the risk inventory (unintended public exposure, scope escalation, CORS misconfiguration, OAuth code/state/redirect flaws, token leakage, webhook SSRF, retry and idempotency), each with prevention and proving test;
   - open decisions in plain language with recommendations;
   - G42 route semantics stay binding: no framework route ownership or conflict arbitration;
   - a milestone plan with file ownership and the verification plan.
2. Coordinator review, reconciliation with task 00020 and owner decisions, then M0 and milestones, rounds 1–4, closeout and a docs follow-up.

## Verification process

Per the coordinated workflow: per-crate coverage of at least 70%, security-negative tests for every risk, gate-tier checks and coordinator re-verification.

## Acceptance criteria

Acceptance criteria: none for the library crate. Any OAuth consent or API key pages proposed by the design will define visual acceptance criteria for owner approval.

## Implementation notes

## Verification notes
