# Resource replay and cancellation

Use `replay` and `resources` with feature `async` (also included by native `ssr`). Replay carries
encoded strings for upstream Resource codecs, without page IDs, HTTP status or application seeds.

```rust
use stackpress_leptos::replay::{ReplayError, ResourceReplay};

fn example() -> Result<(), ReplayError> {
    let values = ResourceReplay::new(vec![(0, r#""Hello""#.into())], 1024)?;
    let text = values.script_json()?;
    assert_eq!(ResourceReplay::decode(&text, 1024)?, values);
    Ok(())
}
```

Each entry is `(resource_id, encoded_value)`. Here the encoded value is a JSON string, including
its quotes; a raw user string is not a substitute for its Resource codec output.

## Operations

| Method/constant | Contract |
| --- | --- |
| `new(Vec<(usize,String)>, byte_limit)` | Validates logical serialized size, count, IDs and duplicates |
| `decode(&str, byte_limit)` | Bounds expanded text before JSON decoding, then repeats admission |
| `script_json(&self)` | Serializes and escapes script terminators/separators for inert JSON insertion |
| `entries(&self)` | Borrows checked pairs; never evaluates code |
| `MAX_RESOURCES` | 64 entries |
| `MAX_RESOURCE_ID` | IDs at most 1024; duplicates are rejected |

**Returns** Constructors return checked `ResourceReplay` or `ReplayError`; `script_json` returns
an owned script-safe string. The selected logical ceiling cannot exceed 1 MiB. Failures distinguish
invalid policy, invalid snapshot, snapshot limit and native cause; `Display` remains fixed safe text.

`resources::cancel_on_cleanup(loader)` belongs inside a Resource fetcher. It wraps a yielding
future, aborts it cooperatively on Owner cleanup, and returns its value or `ResourceFailure`.
The public kind is `Cancelled` or `Panicked`; a panic's native cause is omitted from serialization.
It does not cancel already completed work or roll back side effects. Source replacement requires
the caller's current-result policy; see [async ownership](../guides/async-work.md).
