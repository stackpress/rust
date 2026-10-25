# Parsing and compilation API

These functions turn named text into checked syntax or ordered schema data. They are synchronous,
return owned results, and read no files or process environment. Sources:
[lib.rs](../../src/lib.rs), [stages.rs](../../src/stages.rs).

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, parse};

fn main() -> Result<(), IdeaError> {
    let schema = parse(
        "profile.idea",
        "model Profile { id String }",
        &CompileInputs::default(),
    )?;
    assert_eq!(schema.model["Profile"].columns[0].column_type, "String");
    Ok(())
}
```

Contents: [Text and syntax](#1-text-and-syntax) · [Checked compilation](#2-checked-compilation)

## 1. Text and syntax

Use `parse` for direct config or `parse_ast` when syntax inspection or a later compile stage
is needed. Neither resolves actual declaration references in metadata.

### 1.1. parse

`parse(file: &str, source: &str, inputs: &CompileInputs) -> Result<SchemaConfig, IdeaError>`

Compile one source while retaining `use`, `prop`, and unresolved placeholder values. The opening
example shows the full call.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `file` | `&str` | Logical name for diagnostics; not a filesystem path to open. |
| `source` | `&str` | Complete Idea source text. |
| `inputs` | `&CompileInputs` | Explicit environment, missing-key policy, and budgets. |

**Returns** Ordered `SchemaConfig`, or positioned syntax, duplicate, missing-environment, literal,
or resource failure. Forward/missing metadata targets can remain unresolved at this stage. Use
typed compilation if reference intent must survive composition or JSON exchange.

### 1.2. parse_ast

`parse_ast(source: &SourceFile, limits: ParseLimits) -> Result<Ast, IdeaError>`

Check complete declaration syntax while retaining environment names and typed references.

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SourceFile, parse_ast};

fn main() -> Result<(), IdeaError> {
    let source = SourceFile::new("settings.idea", "prop Settings { title env(\"TITLE\") }");
    let ast = parse_ast(&source, ParseLimits::default())?;
    assert_eq!(ast.syntax().body.len(), 1);
    Ok(())
}
```

The missing environment value is irrelevant to syntax parsing; compilation requires it later.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&SourceFile` | Exact named source, cloned into the returned checked owner. |
| `limits` | `ParseLimits` | Source bytes/name, recursive depth, nodes, and collection policy. |

**Returns** Immutable `Ast`, or a named syntax/duplicate/literal/limit diagnostic. Empty source
is valid. Trailing nondeclaration input is rejected; unknown type and metadata meanings remain open.

## 2. Checked compilation

Both functions recheck the source with the supplied policy and evaluate only the explicit
environment snapshot. They retain imports rather than composing sources.

### 2.1. compile_ast

`compile_ast(ast: &Ast, inputs: &CompileInputs) -> Result<SchemaConfig, IdeaError>`

Normalize checked syntax into ordered config without resolving declaration references.

```rust
use stackpress_idea_parser::{CompileInputs, IdeaError, SourceFile, compile_ast, parse_ast};

fn main() -> Result<(), IdeaError> {
    let mut inputs = CompileInputs::default();
    let ast = parse_ast(
        &SourceFile::new("settings.idea", "prop Settings { title env(\"TITLE\") }"),
        inputs.limits,
    )?;
    inputs.environment.insert("TITLE".into(), "Profiles".into());
    let schema = compile_ast(&ast, &inputs)?;
    assert_eq!(schema.prop["Settings"]["title"], "Profiles");
    Ok(())
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ast` | `&Ast` | Immutable parser-created syntax and retained source. |
| `inputs` | `&CompileInputs` | Environment and current budgets; lower limits still apply. |

**Returns** New `SchemaConfig`, or environment, syntax/structure, duplicate, or budget errors.
The AST and input snapshot remain unchanged. Bare metadata references become `${Name}` strings;
the config alone does not distinguish them from literal strings.

### 2.2. compile_ast_with_references

`compile_ast_with_references(ast: &Ast, inputs: &CompileInputs) -> Result<CompiledSchema, IdeaError>`

Compile config and retain actual reference sites for a composition owner or JSON handoff.

```rust
use stackpress_idea_parser::{
    CompileInputs, IdeaError, SourceFile, compile_ast_with_references, parse_ast,
};

fn main() -> Result<(), IdeaError> {
    let inputs = CompileInputs::default();
    let source = SourceFile::new("profile.idea", "model Profile @label(Labels) { id String }");
    let ast = parse_ast(&source, inputs.limits)?;
    let compiled = compile_ast_with_references(&ast, &inputs)?;
    assert_eq!(compiled.references[0].target, "Labels");
    assert!(!compiled.references[0].forward);
    Ok(())
}
```

`Labels` is absent from this source, so a composer may supply it. This call records intent;
it does not prove that finalization can find the target.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `ast` | `&Ast` | Checked source used for structural site discovery and provenance. |
| `inputs` | `&CompileInputs` | Same compile policy as `compile_ast`. |

**Returns** `CompiledSchema { config, references }`, or the errors from `compile_ast` plus
reference-metadata budget failures. Records retain source order, including nested argument paths.

Related: [inputs](inputs.md), [references](references.md), [finalization](finalization.md).
