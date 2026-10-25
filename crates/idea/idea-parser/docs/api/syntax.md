# Syntax inspection API

Public syntax nodes expose the parser's extracted representation for inspection and serialization.
Only a checked `Ast` can enter compilation. Source: [ast.rs](../../src/ast.rs).

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SourceFile, Statement, parse_ast};

fn main() -> Result<(), IdeaError> {
    let ast = parse_ast(
        &SourceFile::new("profile.idea", "model Profile { id String }"),
        ParseLimits::default(),
    )?;
    let Statement::Declaration(declaration) = &ast.syntax().body[0] else {
        panic!("expected model");
    };
    assert_eq!(declaration.kind, "model");
    assert_eq!(declaration.declarations[0].id.name, "Profile");
    Ok(())
}
```

The AST has the source statement order; normalized config separates declarations into sections.
These shapes are not a promise of TypeScript ESTree parity.

## 1. Common node fields

All syntax structs below derive Debug, Clone, PartialEq, and Serialize. They have public fields
and no public inherent methods. `start: usize` and `end: usize` are half-open source byte offsets.
Nodes with `node_type: &'static str` serialize that field under the key `type`.

Synthetic normalized fields can carry enclosing or zero-width ranges. Do not treat every node
as an exact token slice; use [Token.raw](tokens.md) for exact lexical spelling.

## 2. Statement and declaration nodes

These nodes describe top-level order and declaration payloads. The listed fields supplement
the common `start`/`end` offsets.

| Type | Public fields or variants |
| --- | --- |
| `SchemaToken` | `node_type: &'static str`, `kind: &'static str`, `body: Vec<Statement>`; parser root has `Program` / `schema`. |
| `Statement` | `Declaration(DeclarationToken)` or `Import(ImportToken)`; untagged Serialize. |
| `ImportToken` | `node_type: &'static str`, `specifiers: Vec<()>` (reserved empty list), `source: LiteralToken` (import string). |
| `DeclarationToken` | `node_type: &'static str`, `kind: &'static str`, `mutable: Option<bool>`, `declarations: Vec<DeclaratorToken>`. |
| `DeclaratorToken` | `node_type: &'static str`, `id: IdentifierToken`, `init: ObjectToken`. |

`kind` is `plugin`, `prop`, `enum`, `type`, or `model`. `mutable` is present for types/models
and omitted from serialization when `None`. Parser-produced declarations currently contain one
declarator; the public inspection shape retains a vector.

## 3. Data nodes

Typed data preserves the distinction needed during compilation. All these structs also include
the common offsets and `node_type` field.

| Type | Additional public fields |
| --- | --- |
| `IdentifierToken` | `name: String`; declaration/member identity or an actual reference. |
| `ObjectToken` | `properties: Vec<PropertyToken>` in source order. |
| `PropertyToken` | `kind: &'static str`, `key: IdentifierToken`, `value: DataToken`, `method: Option<bool>`, `shorthand: Option<bool>`, `computed: Option<bool>`. |
| `ArrayToken` | `elements: Vec<DataToken>` in source order. |
| `LiteralToken` | `value: serde_json::Value`, `raw: String`; extracted literal representation. |

Optional property flags are omitted from JSON when `None`. `LiteralToken.raw` is retained
representation rather than a uniform exact source slice: raw Idea string nodes, for example,
use a quoted representation chosen by the extracted parser. Environment nodes retain the complete
`env("NAME")` spelling. Tokens provide a uniform exact `raw` contract.

`DataToken` is an untagged, serializable enum with these variants:

| Variant | Meaning |
| --- | --- |
| `Identifier(IdentifierToken)` | Actual declaration reference, eligible for later resolution. |
| `Literal(LiteralToken)` | Ordinary scalar data. |
| `Environment(LiteralToken)` | Unevaluated environment key and original range. |
| `Object(ObjectToken)` | Ordered object entries. |
| `Array(ArrayToken)` | Ordered nested values. |

Serializing syntax is an inspection/export operation, not a checked import format. There is no
public syntax deserializer or path from caller-assembled nodes to `Ast`.

Related: [SourceFile and Ast](source.md), [stages](stages.md), [typed references](references.md).
