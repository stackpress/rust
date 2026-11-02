# Build-time framework registration

The optional `codegen` feature on framework-schema, framework-sql and framework-admin exposes
ordinary transform registration functions. Put these dependencies in a separate generator
package. The final native app imports generated runtime files; the browser imports only public
contracts and UI. Neither needs the generator, parser, Idea Registry or these helpers.
Pure schema/language contracts and reusable Leptos remain support libraries.

## Registration reference

Every helper has the same signature:
`fn(&mut stackpress_idea_transformer::Registry, stackpress_idea_transformer::ContextTransform) -> stackpress_idea_transformer::Result<()>`.
`ContextTransform` is `fn(&TransformContext<'_>, &mut OutputPlan) -> Result<()>`.
The Result here belongs to `stackpress_idea_transformer`, independently of runtime startup.

| Function | Identity / output partitions | Required earlier transforms |
| --- | --- | --- |
| `stackpress_framework_schema::codegen::register` | `framework-schema` / `contracts`, `metadata` | None |
| `stackpress_framework_sql::codegen::register` | `framework-sql` / `stores` | `framework-schema` |
| `stackpress_framework_admin::codegen::register` | `framework-admin` / `ui`, role-only | `framework-schema` |
| `stackpress_framework_admin::codegen::register_pages` | `framework-admin` / `ui`, including native pages | `framework-schema`, `framework-sql` |

Choose one admin alternative. Selecting both rejects the duplicate identity/partition claim.
Registry planning rejects missing or wrong-order dependencies before invoking any transform.
These helpers only add real metadata and caller-supplied functions to the existing Registry;
they do not run transforms, publish output, select runtime plugins or access a database.

## Replace repeated registry wiring

An application that manually constructed these TransformSpecs can register the same callbacks
through the helpers. The callbacks still own normalization, frozen definitions, diagnostics,
model/operation selection and actual emission. This function accepts those real emitters:

```rust
use stackpress_idea_transformer::{ContextTransform, Registry, Result};

pub fn page_generator(
    schema: ContextTransform,
    stores: ContextTransform,
    pages: ContextTransform,
) -> Result<Registry> {
    let mut registry = Registry::new();
    stackpress_framework_schema::codegen::register(&mut registry, schema)?;
    stackpress_framework_sql::codegen::register(&mut registry, stores)?;
    stackpress_framework_admin::codegen::register_pages(&mut registry, pages)?;
    Ok(registry)
}
```

Continue using the single Registry's `plan_with` and `OutputWorkspace::check` or `install`.
The generator must verify that a page-enabled plan actually includes SQL-owned
`stores/operations.rs` before check/install; selecting SQL alone does not promise operation
output. The [generated admin tool](../../../../examples/generated-admin/tool/src/lib.rs)
demonstrates that check before publication and the explicit nonempty page-model selection.
The [workspace tool](../../../../examples/generated-workspace/tool/src/lib.rs) uses the schema
and SQL helpers without adding admin or an application dependency to its graph.

## Continue using direct emitters

Direct `Schema::normalize_with_diagnostics`, `emit_contracts`, SQL store/operation emitters,
admin `emit_roles` and `emit_pages` remain available. A caller can also retain explicit
`Registry::add(TransformSpec, callback)` when it needs custom identities/partitions; the same
ownership and dependency contracts apply. Helpers add no second writer or dispatcher.
Role-only generation needs no SQL transform. Removed models emit no obsolete page mappings;
no effective page models means no generated runtime Mount/registrar.

## Maintain schema source templates

Schema declarations, create/default preparation, public inputs/projections and typed queries use
compiled-in [structural Rust templates](../src/codegen/templates). The normalization and emission
layer still owns identifiers, types, policy decisions and Rust string-literal encoding. Templates
receive checked values and pre-rendered fragments; inserted source is never parsed as a template.
The `[%` / `%]` delimiters leave Rust braces and embedded `format!` syntax literal.

The generic engine is [stackpress-lib's template API](../../../lib/docs/api/template.md).
It adds no runtime template dependency to the default schema contract graph: the dependency is
selected by `codegen`. Generator context data has a separate 16 MiB encoding budget because
view models repeat descriptors and source fragments. OutputPlan's combined 1 MiB source cap and
the writer's independent 1 MiB formatted cap still apply. Template failures retain their concrete
cause and authored asset path; a template size failure retains `Limit` under the registry's
existing callback error layer.

See [admin code generation](../../framework-admin/docs/api/codegen.md) for catalog/emitter
contracts and [runtime selection](../../framework-sql/docs/guides/select-plugins.md) for the
separate ServerBuilder lifecycle. Runtime startup must never install, migrate or seed a database,
or bind a listener; an explicit build-tool output installation is a separate operation.
