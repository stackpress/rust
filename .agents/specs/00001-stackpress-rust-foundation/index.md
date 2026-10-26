# 00001 Stackpress Rust foundation

Entry point for the Stackpress Rust build program: slices 0–5 coordinated between the owner, a coordinator agent, and an implementer agent, tracked here separately from the KB until the owner approves context promotion. `.agents/context/` remains the source of truth over this spec.

## Spec files

- [Brief](brief.md): load first for the owner's goals as written, scope, non-goals, and inherited sources.
- [Status](status.md): load to check freeze readiness, implementation status, and the next action.
- [Decisions and Gap ledger](decisions.md): load before planning or implementing any slice; holds process decisions, Slice 0+1 owner decisions, the applied design decisions (D1–D8, DX), and the carried open design questions with their status.
- [Proofs](proofs.md): load for proved technical uncertainties, starting with the `Send` handler signature.
- [Slice 3 parser assessment](slice-3-design-assessment.md): load for the pinned core's exact build/test/fixture evidence, dependencies, code-quality findings and extraction recommendation.
- [Slice 3 grammar and compatibility findings](slice-3-design-grammar.md): load for the probe corpus, source-versus-catalog discrepancies, diagnostic/env contracts and proposed corrected grammar/composition.
- [Slice 3 upstream inventory](slice-3-design-inventory.md): load for parser/transformer exports and tests, the language/component/generation catalogs, and the disposition of Slice 1 filesystem/template deferrals.
- [Slice 3 API sketch](slice-3-design-api.md): load for proposed parser/composer/function-plugin/output/wrapper APIs and the one-command Profile author experience; APIs are not implemented yet.
- [Slice 3 generation and dependency contract](slice-3-design-generation.md): load for acyclic bootstrap, pre-build CLI selection, output ownership/recovery, generated-code partitions, the first proof and dependency budget.
- [Slice 3 decisions and recommendations](slice-3-design-decisions.md): load before releasing M0 for pending Q08/Q09 answers, compatibility/scope choices and the checkpoint coverage map.
- [Slice 3 risks and verification](slice-3-design-verification.md): load for R01–R33, M0 and later milestone stops, lane ownership, fail-fast tests, per-crate coverage and rounds 1–4.
- [Slice 4 decisions and recommendations](slice-4-design-decisions.md): load first for the accepted Slice 4 (SQL) scope (G24); it routes to the inventories, test dispositions, driver assessment, API, execution contracts, and risks and milestones.
- [Slice 4 driver assessment](slice-4-design-assessment.md): load for the SQLx, tokio-rusqlite and rusqlite comparison, pinned versions and probe evidence.
- [Slice 4 API](slice-4-design-api.md): load for the standalone query, server service and future store sketches.
- [Slice 4 execution contracts](slice-4-design-execution.md): load for ownership, pooling, transactions, cancellation, values and dialects.
- [Slice 4 risks and milestones](slice-4-design-verification.md): load for the risk register, M0, the two-lane plan and stop gates.
- [Slice 4 core inventory](slice-4-design-inventory-core.md), [framework inventory](slice-4-design-inventory-framework.md), [core tests](slice-4-design-tests-core.md), [integration tests](slice-4-design-tests-integration.md): load for upstream dispositions.
- [Slice 5 decisions and recommendations](slice-5-design-decisions.md): load first for the accepted Slice 5 (UI) scope (G25); it routes to the assessment, architecture, API, inventories, and risks and acceptance.
- [Slice 5 assessment](slice-5-design-assessment.md): load for Leptos and Tauri versions, host tooling, probes and installs.
- [Slice 5 architecture](slice-5-design-architecture.md): load for page lifecycle, routing, snapshots, target dependencies, transports and cancellation.
- [Slice 5 API](slice-5-design-api.md): load for the page, shared component and registration sketches.
- [Slice 5 risks, milestones and acceptance](slice-5-design-verification.md): load for the risk inventory, M0, lanes and owner visual acceptance 00008-001 to 008.
- [Slice 5 view inventory](slice-5-design-inventory-view.md), [Frui inventory](slice-5-design-inventory-frui.md), [language inventory](slice-5-design-inventory-language.md): load for upstream dispositions.

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
- [Slice 3 lane A parser receipt](slice-3-receipt-lane-a-parser.md): load for the corrected-grammar parser, fixture parity, JSON import/export, and the typed reference API.
- [Slice 3 lane B composition receipt](slice-3-receipt-lane-b-compose.md): load for source loaders, `use` composition, the merge-table matrix (including G19), and the pending typed-reference switch.
- [Slice 3 lane C writer receipt](slice-3-receipt-lane-c-writer.md): load for the transform registry, output plan, writer recovery matrix, and limitations.
- [Slice 3 lane D integration receipt](slice-3-receipt-lane-d-integration.md): load for the generated-profile example, codegen DX, consumer proofs, and facade features at the M4 baseline.
- [00005A Server and CLI docs](tasks/00005A-server-and-cli-docs.md): load for the `crates/server/docs` and `tools/cli/docs` documentation task.
- [00006 Slice 3 Idea](tasks/00006-slice-3-idea.md): load for the Idea parser assessment, transformer, and generation task, its Gaps, and its receipts.
- [00006A Idea usage and API docs](tasks/00006A-idea-docs.md): load for the parser, transformer, and wrapper documentation task.
- [00007 Slice 4 SQL](tasks/00007-slice-4-sql.md): load for the SQL, first backend, and framework-sql task.
- [00007A SQL usage and API docs](tasks/00007A-sql-docs.md): load for the SQL core and SQLite documentation task.
- [00007B Consistent where_eq and insert_id Rustdoc](tasks/00007B-sql-where-eq-and-insert-id-docs.md): load for the SQL side quest from the docs review.
- [00008 Slice 5 UI](tasks/00008-slice-5-ui.md): load for the Leptos view/UI and Tauri hosting task.
- [00008A UI usage and API docs](tasks/00008A-ui-docs.md): load for the framework-view/framework-ui documentation task.
- [00009 Profile UI backed by SQLite](tasks/00009-profile-ui-on-sqlite.md): load for wiring the UI example to the SQLite-backed search.
- [00010 Owner review issues #1–#13](tasks/00010-owner-review-issues.md): load for the GitHub review-issue batch across lib, server, CLI and transformer.
- [00011 Slice 6 generated data layer](tasks/00011-slice-6-generated-data-layer.md): load for the Idea-to-SQL stores, operations and migrations task.
- [00012 Slice 7 production servers](tasks/00012-slice-7-production-servers.md): load for the production HTTP server and PostgreSQL/MySQL adapters task.
- [00013 Slice 8 more UI](tasks/00013-slice-8-more-ui.md): load for advanced components, schema-to-UI, language/session, native targets and streamed SSR.
- [Slice 2 lane A rounds receipt](slice-2-receipt-lane-a-rounds.md): load for the server-core refactor rounds 1–4 (module split, lifecycle readability, examples, safety evidence) and final server coverage.
- [Slice 2 lane B CLI receipt](slice-2-receipt-lane-b-cli.md): load for the `stackpress-cli` parsing/control/prompt/TTY implementation, its upstream inventory disposition, rounds 1–4, and verification.
- [Slice 2 lane B CLI execution receipt](slice-2-receipt-lane-b-run.md): load for `run`/`main_with`, stdout/stderr and exit-code policy, output authority, and verification.
- [Slice 2 lane D proof-app receipt](slice-2-receipt-lane-d-app.md): load for the `examples/capability-host` profiles proof application, its example-coverage measurement method, and which proof items are demonstrated.
- [Slice 2 lane C HTTP receipt](slice-2-receipt-lane-c-http.md): load for the buffered HTTP adapter's contract, limitations (e.g. list-valued session cookies need an explicit codec), tests, and coverage.
- [Slice 2 lane D CLI binary receipt](slice-2-receipt-lane-d-cli.md): load for the application `stackpress` binary and the end-to-end CLI = HTTP = direct-call proof.
