# Handle contract mismatch and recovery

Old HTML can outlive its matching JS bundle. Refuse the bootstrap before mounting when its
revision, build, or application data is incompatible; keep useful server HTML visible while
the user chooses how to recover.

```rust
use stackpress_framework_view::{PageAssets, PageSnapshot, ViewError};

fn admit(page_text: &str, asset_text: &str) -> Result<PageSnapshot<String>, ViewError> {
    let assets = PageAssets::decode(asset_text, "demo-1")?;
    let page = PageSnapshot::<String>::decode(page_text, 1, "demo-1")?;
    if assets.base.strip(&page.location).is_none() || page.page.as_str() != "welcome" {
        return Err(ViewError::InvalidSnapshot);
    }
    Ok(page)
}
```

Feed `textContent` from the inert `stackpress-page` and `stackpress-assets` scripts to this
function. The expected build/revision are compiled constants, not values accepted from the seed.
This minimal DTO has no extra semantics; real DTOs need their own checks and canonical-location
comparison. Use `decode_with` when the host selected lower snapshot limits.

## Classify the failure

The framework returns errors; it does not reload or retry. Choose recovery in your bootstrap.

| Failure | Application action |
| --- | --- |
| `ContractMismatch` | Show a refresh-required message and an explicit reload control. |
| `InvalidSnapshot`, `InvalidPolicy`, size errors | Refuse mounting and report a safe bootstrap error. |
| JS/WASM acquisition fails before Rust runs | Handle it in the external bootstrap and preserve server HTML when possible. |
| Valid host rejection from a later action | Show the public host error; keep it distinct from transport/protocol failure. |

The [example bootstrap](../../../../../examples/profile-ui/bootstrap.js) catches startup failure,
adds **Reload page**, and avoids automatic reload loops. Its
[README](../../../../../examples/profile-ui/README.md#asset-and-version-agreement) explains
build manifests and port-free recovery proofs. Those behaviors are example-owned.

You now have a place to stop incompatible data before hydration. Continue with
[browser hydration](../../../framework-ui/docs/guides/hydrate.md) for the mount sequence.
