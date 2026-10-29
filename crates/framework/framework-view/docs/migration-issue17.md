# Leptos extraction migration

G41 removes `stackpress-framework-ui` outright, without a compatibility shim. Add
`stackpress-leptos` from `crates/leptos` for generic components, state, language, resources,
rich content, format/editing and upstream Leptos re-exports. Facade callers use
`stackpress::leptos` with `leptos`, `leptos-ssr`, `leptos-hydrate`, `leptos-csr`,
`leptos-async`, `leptos-stream`, `leptos-rich` or `leptos-format` as applicable.

View integration is explicit and optional. Defaults keep page contracts free of Leptos/server.
Enable `leptos` for `leptos::routing`/presentation, `leptos-async` for page replay,
`leptos-ssr` for synchronous and buffered-async page adapters, `leptos-hydrate` or
`leptos-csr` for browser integration, and `leptos-stream` for opt-in page streaming.
Facade aliases prefix those names with `framework-view-`. Build render modes separately.

| Former UI surface | New owner |
| --- | --- |
| components/state/language/resources/rich/format and upstream re-exports | `stackpress_leptos`, same nested paths |
| render::LeptosRenderer / PreparedPage | `stackpress_framework_view::leptos::render` |
| async_render::AsyncLeptosRenderer | `stackpress_framework_view::leptos::async_render` |
| stream page renderer/body/status script | `stackpress_framework_view::leptos::stream` |
| routing context / CatalogPath | `stackpress_framework_view::leptos::routing` |
| routing BasePath / PageRoute / RouteCatalog / RouteSegment | existing view root exports |
| replay with ViewError/page limits | `stackpress_framework_view::leptos::replay` |
| independent rendering/replay | library generic renderers and `ReplayError`, without page types |

The view startup wrapper retains `ViewError`; direct library initialization returns
`RenderError`. Image display errors now retain a library-local `LocationError` rather than
`ViewError`. Image admission remains strict; broader `safe_href` is unchanged. G41 keeps
location validation and escaping local to each library, guarded by one differential test table.
No additional safety crate or foundation dependency is introduced.

Page, seed, asset, replay and stream marker wires retain their formats. Light/Dark/unset,
one-shot pre-paint sampling and readable no-JS Light remain the issue-15 contracts.
Persistence and authorization stay in view/host adapters; UI state provides no authority.
Existing preferences/flash registration functions remain explicit. The base `plugin::register`
is reserved for issue 14 and is not introduced here. Importing a library registers nothing.

See [page renderer](api/leptos-renderer.md), [routing](api/leptos-routing.md),
[hydration](guides/leptos-hydrate.md), [independent library](../../../leptos/docs/README.md),
and [presentation host](presentation-host.md).

## Choose the replacement dependency

A component-only crate replaces its old UI dependency with `stackpress-leptos`, defaults off,
and keeps matching `ssr`, `hydrate` or `csr` selections. A page host also enables the appropriate
view `leptos-*` adapter. Generated admin templates now emit generic library imports; regenerate
through the writer and check idempotence rather than editing managed output.

The old `framework-ui*` facade features and `framework_ui` alias are removed.
`leptos-ssr` alone exposes generic rendering without view; use `framework-view-leptos-ssr`
for the application page renderer. Default page contracts and preference/flash selections
do not pull Leptos into a consumer.

See [a complete native page](integration-leptos.md) and [the generic independent fixture](../../../../examples/leptos-independent/README.md).
The fixture has its own workspace, two shared dependencies and runtime-only browser graph
audits. Its shell/bootstrap are consumer-owned; no application page types are copied into it.
