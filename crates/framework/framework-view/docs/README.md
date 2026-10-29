# Stackpress View

`stackpress-framework-view` carries explicitly public page data between a native loader,
a renderer, and a browser. It owns checked identities, locations, versions, routes, and
byte limits. Optional modules supply settings, flash notices, access hints and host adapters.
Leptos rendering belongs to [Stackpress UI](../../../leptos/docs/README.md).

These docs describe this checkout: Rust 1.96, edition 2024, local path dependencies,
and `publish = false`. Import `stackpress_framework_view` directly, or use
`stackpress::framework_view` with facade feature `framework-view`.

For the removed preferences crate and theme API changes, read the
[presentation migration note](migration-issue15.md).

## Start here

The [quick start](getting-started.md) creates and decodes a script-safe snapshot without
a runtime or listener. Read [Concepts](concepts.md) next to understand what a page may disclose.

## Choose a task

Follow a guide for an integration task; use the [API index](api/README.md) for exact contracts.

| I want to… | Read |
| --- | --- |
| Select preference/flash host contributions | [Presentation hosts](presentation-host.md) |
| Migrate settings imports or saved themes | [Presentation migration](migration-issue15.md) |
| Register a native GET page and loader | [Add a page](guides/add-a-page.md) |
| Use one route definition on both targets | [Share routes](guides/share-routes.md) |
| Reject mismatched or malformed bootstrap data | [Handle contract mismatch](guides/contract-recovery.md) |
| Hydrate or package shared UI | [UI guides](../../../leptos/docs/README.md#choose-a-task) |

## Features and integration

Default features are empty. Public page contracts, `access` and `theme` are browser-safe;
Leptos rendering and persistence are not acquired by import or feature selection.

| Feature | Adds |
| --- | --- |
| `cookies` | Checked canonical origin/mount and cookie codecs; no server. |
| `preferences` | Locale routing, Light/Dark-or-unset settings, fixed actions and root save state. |
| `flash` | Bounded public notices, cookie codec and per-root deduplication. |
| `server` | Native page registration: PageSpec, PageLoader and ServerBuilderViewExt. |
| `web-preferences` / `web-flash` | Independently selectable native registration functions and typed services. |
| `native-file` | Fixed revision-1 Unix file adapter, independent of server registration. |

The facade forwards `preferences`, `flash`, `native-file`, `server`, `web-preferences`
and `web-flash` as `framework-view-*` features; `cookies` is selected by its dependent modules.
UI's `ssr` selects the native page extension. Server features fail explicitly on wasm32;
pure preference/flash contracts remain usable there. Native files are Unix-only.
See [host composition](presentation-host.md) for configuration, admission and asset ownership.

The [Profile UI example](../../../../examples/profile-ui/README.md) is the canonical integration:
one generated capability feeds SSR, hydrated HTTP actions, and local Tauri actions.
The example owns its DTO validation, query codec, transport, asset serving, and recovery UI.

Source authority: [exports](../src/lib.rs), [policies](../src/policy.rs),
[routes](../src/routes.rs), [native extension](../src/server.rs), and [tests](../tests/).
Markdown examples are compile-checked separately from Rustdoc doctests.

Optional Leptos consumers: [issue-17 migration and features](migration-issue17.md).
