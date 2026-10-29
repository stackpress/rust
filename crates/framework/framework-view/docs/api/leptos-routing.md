# Routing context and CatalogPath

Root-local mount metadata and Leptos matching over View-owned definitions. Source: [routing.rs](../../src/leptos/routing.rs).

```rust
use stackpress_leptos::{leptos::prelude::*, leptos_router::PossibleRouteMatch};
use stackpress_framework_view::leptos::{routing::{
        BasePath, CatalogPath, PageRoute, RouteSegment, provide_route_context, use_route_context,
    }};
fn main() -> Result<(), stackpress_framework_view::ViewError> {
    let route = PageRoute::new(
        "detail",
        &[RouteSegment::Static("ui"), RouteSegment::Param("id")],
    )?;
    let path = CatalogPath::new(route);
    assert!(!path.optional());
    assert!(path.test("/ui/a%2Fb").is_some());
    let owner = Owner::new();
    owner.with(|| {
        provide_route_context(BasePath::new("/demo").expect("fixed mount"));
        assert_eq!(use_route_context().base.as_str(), "/demo");
    });
    owner.cleanup();
    Ok(())
}
```

## Context

`RouteContext { pub base: BasePath }` is Clone, Debug, Default, PartialEq/Eq. Default means root.

| Function | Parameters | Behavior |
| --- | --- | --- |
| `provide_route_context` | `base: BasePath` | Provides checked mount metadata in the current Owner. |
| `use_route_context` | None | Reads nearest context or returns explicit root default. |

**Returns** Providing yields unit; reading yields `RouteContext`. This is metadata only: it
does not configure `Router` base/history or validate the current location. Native rendering
provides the asset mount automatically; the browser bootstrap must supply the same mount.

## CatalogPath

`CatalogPath::new(route: PageRoute)` owns a compiled row and returns Self. The Clone/Debug
wrapper implements `leptos_router::PossibleRouteMatch` using native Leptos segment descriptors.

| Trait operation | Parameters | Behavior |
| --- | --- | --- |
| `optional(&self)` | None | Always false; required segments only. |
| `test(&self, path)` | `&str` | Delegates literal/parameter matching to Leptos, retaining encoded captures. |
| `generate_path(&self, path)` | `&mut Vec<PathSegment>` | Appends the compiled literal/parameter descriptors. |

**Returns** `test` yields `Option<PartialPathMatch<'_>>`; it may be partial, so a direct caller
can inspect completeness. `generate_path` yields unit. Pass the descriptor to a Leptos `Route`;
the surrounding Router owns client route selection. Native Stackpress dispatch stays all-matching.

This module re-exports View's `BasePath`, `PageRoute`, `RouteCatalog`, and `RouteSegment`.
Their construction and URL policy live in [View's reference](README.md).

Related: [share routes](../guides/share-routes.md), [Profile UI](../../../../../examples/profile-ui/README.md).
