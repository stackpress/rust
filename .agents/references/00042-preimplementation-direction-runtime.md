# Historical context: preimplementation direction runtime

Owner: [source catalog](../context/source-catalog.md). Load only to audit the pre-sync setup text, provenance, superseded plans or resolved questions.

This is a lossless demotion of the context at branch base `34be3146ffc3279595595b665bf371205744a3a2`, preserved by Task 00059. Statements about missing implementation, open decisions, versions and support describe that older text and have no current authority. Use the [current context index](../context/index.md) for accepted truth. Only prose links are rebased; fenced examples are unchanged.

Original owner: `.agents/context/project-direction.md`.

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
- Read [accepted source structure](../context/source-structure.md) for the complete tree, package/path catalog, standalone Idea wrapper, SQL/framework-SQL boundary, combined initial Leptos integration/components crate, and acyclic dependency rules.
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

Original owner: `.agents/context/runtime-and-portability.md`.

# Runtime and portability contracts

Authority: author requirements, source-observed Stackpress behavior, and the explicitly marked portability boundaries. No Rust runtime is implemented yet.

## Ordered capability execution

Keep event matching/queue execution separate from transport routing. Stackpress's all-matching prioritized listeners must not silently become a conventional single-winner router. Preserve exact/pattern matches, extracted parameters, priorities, asynchronous sequential mutation, and cooperative false cancellation.

Define equal-priority order explicitly. The source queue's stated FIFO intent and comparator/testing coverage are not sufficient proof of a complete tie contract. Keep invocation metadata local, snapshot or otherwise safely access registrations, and do not hold a global registry lock while awaiting a handler.

An event dispatch outcome (completed, aborted, no listener) differs from response/application status. Native HTTP status, a JSON result envelope, and a terminal exit code are related mappings rather than interchangeable values.

## Request/response and lifecycle

Framework requests/responses are independently constructible with optional native resources. Preserve shared data, headers, request loading, cookies/session revisions, results/rows/totals, errors, redirects, MIME/body forms, and explicit dispatch state where useful.

The general handled-invocation sequence is `request → selected operation → response → output`. Errors from a stage are offered to `error`. Existing Ingest Route can exit early; response is not inherently a finally block. Direct `emit` is lower-level dispatch and must not recursively wrap every nested operation in general lifecycle hooks.

Define top-level CLI/HTTP/Tauri entry behavior, abort/finalization, sent responses, nested context propagation, and failed error-handler fallback before implementation. Keep these unresolved details visible instead of inferring a silent lifecycle redesign.

## Diagnostic intent

Make failure reports useful for named and anonymous handlers. Retain original errors/causes, validation/source-position details, plugin/event/listener/stage/registration metadata, and nested invocation paths. Capture optional native backtraces where supported and meaningful; a thread backtrace is not a complete asynchronous or distributed call history.

Use explicit framework metadata and async-aware instrumentation as appropriate. Result-based errors and `std::error::Error` are idiomatic; preserving an `Exception` name is optional. Panic/abort recovery is a separate problem. Do not turn raw request/session contents into automatic logs.

Read [existing Exception and Rust diagnostic options](../references/00008-discussion-diagnostics-existing-parser.md) when designing reports, and [the source assessment](../context/source-map.md) when locating current error/lifecycle code.

## Generation before compilation

Idea owns syntax, AST, composition, and transform inputs; Stackpress schema/SQL/view/admin/AI packages own interpreted metadata and their generated consumers. Preserve this distinction in the Rust design.

Build-time generator plugins must compile independently of the generated final application. Keep dependencies acyclic and output ownership explicit. Rust generation may emit Rust, SQL, UI, metadata, or docs. The selected Leptos UI changes component output, not necessarily Idea syntax.

Prefer inspectable generated source, deterministic transform order, declared inputs, clean/repeat generation, and intentional stale-file cleanup. A Cargo build script may write generated Rust into OUT_DIR; a pre-build CLI may prepare source/manifests. A running build script cannot retroactively add undeclared dependencies to Cargo resolution.

Revisions are schema history, not proof of applied database state. Keep query/dialect behavior, migration planning/execution, destructive policy, transactions, and population distinct.

## Shared targets

Use shared component and operation code with explicit input/output contracts. Web SSR obtains safe public data, renders HTML, and serializes data for matching hydration. Tauri loads packaged client assets, then uses remote API or permitted local IPC adapters. Choose target behavior through adapters rather than duplicated feature implementations.

Host-specific filesystem, native resources, device permissions, notifications, signing/packaging, and network/process APIs require target-specific integration. Do not assume all optional crates compile for browser WASM. Do not expose unrestricted event execution merely because Tauri or an HTTP endpoint can call the host.

Authentication, authorization, input validation, and output disclosure belong at each access surface and/or the capability's deliberate policy boundary. Internal event dispatch does not authorize a caller automatically; client-side permission display is not enforcement.

## Compatibility limits to decide

JavaScript callable-object syntax, dynamic imports, reflection, arbitrary ECMAScript regex, React components, Node/Electron APIs, and PGlite hosting require redesign or explicit bridging. SQL dialect support does not prove PGlite embedded runtime support. Backend crate/platform documentation is not tested Stackpress integration.

Leptos/Tauri and the [initial source/crate structure](../context/source-structure.md) are selected. The latter records the transport-neutral `server`, independent `sql` family versus `framework-sql`, initial combined `framework-ui`, and build/runtime dependency boundaries. Transport implementation, async handler signature, first database/backend, first serverless adapter, offline/local execution, detailed feature/dependency wiring, and behavioral contracts remain unresolved; accepting a tree is not a Frozen implementation spec.

