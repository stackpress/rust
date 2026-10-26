# PublicStatus

Explicitly disclosed final page status and optional safe text. Source: [policy.rs](../../src/policy.rs).

```rust
use stackpress_framework_view::{PublicStatus, ViewError};
fn main() -> Result<(), ViewError> {
    let missing = PublicStatus::new(404, Some("Profile missing".into()))?;
    assert_eq!(missing.code(), 404);
    assert_eq!(missing.error(), Some("Profile missing"));
    assert_eq!(PublicStatus::default(), PublicStatus::ok());
    Ok(())
}
```

## Operations

The status code is independent of dispatch completion or Abort.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `ok()` / `Default` | None | Code 200, no error text. |
| `new` | `code: u16`, `error: Option<String>` | Code 200–599; message at most 8,192 bytes and only for codes ≥400. |
| `code(&self)` | None | Reads code as `u16`. |
| `error(&self)` | None | Borrows optional plain text as `Option<&str>`. |

**Returns** `new` yields `Result<PublicStatus, ViewError>` with `InvalidSnapshot` for invalid
status/message combinations; other constructors/accessors return the values shown above.
Error text may be absent even on failure. Empty error text is permitted for an error code.
This is deliberately public content, never a developer report.

Serde uses a checked object with `code` and `error`, rejecting unknown fields. The missing
`PageSnapshot.status` field defaults to `ok()`. The type is Clone, Debug, PartialEq/Eq,
Serialize/Deserialize.

Related: [PageSnapshot](page-snapshot.md), [native lifecycle](../concepts.md#lifecycle-remains-the-servers).
