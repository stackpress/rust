# stackpress-framework-view

Define public page snapshots, route catalogs and bounded HTML contracts for Stackpress. Applications explicitly project public data; rendering, preference policies and host adapters are optional.

```rust
use stackpress_framework_view::PageId;
let page = PageId::try_from("profile".to_owned())?;
assert_eq!(page.as_str(), "profile");
# Ok::<(), stackpress_framework_view::ViewError>(())
```

The page ID is a checked catalog key. It is independent of a filesystem path or renderer.

Licence: [Stackpress License](LICENSE).
