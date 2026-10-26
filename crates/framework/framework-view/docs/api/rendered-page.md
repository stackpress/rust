# RenderedPage

Complete buffered document with a final byte check. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_framework_view::{PageLimits, RenderedPage, ViewError};
fn main() -> Result<(), ViewError> {
    let page = RenderedPage::with_limits("<p>Welcome</p>".into(), PageLimits::default())?;
    assert_eq!(page.into_html(), "<p>Welcome</p>");
    Ok(())
}
```

## Operations

Construct only after Head, seed, and assets have been assembled.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `new` | `html: String` | Applies default complete-document ceiling. |
| `with_limits` | `html: String`, `limits: PageLimits` | Validates limits and checks `html.len()`. |
| `into_html(self)` | None | Consumes staged output and transfers its `String`. |

**Returns** Constructors yield `Result<RenderedPage, ViewError>`; invalid policy yields
`InvalidPolicy`, excessive document bytes yield `HtmlLimit`. `into_html` returns `String`.
The wrapper is Debug. It checks size only, not HTML syntax or escaping; those belong to the
renderer. Construction has no native output side effect.

Related: [PageLimits](page-limits.md), [PageRenderer](page-renderer.md).
