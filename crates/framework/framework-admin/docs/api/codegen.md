# Code generation

This module requires `codegen` and belongs only in a build tool. `components() -> Vec<Component>` returns implemented canonical definitions with typed public props, revisions, and private Rust imports. `Component` pairs a `ComponentDefinition` with its `ComponentImport`.

`register_components(registry, replacements) -> SchemaResult<DefinitionRegistry>` consumes and returns the app's catalog. It selects app replacements before defaults, rejects duplicate replacements, and preserves the schema registry's checked prop/import behavior. Canonical names and aliases remain schema-owned.

`emit_roles(ui, definitions, diagnostics, output) -> TransformResult<()>` consumes revision-3 public descriptors through a normal function transform owning `ui`. It preflights every model before contributing a file. It refuses duplicate models/fields/roles, missing or unavailable imports, Rust keywords/default-export conventions, incompatible native controls, unsupported filters, and descriptor/output budgets. Normalization and emission retain the concrete `SchemaError` beneath registry error context.

Every generated model module exports `descriptors()`. A selected field/filter/span function accepts `&Draft` and an instance `&str`; a list/view function accepts `&contracts::public::Public<Model>` and an instance. All return `SchemaResult<AnyView>`. List/view output labels each selected value so adjacent fields remain readable. Omitted roles export no corresponding function. Embedded fieldsets use the same mechanism.

A model with an explicit list selection also exports `table(rows: &[(String, Public<Model>)], instance: &str) -> SchemaResult<AnyView>`. The caller supplies at most 100 unique, nonempty row keys of at most 128 bytes. The function checks all identities before rendering, then uses native stackpress-leptos table slots with the selected list adapters as cells. Headers use the selected labels. Row keys are escaped in attributes and encoded injectively for control IDs. This finite snapshot preserves caller order; it adds no fetching, sort, paging or selection behavior.

Rust import mappings are trusted compiled application choices, checked lexically before emission. A custom symbol's existence and `RoleContext` interface are proved by compiling the generated consumer. Public labels/props never become imports, identifiers, templates, or expressions. Idea diagnostics retain actual UTF-8 ranges; normalized JSON retains file/structural paths without invented coordinates.


`emit_pages(schema, definitions, selected_models, output)` follows `emit_roles` in the same ordinary function transform. It projects the frozen UI contract, rejects duplicate selection and missing public identities, and contributes deterministic `ui/pages.rs` to the same UI partition. Removed models contribute no adapters or routes. Pure per-model and dispatch adapters decode actual generated public create/patch/query DTOs, compose role forms and filters, and render typed public list/detail snapshots.

`ui/pages_host.rs` is a separate explicit native include, omitted from the pure UI aggregate. It registers selected actual generated operations and fixed page/API routes under the application's validated mount. It imports native server/data crates only in the native host. Applications must provide policy, transport, anti-forgery admission, error handling and SQLite lifecycle; generation does not grant authority.
