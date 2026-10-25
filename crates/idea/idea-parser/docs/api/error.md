# IdeaError and diagnostic codes

Parser, compiler, and checked JSON failures return `IdeaError` by value. It implements Display
and `std::error::Error`, with no public inherent constructor or mutator.
Source: [error.rs](../../src/error.rs).

```rust
use stackpress_idea_parser::{CompileInputs, ErrorCode, parse};

fn main() {
    let error = parse("profile.idea", "model Profile {", &CompileInputs::default()).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnexpectedEof);
    assert_eq!(error.start, error.end);
    assert!(error.to_string().starts_with("profile.idea:"));
}
```

The EOF range is zero-width at source end. Display does not print the schema body.

## 1. IdeaError fields

`IdeaError` is Debug, Clone, PartialEq, Eq, and Serialize. Its fields are public:

| Field | Type | Meaning |
| --- | --- | --- |
| `code` | `ErrorCode` | Stable typed failure category. |
| `message` | `String` | Human explanation without automatic source excerpts. |
| `start` | `Option<usize>` | Inclusive byte offset; source-free validation can omit it. |
| `end` | `Option<usize>` | Exclusive byte offset. |
| `file` | `String` | Logical identity; can be empty for source-free failures. |
| `line` | `usize` | One-based line, with CRLF treated as one break. |
| `column` | `usize` | One-based Unicode-scalar column; tabs count as one scalar. |
| `related` | `Box<[SourceSpan]>` | Earlier locations associated with duplicates/related failures. |

Parser-generated coordinates are safe half-open UTF-8 ranges in the supplied source. Checked JSON
structural errors may point to `0..0`; raw JSON syntax errors use their decoded location. Typed
reference validation/resolution can return historical source coordinates stored in the record.

## 2. ErrorCode variants

`ErrorCode` is Debug, Clone, PartialEq, Eq, and Serialize. Serialized variants use the exact
Rust names below; callers should branch on the enum rather than parsing message text.

| Variant | Meaning |
| --- | --- |
| `UnexpectedToken` | Source input does not match the declaration/value grammar. |
| `UnexpectedEof` | A required closer or literal ending is missing. |
| `InvalidDeclaration` | Unsupported declaration kind. |
| `InvalidLiteral` | Malformed/unrepresentable literal or type modifier; includes numeric overflow/nonfinite values. |
| `DuplicateName` | Duplicate declaration/member or duplicate JSON key. |
| `UnknownReference` | Missing, forward, self, or cyclic reference dependency. |
| `MissingProperty` | Required normalized structure is absent. |
| `ValidationError` | Invalid normalized structure/site, outstanding imports at strict finalization, or malformed JSON. |
| `Limit` | Source, nesting, node, collection, expansion, or serialized-byte budget failure. |
| `MissingEnvironment` | An environment key was not explicitly supplied and compatibility fallback is off. |

The category is a lookup aid, not a claim that every entry point can emit every variant.
For example, the current source parser rejects an unknown statement keyword with
`UnexpectedToken`; checked JSON syntax errors use `ValidationError`.

## 3. Rendering and integration

Display produces `file:line:column: message`. It omits source excerpts and environment values;
applications can render source separately when appropriate. Messages can identify names from the
input, so sanitized presentation does not mean every identifier is removed.

There is no parser-specific boxed Result alias or automatic source-map/snippet renderer. Keep
the original text associated with each logical name before slicing coordinates, and handle
source-free errors separately. Deriving Serialize supports explicit diagnostic export, not a
checked diagnostic-import contract.

Related: [diagnostic guide](../guides/diagnostics-and-limits.md), [SourceSpan](source.md#3-sourcespan),
[limits](inputs.md).
