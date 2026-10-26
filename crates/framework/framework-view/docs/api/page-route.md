# PageRoute, RouteSegment, and component encoding

Compiled path descriptions shared by host and client adapters. Source: [routes.rs](../../src/routes.rs).

```rust
use stackpress_framework_view::{
    BasePath, PageRoute, RouteSegment, ViewError, encode_uri_component,
};
fn main() -> Result<(), ViewError> {
    let route = PageRoute::new(
        "detail",
        &[RouteSegment::Static("ui"), RouteSegment::Param("id")],
    )?;
    assert_eq!(route.id().as_str(), "detail");
    assert_eq!(route.segments().len(), 2);
    assert_eq!(route.pattern(), "/ui/:id");
    let base = BasePath::default();
    let location = route.location(&base, &[("id", "a/b")])?;
    assert_eq!(location.as_str(), "/ui/a%2Fb");
    assert_eq!(
        route.captures(&base, &location),
        Some(vec![("id", "a%2Fb")])
    );
    assert_eq!(encode_uri_component("0 & false"), "0%20%26%20false");
    Ok(())
}
```

## Grammar and construction

`RouteSegment::Static(&'static str)` is a literal segment; `Param(&'static str)` is one required
named segment. Segments are Copy/Clone, Debug, PartialEq/Eq. `PageRoute` is Clone and Debug.

`PageRoute::new(id: &str, segments: &'static [RouteSegment])` takes a checked PageId and 1–32
segments. Each name is 1–64 ASCII letters/digits, hyphen, underscore, or dot, excluding exact
`.` and `..`. Duplicate parameter names are invalid. Static slices can come from promoted
literal arrays as shown above. Optional parameters and wildcards are not catalog grammar.

**Returns** `Result<PageRoute, ViewError>`; bad identity is `InvalidSnapshot`, bad grammar is
`InvalidPolicy`. No native handler is registered by constructing a route.

## Operations

The same row supplies patterns, URLs, and data-only capture checks.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `id(&self)` | None | Borrows `&PageId`. |
| `segments(&self)` | None | Returns the compiled `&'static [RouteSegment]`. |
| `pattern(&self)` | None | Builds app-relative Express-style `String`, such as `/ui/:id`. |
| `location(&self, base, params)` | `&BasePath`, `&[(&str, &str)]` | Requires exactly one named value per parameter, with no extras/duplicates. |
| `captures(&self, base, location)` | `&BasePath`, `&PublicLocation` | Strips mount/query, matches exact segment count and literal spelling, returns encoded borrowed captures. |
| `encode_uri_component(value)` | `&str` | Encodes UTF-8 bytes except ASCII letters/digits and `-_.~`; uppercase hex. |

**Returns** Accessors/pattern/encoding return the types above without errors. `location` returns
`Result<PublicLocation, ViewError>`; parameter values must be 1–1,024 bytes and cannot be exact
`.` or `..`. It encodes each value and checks the joined path. `captures` returns
`Option<Vec<(&'static str, &str)>>`, borrowing values from the location; a mismatch returns
None. Neither helper decodes application parameters or validates a query policy.

Native dispatch retains ordinary all-matching semantics; these helpers do not implement a
second native router. Literal pattern validity is also checked when registering `PageSpec`.

Related: [RouteCatalog](route-catalog.md), [routing guide](../guides/share-routes.md).
