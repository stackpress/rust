# Register a transform with explicit inputs

A generator often needs a template snapshot and a known runtime contract. Declare both before
planning so missing inputs and incompatible selections fail before managed output is touched.
The callback remains an ordinary synchronous Rust function.

## Supply the complete selection

This consumer needs the transformer and parser dependencies. The `text` transform reads a
declared auxiliary input; no callback opens a template file implicitly.

```rust
use stackpress_idea_transformer::{
    OutputPlan, Registry, Result, TransformContext, TransformInputs, TransformSelection,
    TransformSpec,
};

fn emit_text(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    assert_eq!(context.id(), "text");
    assert_eq!(context.contract("text-runtime"), Some("1"));
    plan.write("data/message.txt", context.input("templates/message.txt")?)
}

fn main() -> Result<()> {
    let mut inputs = TransformInputs::new();
    inputs.insert("templates/message.txt", "Hello, Ada\n")?;
    let mut selection = TransformSelection::new().with_inputs(inputs);
    selection.contract("text-runtime", "1")?;

    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("text")
            .owns(&["data"])
            .requires_contract("text-runtime", "1"),
        emit_text,
    )?;
    let plan = registry.plan_with(&Default::default(), &selection, vec![])?;
    assert_eq!(plan.inputs(), &["templates/message.txt"]);
    assert_eq!(plan.files().next().unwrap().2, "Hello, Ada\n");
    Ok(())
}
```

The required contract is checked before `emit_text`. `input` records the logical name in the
plan, while its contents stay out of reports. The tool owns reading any native template and
deciding whether its contents are safe to generate.

## Add ordered prerequisites

Use `requires(&["base"])` on a later spec when `base` must be registered earlier. Use
`after_if_present(&["metrics"])` when absence is allowed but selected metrics must precede it.
Planning rejects wrong order, missing required IDs, self-dependencies, and cycles; it never
sorts or auto-loads code. These setters replace their lists; repeated `requires_contract`
calls append distinct contract requirements.

If the root schema declares plugin blocks, declare the exact selected IDs in the same order.
With no root blocks the registry supplies defaults, and `context.config()` returns `None`.
Imported plugin blocks remain inert. See the [G20 table](../concepts.md#31-g20-selection-table).

You have made executable selection, contract expectations, and auxiliary input explicit.
Use [TransformSpec](../api/transform-spec.md) and [TransformContext](../api/transform-context.md)
when adding configuration or additional dependencies.
