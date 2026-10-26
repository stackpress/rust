# PageLimits and byte ceilings

Lowerable output policy. Source: [policy.rs](../../src/policy.rs) and [constants](../../src/lib.rs).

```rust
use stackpress_framework_view::{MAX_HTML_BYTES, MAX_SNAPSHOT_BYTES, PageLimits, ViewError};
fn main() -> Result<(), ViewError> {
    assert_eq!(MAX_SNAPSHOT_BYTES, 1024 * 1024);
    assert_eq!(MAX_HTML_BYTES, 2 * 1024 * 1024);
    let limits = PageLimits {
        snapshot_bytes: 4096,
        html_bytes: 16384,
    };
    limits.validate()?;
    Ok(())
}
```

## Fields and validation

All measurements are UTF-8 bytes, not characters. Limits are inclusive.

| Public field/constant | Type | Default/maximum |
| --- | --- | --- |
| `snapshot_bytes` | `usize` | `MAX_SNAPSHOT_BYTES` = 1,048,576, before script escaping. |
| `html_bytes` | `usize` | `MAX_HTML_BYTES` = 2,097,152, after complete document assembly. |

`Default::default()` creates the standard ceilings. `validate(self)` takes no additional
parameters and rejects either value above its maximum. Zero is valid and deliberately denies
nonempty output; the two limits have no required ordering.

**Returns** `validate` yields `Result<(), ViewError>`, with `InvalidPolicy` for increased limits.
The type is Copy/Clone, Debug, PartialEq/Eq. Decoder encoded-input allowance is six times
`snapshot_bytes`; it still checks reserialized logical size. These are output bounds, not
limits on arbitrary trusted component CPU or intermediate allocations.

Related: [PageSnapshot](page-snapshot.md), [RenderedPage](rendered-page.md), [PageContract](page-contract.md).
