# Accepted monorepo source structure

Authority: the project owner's 2026-10-23 proposal and explicit acceptance of the revised structure and boundaries: "okay all accepted. please update KB with this direction". The [preserved conversation extract](../resources/conversation/2026-10-23-source-structure-acceptance.txt) retains the proposal, response, and approval. This topic supersedes the earlier flat crate-tree and Ingest/Inquire naming proposals; historical upstream names still identify upstream sources.

The owner's 2026-10-28 [issue #16](https://github.com/stackpress/rust/issues/16) groups the existing server packages under `crates/server/`. This changes filesystem paths only; package names, facade exports, features, and runtime contracts remain unchanged.

## Root and workspace

Start directly at the repository root, currently `/Users/cblanquera/server/projects/stackpress/rust`. A displayed `rust/` label means that root; do not add another `rust/` directory. This decision does not request moving or renaming the checkout.

The root `Cargo.toml` contains both `[workspace]` and the `[package]` named `stackpress`. Root `src/lib.rs` is the public facade, and one root `Cargo.lock` is shared by the workspace. Each actual member crate has its own `Cargo.toml` and ordinarily `src/lib.rs`. The grouping directories `crates/framework`, `crates/idea`, `crates/server`, and `crates/sql` have no package manifest; list the nested member paths in the root workspace.

```text
<project-root>/
├── Cargo.toml                    # Workspace + stackpress facade package
├── Cargo.lock
├── src/
│   └── lib.rs                    # Public stackpress:: re-exports
├── crates/
│   ├── framework/                # Former Stackpress framework family
│   │   ├── framework/
│   │   ├── framework-admin/
│   │   ├── framework-api/
│   │   ├── framework-csrf/
│   │   ├── framework-email/
│   │   ├── framework-language/
│   │   ├── framework-schema/
│   │   ├── framework-session/
│   │   ├── framework-sql/
│   │   ├── framework-ui/
│   │   └── framework-view/
│   ├── lib/                      # Foundation
│   ├── idea/
│   │   ├── idea/                 # Standalone convenience wrapper
│   │   ├── idea-parser/
│   │   └── idea-transformer/
│   ├── server/                   # Server family; no package here
│   │   ├── server/               # Former Ingest capability host
│   │   └── http/                 # Optional owned HTTP/1 IO host
│   └── sql/                      # Former Inquire family
│       ├── sql/
│       ├── sql-mysql/
│       ├── sql-postgres/
│       └── sql-sqlite/
├── tools/                        # CLI and build orchestration
├── examples/                     # Applications proving the framework
└── .agents/                      # Knowledge and workflows
```

This is accepted organization, not implementation evidence or a requirement to create empty crates. Introduce the backend adapters as they are implemented. Detailed tools/example application layouts remain implementation work.

## Package and namespace catalog

Folder names, Cargo package names, and Rust paths are independent. Use a `stackpress-` package prefix, retain the shorter directory names, and explicitly re-export selected crates from the root facade. Rust paths use underscores; hyphens are allowed in Cargo package names and feature names. Directory nesting does not automatically create modules or public exports.

| Crate directory under `crates/` | Cargo package | Public facade path |
| --- | --- | --- |
| `framework/framework` | `stackpress-framework` | `stackpress::framework` |
| `framework/framework-admin` | `stackpress-framework-admin` | `stackpress::framework_admin` |
| `framework/framework-api` | `stackpress-framework-api` | `stackpress::framework_api` |
| `framework/framework-csrf` | `stackpress-framework-csrf` | `stackpress::framework_csrf` |
| `framework/framework-email` | `stackpress-framework-email` | `stackpress::framework_email` |
| `framework/framework-language` | `stackpress-framework-language` | `stackpress::framework_language` |
| `framework/framework-schema` | `stackpress-framework-schema` | `stackpress::framework_schema` |
| `framework/framework-session` | `stackpress-framework-session` | `stackpress::framework_session` |
| `framework/framework-sql` | `stackpress-framework-sql` | `stackpress::framework_sql` |
| `framework/framework-ui` | `stackpress-framework-ui` | `stackpress::framework_ui` |
| `framework/framework-view` | `stackpress-framework-view` | `stackpress::framework_view` |
| `lib` | `stackpress-lib` | `stackpress::lib` |
| `idea/idea` | `stackpress-idea` | `stackpress::idea` |
| `idea/idea-parser` | `stackpress-idea-parser` | `stackpress::idea_parser` |
| `idea/idea-transformer` | `stackpress-idea-transformer` | `stackpress::idea_transformer` |
| `server/server` | `stackpress-server` | `stackpress::server` |
| `server/http` | `stackpress-server-http` | `stackpress::server_http_host` |
| `sql/sql` | `stackpress-sql` | `stackpress::sql` |
| `sql/sql-mysql` | `stackpress-sql-mysql` | `stackpress::sql_mysql` |
| `sql/sql-postgres` | `stackpress-sql-postgres` | `stackpress::sql_postgres` |
| `sql/sql-sqlite` | `stackpress-sql-sqlite` | `stackpress::sql_sqlite` |

