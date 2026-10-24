# Task status

Task tracking file of [00001 Stackpress Rust foundation](../index.md). Load to see every task's current status. Ordering and rationale live in the [sprint plan](sprint.md).

Status values follow the spec task implementation workflow: `proposed`, `open`, `started`, `verified`, `accepted`.

| Task | Status | Notes |
| --- | --- | --- |
| [00001 Scaffold and Send handler-signature proof](00001-scaffold-and-send-signature-proof.md) | verified | Acceptance criteria: none. Committed in `e230359`. |
| [00002 Foundation data, events, and values](00002-foundation-data-events-and-values.md) | verified | Acceptance criteria: none. Committed in `e230359`. |
| [00003 Review rounds and closeout](00003-review-rounds-and-closeout.md) | verified | Acceptance criteria: none. Committed in `e230359`. |
| [00004 Lib usage and API docs](00004-lib-usage-and-api-docs.md) | verified | Owner requested 2026-10-24. Acceptance criteria: none. Coordinator-reviewed 2026-10-24. |
| 00004A Response/envelope API consistency | proposed | Side quest discovered while documenting task 00004: envelope JSON conversion ignores custom limits (defect); `sent()`/`result()` state semantics, setters retaining earlier fields, status-only envelope code 0, and optional path reads hiding malformed syntax (owner decision). Task file created on approval. |
| 00005 Slice 2 server + CLI adapter | proposed | Task file created on owner acceptance. |
| 00006 Slice 3 Idea | proposed | Task file created on owner acceptance. |
| 00007 Slice 4 SQL | proposed | Task file created on owner acceptance. |
| 00008 Slice 5 UI | proposed | Task file created on owner acceptance. |
