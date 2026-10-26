# PublicLocation

Checked same-origin pathname/query text. Source: [policy.rs](../../src/policy.rs).

```rust
use stackpress_framework_view::{PublicLocation, ViewError};
fn main() -> Result<(), ViewError> {
    let location = PublicLocation::new("/ui/profiles?id=a%2Fb")?;
    assert_eq!(location.as_str(), "/ui/profiles?id=a%2Fb");
    assert!(PublicLocation::new("//other.example/path").is_err());
    Ok(())
}
```

## Construction and access

This wrapper preserves encoded text; it neither decodes parameters nor normalizes a query.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `new` | `value: impl Into<String>` | Requires a single leading slash and at most 4,096 bytes. |
| `TryFrom<String>` | Owned text | Applies `new` during explicit conversion and serde decoding. |
| `as_str(&self)` | None | Borrows exact text. |
| `From<PublicLocation> for String` | Consumed location | Transfers text without normalization. |

**Returns** Construction yields `Result<PublicLocation, ViewError>`; invalid input returns
`InvalidSnapshot`. Access/conversion yield `&str`/`String`. It is Clone, Debug, PartialEq/Eq,
Serialize/Deserialize.

Input must be ASCII graphic text with complete hexadecimal percent escapes. Raw whitespace,
backslash, fragment `#`, `<`, `>`, double quote, and backtick are rejected. Literal or
percent-encoded dot-only pathname segments (`.` or `..`, case insensitive `%2e`) are rejected.
Encode Unicode and parameter values explicitly. Validation is not application route membership,
authorization, or a general URL canonicalizer.

Related: [BasePath](base-path.md), [PageRoute](page-route.md).