Illustrative future facade source, not existing compiling code:

```rust
// Expose the independently usable foundation through the shared namespace.
pub use stackpress_lib as lib;

// Expose the MySQL integration only when selected for this build.
#[cfg(feature = "sql-mysql")]
pub use stackpress_sql_mysql as sql_mysql;
```

The optional `sql-mysql` feature would enable the optional `stackpress-sql-mysql` dependency as well as its public export. Other optional capabilities follow explicit feature/dependency wiring. Adding a direct dependency alone does not inject it into the facade. Cargo features are additive: another dependency path can also enable a backend. Feature defaults and the complete feature catalog remain open.

Libraries remain directly usable without the facade. Public exports make code available; runtime plugin registration is a separate explicit action. No package names have been reserved or published; registry availability, versions, and publication policy remain open.

## Responsibility and dependency boundaries

- `lib`: the `stackpress-lib` foundation for transport-neutral primitives and event/request/response/error contracts. It must remain independent of database drivers, UI renderers, account features, CLI, and native hosts.
- `server/server`: renamed Ingest responsibility: transport-neutral capability hosting, configuration, registration, and lifecycle. HTTP, CLI, and permitted Tauri calls enter through adapters; the name does not narrow the host to HTTP. Preserve top-level handling versus nested dispatch as described in [runtime and portability](runtime-and-portability.md). The facade's `server-http` feature selects this crate's buffered HTTP conversion only.
- `server/http`: optional owned HTTP/1 IO host, selected through `server-http-host` or `server-http-host-tls` on the facade. TLS remains opt-in; the default facade and buffered HTTP do not acquire this network host.
- `sql/sql`: independently usable query builders, database contracts, dialects, and execution.
- `sql/sql-mysql`, `sql/sql-postgres`, `sql/sql-sqlite`: separate backend adapters isolate dependencies and backend tests. Backend-specific connection handling, parameter binding, SQL differences, schema inspection, and migration behavior must be owned somewhere. Separate crates are the accepted initial organization, although feature-gated modules inside `sql` are a viable alternative if later justified. Existing database libraries may supply protocol/driver implementations. The tree does not select a first backend or prove support for any backend.
- `framework/framework-sql`: integrates the independent SQL facilities with Stackpress schema metadata, generated stores, events, configuration, and lifecycle. Keep it distinct from the lower-level `sql` family.
- `framework/framework-ui`: initially combines Leptos SSR/hydration integration and reusable Leptos components. Separate components and rendering integration internally. Browser builds must not acquire database drivers or native host services through UI dependencies. Split the crate later only when actual dependency or reuse needs justify it; this replaces the earlier mandatory-looking view-leptos/UI split.
- `framework/framework-view`: retains the page/render contract responsibility, distinct from Leptos integration in `framework-ui`.
- `idea/idea`: a turnkey convenience wrapper over independently usable `idea-parser` and `idea-transformer`. Assess the existing Rust parser before extraction or rewrite; see [source map](source-map.md).
- `idea/idea-parser` and `idea/idea-transformer`: parser/AST and transformation responsibilities remain separate from framework-specific schema/SQL/UI interpretation. Build-time transforms must compile before the application they generate. Detailed generator packaging remains open; no additional build crate is mandated by this tree.
- Root `stackpress`: a narrow facade over selected libraries, separate from `framework/framework`. Libraries depend directly on underlying contracts, never back through the facade. Keep the complete graph acyclic; facade convenience must not force every optional capability into every target.

The existing function-based plugin contract, compile-time selection, shared Leptos/Tauri source, and platform/execution adapters remain unchanged. Generation and runtime composition are distinct stages. This accepted structure is not a Frozen behavioral spec and does not itself authorize scaffolding, parser extraction, publishing, deployment, commits, or pushes.

## Technical references retained with acceptance

- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html): root package, nested members, and shared lockfile.
- [Rust public re-exports](https://doc.rust-lang.org/reference/items/use-declarations.html#use-visibility): facade exports.
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html): optional dependencies and additive selection.
