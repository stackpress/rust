# Next-session handoff: Stackpress Rust

Owner: [context index](../context/index.md). Load once at the start of the next session in `/Users/cblanquera/server/projects/stackpress/rust`.

## Start here

Read root [AGENTS.md](../../AGENTS.md), then [the context index](../context/index.md). Use the smallest topic/reference for the task rather than loading the whole KB.

## What exists

- The user's fresh Git repository, existing ignore rules and LICENSE are preserved.
- Complete project-local chrisai-coding and chrisai-knowledge packages at `.agents/skills`, pinned to ChrisAI main `c3be0b52e797f48ba8400893bb4aae693eaeb221`.
- Managed `.agents` rules, glossary, maintenance/spec workflows, references, and validator installed through dry-run/apply.
- Current Rust direction, runtime/portability, style, quality, source-map, provenance, and upstream-reference routers in context.
- [Accepted source structure](../context/source-structure.md) with the later approved root workspace/facade, grouped crate tree, package/path catalog, and dependency boundaries; load before scaffolding or naming Rust crates. The approval is preserved separately from the original setup receipt.
- Full migrated discussion and full source KB documents with provenance; detailed framework/foundation/package inventories.
- A Rust implementation/refactoring workflow, Rustdoc/comment examples, round checklist, and preserved setup request.

## Confirmed decisions

- One Rust monorepo; no mandatory 1:1 mapping to old repositories/npm packages.
- Leptos + Tauri, shared application/UI/operation source across web/mobile/desktop except explicit hardware/host adapters.
- `stackpress-lib` foundation; convenient `stackpress::` public namespace, e.g. `stackpress::sql_mysql` and `stackpress::framework_admin`.
- Work directly at the repository root: root Cargo workspace/package `stackpress` and `src/lib.rs` facade; grouped `crates/framework`, `crates/idea`, `crates/sql`, plus `crates/lib` and `crates/server`. No extra `rust/` directory or checkout move. Ingest becomes `server`; Inquire becomes `sql`. Backend crates are optional and introduced as implemented; `framework-ui` initially combines Leptos integration and components. The accepted structure records independent Idea use, SQL versus framework-SQL ownership, and acyclic dependencies.
- Priority events, false abort, manually constructible mutable request/response, CLI reuse, general request/response/error lifecycle, config and function-based plugins, plugin-contributed Idea generation.
- Compile-time plugin selection, with rebuild/redeploy to disable a plugin.
- Human-readable idiomatic Rust, deliberate patterns/boundaries, Rustdoc, and a lot of accurate explanatory comments per logical block or non-obvious line.
- Natural implementation → meaningful tests → at least three test-verified rounds for logic, responsibilities/abstraction, and consistency/docs/comments; continue targeted rounds while material findings remain.
- 70% development coverage, 90% before publishing. Do not copy old TypeScript >90%-on-every-change policy into Rust.

## Important existing work

Idea already has a Rust parser core on branch `feature/rust-parser-2`, pinned at `6c289d1f3fba17c758baafb2d4dcefd87fd448ea`. It is separate from Node/N-API binding and OS-specific npm distribution. Assess reuse/parity before a rewrite. No branch merge/extraction/test was done here.

## Not implemented or decided

No Cargo manifest/crates/framework runtime/prototype was created by setup or the subsequent KB update. No Rust unit tests, build, coverage, benchmark, SSR/hydration, mobile/desktop, or serverless proof ran. The initial source/package/namespace catalog is accepted; feature defaults/catalog, item-level APIs, detailed dependency wiring, MSRV/edition, first backend/database, handler signature, configuration format, offline/local behavior, lifecycle edge cases, and first implementation scope remain open.

The crate tree in old notes is historical and superseded by [accepted source structure](../context/source-structure.md). The vertical proof remains a proposal. Read [open questions](00039-open-design-questions.md) for outstanding design and proposed proof scope. No behavioral spec is Frozen, and no scaffolding/publication/deployment was authorized by the KB update.

## Continue effectively

The next user request determines implementation scope. If asked to start, propose/implement the smallest coherent slice using the accepted architecture and [programming workflow](../workflows/rust-programming.md). Resolve only genuinely blocking design choices; progress independent work without repeatedly asking to reapprove accepted direction.

Use existing upstream behavior as an informed contract, not literal JavaScript syntax. Trace semantic ownership through generator/runtime/adapter layers. Recheck source hashes/versions before exact implementation claims.

The original discussion notes have been migrated into this project. Use the KB/resource/reference paths, not the former rust-notes folder, as the durable handoff. The [setup receipt](../resources/provenance/setup-receipt.json) and [provenance checks](00038-ingestion-provenance.md) record whether migration and validation completed.
