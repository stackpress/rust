# BasePath

Trusted application mount prefix. Source: [policy.rs](../../src/policy.rs).

```rust
use stackpress_framework_view::{BasePath, ViewError};
fn main() -> Result<(), ViewError> {
    let base = BasePath::new("/demo/v1")?;
    assert_eq!(base.join("/ui/profiles")?.as_str(), "/demo/v1/ui/profiles");
    assert_eq!(base.strip("/demo/v1/ui/profiles"), Some("/ui/profiles"));
    assert_eq!(base.strip("/demo/v10/ui/profiles"), None);
    Ok(())
}
```

## Construction and operations

Root is the empty string: `new("")`, `new("/")`, and `Default` agree.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `new` | `value: impl Into<String>` | At most 1,024 bytes; slash-prefixed nonempty segments, no trailing slash. |
| `as_str(&self)` | None | Borrows the canonical prefix. |
| `join(&self, suffix)` | `suffix: &str` | Checks suffix as a `PublicLocation`, prepends base, then checks combined length/text. |
| `strip(&self, location)` | Borrowed `&str` | Exact prefix removal only if remainder starts with `/` but not `//`. |
| `TryFrom<String>` / `From<BasePath> for String` | Owned text/mount | Checked construction / consumed canonical text. |

**Returns** `new` and `join` yield Results (`InvalidPolicy` for a malformed mount;
location failures from `join` are `InvalidSnapshot`). `strip` yields `Option<&str>` borrowing
the input; it is a prefix helper, not full location validation. A bare `/demo` does not strip
to an empty route, while `/demo/` strips to `/`. Access/conversion yield `&str`/`String`.

Non-root segments allow only ASCII letters/digits, `-`, `_`, `.`, `~`; empty or exact dot/dot-dot
segments are invalid. Query/fragment/percent escapes are not mount syntax. The type is
Default, Clone, Debug, PartialEq/Eq, Serialize/Deserialize with checked decoding.

Related: [PublicLocation](public-location.md), [routing context](leptos-routing.md).
