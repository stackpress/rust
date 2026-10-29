# Task 00020: Slice 9, session, CSRF and email

Task file of [00001 Stackpress Rust foundation](../index.md); status is tracked in [task status](status.md). The owner asked on 2026-10-30 to continue with the pending framework crates. It runs under the [coordinated crate development workflow](../../../workflows/coordinated-crate-development.md) and the [fast build and test workflow](../../../workflows/fast-build-and-test.md).

## Task summary

Design, then build, the three pending security-foundation crates from the [accepted source structure](../../../context/source-structure.md):

- **`framework-session`:** signed identity session distinct from the low-level request session map, authentication (signup/signin/signout and the configured auth flows), role grants, and the `can` permission check.
- **`framework-csrf`:** token generate/clear/validate mechanisms, cookie/session/SSR data placement, timing-safe comparison and the 419 failure.
- **`framework-email`:** configured email-send behaviour as an event, with result/error semantics, behind a Rust transport.

Later slices: `framework-api` (API endpoints, scopes, CORS, OAuth, webhooks) depends on this slice's identity; the aggregate `framework` plugin comes last. Optional AI/MCP and desktop are not in the accepted crate tree.

## Implementation steps

1. Design checkpoint, with no production code, written as `slice-9-design-*.md` per workflow §5.1. It covers:
   - the upstream inventory for `stackpress-session`, `stackpress-csrf` and `stackpress-email` (references 00028, 00041, 00023, 00032 and the current sibling sources), each item mapped to now, a named later slice, or not ported, with a reason;
   - an overlap map against existing Rust code: `stackpress-lib` session/cookies, `framework-view` access/policy/preferences/flash, the admin host's CSRF handling and the review/preferences examples; say what moves, what is reused and what stays;
   - a library assessment with disposable probes outside the repository (signing/JWT or alternative, password hashing, timing-safe comparison, email transport), with pinned versions and new external dependencies listed as an explicit owner request;
   - the API sketch with DX examples, including plugin registration under the #14 conventions and direct use without plugins;
   - the risk inventory (secret handling, token forgery/expiry, timing leaks, session fixation, enumeration, default access policy, email header injection), each with prevention and proving test;
   - open decisions in plain language, each with a recommendation, including the default access policy (the upstream empty-map-permits-all behaviour versus deny-by-default);
   - G42 route semantics stay binding: no framework route ownership or conflict arbitration;
   - a milestone plan for about two lanes with file ownership, and the verification plan.
2. Coordinator review and owner decisions, then M0 and milestones, rounds 1–4, closeout and a docs follow-up.

## Verification process

Per the coordinated workflow: per-crate coverage of at least 70%, security-negative tests for every risk, no secrets in browser graphs or snapshots, gate-tier checks and coordinator re-verification.

## Acceptance criteria

Acceptance criteria: none for the library crates. Any auth pages proposed by the design will define visual acceptance criteria for owner approval.

## Implementation notes

## Verification notes
