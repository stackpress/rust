<!-- kb-import:start -->
> Scope: preserved discussion history. Confirmed current Rust decisions live in [project direction](../context/project-direction.md). Proposed APIs, old paths, and historical status in this source are not implementation instructions.
> Later acceptance: load [accepted source structure](../context/source-structure.md) for the approved root workspace/facade, grouped crate tree, and package/namespace catalog. Statements below about unaccepted layout or earlier Ingest/Inquire names describe the original discussion, not current direction.
> Source: `/Users/cblanquera/server/projects/stackpress/rust-notes/stackpress-rust-feasibility/2026-10-23-stackpress-rust-working-summary.md`; SHA-256 `3bd0d0560371660deb000089dd9114c893552642a969d88414a7ab7fc4a5e8d9`; captured 2026-10-23. [Exact original](../resources/conversation/2026-10-23-stackpress-rust/2026-10-23-stackpress-rust-working-summary.md). Prose links are rebased; source content otherwise retained.
<!-- kb-import:end -->

2026-10-23-stackpress-rust-working-summary.md

# Working Summary

## Summary

The Stackpress author is exploring a Rust version of the framework and its foundation libraries. The aim is to retain the features and programming model the author values: ordered events, mutable transport-neutral requests/responses, plugins, configuration, Idea generation, reusable capabilities, server-first pages, and configured application features.

The author is open to existing libraries only when they fit the requirements precisely. Superficially similar routing, authentication, rendering, or database libraries must not silently narrow Stackpress behavior.

The author also wants reactive UI templates that can serve websites and potentially installed iOS, Android, Windows, macOS, and Linux applications.

## Scope Boundary

Source repositories discussed and inspected during this conversation:

- `/Users/cblanquera/server/projects/stackpress/lib`
- `/Users/cblanquera/server/projects/stackpress/idea`
- `/Users/cblanquera/server/projects/stackpress/ingest`
- `/Users/cblanquera/server/projects/stackpress/inquire`
- `/Users/cblanquera/server/projects/stackpress/reactus`
- `/Users/cblanquera/server/projects/stackpress/stackpress`

The work is feasibility research and architecture discussion. The author requested durable notes in `/Users/cblanquera/server/projects/stackpress/rust-notes/<workspace for note files>`. This workspace fulfills that request.

No implementation, repository initialization, dependency installation, production change, or commitment to complete the port has been authorized by this discussion.

## Current State

As of 2026-10-23:

- The assistant inspected current local source and accepted Stackpress context, and consulted primary Rust/UI documentation.
- Most requested behavior appears implementable in Rust without changing the event/config/plugin architecture.
- No Rust prototype or target build has been run. Feasibility is an engineering assessment, not tested parity.
- The author clarified that plugins are enabled/disabled before compilation.
- JavaScript syntax compatibility is subordinate to readability and architectural intent.
- The author has selected Leptos + Tauri as the UI/installed-app direction.
- Shared web/mobile/desktop source, apart from platform/hardware-specific integration and necessary host adapters, is an explicit design requirement.
- The author requested the name `stackpress-lib` in place of the assistant's earlier `stackpress-core` name.
- General `request`, `response`, and `error` events must remain central lifecycle extension points.
- The existing Idea Rust parser branch was inspected at commit `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`; its parser core and Node/N-API wrapper are separate crates. No parser build/tests were run.
- The author has not accepted the assistant's proposed crate tree, named dependencies, or implementation sequence as a frozen design.
- No relevant previous handover summaries were present in the current repository's `notes/chat-session-handover/`; the requested `rust-notes` directory did not exist before these notes.

## Key Decisions And Confirmed Requirements

### Preserve these capabilities

