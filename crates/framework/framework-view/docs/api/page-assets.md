# PageAssets

Trusted bootstrap metadata; not an asset server or integrity manifest. Source: [policy.rs](../../src/policy.rs).

```rust
use stackpress_framework_view::{BasePath, PageAssets, PageContract, PublicLocation, ViewError};
fn main() -> Result<(), ViewError> {
    let assets = PageAssets {
        build: "demo-1".into(),
        base: BasePath::new("/demo")?,
        module: PublicLocation::new("/assets/start.js")?,
        stylesheet: None,
    };
    assets.validate(&PageContract::new(1, "demo-1")?)?;
    assert_eq!(
        PageAssets::decode(&assets.script_json()?, "demo-1")?,
        assets
    );
    Ok(())
}
```

## Fields

All fields are public; serde rejects unknown fields and uses checked location/mount decoding.

| Field | Type | Meaning |
| --- | --- | --- |
| `build` | `String` | Exact agreement with the page contract build. |
| `base` | `BasePath` | Trusted mount applied explicitly to resource paths. |
| `module` | `PublicLocation` | App-relative module pathname; no query. |
| `stylesheet` | `Option<PublicLocation>` | Optional app-relative stylesheet pathname; no query. |

## Operations

These methods check metadata, not the bytes served at a URL.

| Operation | Parameters | Checks |
| --- | --- | --- |
| `validate(&self, contract)` | `&PageContract` | Contract validity, exact build, no resource query, checked mounted paths. |
| `script_json(&self)` | None | Valid assets under a revision-1 default contract; serialized metadata ≤16 KiB, then standard script escaping. |
| `decode(json, build)` | `&str`, expected `&str` | Input ≤16 KiB, strict decoding and asset validation against expected build. |

**Returns** Validation yields `Result<(), ViewError>`; encoding yields `Result<String, ViewError>`;
decoding yields `Result<PageAssets, ViewError>`. Drift is `ContractMismatch`; malformed JSON,
oversized metadata, or query-bearing resources are `InvalidPolicy`; unsafe combined location
can yield `InvalidSnapshot`. The decode ceiling applies to encoded input too. The type is
Clone, Debug, PartialEq/Eq, Serialize/Deserialize.

Source fingerprints, file hashes/MIME, immutable serving, and missing-file recovery belong to
the [example integration](../../../../../examples/profile-ui/README.md#asset-and-version-agreement).

Related: [PageContract](page-contract.md), [BasePath](base-path.md).
