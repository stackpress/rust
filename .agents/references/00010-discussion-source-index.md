<!-- kb-import:start -->
> Scope: preserved discussion history. Confirmed current Rust decisions live in [project direction](../context/project-direction.md). Proposed APIs, old paths, and historical status in this source are not implementation instructions.
> Later acceptance: load [accepted source structure](../context/source-structure.md) for the current root layout and renamed public paths such as `stackpress::server` and `stackpress::sql_mysql`. Earlier namespace examples below remain historical source material.
> Source: `/Users/cblanquera/server/projects/stackpress/rust-notes/stackpress-rust-feasibility/README.md`; SHA-256 `62d14124c3c2fc0ddb76514ff9c0045a78387af1fc302a0584a584a8fb5fa5b2`; captured 2026-10-23. [Exact original](../resources/conversation/2026-10-23-stackpress-rust/README.md). Prose links are rebased; source content otherwise retained.
<!-- kb-import:end -->

# Stackpress Rust feasibility notes

Detailed working notes from the conversation on 2026-10-23 (Asia/Manila).

These notes preserve the author's requirements, current preferences, research findings, assistant proposals, and unresolved decisions. They are not an approved implementation specification or evidence that a Rust port has been built.

## Read first

1. [Working summary](00004-discussion-working-summary.md): objective, confirmed requirements, changes in direction, open questions, and proposed next steps.
2. [Feature inventory and feasibility](00005-discussion-feature-feasibility.md): the requested behavior and additional features across all six repositories.
3. [Crate and plugin architecture](00006-discussion-crate-plugin-proposals.md): proposed workspace, dependencies, compile-time plugin selection, generation, and readability conventions.
4. [Leptos, Tauri, and Vite](00007-discussion-frontend-decision-history.md): current UI preference, the actual comparison, platform boundaries, tradeoffs, and a proposed proof.
5. [Diagnostics and existing Rust parser](00008-discussion-diagnostics-existing-parser.md): async event error reports, the existing parser branch, and its distribution boundary.
6. [Cargo naming and namespace](00009-discussion-cargo-namespace.md): the requested `stackpress::` public namespace, Cargo package names, re-exports, and feature selection.

## Latest direction

- Preserve Stackpress's architecture and feature intent rather than JavaScript syntax.
- Plugins are selected before compilation. Loading new executable plugins into an already compiled application is not a requirement.
- Human readability, deliberate design patterns, consistent code patterns, separation of responsibility, abstraction, and extensive useful comments are priorities.
- Leptos + Tauri is the author's selected direction. This is an architecture decision, not evidence of a working integration.
- Share UI, application operations, contracts, and suitable domain code across web/mobile/desktop. Hardware/platform integrations and execution-host adapters remain at explicit boundaries.
- Use `stackpress-lib`, replacing the earlier proposed name `stackpress-core`.
- Preserve the general `request` (before operation), `response` (after operation), and `error` lifecycle events already present in Ingest.
- Plugin organization is recommended, not mandatory. The essential contract is a registration function; no plugin class, trait implementation, or fixed folder set is required for every plugin.
- Idea already has Rust parser work on `feature/rust-parser-2`; reuse must be assessed before writing another parser. Its Rust core is separate from its N-API binding and platform-specific npm packaging.
- Existing Idea, Inquire, and Stackpress repositories are monorepos. Repository, Cargo package, crate, plugin, and executable boundaries need not map one-to-one.
- Follow idiomatic Rust rather than a literal project-by-project conversion.
- Present the convenient public API under `stackpress::`. Rust identifiers use underscores, so the requested `stackpress::inquire-mysql` is expressed as `stackpress::inquire_mysql`. Cargo packages can retain hyphenated names such as `stackpress-inquire-mysql`; the facade explicitly re-exports them.
- Dioxus was the assistant's earlier recommendation. The user subsequently expressed concern about its direction and dislike of its syntax. Do not carry forward the earlier recommendation as an accepted choice.
- Tauri and Leptos perform different jobs and can be used together. Vite is a frontend build tool; a comparison with Leptos needs to identify the JavaScript UI framework it would build.
- No port, prototype, target build, benchmark, or dependency installation was performed in this conversation. Only these notes were created.

## Maintenance

Use the dated working summary as the conversation snapshot. Update technical detail notes when the author makes decisions or new evidence changes a finding. Record proposed and accepted directions separately. Do not infer authorization to implement, commit, publish, or deploy from the existence of these notes.
