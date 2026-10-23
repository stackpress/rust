<!-- kb-import:start -->
> Scope: preserved discussion history. Confirmed current Rust decisions live in [project direction](../context/project-direction.md). Proposed APIs, old paths, and historical status in this source are not implementation instructions.
> Later acceptance: load [accepted source structure](../context/source-structure.md) for the current root facade, grouped crate tree, renamed server/SQL families, and initial combined framework UI. The body below remains the unchanged historical proposal.
> Source: `/Users/cblanquera/server/projects/stackpress/rust-notes/stackpress-rust-feasibility/crate-and-plugin-architecture.md`; SHA-256 `daabcbff8b1cfb62e06ca08779e180339331a5ce83312683eaef6055be7e9d0b`; captured 2026-10-23. [Exact original](../resources/conversation/2026-10-23-stackpress-rust/crate-and-plugin-architecture.md). Prose links are rebased; source content otherwise retained.
<!-- kb-import:end -->

# Proposed crate and plugin architecture

Discussion snapshot: 2026-10-23. Leptos + Tauri, shared cross-platform source, compile-time plugin selection, the name `stackpress-lib`, and minimal unopinionated plugin entrypoints are author-confirmed. The detailed crate topology and APIs below remain proposals.

## Organizing principle

Use one Cargo workspace with focused libraries, an aggregate facade, and application target packages. Libraries remain independently reusable rather than eventually merging their implementation into a large final crate.

The final application composes selected crates and plugins. Server executables, browser assets/WASM, desktop bundles, and mobile applications are separate output artifacts, even if they share much of their source.

Keep the architecture event-driven, configuration-driven, and plugin-based internally as well as externally. Ordinary named functions and explicit registrations should remain easy to find.

## Eventual layout

Names below are illustrative; no Rust crates have been created.

```text
stackpress-rs/
├── Cargo.toml
├── crates/
│   ├── stackpress-lib/
│   ├── stackpress-ingest/
│   ├── stackpress-idea/
│   ├── stackpress-idea-build/
│   ├── stackpress-inquire/
│   ├── stackpress-schema/
│   ├── stackpress-sql/
│   ├── stackpress-view/
│   ├── stackpress-view-leptos/
│   ├── stackpress-session/
│   ├── stackpress-api/
│   ├── stackpress-admin/
│   ├── stackpress-language/
│   ├── stackpress-email/
│   ├── stackpress-csrf/
│   └── stackpress/
├── adapters/
│   ├── stackpress-http/
│   └── stackpress-lambda/
├── tools/
│   └── stackpress-cli/
├── examples/
│   └── blog/
│       ├── contracts/
│       ├── generated/
│       ├── plugins/
│       ├── ui/
│       └── targets/
│           ├── server/
│           └── tauri/
└── docs/
```

A Tauri target can own its desktop/mobile configuration and platform-generated projects. Separate Rust target crates are useful only when their dependencies or source ownership justify the separation. The tree does not require distinct duplicate mobile and desktop implementations.

The earlier proposal named `stackpress-view-dioxus`; the author subsequently selected Leptos + Tauri. Use `stackpress-view-leptos` in the current proposal. The author renamed the proposed foundation from `stackpress-core` to `stackpress-lib`.

The current Idea, Inquire, and Stackpress repositories are monorepos. One repository can contain several packages/crates, and a Rust package can expose a library plus binaries. This proposed workspace is a coordination choice, not a rule that each existing repository must become one crate or that every current npm package must have a Rust counterpart. Choose boundaries by responsibility, dependency direction, reuse, and target compatibility. Shared repository layout versus separately published foundational repositories remains open.

## Responsibilities

