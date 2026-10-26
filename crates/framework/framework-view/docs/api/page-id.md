# PageId

Checked compiled catalog identity. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_framework_view::{PageId, ViewError};
fn main() -> Result<(), ViewError> {
    let id = PageId::try_from("profile-detail".to_owned())?;
    assert_eq!(id.as_str(), "profile-detail");
    let text: String = id.into();
    assert_eq!(text, "profile-detail");
    Ok(())
}
```

## Construction and access

Use an identity to select a compiled page, never a source/module path.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `TryFrom<String>::try_from` | Owned catalog name | Accepts 1–64 ASCII bytes, letters/digits/hyphen only; returns `InvalidSnapshot` otherwise. |
| `as_str(&self)` | None | Borrows the exact checked name. |
| `From<PageId> for String` | Consumed identity | Transfers the owned string. |

**Returns** Construction yields `Result<PageId, ViewError>`; access yields `&str` and conversion
yields `String`. Serde uses the checked string conversion on decode. The type is Clone, Debug,
PartialEq/Eq, Serialize/Deserialize. Identity uniqueness is a catalog/startup check, not a
property of an individual `PageId`.

Related: [PageRoute](page-route.md), [PageSpec](page-spec.md).
