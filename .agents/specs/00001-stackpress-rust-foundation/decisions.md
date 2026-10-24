# Decisions and Gap ledger

Spec file of [00001 Stackpress Rust foundation](index.md). Load before planning or implementing any slice: every material Gap is a question paired with an accepted decision, an assumption to verify, or an explicit unresolved status and owner. Spec decisions stay here, separate from `.agents/context/`, until the owner approves promotion. Where this ledger and [context](../../context/index.md) conflict, context wins and the conflict becomes a new Gap.

## Process and coordination

| ID | Gap | Status | Decision / evidence |
| --- | --- | --- | --- |
| G01 | Who implements and who reviews? | Accepted (owner, 2026-10-24) | The implementer agent (Codex `gpt-6.1-sol`, thinking `xhigh`, via Paseo) writes all code, tests, and refactor rounds. The coordinator (Claude Opus) gives direction from the KB and reviews DX/API ergonomics, breadth against upstream, KB conformance, and docs/comments, re-running checks to verify claims. The owner decides public-behavior and product questions. |
| G02 | What is the implementer's safety scope? | Accepted (owner, 2026-10-24) | Safety, soundness, and Rust best practice across all code with an open-ended mandate and a soundness veto. Named prior failures (deadlock, concurrent nested dispatch, poisoning, `Clone` silently aliasing) are a required floor, not the scope; the implementer builds its own risk inventory per slice. |
| G03 | Which sources may inform this trial? | Accepted (owner, 2026-10-24) | Only branch `trial/3`, the KB, and upstream sibling checkouts. Other `trial/*` branches and their stale `target/` output are never read or used as evidence. |
| G04 | Where are planning records and receipts tracked? | Accepted (owner, 2026-10-24); supersedes the earlier `docs/trial-3/` choice | In this spec package, separate from the KB, "until all is well". The former `docs/trial-3/` design and receipts were moved losslessly into the design and task files. No context promotion until the owner approves it. |
| G05 | How are milestones gated? | Accepted (coordinator, 2026-10-24) | A design checkpoint before code, a stop after the riskiest proof, a stop before refactor rounds, then rounds 1–4 and a closeout receipt, each with coordinator re-verification. |

## Slice 0+1 owner decisions

| ID | Gap | Status | Decision / evidence |
| --- | --- | --- | --- |
| G06 (D1) | Must handler dispatch futures be `Send`? | Accepted (owner): require `Send` now | Coordinator probe on rustc 1.96 stable showed named `async fn` with a lifetime-indexed helper trait compiles with `Send` futures; `AsyncFn` with a `Send` returned-future bound needs unstable `async_fn_traits`; a closure returning a borrowing `async move` fails on lifetimes; a closure through an identity boxing helper compiles. Proved in production by [P1](proofs.md). |
| G07 (D8) | How should cloning work for data containing native values? | Accepted (owner): Clone + SharedNative | Ordinary Value/Nest data clones deeply; only `Value::Shared(SharedNative)` shares an Arc payload, visible in the type. Request/Response stay non-Clone. |
| G08 (D2–D6) | Accept deliberate deviations from upstream behavior? | Accepted (owner): all | Builder-time registration then immutable table; captures stay in Invocation for raw emit; strict parsers/limits and upstream bug fixes; one-attempt load/delivery without retry; cookie removals keep scope; public errors omit developer diagnostics. |
| G09 (D7) | What edition and MSRV? | Accepted (owner): declare 1.96 for now | Edition 2024, resolver 3, `rust-version = "1.96"` (the installed, verifiable compiler). Revisit before publication. |
| G10 | Which coordinator DX changes apply? | Accepted (coordinator review) | Dot-path and typed getters; `Serialize` results/rows/json; one-line priority/label registration; bootstrap-mapping example; uniform Nest naming; infallible `Method` constants; `Invocation::param`; readable `DataPath` display; Response accessor-guard docs; in-place Nest mutation; conventional doc-example formatting and 100-column wrapping. |
| G11 | What is the slice plan? | Accepted (owner/coordinator) | Slice 0 workspace; Slice 1 `stackpress-lib`; Slice 2 `stackpress-server` + CLI adapter in `tools/`; Slice 3 Idea (parser assessment, transformer, generation, filesystem, templates); Slice 4 SQL; Slice 5 Leptos/Tauri UI. See the [sprint plan](tasks/sprint.md). |

## Carried design questions for later slices

These come from [open design questions](../../references/00039-open-design-questions.md). Numbers match that reference.

