# Registry API

Registry selects compiled function transforms and executes them sequentially after preflight.
`TransformRegistry` is an alias for the same type. Registry implements Default.
Source: [registry.rs](../../src/registry.rs).

```rust
use stackpress_idea_parser::SchemaConfig;
use stackpress_idea_transformer::{
    GeneratedFile, OutputPlan, Registry, Result, TransformContext, TransformSelection,
    TransformSpec,
};
fn emit(_: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    plan.write("data/profile.txt", "Profile\n")
}
fn legacy(_: &SchemaConfig) -> Result<Vec<GeneratedFile>> {
    Ok(vec![GeneratedFile::new("data/profile.txt", "Profile\n")])
}
let mut registry = Registry::new();
```

Each following block uses that setup independently. Contents:
[1. Selection](#1-selection) · [2. Planning](#2-planning) · [3. Callback aliases](#3-callback-aliases)

## 1. Selection

The application chooses code before planning. IDs are distinct from Rust symbols and source
plugin config cannot load uncompiled functions.

### 1.1. Registry::new

`Registry::new() -> Self`

Start a compiled selection.

```rust
let registry = Registry::new();
assert!(
    registry
        .plan(&Default::default(), vec![])?
        .executed()
        .is_empty()
);
```

**Returns** An empty registry. With no root plugin blocks it supports aggregate-only output.

### 1.2. add

`add(&mut self, spec: TransformSpec, transform: ContextTransform) -> Result<()>`

Select a contextual function and grant exclusive partitions.

```rust
registry.add(TransformSpec::new("profile").owns(&["data"]), emit)?;
assert_eq!(
    registry.plan(&Default::default(), vec![])?.executed(),
    &["profile"]
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `spec` | `TransformSpec` | ID, partitions, prerequisites, and runtime contracts. |
| `transform` | `ContextTransform` | Named synchronous function receiving context and plan. |

**Returns** Unit, or a metadata/duplicate/partition/budget error with no partial registration.
Presence, ordering, and contract matching are deferred until planning. At most 1,024 transforms
are allowed; IDs are valid portable paths of at most 128 bytes, unique, and not `aggregate`.

### 1.3. register

`register(&mut self, id: &str, partitions: &[&str], transform: Transform) -> Result<()>`

Use the compatible schema-to-files callback when contextual inputs are unnecessary.

```rust
registry.register("profile", &["data"], legacy)?;
assert_eq!(
    registry.plan(&Default::default(), vec![])?.files().count(),
    1
);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `id` | `&str` | Unique compiled transform ID. |
| `partitions` | `&[&str]` | Exclusive single-component output partitions. |
| `transform` | `Transform` | Function returning proposed complete files. |

**Returns** Unit or the same atomic registration failures as add. An empty partition list is
valid, but then the callback cannot emit files. This form supplies no prerequisite/contract metadata.

## 2. Planning

The complete [G20 table](../concepts.md#31-g20-selection-table) applies: no root plugin blocks
uses all registry defaults, an empty registry supports aggregate-only output, and present root
blocks must exactly match every registered ID in listed order. Imported blocks are inert.

### 2.1. plan

`plan(&self, schema: &SchemaConfig, aggregates: Vec<GeneratedFile>) -> Result<OutputPlan>`

Plan without auxiliary inputs or runtime contract declarations.

```rust
registry.add(TransformSpec::new("profile").owns(&["data"]), emit)?;
let plan = registry.plan(
    &SchemaConfig::default(),
    vec![GeneratedFile::new("index.txt", "Profiles\n")],
)?;
assert_eq!(plan.files().count(), 2);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `schema` | `&SchemaConfig` | Config view; finalize references beforehand when needed. |
| `aggregates` | `Vec<GeneratedFile>` | App-owned root files, claimed before callbacks. |

**Returns** A checked in-memory plan or selection, callback, path, ownership, or budget errors.
Equivalent to plan_with using a default TransformSelection. No output workspace is touched.

### 2.2. plan_with

`plan_with(&self, schema: &SchemaConfig, selection: &TransformSelection, aggregates: Vec<GeneratedFile>) -> Result<OutputPlan>`

Supply explicit runtime contracts and immutable auxiliary inputs.

```rust
registry.add(TransformSpec::new("profile").owns(&["data"]), emit)?;
let selection = TransformSelection::new();
let plan = registry.plan_with(&SchemaConfig::default(), &selection, vec![])?;
assert_eq!(plan.executed(), &["profile"]);
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `schema` | `&SchemaConfig` | Same immutable schema for every callback. |
| `selection` | `&TransformSelection` | Contracts and declared snapshots, not executable selection. |
| `aggregates` | `Vec<GeneratedFile>` | Root-only app aggregates; no collision with partition names. |

**Returns** A fresh OutputPlan. All selection/dependency/contract checks precede transform
callbacks; callbacks execute once in registration order, and automatic module indexes finish
after them. Prerequisites must occur earlier; optional IDs may be absent. Contextual failures
preserve the original cause under a safe phase message; legacy errors retain their direct
behavior. Panics propagate. Direct IO performed by trusted callbacks lies outside plan guarantees.

## 3. Callback aliases

`Transform = fn(&SchemaConfig) -> Result<Vec<GeneratedFile>>` is the compatible minimal form.
`ContextTransform = fn(&TransformContext<'_>, &mut OutputPlan) -> Result<()>` is the contextual
form. They are function pointers, not captured closures, trait plugins, async futures, or
dynamic libraries. A callback receives authority only while the registry invokes it.

Related: [TransformSpec](transform-spec.md), [selection](transform-selection.md),
[context](transform-context.md), [output plan](output-plan.md).
