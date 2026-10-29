# Stackpress View API

Page contracts are exported from `stackpress_framework_view`. Optional presentation
capabilities live in explicit public modules; their implementation helpers remain private.
The three native page rows require `server`; presentation host functions require their
separate host features. Core page/access/theme contracts are browser-safe by default.

## Surfaces

Use these pages for inputs, defaults, returns, and failures. Guides remain task-oriented.

| Surface | Public coverage |
| --- | --- |
| [Presentation modules](../migration-issue15.md) | preferences, flash, access, adapters and feature/import map |
| [Presentation host registration](../presentation-host.md) | typed Config/Admission/Service APIs, register functions and asset composition |
| [Initial theme](../migration-issue15.md) | theme asset identity and checked bootstrap_tag |
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
and the [UI API](../../../../leptos/docs/api/README.md).
