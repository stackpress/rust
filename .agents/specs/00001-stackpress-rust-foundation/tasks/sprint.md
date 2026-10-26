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
| [00004](00004-lib-usage-and-api-docs.md) | 1 | `crates/lib/docs`: getting started, concepts, task guides, and API reference, written to the chrisai-docing standard as plain Markdown. | verified |
| [00004A](00004A-envelope-json-limits.md) | 1 | Side quest of 00004: envelope JSON conversion respects custom limits. | verified |
| [00005](00005-slice-2-server-and-cli.md) | 2 | `stackpress-server` capability host (config, plugins, top-level lifecycle), buffered HTTP adapter, CLI adapter in `tools/`, and the `examples/capability-host` proof app. | verified |
| [00005A](00005A-server-and-cli-docs.md) | 2 | Follow-up: server and CLI usage/API docs (chrisai-docing). | verified |
| [00006](00006-slice-3-idea.md) | 3 | Idea parser assessment against current fixtures, transformer, and generation of one model. | verified |
| [00006A](00006A-idea-docs.md) | 3 | Follow-up: Idea parser, transformer and wrapper usage/API docs (chrisai-docing). | verified |
| [00007](00007-slice-4-sql.md) | 4 | `stackpress-sql`, first backend, and `framework-sql` integration; parallel with 00008 (G23). | verified |
| [00007A](00007A-sql-docs.md) | 4 | Follow-up: SQL core and SQLite usage/API docs (chrisai-docing). | verified |
| [00007B](00007B-sql-where-eq-and-insert-id-docs.md) | 4 | Side quest of 00007A: `Select::where_eq` appends like Update/Delete; current `insert_id` Rustdoc. | started |
| [00008](00008-slice-5-ui.md) | 5 | `framework-view`/`framework-ui` Leptos SSR with hydration, and a Tauri client; parallel with 00007 (G23). | started |

Proposed tasks get task files only after the owner accepts them. Proposed numbers were shifted by one on 2026-10-24 when the owner requested task 00004; none had been accepted, so no stable number changed.
