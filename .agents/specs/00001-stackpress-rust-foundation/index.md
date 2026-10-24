# 00001 Stackpress Rust foundation

Entry point for the Stackpress Rust build program: slices 0–5 coordinated between the owner, a coordinator agent, and an implementer agent, tracked here separately from the KB until the owner approves context promotion. `.agents/context/` remains the source of truth over this spec.

## Spec files

- [Brief](brief.md): load first for the owner's goals as written, scope, non-goals, and inherited sources.
- [Status](status.md): load to check freeze readiness, implementation status, and the next action.
- [Decisions and Gap ledger](decisions.md): load before planning or implementing any slice; holds process decisions, Slice 0+1 owner decisions, the applied design decisions (D1–D8, DX), and the carried open design questions with their status.
- [Proofs](proofs.md): load for proved technical uncertainties, starting with the `Send` handler signature.

## Slice 1 design

The approved Slice 0+1 design, moved losslessly from the former `docs/trial-3/slice-1-design.md`:

- [Authority, scope, and upstream inventory](design-upstream-inventory.md): load when checking which upstream `lib` exports, types, and tests are implemented, deferred to a later slice, or intentionally not ported.
- [Public API and execution contract](design-api-contract.md): load for package layout, handler registration and ownership, the `Send` signature, execution semantics, and matcher grammar.
- [Data, request/response, and error contract](design-values-and-errors.md): load for Value/Nest and parser rules, request/response/session values, and the error/diagnostic model.
- [Risk inventory and verification plan](design-risks-and-verification.md): load for the risk table with required proofs, milestone order, watchdog rules, review rounds, coverage, and the receipt template.

## Slice 2 design checkpoint

Approved with G12–G16 and coordinator DX changes; only M0 is released. Load the decision checkpoint and task receipt before advancing:

- [Upstream host inventory](slice-2-design-upstream-host.md): load for Ingest exports/types/tests, plugin/loading facets, and precise source lifecycle discrepancies.
- [Upstream adapter and terminal inventory](slice-2-design-upstream-adapters.md): load for HTTP/WHATWG, Stackpress commands, lib Terminal/control/input, tested behavior and every Slice 1 deferral.
- [Host API and ownership](slice-2-design-api.md): load for crate/module layout, function plugins, startup, typed config/services, examples and additive lib context changes.
- [Lifecycle, failures and diagnostics](slice-2-design-lifecycle.md): load for top-level versus raw dispatch, stage/abort/error tables, finalization, terminal fallback and correlation.
- [CLI, HTTP and dependencies](slice-2-design-adapters.md): load for native envelopes, capture precedence, operator syntax/output/exit, compile-time app composition, prompts and feature/dependency boundaries.
- [Risks, milestones and verification](slice-2-design-verification.md): load for R01–R33, M0 and later stops, per-crate coverage, watchdogs, rounds 1–4 and possible four-lane file ownership.
- [Open decisions and recommendations](slice-2-design-decisions.md): load for proposed Q04/Q05/Q10/Q14 answers, partial Q01 scope, pending owner choices and checkpoint coverage.

## Tasks

- [Sprint plan](tasks/sprint.md): load for task order, rationale, the planning rubric mapping, and proposed later slices.
- [Task status](tasks/status.md): load for current task statuses.
- [00001 Scaffold and Send signature proof](tasks/00001-scaffold-and-send-signature-proof.md): load for the M0 work and its full receipt.
- [00002 Foundation data, events, and values](tasks/00002-foundation-data-events-and-values.md): load for the M1–M3 work and its full receipt.
- [00003 Review rounds and closeout](tasks/00003-review-rounds-and-closeout.md): load for refactoring rounds 1–4, the follow-up, and the Slice 0+1 closeout receipt.
- [00004 Lib usage and API docs](tasks/00004-lib-usage-and-api-docs.md): load for the `crates/lib/docs` documentation task, its standards, and its review results.
- [00004A Envelope JSON limits](tasks/00004A-envelope-json-limits.md): load for the side quest that makes envelope JSON conversion honor custom limits.
- [00005 Slice 2 server and CLI](tasks/00005-slice-2-server-and-cli.md): load for the `stackpress-server` capability host and CLI adapter task, its Gaps, and its receipts.
- [Slice 2 lane B CLI receipt](slice-2-receipt-lane-b-cli.md): load for the `stackpress-cli` parsing/control/prompt/TTY implementation, its upstream inventory disposition, rounds 1–4, and verification.
- [Slice 2 lane D proof-app receipt](slice-2-receipt-lane-d-app.md): load for the `examples/capability-host` profiles proof application, its example-coverage measurement method, and which proof items are demonstrated.
- [Slice 2 lane C HTTP receipt](slice-2-receipt-lane-c-http.md): load for the buffered HTTP adapter's contract, limitations (e.g. list-valued session cookies need an explicit codec), tests, and coverage.
