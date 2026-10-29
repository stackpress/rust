# PageContract

Trusted version, initial display settings, and limits. Source: [policy.rs](../../src/policy.rs).

```rust
use stackpress_framework_view::{PageContract, ViewError};
fn main() -> Result<(), ViewError> {
    let mut contract = PageContract::new(1, "demo-1")?;
    contract.locale = "fr-CA".into();
    contract.theme = Some("dark".into());
    contract.limits.snapshot_bytes = 4096;
    contract.validate()?;
    Ok(())
}
```

## Construction and fields

`new(revision: u32, build: impl Into<String>)` validates the supplied version and installs
`en`, `light`, and default `PageLimits`.

| Public field | Type | Accepted values |
| --- | --- | --- |
| `revision` | `u32` | Nonzero app wire revision. |
| `build` | `String` | 1–128 ASCII letters/digits, hyphen, underscore, or dot. |
| `locale` | `String` | 1–35 bytes; nonempty ASCII alphanumeric parts separated by hyphens. |
| `theme` | `Option<String>` | Some(`light`/`dark`) is explicit; None uses readable Light SSR and an optional one-shot head asset. |
| `limits` | `PageLimits` | No increase above framework ceilings. |

**Returns** `new` yields `Result<PageContract, ViewError>`. `validate(&self)` takes no additional
parameters and yields `Result<(), ViewError>`; malformed settings return `InvalidPolicy`.
Fields are public, so call validation after changing them. Native page registration and the
Leptos renderer validate again. The type is Clone, Debug, PartialEq/Eq.

Page locale syntax is narrower than `Language`'s standalone key syntax. Use a value accepted
by both when putting language data into a page.

Related: [PageAssets](page-assets.md), [Language](../../../../leptos/docs/api/language.md).
