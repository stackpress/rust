# Lexical token API

`tokenize` uses the parser's scanners to return exact source spelling and coordinates, without
checking full declaration grammar or evaluating environment values. Source: [token.rs](../../src/token.rs).

```rust
use stackpress_idea_parser::{IdeaError, ParseLimits, SourceFile, TokenKind, tokenize};

fn main() -> Result<(), IdeaError> {
    let source = SourceFile::new("tokens.idea", "env(\"TITLE\")");
    let tokens = tokenize(&source, ParseLimits::default())?;
    assert_eq!(tokens[0].kind, TokenKind::Environment);
    assert_eq!(tokens[0].raw, "env(\"TITLE\")");
    Ok(())
}
```

## 1. tokenize

`tokenize(source: &SourceFile, limits: ParseLimits) -> Result<Vec<Token>, IdeaError>`

Scan lexical data, skipping whitespace and comments. A bare scalar can tokenize successfully
even though it is not a complete Idea schema.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `source` | `&SourceFile` | Named exact text used for raw spelling and spans. |
| `limits` | `ParseLimits` | Source-byte, token-node, and delimiter-nesting budgets. |

**Returns** Owned tokens in source order, or lexical/literal/namespace/comment/resource errors.
Empty text returns an empty vector. Success does not establish balanced delimiters or valid
declarations. No environment key needs to be supplied.

## 2. Token shapes

`Token` is Debug, Clone, PartialEq, and Serialize. Its public fields are:

| Field | Type | Meaning |
| --- | --- | --- |
| `kind` | `TokenKind` | Lexical category. |
| `value` | `TokenValue` | Owned typed payload. |
| `span` | `SourceSpan` | Named half-open byte range. |
| `raw` | `String` | Exact source slice, including delimiters belonging to this token. |

`TokenKind` is Debug, Clone, PartialEq, Eq, and Serialize:

| Variant | Meaning |
| --- | --- |
| `Word` | Identifier, keyword, or complete validated type spelling such as `String[]?`. |
| `Literal` | Raw string, exact number, boolean, or null. |
| `Environment` | Entire `env("NAME")` expression, with unevaluated key. |
| `Attribute` | Valid lowercase metadata namespace. |
| `Delimiter` | Bracket, brace, parenthesis, or immutable marker. |

`TokenValue` is Debug, Clone, PartialEq, and Serialize:

| Variant | Meaning |
| --- | --- |
| `Text(String)` | Identifier or metadata namespace text. |
| `Literal(serde_json::Value)` | Scalar data, or a string containing an environment key. |
| `Symbol(char)` | One grammar delimiter. |

These types have no public inherent methods or checked token-import API. Tokenization shares
the scanners for Idea's raw strings, exact numbers, comments, environment expressions, and names;
it does not introduce a different language.

Related: [syntax parsing](stages.md#12-parse_ast), [source coordinates](source.md),
[syntax inspection](syntax.md).
