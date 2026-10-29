# Stackpress Leptos

Build a component once, then render it natively, hydrate it, or mount it in a browser/WebView.
`stackpress-leptos` supplies components, root-scoped state, checked translation, optional
rich content/formatting and rendering kernels. Your application supplies initial values,
callbacks, document framing and transport. This checkout uses local paths and `publish = false`.

Start with [a first native render](getting-started.md). The [independent consumer](../../../examples/leptos-independent/README.md)
contains a complete shared root and finite browser proofs with its own workspace. It depends
on this library and pinned upstream Leptos, without a Stackpress application host.

## Choose a task

Use guides for an action and [API reference](api/README.md) for exact signatures and limits.

| I want to… | Read |
| --- | --- |
| Understand root lifetime and render ownership | [Concepts](concepts.md) |
| Compose labelled controls and local state | [Compose a root](guides/compose-a-root.md) |
| Supply language data and translated slots | [Translate](guides/translate.md) |
| Render eager or asynchronous native content | [Render native HTML](guides/render-native.md) |
| Adopt SSR nodes or mount a fresh client root | [Hydrate and mount](guides/hydrate-and-mount.md) |
| Cancel pending loaders and publish current suggestions | [Own async work](guides/async-work.md) |
| Stream through bounded owned chunks | [Stream HTML](guides/stream-html.md) |
| Display checked rich content or edit typed decimals | [Optional helpers](guides/optional-helpers.md) |
| Integrate page contracts, routes and hosts | [View integration](../../framework/framework-view/docs/integration-leptos.md) |

## Select one render mode

Defaults are empty. Choose modes per artifact; SSR, hydrate and CSR are mutually exclusive.
Leptos macros need a direct `leptos = "=0.8.22"` dependency with defaults off and the same mode.

| Feature | Capability | Facade feature |
| --- | --- | --- |
| none | Components, state, language, checked locations and escaping | `leptos` |
| `ssr` | Native sync and buffered async kernels; includes generic replay/resources | `leptos-ssr` |
| `hydrate` | Adopt matching SSR markup in WASM | `leptos-hydrate` |
| `csr` | Mount a new WASM component tree | `leptos-csr` |
| `async` | Cooperative resources, checked replay and async suggestions | `leptos-async` |
| `stream` | Native supervised streaming; implies `ssr` | `leptos-stream` |
| `rich` | Closed sanitizer and Markdown policy | `leptos-rich` |
| `format` | Explicit-locale decimal/date/time/phone display and editing | `leptos-format` |

Build hydrate and CSR separately. Do not use `--all-features`. Native startup requires an
active Tokio runtime and explicit `initialize_ssr`; importing a crate starts no runtime,
listener or registration. Generic APIs never acquire cookies, preferences or authority.

Source authority: [exports](../src/lib.rs), [ownership](../src/render.rs), and
[the independent proofs](../../../examples/leptos-independent/proof.py). Neutral CSS and
[vendored Lucide notices](../third-party/lucide/NOTICE) retain their licenses. The
[page migration](../../framework/framework-view/docs/migration-issue17.md) covers retired paths.
