# PageLoad

Owned loader outcome, separate from errors and application status. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_framework_view::PageLoad;
fn visible(load: PageLoad<String>) -> Option<String> {
    match load {
        PageLoad::Ready(data) => Some(data),
        PageLoad::Abort => None,
    }
}
```

## Variants

The loader returns this inside the foundation `Result`.

| Variant | Payload | Native behavior |
| --- | --- | --- |
| `Ready(T)` | Owned public page data | Render if the response is still Open. |
| `Abort` | None | Preserve current response and return `Flow::Abort`, skipping rendering. |

**Returns** The enum does not carry an error. Use a loader `Err` for failure; use `PublicStatus`
for an explicitly public page status. `Abort` does not mean rollback, and `Ready` after a
loader stops/sends the response does not cause rendering. The type implements Debug.

Related: [PageLoader](page-loader.md), [lifecycle table](../concepts.md#lifecycle-remains-the-servers).
