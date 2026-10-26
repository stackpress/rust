# Stackpress UI

`stackpress-framework-ui` supplies shared Leptos controls, layouts, scoped state, translation,
routing adapters, and buffered native rendering. It consumes the public contracts owned by
[Stackpress View](../../framework-view/docs/README.md).

These pages describe this checkout: Rust 1.96, edition 2024, unpublished local path dependencies.
Direct imports use `stackpress_framework_ui`; the optional facade is `stackpress::framework_ui`.

## Start here

Use [Getting started](getting-started.md) to render a complete HTML document in process.
Then read [Concepts](concepts.md) for mode selection, ownership, Head, and client roots.

## Choose a task

The [API reference](api/README.md) lists every framework-owned public surface.
For integration, start with the result you need:

| I want to… | Read |
| --- | --- |
| Register a native GET page | [View's page guide](../../framework-view/docs/guides/add-a-page.md) |
| Reuse server data in a browser | [Hydrate a page](guides/hydrate.md) |
| Package shared UI with local operations | [Package with Tauri](guides/tauri.md) |
| Reject old or invalid bootstrap data | [View's recovery guide](../../framework-view/docs/guides/contract-recovery.md) |
| Compose controls and root-scoped state | [Compose a UI root](guides/compose-a-root.md) |
| Translate text and counted phrases | [Translate and pluralize](guides/translate.md) |

## Select one render mode

Default features are empty. Build the targets separately; feature unification makes combined
modes invalid. Do not use `--all-features`.

| Direct feature | Target and purpose | Facade feature |
| --- | --- | --- |
| None | Shared APIs without choosing a rendering mode; no native renderer. | `framework-ui` |
| `ssr` | Native buffered SSR; enables Leptos/meta/router SSR and View's `server` extension. | `framework-ui-ssr` |
| `hydrate` | Browser WASM adopting matching server markup. | `framework-ui-hydrate` |
| `csr` | Browser/WebView WASM creating client markup, including packaged Tauri UI. | `framework-ui-csr` |

For `view!` and `#[component]`, add direct `leptos = "=0.8.22"` with defaults off and the
same chosen mode: macro expansion requires the crate name even with re-export imports.

Any pair of modes is a compile error; `ssr` on wasm32 is also a compile error.
Hydrate/CSR have no native host dependency through this crate, but still require real browser
entrypoints for DOM work. A native `cargo check` alone does not verify those entrypoints.

The [Profile UI README](../../../../examples/profile-ui/README.md) is the canonical integration
and build/proof entrypoint. The framework does not supply a universal operation client,
Tauri host, asset server, persistence, or general asynchronous component SSR.

Source authority: [exports](../src/lib.rs), [rendering](../src/render.rs),
[components](../src/components/mod.rs), [state](../src/state.rs), [language](../src/language.rs),
[routing](../src/routing.rs), and [integration tests](../tests/).
Markdown examples are checked separately from Rustdoc doctests.
