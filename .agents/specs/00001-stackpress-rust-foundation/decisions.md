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

## Slice 2 owner decisions

Decided by the owner on 2026-10-24 after coordinator review of the [Slice 2 design decisions](slice-2-design-decisions.md). Details live in the linked Slice 2 design files.

| ID | Gap | Status | Decision / evidence |
| --- | --- | --- | --- |
| G12 (Q05) | Top-level lifecycle rules? | Accepted (owner): all four as proposed | A failed `request` stage skips the operation. An operation Abort stops the lifecycle, including the `response` stage; this deliberately fixes upstream discarding operation aborts. Any error replaces the response with a fresh public error response, discarding partial body, headers, and cookies. An `error`-listener failure keeps both failures, never recurses, and falls back to a sanitized generic 500. Full table in [lifecycle](slice-2-design-lifecycle.md). |
| G13 (S2-D01, D04, D05) | What does Slice 2 include? | Accepted (owner): HTTP + prompts now | The buffered, feature-gated, in-process HTTP adapter (no listener) and full terminal prompt/control support with an optional crossterm TTY driver are in scope, implemented as parallel lanes after M0. A real network gateway, streaming, and serverless are deferred to a separate later task. |
| G14 (S2-D02, D03) | CLI packaging and syntax? | Accepted (owner): app binary only; operator flags before `emit` | Ship the reusable `stackpress-cli` library and have each application compile its own `stackpress` binary (the proof app in `examples/capability-host`). No empty stock binary. Operator flags (`-v`, `-f`, `-b`) come before `emit`; every token after the event is capability data. Exit code policy as in [adapters](slice-2-design-adapters.md). |
| G15 (Q10, Q04, S2-D07, Q14, Q01) | Host policies? | Accepted (owner): all as proposed | **Config:** typed JSON sections layered defaults → files → code, with unknown top-level sections rejected. **Plugins:** run in the listed order, and a wrong dependency order is an error, not an automatic reorder. **Routes:** captures override same-named request data keys. **Correlation:** per-host IDs only, with no distributed tracing yet. **First end-to-end proof:** an in-memory `profile-search` via CLI and HTTP. |
| G16 (S2-D06, S2-D08) | Lib changes and dependency budget? | Accepted (coordinator, within authority) | Additive borrowed host context on `Invocation` (`emit_with_context`/`resolve_with_context`/`Invocation::context`), plus `Handler`/`TaskHandler` forwarding only if M0 compiler evidence requires it. Dependencies as budgeted in [adapters](slice-2-design-adapters.md); additions return for review. |
| G17 (G12 clarification) | What happens when top-level input fails before any normalized `Request` exists (e.g. `call` with a non-object input, or HTTP too malformed to normalize)? | Accepted (coordinator, 2026-10-24; consistent with the approved HTTP malformed-request rule) | Allocate a correlation ID, record the Input-stage failure in `HandleReport`, and return a fresh sanitized public error response with status 400. Run no `request`/`error`/`response` callbacks and never fabricate a `Request`. Input failures after normalization (load or body parse) follow the full G12 table. Raised by lane A during M2. |

## Slice 3 owner decisions

Decided by the owner on 2026-10-24 after coordinator review of the [Slice 3 design decisions](slice-3-design-decisions.md).