| ID | Gap | Status | Owner / current position |
| --- | --- | --- | --- |
| Q01 | Which first vertical capability/model and which first native target? | Unresolved | Owner. Assumption to verify: the profile-search-like capability proposed in 00039. |
| Q02 | Which operations run remotely, locally, or both; is offline required? | Unresolved | Owner, before Slice 5. |
| Q03 | Precise async handler signature and context access? | Answered for Slice 1 | G06 and the [API contract](design-api-contract.md); host context enrichment remains Slice 2. |
| Q04 | Tie order, matching precedence, handles, plugin dependencies, registration conflicts? | Partly answered | Ties run in registration order; handles, replacement, and removal implemented. Plugin dependencies and registration conflicts unresolved (Slice 2). |
| Q05 | Top-level CLI/HTTP/Tauri lifecycle versus nested emit; abort, finalization, sent state, error-handler failure? | Unresolved | Slice 2 design checkpoint. Slice 1 dispatch runs no lifecycle hooks. |
| Q06 | First database/backend, first serverless adapter, PGlite? | Unresolved | Owner, before Slice 4. Assumption to verify: SQLite first for test simplicity. |
| Q07 | ECMAScript regex compatibility versus a Rust path-pattern contract? | Answered | Rust `regex` syntax with explicit options; no ECMAScript lookaround/backreference parity (D3, [API contract](design-api-contract.md) B4). |
| Q08 | Rust parser grammar/schema parity and extraction strategy? | Unresolved | Slice 3: assess the pinned `feature/rust-parser-2` core against current fixtures first. |
| Q09 | Generator packaging/order, output cleanup, partitions, build/runtime consistency? | Unresolved | Slice 3. |
| Q10 | Configuration authoring/loading/merging and runtime validation? | Unresolved | Slice 2. |
| Q11 | Leptos routing, metadata/Head, server props/hydration, Tauri operation adapter? | Unresolved | Slice 5. |
| Q12 | Which React/Frui/r22n behavior to reimplement? | Unresolved | Owner, before Slice 5. |
| Q13 | Feature catalog/defaults, item-level APIs, dependency wiring, MSRV/edition, versions, publication? | Partly answered | Edition 2024 and MSRV 1.96 (G09); no feature flags yet; the rest unresolved. |
| Q14 | Error/report type, tracing, caller metadata, backtraces, disclosure, correlation? | Partly answered | Slice 1 `Error` with cause chain, registration sites, invocation frames, public/private separation; no tracing dependency. Tracing and cross-process correlation unresolved (Slice 2). |
| Q15 | Target/build/test matrix and platform exclusions? | Partly answered | Only native `aarch64-apple-darwin` verified; matrix unresolved. |

## Slice 1 design decisions applied

The following section was moved losslessly from `docs/trial-3/slice-1-design.md` section D (lines 617–638) at commit `e230359`; headings are demoted one level and links rebased.

### D. Decisions applied

The owner/coordinator approved implementation with the following changes on 2026-10-24. These replace the previous open recommendations; no further product choice is pending for M0.

| Decision | Accepted contract |
| --- | --- |
| **D1** | Send dispatch futures now. Named async fns register directly through lifetime-indexed Fn/Future adaptation; closures use one small higher-ranked boxing helper. No unstable associated-type bounds or unsafe workaround. Stop with compiler evidence if the real three-argument contract cannot be proved. |
| **D2** | Builder-time mutation/removal/composition and handle replacement, immutable runtime table; listener-local captures for raw emit. |
| **D3** | Approved explicit parser/matcher grammar, limits and documented bug corrections. |
| **D4** | Approved staged one-attempt load/delivery and finalization guards; no implicit retries. |
| **D5** | Approved breadth and neutral resolve; use corrected later slices: 2 server+CLI, 3 Idea/generation/filesystem/templates, 4 SQL, 5 UI. |
| **D6** | Preserve cookie removal scope; default public error views omit developer diagnostics. |
| **D7** | Edition 2024, resolver 3, rust-version 1.96; verify installed stable, install no toolchain. |
| **D8** | Value/Nest/data-bearing values are Clone, ordinary nodes deeply independent; only the explicitly named Shared(SharedNative) variant shares an Arc payload. Request/Response remain non-Clone to avoid duplicating attempt state. |
| **DX** | Literal get plus common string dot-path reads/typed reads/setters, scalar accessors; Serialize-based results/rows/json plus explicit Value APIs; one-line priority/label registration; bootstrap mapping example. |

String dot-path reads parse internally. `path` returns None on invalid/missing paths; `path_as` and mutation return an invalid-path error rather than silently discarding it. Parsed DataPath remains optional for reuse/literal segment addressing. `results`/`rows`/`json` accept borrowed Serialize values including typed structs and serde_json::Value; serialization errors return Result before any state mutation. Value implements Serialize for its JSON variants and deliberately rejects non-JSON variants with useful paths; `results_value`/body-value methods preserve those values in-process without forcing serialization. `rows` verifies an array result rather than trusting arbitrary Serialize output.

No HTTP framework/database/application/UI/lifecycle implementation is added to Slice 1. The soundness veto and the separately scheduled reviews remain in effect.

M1–M3 implementation refinements: one-shot loaders/senders take isolated owned data snapshots so caller-borrowed native readers/writers remain possible; envelope cloning/moving is infallible and wire conversion is explicitly fallible; optional `errors: Option<Value>` distinguishes absent errors from an explicit empty object. Data/path recursion has a hard ceiling of 64 in addition to caller-lowered budgets. These details and current verification are recorded in the [M1–M3 checkpoint receipt](tasks/00002-foundation-data-events-and-values.md); post-M3 rounds are still pending.
