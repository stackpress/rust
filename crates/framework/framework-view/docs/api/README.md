# Stackpress View API

All public items are exported from `stackpress_framework_view`; implementation modules are
private. Related enums/functions are documented with their owning surface. The final three
rows require native feature `server`; the other contracts are client-safe by default.

## Surfaces

Use these pages for inputs, defaults, returns, and failures. Guides remain task-oriented.

| Surface | Public coverage |
| --- | --- |
| [PageId](page-id.md) | PageId; checked string conversions |
| [PageLoad](page-load.md) | Ready(T), Abort |
| [PageSnapshot](page-snapshot.md) | fields; validate, script_json[_with], decode[_with] |
| [PageContract](page-contract.md) | fields; new, validate |
| [Limits](page-limits.md) | PageLimits; MAX_SNAPSHOT_BYTES, MAX_HTML_BYTES |
| [PublicLocation](public-location.md) | new, as_str, checked string conversions |
| [BasePath](base-path.md) | new, as_str, join, strip, checked string conversions |
| [PublicStatus](public-status.md) | ok/default, new, code, error |
| [PageAssets](page-assets.md) | fields; validate, script_json, decode |
| [PageRoute](page-route.md) | RouteSegment; new, id, segments, pattern, location, captures; encode_uri_component |
| [RouteCatalog](route-catalog.md) | new, base, routes, get, pattern |
| [Rendering boundary](page-renderer.md) | PageRenderer<T>; RenderFuture |
| [RenderedPage](rendered-page.md) | new, with_limits, into_html |
| [Errors](errors.md) | ViewError; from_source, Display/Error, equality |
| [PageLoader](page-loader.md) | Native named async loader blanket contract |
| [PageSpec](page-spec.md) | Native get and location |
| [ServerBuilderViewExt](server-builder-view-ext.md) | Native page registration |

## Example conventions

Rust blocks include their own imports and are independent programs or named consumer functions.
Native registration examples select `server`; renderer integration also selects UI `ssr`.
No example needs a socket listener. These Markdown fences are compile-checked independently
of Cargo's Rustdoc test discovery.

The facade path is `stackpress::framework_view` with `framework-view`, or
`framework-view-server` for the native extension. See [feature selection](../README.md#features-and-integration)
and the [UI API](../../../framework-ui/docs/api/README.md).
