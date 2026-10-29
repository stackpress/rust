# Stackpress Leptos

`stackpress-leptos` supplies reusable Leptos components, root-scoped state, translation,
formatting, sanitized rich content and explicit native rendering. It has no Stackpress view,
server, preference, schema, SQL, admin or facade dependency. Nothing is published.

Start with [Getting started](getting-started.md), then [Concepts](concepts.md) and the
[API index](api/README.md). Direct imports use `stackpress_leptos`; the optional facade is
`stackpress::leptos`.

| Feature | Capability | Facade |
| --- | --- | --- |
| none | Components, language, state, checked locations and escaping | `leptos` |
| `ssr` | Native synchronous and buffered async rendering, explicit executor startup | `leptos-ssr` |
| `hydrate` | Browser adoption of matching SSR markup | `leptos-hydrate` |
| `csr` | Browser/WebView mounting | `leptos-csr` |
| `async` | Cooperative resources, replay and suggestions | `leptos-async` |
| `stream` | Opt-in supervised native streaming; implies SSR | `leptos-stream` |
| `rich` / `format` | Closed rich-content policy / deterministic display and editing | `leptos-rich` / `leptos-format` |

Defaults are empty. SSR, hydrate and CSR are mutually exclusive; build them separately,
never with `--all-features`. Leptos macros require a direct pinned upstream `leptos`
dependency with the same selected mode. Browser execution requires actual WASM artifacts.

For Stackpress page contracts, assets, mounted routing, persistence and host registration,
use [view's explicit integration](../../framework/framework-view/docs/migration-issue17.md).
The [Profile example](../../../examples/profile-ui/README.md) demonstrates that application
integration; it is not a dependency of this library.

[Compose a root](guides/compose-a-root.md) and [translate phrases](guides/translate.md) without
adopting an application protocol. CSS constants and [vendored Lucide assets](../third-party/lucide/NOTICE)
retain their existing bytes and licenses. Source authority is [lib.rs](../src/lib.rs).