1. Event emitters with priorities and cooperative abort by returning false.
2. Express-style method/path routing implemented through events, with framework request/response values.
3. Manually constructible and mutable Request/Response objects for CLI and in-process use.
4. CLI invocation such as `stackpress emit profile-search --id abc123` using the same named capability as other surfaces.
5. Plugin organization and enable/disable selection.
6. Event-driven error handling.
7. Idea declarations and plugin-contributed code generation before final application compilation.
8. Server and serverless transport adaptation.
9. Reactive templates, server-first website rendering, and server props.
10. Framework internals that follow the same event/config/plugin conventions available to applications.
11. Built-in account/auth/session, API, roles, and permissions driven through configuration.
12. An itemized assessment of the six source repositories, including features beyond the initial list.

### Plugin selection is a build-time concern

The author's examples are:

- Keep experimental work in the codebase but disabled until ready.
- Disable an error-prone production plugin until fixed.
- Add features without changing core plugins.
- Slightly alter an existing plugin without editing the original.
- Incrementally extend functionality beyond MVP.
- Copy plugins or use them as a basis between projects.
- Integrate third parties without disrupting main application logic.

The production consequence is rebuild and redeploy after changing the selected plugin set. Ordinary compile-time plugins share a process when enabled; organizational boundaries are not crash containment.

### Readability matters more than JavaScript syntax

The author's explicit priorities are methodically chosen design patterns, consistent code patterns, separation of responsibility, abstraction, and a lot of comments so human programmers can easily understand the code.

Do not reinterpret this as requiring sparse comments, maximal terseness, or heavy macro-based APIs. Preserve deliberate explanatory comments about intent, contracts, execution order, assumptions, and non-obvious implementation details.

### Selected UI direction and shared code

The author said: "Leptos looks good actually. I can see myself using it."

The author subsequently said "okay sold with leptos + tauri." Leptos + Tauri is now selected. The author also requires the same application source to be usable on web/mobile/desktop, except hardware-specific code. Preserve shared UI and capability/domain code with explicit host/transport adapters; do not assume identical binaries or that server-only behavior runs inside the browser. No implementation or full UI rewrite has been requested yet.

### Idiomatic Rust and plugin flexibility

The author is comfortable with changing package/project boundaries and does not require a 1:1 conversion. Idea, Inquire, and Stackpress are existing monorepos; do not treat each repository as one package or crate.

The preferred library name is `stackpress-lib`. A plugin needs a registration function, analogous to the old default-exported function. `events/`, `pages/`, `views/`, and `tests/` are recommended organization, not mandatory directories. A plugin can be a local module or reusable crate; adding a class/trait obligation would change the intended minimal contract.

### Event diagnostics

The author asked whether Rust can address poor anonymous/async event stack reports, referring to `lib/src/Exception.ts`. Preserve structured status/validation/source information, underlying causes, and useful human-readable diagnostic context. The assistant proposes automatic plugin/event/handler metadata, async tracing, and optional native backtraces. Exact Rust error type and dependencies remain open.

### Cargo and public namespace

The author wants the `stackpress` namespace, illustrated as `stackpress::inquire-mysql`. Preserve this public API intent using valid Rust syntax: `stackpress::inquire_mysql`. Separate hyphenated Cargo packages can remain independently usable and be explicitly re-exported by the `stackpress` facade. Cargo package names do not contain `::`; this is a Rust API namespace, not an npm-style package scope. The assistant proposes feature-gating optional facade exports such as the MySQL adapter. Exact package availability, exports, and feature names have not been reserved or implemented.

## Assistant Proposals And Assumptions

These are recommendations from the discussion, not author-approved decisions:

- A single Cargo workspace containing focused crates and a small aggregate `stackpress` facade.
- A custom dispatcher preserving Stackpress event behavior rather than delegating semantic control to a conventional Rust HTTP router.
- Compiled plugin crates or feature-gated modules, selected explicitly by the application build.
- Separate generation/build dependencies from generated application runtime dependencies.
- A renderer-neutral view contract, with a Leptos adapter implementing the selected UI direction.
- Website SSR/hydration and installed-app rendering as different hosts for shared UI source.
- Explicit host adapters around the author-selected Tauri shell; a JS frontend is a historical alternative, not the current direction.
- A vertical proof before large-scale porting.

Names in the proposed crate tree are illustrative. No crates have been created or names reserved.

## Conflicts Or Changes

