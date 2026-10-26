# PageSnapshot

Strict public envelope and script-safe JSON boundary. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_framework_view::{PageId, PageLimits, PageSnapshot, PublicStatus, ViewError};
fn main() -> Result<(), ViewError> {
    let page = PageSnapshot {
        revision: 1,
        build: "demo-1".into(),
        page: PageId::try_from("welcome".to_owned())?,
        location: "/ui/welcome".into(),
        locale: "en".into(),
        theme: "light".into(),
        status: PublicStatus::ok(),
        data: "<public text>".to_owned(),
    };
    page.validate(1, "demo-1")?;
    let text = page.script_json()?;
    assert_eq!(
        PageSnapshot::<String>::decode(&text, 1, "demo-1")?.data,
        page.data
    );
    let limits = PageLimits {
        snapshot_bytes: 4096,
        html_bytes: 16384,
    };
    let bounded = page.script_json_with(limits)?;
    let decoded = PageSnapshot::<String>::decode_with(&bounded, 1, "demo-1", limits)?;
    assert_eq!(decoded.page.as_str(), "welcome");
    Ok(())
}
```

Contents: [Fields](#1-fields) · [Validation](#2-validation) · [Encoding](#3-encoding) · [Decoding](#4-decoding)

## 1. Fields

The envelope rejects unknown fields. Data validation remains the application's responsibility.

| Public field | Type | Meaning |
| --- | --- | --- |
| `revision` | `u32` | Nonzero app wire revision. |
| `build` | `String` | Matching asset build identity. |
| `page` | `PageId` | Checked compiled identity. |
| `location` | `String` | Public encoded pathname/query, checked on validation. |
| `locale`, `theme` | `String` each | Explicit initial settings under `PageContract` syntax. |
| `status` | `PublicStatus` | Public status/error; defaults to OK if absent on decode. |
| `data` | `T` | Explicit public projection only. |

The type implements Debug and conditional Serialize/Deserialize. It does not automatically
reject unknown fields inside `T`; use a strict DTO if your application needs that contract.

## 2. Validation

`validate(&self, revision: u32, build: &str)` compares exact expected revision/build first,
then validates public location and version/display metadata. It does not compare the location
to a route catalog, or the locale/theme to another contract.

**Returns** `Result<(), ViewError>`: `ContractMismatch` on version drift, `InvalidSnapshot` for
bad location, or `InvalidPolicy` for bad contract metadata. It does not measure `T` or JSON size.

## 3. Encoding

`script_json(&self)` requires `T: Serialize` and uses default limits.
`script_json_with(&self, limits: PageLimits)` accepts the lower application policy.
Both validate metadata and serialize the full envelope before checking logical bytes.

**Returns** `Result<String, ViewError>` containing inert script text. Escapes `<`, `>`, `&`,
U+2028, and U+2029 with JSON Unicode escapes, preserving exact roundtrip values. Invalid
policy/metadata is rejected, serialization failure retains a native cause in `Failure`, and
logical excess yields `SnapshotLimit`. Embed only as text of an inert JSON script, not as an
HTML attribute or executable JS expression. Final HTML needs its own limit.

## 4. Decoding

`decode(json: &str, revision: u32, build: &str)` and
`decode_with(json: &str, revision: u32, build: &str, limits: PageLimits)` require
`T: DeserializeOwned + Serialize`. The explicit policy must match the host's selected snapshot
budget. Input is script `textContent`; expected versions come from compiled client code.

**Returns** `Result<PageSnapshot<T>, ViewError>`. Invalid policy, malformed JSON, encoded or
logical excess, and metadata/version failures are rejected before returning data. Encoded input
may be at most six times the snapshot ceiling; reserialization must still fit the logical
ceiling. Malformed JSON/unknown envelope fields return `InvalidSnapshot`. Validate application
data, page membership, and canonical location before mounting.

Related: [limits](page-limits.md), [recovery guide](../guides/contract-recovery.md).
