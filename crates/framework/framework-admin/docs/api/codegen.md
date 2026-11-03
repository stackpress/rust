# Code generation

This module requires `codegen` and belongs only in a build tool. `components() -> Vec<Component>` returns implemented canonical definitions with typed public props, revisions, and private Rust imports. `Component` pairs a `ComponentDefinition` with its `ComponentImport`.

`register_components(registry, replacements) -> SchemaResult<DefinitionRegistry>` consumes and returns the app's catalog. It selects app replacements before defaults, rejects duplicate replacements, and preserves the schema registry's checked prop/import behavior. Canonical names and aliases remain schema-owned.

`emit_roles(ui, definitions, diagnostics, output) -> TransformResult<()>` consumes revision-3 public descriptors through a normal function transform owning `ui`. It preflights every model before contributing a file. It refuses duplicate models/fields/roles, missing or unavailable imports, Rust keywords/default-export conventions, incompatible native controls, unsupported filters, and descriptor/output budgets. Normalization and emission retain the concrete `SchemaError` beneath registry error context.

Every generated model module exports `descriptors()`. A selected field/filter/span function accepts `&Draft` and an instance `&str`; a list/view function accepts `&contracts::public::Public<Model>` and an instance. All return `SchemaResult<AnyView>`. List/view output labels each selected value so adjacent fields remain readable. Omitted roles export no corresponding function. Embedded fieldsets use the same mechanism.

A model with an explicit list selection also exports `table(rows: &[(String, Public<Model>)], instance: &str) -> SchemaResult<AnyView>`. The caller supplies at most 100 unique, nonempty row keys of at most 128 bytes. The function checks all identities before rendering, then uses native stackpress-leptos table slots with the selected list adapters as cells. Headers use the selected labels. Row keys are escaped in attributes and encoded injectively for control IDs. Models with identities also receive a default sticky-right detail-action column through the [root row-action context](pages.md). Denied links leave empty cells. This finite snapshot preserves caller order; it adds no fetching, sort, paging or selection behavior.

Rust import mappings are trusted compiled application choices, checked lexically before emission. A custom symbol's existence and `RoleContext` interface are proved by compiling the generated consumer. Public labels/props never become imports, identifiers, templates, or expressions. Idea diagnostics retain actual UTF-8 ranges; normalized JSON retains file/structural paths without invented coordinates.

`emit_pages(schema, definitions, selected_models, output)` follows `emit_roles` in the same ordinary function transform. It projects the frozen UI contract, rejects duplicate selection and missing public identities, and contributes deterministic `ui/pages.rs` to the same UI partition. Removed models contribute no adapters or routes. Pure per-model and dispatch adapters decode actual generated public create/patch/query DTOs, compose role forms and filters, and render typed public list/detail snapshots.

`ui/pages_host.rs` is a separate explicit native include, omitted from the pure UI aggregate. It registers selected actual generated operations and fixed page/API routes under the application's validated mount. It imports native server/data crates only in the native host. Applications must provide policy, transport, anti-forgery admission, error handling and SQLite lifecycle; generation does not grant authority.

The [build-time registration reference](../../../framework-schema/docs/README.md) describes the separate schema/SQL/admin transform helpers, their partitions and dependency order.

## Integrate the emitters

Call the emitters while an ordinary registered transform owns the `ui` partition.
The application's normalization and diagnostic map remain inputs to generation:

```rust
use stackpress_framework_admin::codegen::{emit_pages, emit_roles};
use stackpress_framework_schema::{
    Schema,
    codegen::{DefinitionRegistry, DiagnosticMap, transform_error},
};
use stackpress_idea_transformer::{OutputPlan, Result};

pub fn emit_selected_admin(
    schema: &Schema,
    definitions: &DefinitionRegistry,
    diagnostics: &DiagnosticMap,
    output: &mut OutputPlan,
) -> Result<()> {
    let ui = schema.project_ui().map_err(transform_error)?;
    emit_roles(&ui, definitions, diagnostics, output)?;
    emit_pages(schema, definitions, &["Profile"], output)
}
```

This application function runs complete role/import preflight before emitting page
adapters for Profile. Register real schema and SQL transforms before selecting
`register_pages`; the application must verify actual generated SQL operations.
For role-only generation, use `register` and omit `emit_pages`.

| Input | Used by | Contract |
| --- | --- | --- |
| `ui: &UiSchema` | `emit_roles` | Revision-3 public projection; sensitive/private fields are excluded. |
| `schema: &Schema` | `emit_pages` | The normalized schema that produced the role projection. |
| `definitions: &DefinitionRegistry` | Both | The same checked component catalog and application policy used during normalization. |
| `diagnostics: &DiagnosticMap` | `emit_roles` | Actual composed Idea coordinates or normalized JSON structural paths. |
| `selected: &[&str]` | `emit_pages` | Ordered model names, at most 128 and without duplicates; absent models are omitted. |
| `output: &mut OutputPlan` | Both | The current transform's granted `ui` partition; the ordinary managed writer installs it. |

**Returns:** both emitters return `stackpress_idea_transformer::Result<()>` after
contributing source to the plan. They do not install a root, open a database or
start a host. Runtime registration still requires the application's explicit call.

## Templates, errors and limits

Rust structure lives in adjacent `templates/*.rs.tpl` assets using `[% %]`
delimiters. Each emitter lazily compiles a used asset once within that call.
Subsequent renders borrow fresh data and start independent lookup/work/output
state. There is no global cache or runtime interpretation of schema text.

An internal view model may occupy at most 8 MiB of compact encoded JSON, including
keys, punctuation and escaping. Each rendered asset remains bounded to 1 MiB;
the existing aggregate OutputPlan and formatted-artifact limits also remain 1 MiB.
Descriptors, model/field counts and normalizer constraints retain their own limits.

Template failures retain their authored static asset identity and concrete
`stackpress_lib::TemplateError` kind/span beneath `SchemaError`. A template limit
returns a direct emitter `TransformErrorKind::Limit`; Registry preserves its usual
outer Callback context. Other schema failures keep the existing schema error policy.
No rejected label, prop, template text or render data is included in these contexts.

Read [change generated admin source](../guides/change-generated-source.md) for the
maintainer workflow. [Generated native page-host registration](pages-host.md)
documents the separate native include and ordinary plugin function contract.