- Earlier assistant advice favored retaining Reactus/React initially to preserve existing components. The author clarified that literal JS compatibility is not the priority and is interested in Leptos. React retention is now an alternative, not the default assumption.
- The assistant then recommended Dioxus as a unified-platform candidate. The author subsequently expressed concern that its owners are pivoting and disliked its syntax, then chose Leptos + Tauri. Preserve the pivot statement as the author's concern; it was not independently verified.
- The earlier runtime/dynamic-plugin discussion is no longer a requirement. Compile-time selection is the stated model.
- Leptos versus Tauri is not an either/or choice. Leptos supplies UI/reactivity; Tauri supplies a native application host. Vite supplies build/development tooling for a chosen frontend.

## Open Questions

1. Must installed applications use native OS widgets, or is a system WebView acceptable?
2. How should the selected Leptos renderer integrate page selection, metadata, hydration, and build tooling with Stackpress?
3. How should the website and Tauri client select remote versus local operation adapters while sharing source?
4. Which existing React/Frui/r22n components must be preserved versus rewritten?
5. Do installed apps call a remote server, embed selected local capabilities, or support both?
6. Is offline operation required, and for which operations?
7. Which serverless platform is the first supported target?
8. Which database is first, and is exact embedded PGlite behavior required?
9. Which route/regex behaviors require literal parity, including arbitrary JavaScript regex syntax?
10. What is the intended response to a missing required plugin or conflicting replacement registration?
11. How should build-time configuration select Cargo features and generator plugins from one application selection?
12. Which crate boundaries should exist in the first proof versus later releases?
13. Should direct `emit` and nested `resolve` bypass the general request/response lifecycle, as distinct from a top-level handled invocation? Define this deliberately; wrapping every nested event risks recursive lifecycle execution.
14. What happens when a lifecycle listener aborts, a response is already sent, or the error handler itself fails? Preserve original failure context and avoid recursive error dispatch.
15. What parity work remains on the existing Rust parser branch? Source separation is verified; completeness is not.

## Blockers Or Risks

- No measured implementation or compatibility proof exists yet.
- SSR in a browser and local rendering in a packaged Tauri app are different execution modes.
- Rust/WASM plus JavaScript interop may be more work than a JS frontend for some UI dependencies.
- Rust async handler ergonomics and nested event invocation need a careful internal design.
- Cargo features are additive; accidental transitive/default features can include unwanted dependencies.
- Generator dependency cycles and inconsistent build/runtime plugin selections can break compilation.
- PGlite, arbitrary JS regexes, V8 reflection, callable objects, dynamic imports, and React components are explicit compatibility boundaries.
- A plugin can be disabled without undoing database migrations or cleaning up data it already created. Such state needs an explicit policy if it matters.

## Next Actions

- Action: If implementation is requested, prove the selected Leptos + Tauri stack with shared component/operation source and distinct web/Tauri host adapters.
  Owner: Not assigned.
  Target timing: Not specified; proposed only.
- Action: Draft a small parity contract for event matching, order, cancellation, nested dispatch, mutable request/response, and error handling.
  Owner: Not assigned.
  Target timing: Before implementation; proposed only.
- Action: If implementation is requested, prove one generated model across CLI, HTTP, a Leptos SSR page, and a packaged client.
  Owner: Not assigned.
  Target timing: Not specified; proposed only.
- Action: Decide target platforms and database before making broad support claims.
  Owner: Stackpress author, with implementation/research support if requested.
  Target timing: Not specified.

## References

- [Feature inventory](00005-discussion-feature-feasibility.md)
- [Crate and plugin architecture](00006-discussion-crate-plugin-proposals.md)
- [Frontend comparison](00007-discussion-frontend-decision-history.md)
- [Diagnostics and existing Rust parser](00008-discussion-diagnostics-existing-parser.md)
- [Cargo naming and namespace](00009-discussion-cargo-namespace.md)
- Current conversation is the primary authority for requirements and preferences.
- Source and external links in the companion notes support technical findings and remain subject to version drift.
