# Finalization API

Finalization resolves declaration references and returns the sections needed by downstream tools.
It performs no source loading or executable plugin activation. Sources:
[lib.rs](../../src/lib.rs), [stages.rs](../../src/stages.rs), [references.rs](../../src/references.rs).

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, SourceFile, finalize_ast, parse_ast};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let ast = parse_ast(
        &SourceFile::new(
            "profile.idea",
            "prop Labels { title \"Profiles\" } model Profile @label(Labels) { id String }",
        ),
        inputs.limits,
    )?;
    let schema = finalize_ast(&ast, &inputs)?;
    assert!(schema.model.contains_key("Profile"));
    Ok(())
}
```

Contents: [FinalSchema](#1-finalschema) · [Entry points](#2-finalization-entry-points)

## 1. FinalSchema

Debug, Clone, PartialEq, Serialize, and Default. It has no public inherent methods. It contains
the following ordered maps, and omits empty sections during serialization:

| Public field | Type | Meaning |
| --- | --- | --- |
| `plugin` | `IndexMap<String, PluginConfig>` | Plugin configuration data. |
| `r#enum` | `IndexMap<String, EnumConfig>` | Scalar choice maps. |
| `r#type` | `IndexMap<String, TypeConfig>` | Reusable fieldsets with resolved metadata sites. |
| `model` | `IndexMap<String, ModelConfig>` | Models with resolved metadata sites. |

There are no `prop` or `use` fields. Referenced prop data is materialized at its destination
before props disappear. Final data still contains literal `${Name}` strings when they were
ordinary strings; their presence does not establish an unresolved reference.

## 2. Finalization entry points

Choose based on ownership of imports and reference intent. All return new owned data without
mutating their borrowed inputs.

### 2.1. Entry selection

The compatibility API and staged APIs deliberately have different import policies:

| Entry | Input | Outstanding imports |
| --- | --- | --- |
| `finalize` | Named source text | Discards directives without loading them. |
| `finalize_ast` | Checked single-source syntax | Rejects them with `ValidationError`. |
| `finalize_with_references` | Composed config plus typed sites | Rejects them; composer must remove them. |

Prefer `finalize_ast` for a strict standalone source and the transformer for multi-source use.
No successful compatibility finalization proves that an import exists or was merged.

### 2.2. finalize

`finalize(file: &str, source: &str, inputs: &CompileInputs) -> Result<FinalSchema, IdeaError>`

Compile source and resolve references to prior declarations, preserving the pinned compatibility
behavior for `use` directives.

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, finalize};

fn main() -> Result<(), IdeaError> {
    let schema = finalize(
        "profile.idea",
        "use \"unloaded.idea\" model Profile { id String }",
        &CompileInputs::default(),
    )?;
    assert!(schema.model.contains_key("Profile"));
    Ok(())
}
```

This succeeds without reading `unloaded.idea`; the directive was discarded.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `file` | `&str` | Logical source identity. |
| `source` | `&str` | Complete Idea source. |
| `inputs` | `&CompileInputs` | Explicit environment and current resource policy. |

**Returns** `FinalSchema`, or syntax, environment, duplicate, unknown/prior-reference, literal,
and expansion/resource failures. Reference targets must already be visible in source order;
quoted placeholder-looking strings remain literal.

### 2.3. finalize_ast

`finalize_ast(ast: &Ast, inputs: &CompileInputs) -> Result<FinalSchema, IdeaError>`

Finalize checked single-source syntax with an explicit no-import boundary. The opening example
shows a complete call with a prior prop target.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ast` | `&Ast` | Checked source and typed syntax. |
| `inputs` | `&CompileInputs` | Explicit environment and rechecked budgets. |

**Returns** `FinalSchema`, or an outstanding-import diagnostic at the first directive, same-source
forward/unknown references, syntax/environment, or resource failures. The import check precedes
compilation, so missing environment data does not hide an outstanding import.

### 2.4. finalize_with_references

`finalize_with_references(schema: &SchemaConfig, references: &[Reference], inputs: &CompileInputs) -> Result<FinalSchema, IdeaError>`

Resolve exactly the supplied structural sites after composition. Other strings stay literal.

```rust
use stackpress_idea_parser::{
    AttributeValue, CompileInputs, IdeaError, SourceFile, compile_ast_with_references,
    finalize_with_references, parse_ast,
};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let ast = parse_ast(
        &SourceFile::new("profile.idea", "model Profile @label(Labels) { id String }"),
        inputs.limits,
    )?;
    let mut compiled = compile_ast_with_references(&ast, &inputs)?;
    // Supply an external target as a composer would; this example performs no loading.
    compiled.config.prop.insert(
        "Labels".into(),
        [("title".into(), "Profiles".into())].into(),
    );
    let schema = finalize_with_references(&compiled.config, &compiled.references, &inputs)?;
    let AttributeValue::Array(values) = &schema.model["Profile"].attributes["label"] else {
        panic!("label has arguments");
    };
    assert_eq!(values[0]["title"], "Profiles");
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `schema` | `&SchemaConfig` | Bounded normalized composed data with no outstanding imports. |
| `references` | `&[Reference]` | Exactly the surviving actual sites, with original provenance. |
| `inputs` | `&CompileInputs` | Uses `limits`; environment is already compiled and is not re-evaluated here. |

**Returns** `FinalSchema`, or invalid config/sites, outstanding imports, missing targets,
same-source forward/self references, dependency cycles, or depth/byte/copy-budget failures.
Unknown/forward/cyclic reference failures use `UnknownReference`; malformed sites use
`ValidationError`. Reference failures retain original stored coordinates where available.

Validation runs before resolution: config first, sites second, outstanding imports third.
Repeated/cached copies still spend the expansion budget, and resulting nesting/output size is
checked. Input data remains unchanged on failure.

Related: [references](references.md), [reference guide](../guides/resolving-references.md),
[schema shapes](schema.md).
