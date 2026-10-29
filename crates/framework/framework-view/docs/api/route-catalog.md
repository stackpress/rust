# RouteCatalog

Checked inventory and one trusted mount. Source: [routes.rs](../../src/routes.rs).

```rust
use stackpress_framework_view::{BasePath, PageRoute, RouteCatalog, RouteSegment, ViewError};
fn main() -> Result<(), ViewError> {
    let catalog = RouteCatalog::new(
        BasePath::new("/demo")?,
        vec![PageRoute::new(
            "welcome",
            &[RouteSegment::Static("welcome")],
        )?],
    )?;
    assert_eq!(catalog.routes().count(), 1);
    assert_eq!(catalog.pattern("welcome")?.as_str(), "/demo/welcome");
    assert!(catalog.get("unknown").is_none());
    Ok(())
}
```

## Construction and lookup

The catalog preserves definition order and does not register handlers.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `new` | `base: BasePath`, `routes: Vec<PageRoute>` | Requires 1–128 rows; rejects duplicate page identities or identical pattern strings. |
| `base(&self)` | None | Borrows `&BasePath`. |
| `routes(&self)` | None | Ordered iterator of `&PageRoute`. |
| `get(&self, id)` | `&str` | Finds an exact known identity. |
| `pattern(&self, id)` | `&str` | Resolves identity and joins its native pattern to the mount. |

**Returns** `new` yields `Result<RouteCatalog, ViewError>` (`InvalidPolicy` on inventory errors).
`get` yields `Option<&PageRoute>`. `pattern` yields `Result<PublicLocation, ViewError>`;
unknown ID is `InvalidPolicy`, and overlong mounted text is `InvalidSnapshot`. Other returns
are shown above. The catalog is Clone and Debug.

Uniqueness compares exact pattern strings. It does not reject every semantically overlapping
pattern or choose native dispatch precedence; that remains normal server routing. Build a
Leptos descriptor from a known row with UI's `CatalogPath`.

Related: [PageRoute](page-route.md), [CatalogPath](leptos-routing.md).
