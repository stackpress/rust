# One component tree, explicit execution hosts

A server request, a browser tab, and a packaged WebView have different lifetimes. Share the
component source and public data while letting each host own loading, transport, and disposal.
The [quick start](../../../leptos/docs/getting-started.md) demonstrates the native render lifetime.

## Native loading happens before component evaluation

The native [page loader](api/page-loader.md) borrows the live
request/response/invocation and returns owned public data. `LeptosRenderer` evaluates a
synchronous component factory in a fresh request-local Owner. It does not retain a host
invocation in a Resource or carry a live Owner across the final await.

Call `initialize_ssr` during startup inside an active Tokio runtime. Rendering and
`PreparedPage::finish` also require Tokio runtime context. The helper is idempotent;
initializing a dependency executor by another route does not mark this boundary ready.
Missing startup/runtime returns `ViewError::ExecutorUnavailable`.

## Head must own its SSR values

Native metadata assembly happens after render signals are disposed. Resolve a signal during
component evaluation and pass its owned string to `Title`, for example `text=title.get()`.
Do the same for Head getters and formatters: none may read disposed reactive state later.

Deferred reactive SSR Head is outside the current contract and remains S5-F6. A panic during
owned Head assembly becomes a safe `ViewError::Failure`, with a native diagnostic cause and
no retry. A panic in trusted synchronous component evaluation propagates after cleanup;
it is not converted to a recoverable page error by that Head catch boundary.

`leptos_meta` owns metadata in each render/root. The renderer provides its context, canonical
link, optional stylesheet, and title escaping. `LayoutHead` is a visible HTML header, separate
from document Head. Do not insert caller-controlled raw Head HTML.

## Cleanup precedes the metadata await

The renderer disposes its Owner on success, component Result error, and ordinary unwind,
then restores the caller's previous thread-local Owner or clears the render context. Discarding
the prepared future leaves no live request render signals. Independent concurrent requests
use independent roots; there is no global render lock.

This is not a sandbox for arbitrary component code, panicking cleanup callbacks/destructors,
panic-abort builds, or unbounded CPU/allocation. Output byte ceilings apply to the staged seed
and complete document. The [cleanup tests](../tests/leptos_cleanup.rs) cover the supported exits.

## Client state belongs to one root

Create `ThemeState`, `Notifications`, and `LanguageContext` inside the owning Leptos root,
then provide them to descendants. Values start explicitly from the public seed. No cookie,
browser preference, fetch, timer, or persistence is inferred.

Mutations are synchronous actions in that root's UI execution context. Marshal host/async
completions back to that root before publishing them. Signal handle types being Send/Sync do
not make a single root a concurrent cross-thread queue; arbitrary simultaneous writers to
one signal root are outside the contract. Distinct roots remain independent.

## Hydrate and CSR start differently

Hydration admits the bounded matching snapshot and assets, then adopts the server's initial
tree without an initial refetch. CSR creates explicit initial client state and mounts new
markup. Packaged Tauri CSR then calls a permitted local operation adapter; the website uses
HTTP. Neither mode silently falls back to the other's host.

One compiled route catalog feeds native page patterns and `CatalogPath`. A checked
`RouteContext` supplies mount metadata, while the Leptos `Router` still needs its own matching
base/history configuration. Authorization stays at each host access surface.

You now know which values may cross a render await and which state must stay in a root.
Use [hydration](guides/leptos-hydrate.md), [root composition](../../../leptos/docs/guides/compose-a-root.md), or
[Tauri packaging](guides/leptos-tauri.md) for the next integration step.