| ID | Gap | Status | Decision / evidence |
| --- | --- | --- | --- |
| G18 (S3-D01, S3-D02, Q08) | Extract, depend on, or rewrite the pinned Rust parser, and which TS behaviors to keep? | Accepted (owner): extract, repair, correct grammar | Copy `idea_parser_core` (pinned `6c289d1`) into `crates/idea/idea-parser` with provenance and license retention. Fix bounded recursion (no stack-overflow abort), plugin/declaration order, env values in every scalar position, and diagnostic spans. Adopt the explicit corrected grammar in [grammar](slice-3-design-grammar.md) rather than reproducing TS quirks. Unchanged reuse remains vetoed. |
| G19 (S3-D03) | Does `!` protect an immutable local declaration from imported replacement under `use`? | Accepted (owner): **match current upstream code** | Reproduce the TypeScript `Transformer.schema` behavior, in which an imported child replaces an immutable local declaration. This is a deliberate exception to the KB catalog wording in reference 00022, and it must be documented and tested explicitly. All other merge-table rules in [grammar](slice-3-design-grammar.md) stand. |
| G20 (S3-D04, S3-D05, S3-D07, Q09) | How does generation run, what does the first proof own, and what output promises apply? | Accepted (owner): separate codegen tool | **Tool:** an independent app-owned pre-build codegen binary, run as `cargo run -p <app>-codegen -- emit generate` through `stackpress_cli::main_with`. **Output:** inspectable Rust in a dedicated, marked output root with per-plugin partitions and a single aggregate owner; hand-written files are never overwritten; only promised owned outputs are removed; `--check` detects stale output; there is no crash-atomic guarantee. **Plugins:** function-registered transforms in the listed order, matching the `.idea` plugin-block order. **First proof:** a new `examples/generated-profile` whose `Profile` model generates public types plus the `profile-search` operation and route, with CLI/HTTP/direct equality to Slice 2, byte-stable repeat generation, and model removal producing 404. No `framework-schema` crate yet. `build.rs`/`OUT_DIR` is deferred. |
| G21 (S3-D06) | Build a general template engine now? | Accepted (owner): defer | Implement only native source resolution and owned output IO in the transformer. The template engine, helpers, and render are a separate future task when a generator needs one. |
| G22 (S3-D08) | Slice 3 dependency budget? | Accepted (coordinator, within authority) | serde/serde_json with order support, `tempfile` for staged installs, SHA-256 for owned-file drift checks, and the existing CLI/server/tokio for the tool adapter. These are build-time only, with no new edges in the runtime defaults. Additions return for review. |
| G23 | Run Slices 4 and 5 in parallel? | Accepted (owner, 2026-10-25) | Yes. Both design checkpoints run in parallel now, and implementation is parallel after owner approval of both designs. **Shared-crate rule:** any change to `stackpress-server` (including the HTTP adapter), `stackpress-lib`, the CLI or the Idea crates goes through the coordinator and lands in sequence. Lanes stop and report rather than edit those crates. The total stays at 4 Sol agents, about two lanes per slice during implementation. The coordinator resolves shared manifest, lockfile and facade conflicts at merge. |

## Carried design questions for later slices

These come from [open design questions](../../references/00039-open-design-questions.md). Numbers match that reference.

| ID | Gap | Status | Owner / current position |
| --- | --- | --- | --- |
| Q01 | Which first vertical capability/model and which first native target? | Partly answered (G15) | First capability: in-memory `profile-search` via CLI and HTTP. Generated model (Slice 3) and the first Tauri target (Slice 5) remain open. |
| Q02 | Which operations run remotely, locally, or both; is offline required? | Unresolved | Owner, before Slice 5. |
| Q03 | Precise async handler signature and context access? | Answered for Slice 1 | G06 and the [API contract](design-api-contract.md); host context enrichment remains Slice 2. |
| Q04 | Tie order, matching precedence, handles, plugin dependencies, registration conflicts? | Answered (G15) | Ties run in registration order; handles implemented in Slice 1. Plugins have unique IDs, run in the listed order, and wrong order or missing dependencies are startup errors; singleton config/service conflicts error; duplicate listeners stay all-matching. |
| Q05 | Top-level CLI/HTTP/Tauri lifecycle versus nested emit; abort, finalization, sent state, error-handler failure? | Answered for CLI/HTTP (G12) | See [lifecycle](slice-2-design-lifecycle.md). Tauri entry is defined in Slice 5. |
| Q06 | First database/backend, first serverless adapter, PGlite? | Unresolved | Owner, before Slice 4. Assumption to verify: SQLite first for test simplicity. |
| Q07 | ECMAScript regex compatibility versus a Rust path-pattern contract? | Answered | Rust `regex` syntax with explicit options; no ECMAScript lookaround/backreference parity (D3, [API contract](design-api-contract.md) B4). |
| Q08 | Rust parser grammar/schema parity and extraction strategy? | Answered (G18, G19) | Extract and repair the pinned core with the corrected grammar; immutable `!` follows upstream code replacement. M0 proves the policy. |
| Q09 | Generator packaging/order, output cleanup, partitions, build/runtime consistency? | Answered (G20, G21) | Independent app-owned codegen tool, owned output root with explicit cleanup, ordered function transforms, `--check` freshness; template engine deferred. |
| Q10 | Configuration authoring/loading/merging and runtime validation? | Answered (G15) | Typed package-owned JSON sections, layered and validated at startup; unknown sections rejected. |
| Q11 | Leptos routing, metadata/Head, server props/hydration, Tauri operation adapter? | Unresolved | Slice 5. |
| Q12 | Which React/Frui/r22n behavior to reimplement? | Unresolved | Owner, before Slice 5. |
| Q13 | Feature catalog/defaults, item-level APIs, dependency wiring, MSRV/edition, versions, publication? | Partly answered | Edition 2024 and MSRV 1.96 (G09); no feature flags yet; the rest unresolved. |
| Q14 | Error/report type, tracing, caller metadata, backtraces, disclosure, correlation? | Partly answered (G15) | Slice 1 `Error` plus per-host correlation IDs and lifecycle stage in reports. Tracing and cross-process correlation remain unresolved. |
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