| Crate | Responsibility |
| --- | --- |
| stackpress-lib | Transport-neutral request/response, errors/status, data, priority queue, fundamental emitter contracts |
| ingest | Host, capability bus, event routes, plugins/services, config, bootstrap, request lifecycle |
| idea | Language grammar/parser/AST/compiler/composition/diagnostics |
| idea-build | Generator API, ordered transforms, build integration, generated-file management |
| inquire | Query/schema builders, dialect and connection contracts, diff, execution hooks |
| schema | Stackpress metadata semantics and normalized model/column helpers |
| sql | Generated stores/actions/events and database lifecycle |
| view | Page selection, view snapshot, render contract |
| view-leptos | Selected Leptos SSR/hydration/component integration |
| session/api/admin/etc. | Their own runtime behavior, configuration, and generation contributions |
| HTTP/Lambda adapters | Host-specific normalization, request bodies, response/stream conversion |
| stackpress | Narrow exports and deliberately selected framework composition |
| CLI | Project setup, generation/build orchestration, terminal invocation |

Keep underlying libraries independently usable. Do not force an Inquire user to depend on authentication or a Leptos user to depend on the database layer.

## Public namespace and Cargo names

The author wants application-facing imports under `stackpress::`, illustrated as `stackpress::inquire-mysql`. The valid Rust path is `stackpress::inquire_mysql`. Cargo package names may retain hyphens, such as `stackpress-inquire-mysql`, while source crate/module identifiers use underscores.

The facade explicitly re-exports separately implemented crates (`pub use stackpress_inquire_mysql as inquire_mysql;`), conditional on selected features for optional adapters. Installing a second dependency alone does not automatically add a module to the stackpress facade. Standalone crate usage remains possible. This reconciles shared public imports with separate implementation/publishing boundaries.

See [Cargo naming and namespace](00009-discussion-cargo-namespace.md) for illustrative manifests, re-exports, and unresolved publishing details.

## Dependency rules

1. `stackpress-lib` has no required dependency on a UI renderer, account system, database driver, CLI, or native host.
2. Runtime plugins depend on framework contracts; `stackpress-lib` does not import every plugin.
3. Renderer adapters depend on view contracts. Renderer-specific types stay in adapters or UI packages where practical.
4. Target hosts depend on selected features and adapters.
5. Shared browser/mobile contracts expose serializable public data, not native resources, DB connections, or server service containers.
6. Generated source is partitioned by consumer: contracts, server runtime, UI, and optional external tools.
7. Generation dependencies compile before the generated application and cannot depend cyclically on it.
8. Built-in plugin composition remains explicit; the facade should not become a hidden service locator that implicitly enables everything.

Suggested runtime direction:

```text
Application target
  → selected runtime plugins and transport/view adapters
    → host/schema/query/view contracts
      → stackpress-lib
```

## Compile-time plugin selection

Confirmed author model: select plugins before compiling. There is no present requirement for native dynamic libraries, WASM plugin execution, runtime package installation, or hot unload.

Illustrative application manifest:

```toml
[features]
default = []
payments = ["dep:payments-plugin"]

[dependencies]
payments-plugin = { path = "../plugins/payments", optional = true }
```

Illustrative registration:

```rust
#[cfg(feature = "payments")]
app.plugin(payments_plugin::plugin)?;
```

These are proposed APIs, not existing compiling code. A disabled out-of-line module and its optional dependency can be excluded from the selected build. Merely skipping runtime registration would not exclude its compilation. Dependencies can still be enabled by another selected path because Cargo features are additive.

Build tooling should keep runtime plugin selection, generator plugin selection, generated registries, and migrations coherent. Build scripts may generate source, but cannot retroactively add undeclared Cargo dependencies to an ongoing build. A CLI/configuration step may prepare manifests and registries before invoking Cargo; alternatively, declare optional dependencies up front and select their features.

Disabling a faulty production plugin requires rebuilding and deploying. It does not rewind its persisted state. Define required/optional dependencies and fail clearly if a selected plugin requires one that is absent.

## Plugin use cases and extension policy

| Author scenario | Intended approach |
| --- | --- |
| Experimental work | Source present, module/dependencies excluded from production selection |
| Production problem | Exclude plugin, rebuild, deploy; inspect affected dependents |
| Additional capabilities | Register new events/services/routes/transforms |
| Slightly change existing behavior | Documented before/after hooks, explicit decorators/replacements, cooperative cancellation |
| Extend MVP | Add plugin through the same registration contract |
| Cross-project reuse | Copy a self-contained local plugin or depend on its crate |
| Third-party integration | Keep external credentials/transport/payload conversion within integration-owned code |

