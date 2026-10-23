# Open design questions and first-proof proposal

Owner: [project direction](../context/project-direction.md). Load when preparing an implementation brief. Questions and suggested proofs here are not accepted design answers or a Frozen implementation spec.

## Settled; do not re-ask by default

- One monorepo at the new Rust repository.
- Leptos + Tauri; shared web/mobile/desktop source with explicit hardware/host boundaries.
- `stackpress-lib`; convenient `stackpress::` imports, using valid Rust identifiers.
- Root Cargo workspace and root `stackpress` facade, grouped framework/Idea/SQL crates, `server`/`sql` naming, complete initial package/namespace catalog, separate optional backend adapters as implemented, and initial combined `framework-ui`. Read [accepted source structure](../context/source-structure.md) for the tree and dependency/responsibility boundaries; do not re-ask the earlier layout questions.
- Compile-time plugin selection; disable via rebuild/redeploy.
- Registration function is the essential plugin contract; folder conventions remain optional.
- Idiomatic Rust rather than a literal JS/package conversion.
- Rich Rustdoc/comments and several test-verified refactoring rounds.
- 70% development coverage and 90% publication-preparation coverage.

## Design work still needed

1. Which first vertical capability/model and which first native target?
2. Which operations execute remotely, locally, or both? Is offline storage/synchronization required?
3. What is the precise async handler signature and context access model? Two req/res arguments versus a typed props/context shape remains a design choice.
4. Equal-priority tie order, matching/capture precedence, listener replacement handles, plugin dependencies, and registration conflicts.
5. Top-level CLI/HTTP/Tauri lifecycle versus nested emit/resolve; abort, response finalization, sent-state, and error-handler failure behavior.
6. First database/backend and first serverless adapter; whether exact PGlite integration is required.
7. Required ECMAScript-regex compatibility versus a deliberate Rust path-pattern contract.
8. Rust parser grammar/schema parity with current fixtures, import/environment behavior, and extraction strategy from the branch.
9. Generator plugin packaging/order, output cleanup/ownership, shared/generated code partitions, and build/runtime selection consistency.
10. Configuration authoring/loading/merging and runtime validation; typed Rust composition versus data files with registered mechanisms.
11. Leptos routing/page selection, metadata/Head, server props and hydration integration; Tauri operation adapter and platform integrations.
12. Current React/Frui/r22n behavior to reimplement, especially role-specific generated fields/views, localization, notifications, accessibility, and account flows.
13. Feature catalog/defaults, item-level public APIs, detailed dependency wiring, MSRV/edition, versions, and registry ownership/publication policy. The initial crate/package/module-path catalog is accepted in [source structure](../context/source-structure.md); further crates require actual implementation needs rather than copying the old proposal.
14. Error/report type and tracing dependencies, caller metadata, source-chain/backtrace capture, public diagnostic disclosure, and cross-process correlation.
15. Explicit target/build/test matrix, generated-code coverage treatment, and any tool-specific platform exclusions.

## Suggested first proof, not a scheduled commitment

Use one Idea model and one profile-search-like capability to demonstrate typed generation, a minimal function plugin, priority/abort behavior, manually constructed request/response, CLI and HTTP calls, error reporting, a Leptos SSR page with matching hydration, and a Tauri client built from shared UI/operation contracts.

Prove a disabled plugin is excluded and a customization plugin changes behavior without editing the original. Test nested/concurrent event diagnostics and response/lifecycle order. Repeat generation and compile generated output. Add more adapters/targets after the first proof rather than declaring all-platform parity from one successful build.

Do not copy this proposal directly into code as a full spec. Use the user's next requested scope to choose the smallest coherent contract and resolve blocking questions while progressing independent work.
