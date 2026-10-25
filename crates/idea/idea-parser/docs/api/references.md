# Compiled schemas and references

`CompiledSchema` retains reference intent beside the compatibility config. Use it when composing
sources or persisting unresolved metadata. Sources: [stages.rs](../../src/stages.rs),
[references.rs](../../src/references.rs), [json.rs](../../src/json.rs).

```rust
use stackpress_idea_parser::{
    CompileInputs, IdeaError, SourceFile, ValuePath, compile_ast_with_references, parse_ast,
};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let ast = parse_ast(
        &SourceFile::new(
            "profile.idea",
            "model Profile @label([Labels]) { id String }",
        ),
        inputs.limits,
    )?;
    let compiled = compile_ast_with_references(&ast, &inputs)?;
    assert_eq!(
        compiled.references[0].path,
        [ValuePath::Index(0), ValuePath::Index(0)]
    );
    Ok(())
}
```

The path first selects argument zero, then its array element zero. It does not select a
column by numeric position.

Contents: [Data shapes](#1-data-shapes) · [Envelope JSON](#2-compiledschema-json-operations) ·
[Validation](#3-validation-and-composition-contract)

## 1. Data shapes

These public fields are owned and editable for the composition owner. Editing them does not
bypass validation at checked export/import or typed finalization.

### 1.1. CompiledSchema

Debug, Clone, PartialEq, and Serialize. It has two public fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `config` | `SchemaConfig` | Ordered non-final data; reference sites use `${Name}` strings. |
| `references` | `Vec<Reference>` | Only these structurally addressed sites are eligible for substitution. |

The config alone loses reference intent. A quoted `${Name}` string remains literal unless a
caller explicitly supplies a corresponding record; checked import cannot reconstruct historical intent.

### 1.2. Reference

Debug, Clone, PartialEq, Eq, Serialize, and Deserialize; unknown JSON fields are rejected.
It has no public inherent methods.

| Field | Type | Meaning |
| --- | --- | --- |
| `kind` | `DeclarationKind` | Owning type or model section. |
| `declaration` | `String` | Owning declaration name. |
| `column` | `Option<String>` | Named column, or `None` for declaration metadata. |
| `attribute` | `String` | Metadata namespace containing the site. |
| `path` | `Vec<ValuePath>` | Argument index followed by nested array indices/object keys. |
| `target` | `String` | Declaration to resolve. |
| `span` | `SourceSpan` | Original source coordinates, retained after composition. |
| `forward` | `bool` | Target declared at or after the owner in the same source. |

Targets absent from the originating source are not marked forward; a composer may supply them.
Self and same-source forward references remain invalid during finalization.

### 1.3. DeclarationKind and ValuePath

`DeclarationKind` is Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, and Deserialize, with
`Type` and `Model` variants. JSON uses lowercase `"type"` / `"model"`.

`ValuePath` is Debug, Clone, PartialEq, Eq, Hash, Serialize, and Deserialize:

| Variant | Meaning | JSON example |
| --- | --- | --- |
| `Index(usize)` | Argument or nested-array element. | `{"Index":0}` |
| `Key(String)` | Nested object member. | `{"Key":"label"}` |

Both enums have no public inherent methods. A valid path starts with `Index`, because its
root is a method's argument list.

## 2. CompiledSchema JSON operations

The checked envelope has exactly two required fields, `config` and `references`. Empty data is
`{"config":{},"references":[]}`; unknown envelope fields are rejected.

### 2.1. to_json

`to_json(&self, limits: ParseLimits) -> Result<String, IdeaError>`

Export the config and typed sites together after validation.

```rust
use stackpress_idea_parser::{CompiledSchema, IdeaError, ParseLimits, SchemaConfig};

fn main() -> Result<(), IdeaError> {
    let compiled = CompiledSchema {
        config: SchemaConfig::default(),
        references: Vec::new(),
    };
    assert_eq!(
        compiled.to_json(ParseLimits::default())?,
        r#"{"config":{},"references":[]}"#,
    );
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `limits` | `ParseLimits` | Config, site-metadata, and encoded-output budgets. |

**Returns** Owned compact envelope JSON, or invalid config/site/coordinate and resource failures.
Export does not require targets to exist or resolve them; those checks occur at finalization.

### 2.2. from_json

`CompiledSchema::from_json(source: &str, limits: ParseLimits) -> Result<Self, IdeaError>`

Decode the bounded duplicate-aware envelope, validating config and site destinations.

```rust
use stackpress_idea_parser::{CompiledSchema, IdeaError, ParseLimits};

fn main() -> Result<(), IdeaError> {
    let compiled =
        CompiledSchema::from_json(r#"{"config":{},"references":[]}"#, ParseLimits::default())?;
    assert!(compiled.references.is_empty());
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&str` | Complete envelope JSON, not plain schema JSON or AST JSON. |
| `limits` | `ParseLimits` | Input/value and typed-site policy. |

**Returns** New `CompiledSchema`, or JSON, shape, duplicate, site, coordinate, or limit errors
under `<json>` where applicable. Some site-validation failures retain the site's stored source
coordinates. There is no named-envelope import method. Historical source text is not loaded.

## 3. Validation and composition contract

Each site must name an existing type/model, optional column, and argument-array attribute. Its
nonempty path must reach exactly `${target}`. Paths must have valid kinds/indices/keys; duplicate
structural destinations, stale placeholders, invalid names, reversed/oversized ranges, zero
line/column, and exhausted budgets are rejected.

Site identity uses declaration kind/name, column name, attribute, and path, rather than span.
A composer must keep only surviving sites, preserve their original spans/forward flags, and
remove composed `use` directives before [typed finalization](finalization.md#24-finalize_with_references).
Checked validation authenticates structure, not the relationship of historical spans to absent text.

Related: [reference guide](../guides/resolving-references.md), [schema JSON](schema.md),
[compilation](stages.md#22-compile_ast_with_references).
