# TransformContext API

TransformContext is a borrowed immutable view constructed only by Registry for one contextual
callback. Its fields are private. Source: [registry/context.rs](../../src/registry/context.rs).

```rust
use stackpress_idea_transformer::{
    OutputPlan, Registry, Result, TransformContext, TransformInputs, TransformSelection,
    TransformSpec,
};
fn inspect(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    let schema = context.schema();
    assert!(schema.model.is_empty());
    assert_eq!(context.id(), "text");
    assert!(context.config().is_none());
    assert_eq!(context.contract("text-runtime"), Some("1"));
    plan.write("data/message.txt", context.input("templates/message.txt")?)
}
let mut inputs = TransformInputs::new();
inputs.insert("templates/message.txt", "Hello\n")?;
let mut selection = TransformSelection::new().with_inputs(inputs);
selection.contract("text-runtime", "1")?;
let mut registry = Registry::new();
registry.add(TransformSpec::new("text").owns(&["data"]), inspect)?;
let plan = registry.plan_with(&Default::default(), &selection, vec![])?;
assert_eq!(plan.inputs(), &["templates/message.txt"]);
```

The example exercises every accessor inside a real registry invocation. Contexts cannot be
constructed by callers or retained as write authority after callbacks finish. Lifetimes below
use the context's `'a` schema/selection borrow.

## 1. schema

`schema(&self) -> &'a SchemaConfig`

Borrow the shared schema, as `let schema = context.schema()` does above.

**Returns** The immutable config used by all callbacks. Direct registry callers choose whether
it is finalized; the wrapper always supplies its finalized SchemaConfig view with imports and
props consumed. This accessor does not parse, load, or finalize anything.

## 2. id

`id(&self) -> &'a str`

Inspect the compiled identity; `context.id()` returns `"text"` in the example.

**Returns** The executing registration's ID, independent of its Rust function symbol.

## 3. config

`config(&self) -> Option<&'a PluginConfig>`

Read the current root plugin block. The example verifies `context.config().is_none()` for a
registry-default run with no root blocks.

**Returns** `Some` for this ID's root plugin config, or `None` when root blocks are absent.
PluginConfig and its data are [parser-owned](../../../idea-parser/docs/api/README.md).
The accessor never enables imported or uncompiled plugins.

## 4. contract

`contract(&self, id: &str) -> Option<&'a str>`

Inspect an explicitly supplied runtime revision, as `context.contract("text-runtime")` does.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&str` | Exact selected runtime contract ID. |

**Returns** The supplied revision or `None`, without reading process state. Declaring a
TransformSpec requirement makes the registry validate presence/revision before this callback.

## 5. input

`input(&self, name: &str) -> Result<&'a str>`

Consume a predeclared snapshot. The example's `context.input("templates/message.txt")?` feeds
the text to a fresh file claim.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `&str` | Exact logical name in TransformInputs. |

**Returns** Borrowed immutable text and records its logical name for deterministic reporting.
Undeclared names return TransformError; no filesystem lookup or automatic fallback occurs.
Repeated reads deduplicate the name in the final sorted access log. Values stay out of the
log, but the callback deliberately writes one into output in this example.

Related: [selection](transform-selection.md), [registry](registry.md),
[declared-input guide](../guides/registering-transforms.md). These APIs do not sandbox trusted
Rust code that directly reads files or process environment.
