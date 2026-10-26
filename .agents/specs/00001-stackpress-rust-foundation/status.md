# Status

Spec file of [00001 Stackpress Rust foundation](index.md). Load to see freeze readiness, implementation status, and the next action.

## Freeze status

**Not Frozen.** The Slice 0+1 scope was accepted by the owner and implemented under direct authorization before this spec existed; this package records that work retroactively and tracks the remaining slices.

| Work item | Status | Next action |
| --- | --- | --- |
| Brief | Complete | Update when the owner changes program goals. |
| Decisions and Gaps | Slice 0+1 answered; Q01, Q02, Q05, Q06, and Q08–Q12 unresolved; Q04 and Q13–Q15 partly answered | Resolve the Slice 2 Gaps (Q04, Q05, Q10, Q14) in its design checkpoint. |
| Research | Slice 1 upstream `lib` inventory complete in the [upstream inventory](design-upstream-inventory.md); later slices not started | Inventory upstream Ingest/server and CLI contracts for Slice 2. |
| Proofs | P1 proved; later slices TBD | Add proofs per slice in [proofs](proofs.md). |
| User journeys | Not applicable for Slices 0–4 (libraries, host, and generation with no human actor flows) | Run the user journeys workflow before Slice 5 UI and any session/auth work. |
| Context promotion | Deliberately deferred by owner (G04) | Promote only after the owner approves. |

## Implementation status

**accepted.** The Slice 0+1 plan is accepted and its tasks are verified; Slices 2–5 are proposed in the [sprint plan](tasks/sprint.md). Per-task status lives in [task status](tasks/status.md).

| Slice | State | Evidence |
| --- | --- | --- |
| 0 + 1 | Verified; committed on `trial/3` as `e230359` | Tasks 00001–00003; `stackpress-lib` line coverage 92.95%; 37 integration tests and 263 doctests passing. |
| 2 | Verified; committed on `trial/3` | Task 00005; 143 workspace tests, 394 doctests; coverage lib 92.96%, server 92.62%/94.23% (http), CLI 96.32%, example 100%; CLI = HTTP = direct-call proof. |
| 3 | Verified; committed on `trial/3` | Task 00006; 262 workspace tests, 430 doctests; coverage parser 94.44%, transformer 96.38%, wrapper 100%, codegen 95.27%, generated app 81.31%; imports compose end to end; generated app has no generator dependency. |
| 4 | Verified; committed on `trial/3` | Task 00007; 423 workspace tests, 854 doctests; coverage sql 95.43%, sql-sqlite 95.41%, sql-profile example 89.53%; live PostgreSQL/MySQL renderer validation 16/16; default facade driver-free. |
| 5 | Started (task 00008): M0 | Owner accepted 2026-10-25; parallel with Slice 4 under G23. |

## Next action

In flight:

- **Slice 4, lane A M2:** the network renderers, validated against the owner's local PostgreSQL 12.2 and MySQL 8.0.27.
- **Slice 5, lane B:** integration and M2 completion.

The compile-fail harness fix was merged as `1e957a4` ([receipt](fix-compile-fail-harness-receipt.md)). Both harnesses now use trybuild against the exact build. The coordinator reproduced the failure with three mixed-feature builds in one target, and the same sequence now passes.
