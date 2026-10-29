# Share routes between native and browser code

A detail link should mean the same thing before and after hydration. Define its path once,
then derive both native registration and client matching from that compiled description.

```rust
use stackpress_framework_view::{BasePath, PageRoute, RouteCatalog, RouteSegment, ViewError};

fn main() -> Result<(), ViewError> {
    let detail = PageRoute::new(
        "detail",
        &[
            RouteSegment::Static("ui"),
            RouteSegment::Static("profiles"),
            RouteSegment::Param("id"),
        ],
    )?;
    let catalog = RouteCatalog::new(BasePath::new("/demo")?, vec![detail])?;
    assert_eq!(catalog.pattern("detail")?.as_str(), "/demo/ui/profiles/:id");
    let route = catalog.get("detail").ok_or(ViewError::InvalidPolicy)?;
    let location = route.location(catalog.base(), &[("id", "a/b")])?;
    assert_eq!(location.as_str(), "/demo/ui/profiles/a%2Fb");
    assert_eq!(
        route.captures(catalog.base(), &location),
        Some(vec![("id", "a%2Fb")])
    );
    Ok(())
}
```

The parameter is encoded as one component, so its slash does not add a route segment.
Captures stay encoded; decode once using the application's strict codec. The catalog rejects
duplicate IDs and identical pattern strings, but does not prove all routes are unambiguous.

Pass `catalog.pattern(id)` to `PageSpec::get` and a cloned row to UI's
[`CatalogPath`](../api/leptos-routing.md). Supply the same `BasePath` to
assets, the route context, the Leptos `Router` base, and operation URLs. Providing route context
alone does not configure a `Router`.

Queries are app policy. The [canonical example](../../../../../examples/profile-ui/README.md)
keeps search and return-query encoding in shared source rather than teaching the framework
about Profile. You can now derive mounted patterns and links from one definition.
