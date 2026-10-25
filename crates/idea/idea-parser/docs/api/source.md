# Source, spans, and checked syntax

`SourceFile` owns named UTF-8 text; `Ast` owns checked immutable syntax. `SourceSpan` stores
coordinates without embedding source text. Source: [stages.rs](../../src/stages.rs).

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SourceFile, parse_ast};

fn main() -> Result<(), IdeaError> {
    let source = SourceFile::new("profile.idea", "model Profile { id String }");
    let ast = parse_ast(&source, ParseLimits::default())?;
    assert_eq!(ast.source(), &source);
    assert_eq!(ast.syntax().body.len(), 1);
    Ok(())
}
```

The AST retains its own source identity and text. It does not borrow the caller's `SourceFile`.

Contents: [SourceFile](#1-sourcefile) · [Ast](#2-ast) · [SourceSpan](#3-sourcespan)

## 1. SourceFile

`SourceFile` is Debug, Clone, PartialEq, and Eq. Fields are private. Construction performs no
IO and does not validate resource limits; bound caller input before construction if indexing
allocation itself needs a host-side limit.

### 1.1. new

`SourceFile::new(name: impl Into<String>, text: impl Into<String>) -> Self`

Own a logical diagnostic name and unchanged source text, building line/scalar indexes.

```rust
use stackpress_idea_parser::SourceFile;

fn main() {
    let source = SourceFile::new("memory.idea", "// café\r\n");
    assert_eq!(source.name(), "memory.idea");
    assert_eq!(source.text(), "// café\r\n");
}
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `name` | `impl Into<String>` | Caller-owned identity; use distinct names for distinct sources. |
| `text` | `impl Into<String>` | Exact UTF-8 text, including original line endings. |

**Returns** An owned source. No parser policy, filesystem resolution, or normalization runs here.

### 1.2. name and text

`name(&self) -> &str` and `text(&self) -> &str` borrow the retained identity and text. The preceding
example demonstrates both accessors; neither changes the source or allocates a new string.

**Returns** A borrow tied to the `SourceFile`, with no fallible operation.

## 2. Ast

`Ast` is Debug, Clone, and PartialEq. Only [parse_ast](stages.md#12-parse_ast) constructs it.
There is no public mutable syntax accessor, unchecked constructor, or JSON import for `Ast`.

### 2.1. source and syntax

`source(&self) -> &SourceFile` and `syntax(&self) -> &SchemaToken` inspect the checked owner.

```rust
use stackpress_idea_parser::{DataToken, IdeaError, ParseLimits, SourceFile, Statement, parse_ast};

fn main() -> Result<(), IdeaError> {
    let source = SourceFile::new("settings.idea", "prop Settings { title env(\"TITLE\") }");
    let ast = parse_ast(&source, ParseLimits::default())?;
    assert_eq!(ast.source().name(), "settings.idea");
    let Statement::Declaration(declaration) = &ast.syntax().body[0] else {
        panic!("expected prop");
    };
    assert!(matches!(
        declaration.declarations[0].init.properties[0].value,
        DataToken::Environment(_),
    ));
    Ok(())
}
```

Environment syntax remains typed without requiring a value. Detached public nodes can be cloned
or serialized for inspection, but cannot replace the syntax within the checked AST.

**Returns** Immutable borrows tied to the `Ast`. Access does not compile or evaluate anything.

## 3. SourceSpan

`SourceSpan` is Debug, Clone, PartialEq, Eq, Serialize, and Deserialize, with unknown JSON fields
rejected. It has these public fields and no public inherent methods:

| Field | Type | Meaning |
| --- | --- | --- |
| `file` | `String` | Caller-supplied logical source identity. |
| `start` | `usize` | Inclusive UTF-8 byte offset. |
| `end` | `usize` | Exclusive UTF-8 byte offset. |
| `line` | `usize` | One-based line; CRLF is one break, standalone CR also breaks. |
| `column` | `usize` | One-based Unicode-scalar column; a tab counts as one scalar. |

Parser-produced spans use valid character boundaries within their original source. Deserializing
coordinates alone does not validate them against historical text. Checked compiled-envelope import
validates ranges and coordinate structure, without loading that text.

Related: [syntax nodes](syntax.md), [tokens](tokens.md), [diagnostics](error.md).