Priority listeners support preprocessing and postprocessing. Replacing a handler or service should use an explicit replacement mechanism rather than depend on accidental duplicate registration order. Give listeners stable identities/handles if removal/replacement is a required extension contract.

Use interfaces/traits where substitution is real. A plugin boundary does not require a trait for every struct or helper.

## Plugin source organization

The original convention was intentionally unopinionated: a selected file/module default-exported a registration function; directories were suggestions. Rust should preserve this minimal entry contract, not require a plugin class, a trait implementation, or all the folders below.

An independently reusable plugin crate can keep the author's suggested top-level layout:

```text
plugins/payments/
├── Cargo.toml
├── plugin.rs
├── events/
├── pages/
├── views/
└── tests/
```

Cargo's conventional `src/lib.rs` is optional. A crate can declare:

```toml
[lib]
path = "plugin.rs"
```

The Rust analogue of a default-exported function can be a public function with a consistent name:

```rust
// Proposed API, not implemented framework code.
pub fn plugin(app: &mut AppBuilder) -> Result<()> {
    // Register any services, lifecycle listeners, events, routes, or generators.
    Ok(())
}
```

The application selects `payments::plugin`, `search::plugin`, or another typed function path. An app-local plugin can be a module with only `plugin.rs`; it does not need its own Cargo manifest. A standalone published crate needs a Cargo manifest as packaging metadata, not as an extra framework plugin contract. No directories are required if a registration function can express the whole plugin.

`events/`, `pages/`, `views/`, `tests/`, and optional service/integration/config modules remain recommended organization when useful. Rust module files must explicitly include their children; a folder is not automatically discovered code. A source generator may produce those module declarations if adopted deliberately.

Registration declares contributions. Event handlers translate request/response conventions. Services and integration modules are optional organization, not required framework layers in every plugin.

Small local plugins can be modules rather than crates. Use separate crates when optional dependencies, reuse, independent tests, or stable imports justify them.

A plugin that contributes generation may have a companion `payments-codegen` crate. A reusable family can separate shared contracts, server behavior, UI, and codegen where necessary; do not impose four packages on every small plugin.

The old package.json entries allowed folder/module resolution (`plugins/payments`, `plugins/payments/plugin`, a package such as `stackpress`). Rust uses compile-time module/crate resolution and explicit function registration. If path-list configuration is desirable, a pre-build tool can generate typed registrations from already declared dependencies/modules. An arbitrary string in runtime config cannot instantiate previously uncompiled Rust code, and no such dynamic-loading requirement exists here.

## General lifecycle extension events

The author clarified that Ingest already provides the broad extension events required:

```text
normalized top-level invocation
  → request
  → selected route/custom event listeners
  → response
  → output adapter

application failure in a stage
  → structured error on response
  → error event
  → handled outcome or terminal failure policy
```

These are existing contracts to preserve, not proposed new substitutes. Event priorities and cooperative abort apply to the selected pipeline. Domain-specific before/after events may be provided by a plugin, but they are not mandatory for every operation.

Source distinction: current Route performs prepare/process/shutdown for handled transport requests; direct `emit` is the lower-level event dispatcher. Do not automatically wrap request/response around every nested emit/resolve, which could recurse or duplicate guards/rendering. Define a top-level lifecycle boundary and direct/nested dispatch contract for CLI, HTTP, and permitted local Tauri calls.

Decide explicitly whether an abort skips the response stage or invokes a separate finalization path, and what happens when the response is already sent. Do not claim response always runs like a finally block; existing Route can exit early. Error-handler failure must preserve the original failure and avoid recursively emitting error forever.

Capture plugin/event/handler metadata around each invocation for diagnostic reporting. See [diagnostics](00008-discussion-diagnostics-existing-parser.md).

## Generation and lifecycle

Preserve the conceptual stages:

1. Read Idea and application build/plugin selection.
2. Compose schema imports and normalize Stackpress metadata.
3. Enabled generator plugins contribute ordered transforms.
4. Emit source and registries for their consumers.
5. Compile the selected runtime, shared UI/contracts, and platform targets.
6. Runtime bootstrap registers selected plugins and resolves config/listen/route conventions.

