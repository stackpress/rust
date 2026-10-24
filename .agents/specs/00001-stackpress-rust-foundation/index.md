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

## Tasks

- [Sprint plan](tasks/sprint.md): load for task order, rationale, the planning rubric mapping, and proposed later slices.
- [Task status](tasks/status.md): load for current task statuses.
- [00001 Scaffold and Send signature proof](tasks/00001-scaffold-and-send-signature-proof.md): load for the M0 work and its full receipt.
- [00002 Foundation data, events, and values](tasks/00002-foundation-data-events-and-values.md): load for the M1–M3 work and its full receipt.
- [00003 Review rounds and closeout](tasks/00003-review-rounds-and-closeout.md): load for refactoring rounds 1–4, the follow-up, and the Slice 0+1 closeout receipt.
