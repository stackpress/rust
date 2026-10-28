# Task status

Task tracking file of [00001 Stackpress Rust foundation](../index.md). Load to see every task's current status. Ordering and rationale live in the [sprint plan](sprint.md).

Status values follow the spec task implementation workflow: `proposed`, `open`, `started`, `verified`, `accepted`.

| Task | Status | Notes |
| --- | --- | --- |
| [00001 Scaffold and Send handler-signature proof](00001-scaffold-and-send-signature-proof.md) | verified | Acceptance criteria: none. Committed in `e230359`. |
| [00002 Foundation data, events, and values](00002-foundation-data-events-and-values.md) | verified | Acceptance criteria: none. Committed in `e230359`. |
| [00003 Review rounds and closeout](00003-review-rounds-and-closeout.md) | verified | Acceptance criteria: none. Committed in `e230359`. |
| [00004 Lib usage and API docs](00004-lib-usage-and-api-docs.md) | verified | Owner requested 2026-10-24. Acceptance criteria: none. Coordinator-reviewed 2026-10-24. |
| [00004A Envelope JSON conversion respects custom limits](00004A-envelope-json-limits.md) | verified | Side quest of 00004, approved by the owner 2026-10-24 and scoped to the envelope-limits defect only; the other DX observations remain documented behavior. Acceptance criteria: none. Coordinator-verified 2026-10-24. |
| [00005 Slice 2 server and CLI](00005-slice-2-server-and-cli.md) | verified | Owner accepted 2026-10-24; four parallel lanes merged; coordinator closeout verified 2026-10-24. Acceptance criteria: none. |
| [00005A Server and CLI docs](00005A-server-and-cli-docs.md) | verified | Owner accepted 2026-10-24; coordinator-reviewed 2026-10-24. Acceptance criteria: none. |
| [00006 Slice 3 Idea](00006-slice-3-idea.md) | verified | Owner accepted 2026-10-24; four parallel lanes merged; coordinator closeout verified 2026-10-25. Acceptance criteria: none. |
| [00006A Idea usage and API docs](00006A-idea-docs.md) | verified | Owner requested 2026-10-25; 43 pages across parser, transformer and wrapper; coordinator-reviewed 2026-10-25. Acceptance criteria: none. |
| [00007 Slice 4 SQL](00007-slice-4-sql.md) | verified | Owner accepted 2026-10-25 (G24/G26); M0–M3 and rounds 1–4 merged; coordinator closeout verified 2026-10-26. Acceptance criteria: none. |
| [00007A SQL usage and API docs](00007A-sql-docs.md) | verified | Owner requested 2026-10-26; 40 pages across core and SQLite; coordinator-reviewed 2026-10-26. Acceptance criteria: none. |
| [00007B Consistent where_eq and insert_id Rustdoc](00007B-sql-where-eq-and-insert-id-docs.md) | verified | Side quest of 00007A, owner-approved and coordinator-verified 2026-10-26. Acceptance criteria: none. |
| [00008 Slice 5 UI](00008-slice-5-ui.md) | accepted | Owner accepted 2026-10-25 (G25); M0–M4 and rounds 1–4 merged; coordinator closeout verified and owner visual acceptance 2026-10-26. |
| [00008A UI usage and API docs](00008A-ui-docs.md) | verified | Owner requested 2026-10-26; 52 pages across framework-view and framework-ui; coordinator-reviewed 2026-10-26. Acceptance criteria: none. |
| [00009 Profile UI backed by SQLite](00009-profile-ui-on-sqlite.md) | verified | Owner requested 2026-10-26 (option B); coordinator-verified 2026-10-26; both backends pass all proofs. |
| [00010 Owner review issues #1–#13](00010-owner-review-issues.md) | verified | All 13 fixed in three passes; coordinator-verified 2026-10-26; notes posted on each GitHub issue (open until on `main`). Acceptance criteria: none. |
| [00011 Slice 6 generated data layer](00011-slice-6-generated-data-layer.md) | verified | M0–M4 and closeout verified (CRUD stores, migrations with G35 sequence preservation, quickstart walkthrough, public pre-IO refusals). G37/G38 explicit string storage length (schema contract 3) merged and verified 2026-10-28. Library work, no acceptance criteria. |
| [00012 Slice 7 production servers](00012-slice-7-production-servers.md) | verified | HTTP and SQL lanes implemented, integrated with generated stores, and closed out (four review rounds each); coverage 92–97%. Library work, no acceptance criteria. |
| [00013 Slice 8 more UI](00013-slice-8-more-ui.md) | accepted | Owner visual acceptance 2026-10-28: web review preview after the first-walkthrough fixes, and the installed iOS review (plugin/disabled). Every lane implemented and verified. Android runtime unverified by owner decision; G39 (no persistent System theme) lands with issue #15. |
| [00014 SQLite contention-sensitive test](00014-sqlite-contention-test.md) | verified | Load-sensitive tests made deterministic; merged 2026-10-27. |
| [00015 Issue #16 server crate grouping](00015-receipt-issue16-server-layout.md) | verified | Owner requested 2026-10-28 (GitHub #16). `crates/server` → `crates/server/server`, `crates/server-http` → `crates/server/http`. Package names, imports, features, dependency graphs and Cargo.lock unchanged. The coordinator independently reran the full workspace including `tcp_watchdog` (740 passed), generators, quickstart, profile-ui and admin checks, recorded through the benchmark workflow. Merged 2026-10-28; the GitHub issue stays open until the change is on `main`. |
| [00016 Issue #15 presentation in framework-view](00016-issue15-design.md) | M1 verified; M2 next | Owner requested 2026-10-28 (GitHub #15; decisions G39, G40). M1 ([receipt](00016-issue15-m1.md)): `stackpress-framework-preferences` removed; preferences/flash/access/adapters live in framework-view; theme is Light/Dark only with a one-shot pre-paint OS sample for unset pages and Light without JS. Coordinator reran fmt, workspace clippy, full workspace tests including `tcp_watchdog` (744 passed), doctests (939), generators, golden comparison, lock contract, and the preferences/first-paint/gallery/Profile browser proofs, recorded through the benchmark workflow. Merged 2026-10-28. Next: M2 plugin registrations `framework_view::{preferences,flash}::register`; then three review rounds and the owner re-check. |