Generation is not runtime initialization. A build-time contribution must not require a live database or a fully compiled final application unless an explicit tool workflow chooses that behavior.

Standalone CLI generation gives inspectable artifacts. Optional Cargo `build.rs` integration should use `OUT_DIR` and declare relevant inputs. Keep deterministic ordering and owned-output cleanup explicit. Clean/repeat generation should not duplicate registrations or leave removed model modules active.

## UI and capability invocation

Proposed boundary:

```text
Leptos UI
  → application operation interface
    → HTTP transport for remote server calls
    → Tauri command/IPC adapter for permitted local calls
      → Stackpress event dispatcher
        → selected plugin operation
```

This is a design proposal. It does not imply every app supports local execution or offline data.

The author explicitly requires source reuse across web/mobile/desktop, except hardware-specific code. Use shared Leptos components, public contracts, applicable validation, and domain operations. Put execution-host and platform differences in adapters. The same operation implementation can run on the remote server or in a native host when its dependencies/policy allow; a web client invokes the remote implementation rather than compiling server-only database/secret code into browser WASM. Shared source does not mean every operation executes locally on every target.

The website can use Leptos SSR plus hydration. The Tauri app can use shared Leptos components compiled to browser/WASM assets. Server-only code remains outside the packaged frontend. Local Rust capabilities can run in Tauri's Rust host when explicitly selected.

Tauri command authorization and remote API authorization are distinct edges. Avoid exporting a generic unrestricted "emit any event" command to untrusted clients. Map approved commands to capabilities and apply caller policy.

Leptos server functions, if used, should delegate to the shared operation/event layer rather than quietly become a second domain implementation.

## Readability contract

Confirmed author priority: methodically chosen design patterns, consistent code patterns, separation of responsibility, abstraction, and a lot of comments for human programmers.

Recommended conventions:

- Named handlers and explicit registration.
- Consistent module names, error flow, and configuration ownership.
- Concrete public types where they clarify behavior.
- Traits at actual adapter/service substitution boundaries.
- Useful comments on intent, contracts, assumptions, order, ownership, and non-obvious mechanics.
- Small orchestration functions that expose the sequence of steps.
- Inspectable generated source and intermediate forms.
- Minimal hidden control flow in macros.
- Async type erasure, lifetimes, and registry machinery contained inside framework internals where possible.

The goal is not recreating JS syntax. It is preserving understandable control flow and responsibility boundaries in idiomatic Rust.

## Proposed first proof

Do not create all eventual crates before proving the boundaries. Begin with `stackpress-lib`, host, existing Idea parser/build, one minimal generated operation, and a small application facade/CLI. Introduce query and view implementations as the proof needs them. Assess the existing parser branch before starting a parser rewrite.

Proof checklist:

1. One Idea model produces compilable public types and an operation registration.
2. Multiple exact/pattern listeners run in priority order with defined ties.
3. Returning false stops the remaining pipeline.
4. Nested dispatch works without global locks or lost invocation metadata.
5. CLI and HTTP can invoke the same profile-search operation.
6. Manually constructed requests/responses can be mutated before output.
7. Result errors reach the error event; handled/unhandled outcomes are explicit.
8. A disabled experimental plugin is excluded from the selected build.
9. A customization plugin changes behavior without editing the original.
10. Repeat generation is stable and generated code compiles.
11. A Leptos page renders initial server HTML and hydrates from the same safe data.
12. The same component source builds into a Tauri client on a chosen first platform.
13. Additional platforms/database/serverless adapters receive their own proofs before support claims.
14. request/response/error lifecycle order and abort behavior are explicit and do not repeat for every nested event call.
15. A failure report identifies plugin, handler, source registration, nested invocation path, and original cause even for a closure and after an await.

## References

- [Cargo workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) — background reference; verify details when implementing.
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- [Cargo library target path](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#library)
- [Current Stackpress aggregate composition](../../../stackpress/packages/stackpress/src/plugin.ts)
- [Current schema plugin contribution](../../../stackpress/packages/stackpress-schema/src/plugin.ts)
- [Current architectural principles](00011-upstream-identity-and-principles.md)
