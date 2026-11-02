# Build-host code generation

`stackpress_framework_sql::codegen` requires the `codegen` Cargo feature. It emits
native stores from an already normalized `Schema`. The feature activates the
existing optional `stackpress-lib` dependency for its template engine; it does not
require `server`. Selecting generated host handlers requires `server` in the
consuming app and explicit operation registration there.

## Register a transform

`register(registry, transform) -> Result<()>` adds `framework-sql`, claims the
`stores` partition and requires the `framework-schema` transform. The caller
supplies the callback and its frozen definition registry. Register the schema
transform before planning; this helper neither normalizes schemas nor installs
output.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `registry` | `&mut Registry` | Build-tool transform registry. |
| `transform` | `ContextTransform` | Function callback receiving schema context and an output plan. |

**Returns** `Ok(())` when registration succeeds. Duplicate IDs or partition claims
return the registry's error. Dependency order is checked during planning.

This minimal callback uses standard definitions. An app with custom providers,
validators or exposure rules must supply its own frozen definitions at the same
normalization boundary as its schema transform.

```rust
use stackpress_framework_schema::Schema;
use stackpress_framework_schema::codegen::{
    DefinitionRegistry, NormalizeOptions, transform_error,
};
use stackpress_framework_sql::codegen::{emit_stores, register};
use stackpress_idea_transformer::{OutputPlan, Registry, Result, TransformContext};

fn stores(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let schema = Schema::normalize(
        context.schema(),
        &DefinitionRegistry::standard(),
        NormalizeOptions::default(),
    )
    .map_err(transform_error)?;
    emit_stores(&schema, output)
}

pub fn register_stores(registry: &mut Registry) -> Result<()> {
    register(registry, stores)
}
```

## Emit stores and handlers

| Function | Additional arguments | Output |
| --- | --- | --- |
| `emit_stores(schema, output)` | None | Native stores, finite eager views, selections and aggregate storage definitions. |
| `emit_stores_with_operations(schema, output)` | None | The same stores plus named host handlers in `stores/operations.rs`. |
| `emit_stores_with_diagnostics(schema, output, diagnostics, with_operations)` | `&DiagnosticMap`, `bool` | The selected output, with winning Idea coordinates or JSON structural paths on model/projection diagnostics. |

Every function takes `schema: &Schema` and `output: &mut OutputPlan`, and returns
`Result<()>`. The plan must be used under authority for `stores`; normal registry
callbacks supply that authority. Output stays in the plan until the managed
workspace validates, formats and installs it. An emitter error can leave earlier
claims in that in-memory plan; discard a failed plan. It cannot partially install
an output root.

**Returns** `Ok(())` after adding the selected source and module contributions to
the plan. To select handlers in the callback above, replace its final call with
`stackpress_framework_sql::codegen::emit_stores_with_operations(&schema, output)`.
Use the diagnostics variant when the build tool has retained a `DiagnosticMap`.

Failures include schema projection/storage errors, unsupported default selections,
template failures and output ownership/resource refusals. Each template failure
retains a static SQL asset label and a concrete `TemplateError` cause. Its resource
refusals map to `TransformErrorKind::Limit`; other template failures map to
`Callback`. Existing model/projection wrappers still apply. `Registry::plan_with`
adds an outer `Callback`, so inspect the cause chain when diagnosing a direct
emitter limit.

## Obtain storage definitions

`storage_definitions(schema) -> Result<Vec<TableDefinition>>` returns the exact
ordered storage catalog used by generated source and durable revision snapshots.
It checks complete primary keys, relation columns, portable constraint names,
storage lengths and foreign-key width agreement. It performs no database IO.

`mysql_storage_definitions(schema, collation, page_bytes)` returns the same result
type with explicit MySQL target facts and whole-catalog policy validation.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `schema` | `&Schema` | Normalized app-owned schema, shared by both functions. |
| `collation` | `&str` | App-selected supported utf8mb4 collation for MySQL validation. |
| `page_bytes` | `u32` | App-selected InnoDB page size. |

**Returns** checked definitions in model order. Definition/name/relation failures and
incompatible storage widths return errors. The MySQL variant also retains the
concrete DDL policy cause for invalid target facts or catalog/index constraints.
See [explicit string storage length](storage-length.md) for supported target facts
and physical limits.

## Related contracts

- [Changing generated Rust source](../guides/change-generated-source.md) describes assets, contexts and compatibility checks.
- [Registering operations](../guides/register-operations.md) describes the app-owned runtime boundary.
- [Template engine reference](../../../../lib/docs/api/template.md) describes the underlying bounded renderer.
- [Managed module generation](../../../../idea/idea-transformer/docs/guides/generating-modules.md) describes ownership and installation.
