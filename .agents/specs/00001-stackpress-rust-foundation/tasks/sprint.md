# Sprint plan

Task planning file of [00001 Stackpress Rust foundation](../index.md). Load for task order, rationale, and how the maintainability-first planning rubric applies. Task statuses live in [task status](status.md).

## Ordering rationale

Build foundations first and add each crate only when it has real code. Each slice must leave the workspace compiling, tested, and at or above the 70% development coverage floor before the next begins. Every slice follows the same gated sequence: design checkpoint, riskiest proof, implementation milestones, stop before refactor rounds, rounds 1–4, then a closeout receipt, each with coordinator re-verification.

## Planning rubric applied

The rubric layers in the spec task implementation workflow map onto this program as follows. Layers that don't apply are named with the reason.

| Rubric layer | Application |
| --- | --- |
| 1. Reusable foundation | Slice 1 `stackpress-lib`: data, queues, events, request/response, status, errors. |
| 2. Temporary placeholders | M0 signature fixtures, removed in task 00002. No other placeholders are planned. |
| 3. Reusable units | Slice 1 parsers and matchers; later backend and host adapters. |
| 4. Composition structures | Slice 2 server lifecycle and plugin composition; Slice 5 layouts. |
| 5. Event/workflow handling | Slice 1 dispatch; Slice 2 top-level lifecycle and CLI commands. |
| 6. Primary outputs | The profile-search vertical proof across CLI and HTTP (Q01), then UI. |
| 7–8. External-system placeholders, then real logic | Slice 4 database backends; Slice 5 Tauri and remote adapters. |
| 9. Maintainability refactoring | Rounds 1–4 in every slice. |

Acceptance criteria: none for library tasks, because they have no human-reviewable visual output; they finish at `verified`. UI tasks in Slice 5 will define visual acceptance criteria.

## Tasks

| Task | Slice | Summary | Status |
| --- | --- | --- | --- |
| [00001](00001-scaffold-and-send-signature-proof.md) | 0 + 1 (M0) | Workspace scaffold and `Send` handler-signature proof. | verified |
| [00002](00002-foundation-data-events-and-values.md) | 1 (M1–M3) | Data, queues/events/diagnostics, request/response/session, `resolve`, facade. | verified |
| [00003](00003-review-rounds-and-closeout.md) | 1 | Refactoring rounds 1–4, follow-up, closeout. | verified |
| 00004 | 2 | `stackpress-server` capability host (config, plugins, top-level lifecycle) and CLI adapter in `tools/`. | proposed |
| 00005 | 3 | Idea parser assessment against current fixtures, transformer, and generation of one model. | proposed |
| 00006 | 4 | `stackpress-sql`, first backend, and `framework-sql` integration. | proposed |
| 00007 | 5 | `framework-view`/`framework-ui` Leptos SSR with hydration, and a Tauri client. | proposed |

Proposed tasks get task files only after the owner accepts them.
