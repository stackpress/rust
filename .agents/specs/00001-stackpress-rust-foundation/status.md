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
| 5 | Accepted; committed on `trial/3` | Task 00008; owner visual acceptance 2026-10-26; 442 workspace tests, 855 doctests; coverage framework-view 94.52%, framework-ui 98.12%; SSR, hydration, CSR and a packaged macOS app with networking-free local operation. |

## Next action

The owner accepted all recommendations of the Slice 6, 7 and 8 designs (tasks 00011–00013; G27–G30) and approved the iOS and Android tooling, now installed. Their first proof steps (M0) run in parallel. The owner is reviewing GitHub issues #1–#13, which are fixed on `trial/3`.
