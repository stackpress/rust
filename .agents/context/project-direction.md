# Stackpress Rust project direction

Authority: direct project-owner decisions in the preserved conversation, [setup request](../resources/conversation/2026-10-23-setup-request.txt), and [source-structure acceptance](../resources/conversation/2026-10-23-source-structure-acceptance.txt). These govern the Rust target; source-language behavior is referenced separately.

Read [the next-session handoff](../references/00040-session-handoff.md) when continuing in the new repository; it records the installed surface, authoritative decisions, outstanding design, and setup receipt.

## Identity and intent

The author wrote the Stackpress libraries and wants to retain their useful features and understandable event/config/plugin architecture in Rust. Readability is a core requirement: deliberate design patterns, consistent patterns, separation of responsibility, abstraction, Rustdoc, and a lot of explanatory comments.

The project is one monorepo at `/Users/cblanquera/server/projects/stackpress/rust`. Existing repository and npm package boundaries do not dictate Rust crates. Use idiomatic Rust rather than mechanically copying classes, module systems, layouts, or JavaScript behavior.

The underlying goal is reusable named capabilities with adapters for CLI, HTTP, pages/admin, APIs, MCP, installed applications, and plugins. Exact additional package implementation/release scope still needs planning. Existing features are an assessment inventory, not a promise to build every package immediately.

## Confirmed foundations

- Event listeners have priorities and cooperative abort by false; handlers run sequentially where mutation/order are observable.
- Routes use Express-style method/path event matching and framework request/response values.
- Request/Response can be constructed without sockets, mutated, and used from terminal/in-process invocation, including `stackpress emit profile-search --id abc123`.
- Preserve general request/response/error lifecycle extension points.
- Plugins contribute services, events, routes, policy, and generation; the framework itself follows the same conventions.
- Idea drives plugin-extensible generation before final application compilation.
- Retain configured auth/account/session, API, roles/permissions, and server props as feature intent; choose idiomatic mechanisms and prove parity where required.
- Existing compatible libraries are welcome only behind contracts that actually satisfy the requirements. Do not silently trade away semantics for a superficially similar framework.

## UI and cross-platform code

Leptos + Tauri is selected. Share Leptos components, public contracts, applicable validation, and operation/domain source across website/mobile/desktop. Hardware and execution-host differences belong at explicit boundaries. Platform-specific entrypoints/build outputs are expected; duplicate independent domain implementations are not.

Websites render initial HTML on the server and hydrate in the browser. Installed Tauri applications normally load packaged frontend assets and render locally. Calls reach remote capabilities or explicitly selected native-host capabilities through adapters. Shared source does not give browser WASM direct server-secret/database access.

The author disliked Dioxus syntax and expressed concerns about its direction. This superseded an assistant recommendation. The claimed maintainer pivot was not independently verified and is not a project fact to repeat.

## Cargo and naming

- Name the foundation `stackpress-lib`.
- Start directly at the repository root with a Cargo workspace and root package `stackpress`; root `src/lib.rs` supplies the public facade. Do not add an extra `rust/` directory or move the checkout.
- Use the accepted grouped `crates/framework/`, `crates/idea/`, `crates/server/`, and `crates/sql/` families, plus `crates/lib/`. The server family contains `crates/server/server/` and `crates/server/http/`; its grouping directory has no package. Ingest becomes `server`; Inquire becomes `sql`; the framework family uses `framework-*` names.
- Provide convenient `stackpress::` imports using valid Rust identifiers, e.g. `stackpress::sql_mysql` and `stackpress::framework_admin`. Cargo packages use the `stackpress-` prefix, e.g. `stackpress-sql-mysql`; explicit facade re-exports and optional feature selection supply the public paths.
- Independent crate use remains possible; a shared public namespace does not require centralized implementation.
- Read [accepted source structure](source-structure.md) for the complete tree, package/path catalog, standalone Idea wrapper, SQL/framework-SQL boundary, combined initial Leptos integration/components crate, and acyclic dependency rules.
- The initial source/package/namespace catalog is accepted. Feature catalog/defaults, detailed dependency graph, versions, registry availability, and publication plan remain unresolved. Backend crates are introduced as implemented; their placement does not prove backend support. No names were reserved or packages published.

## Plugin contract

The essential contract is a registration function. Recommended `events/`, `pages/`, `views/`, and `tests/` organization is optional. A small plugin may be one module/file; a reusable crate may point `[lib] path` at root `plugin.rs`. Do not require a class, a trait implementation, or service/config/integration directories for every plugin.

Selection occurs before compilation. Disabling a production plugin requires rebuild/redeploy, as explicitly confirmed. This supports experimental code, incremental features, customization, cross-project copying/reuse, and third-party integrations without changing core plugins. Enabled plugins share the process; crate boundaries are not crash isolation.

General lifecycle listeners and priorities remain the primary broad extension mechanism. Explicit service/handler replacement or decorator APIs are design candidates, not already approved mandatory interfaces. Disabling a plugin does not automatically undo stored data or database migrations.

## Existing parser and ecosystem

Assess `stackpress/idea` branch `feature/rust-parser-2` before writing another parser. Its `idea_parser_core` is separate from N-API bindings. A Rust consumer does not inherently need the OS-specific npm packages that stopped the earlier Node distribution effort. Current branch source separation was inspected; no parser compilation/parity test has been performed here.

Idea, Inquire, and Stackpress are monorepos; Ingest/Reactus also have workspace examples. Preserve semantic ownership while reorganizing as appropriate for Rust. Frui/r22n component/phrase contracts inform UI parity, but their React code is not reusable Leptos component code.

## Current authorized work and unresolved design

The initial setup installed project-local skills, populated the KB, migrated the notes, and established style/workflows. The subsequent source-structure acceptance authorizes recording the agreed direction in the KB. Neither pass scaffolds a Cargo workspace, implements features, merges the parser branch, publishes packages, deploys, commits, or pushes.

Read [open design questions](../references/00039-open-design-questions.md) when preparing the first implementation task; the questions and proposed vertical proof are planning material, not accepted answers or a Frozen spec.

The [full discussion reference](../references/00004-discussion-working-summary.md) retains requirements and decision history; load it when exact background, superseded suggestions, or prior open questions matter. The [feature assessment](../references/00005-discussion-feature-feasibility.md) retains the six-repository inventory; load it when checking whether a capability was considered.
