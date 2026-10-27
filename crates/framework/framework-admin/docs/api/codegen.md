# Code generation

This module requires `codegen` and belongs only in a build tool. `components() -> Vec<Component>` returns implemented canonical definitions with typed public props, revisions, and private Rust imports. `Component` pairs a `ComponentDefinition` with its `ComponentImport`.

`register_components(registry, replacements) -> SchemaResult<DefinitionRegistry>` consumes and returns the app's catalog. It selects app replacements before defaults, rejects duplicate replacements, and preserves the schema registry's checked prop/import behavior. Canonical names and aliases remain schema-owned.

`emit_roles(ui, definitions, diagnostics, output) -> TransformResult<()>` consumes revision-2 public descriptors through a normal function transform owning `ui`. It preflights every model before contributing a file. It refuses duplicate models/fields/roles, missing or unavailable imports, Rust keywords/default-export conventions, incompatible native controls, unsupported filters, and descriptor/output budgets. Normalization and emission retain the concrete `SchemaError` beneath registry error context.

Every generated model module exports `descriptors()`. A selected field/filter/span function accepts `&Draft` and an instance `&str`; a list/view function accepts `&contracts::public::Public<Model>` and an instance. All return `SchemaResult<AnyView>`. List/view output labels each selected value so adjacent fields remain readable. Omitted roles export no corresponding function. Embedded fieldsets use the same mechanism.

A model with an explicit list selection also exports `table(rows: &[(String, Public<Model>)], instance: &str) -> SchemaResult<AnyView>`. The caller supplies at most 100 unique, nonempty row keys of at most 128 bytes. The function checks all identities before rendering, then uses native framework-ui table slots with the selected list adapters as cells. Headers use the selected labels. Row keys are escaped in attributes and encoded injectively for control IDs. This finite snapshot preserves caller order; it adds no fetching, sort, paging or selection behavior.

Rust import mappings are trusted compiled application choices, checked lexically before emission. A custom symbol's existence and `RoleContext` interface are proved by compiling the generated consumer. Public labels/props never become imports, identifiers, templates, or expressions. Idea diagnostics retain actual UTF-8 ranges; normalized JSON retains file/structural paths without invented coordinates.
