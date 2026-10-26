# Stackpress View

`stackpress-framework-view` carries explicitly public page data between a native loader,
a renderer, and a browser. It owns checked identities, locations, versions, routes, and
byte limits. Leptos rendering belongs to [Stackpress UI](../../framework-ui/docs/README.md).

These docs describe this checkout: Rust 1.96, edition 2024, local path dependencies,
and `publish = false`. Import `stackpress_framework_view` directly, or use
`stackpress::framework_view` with facade feature `framework-view`.

## Start here

The [quick start](getting-started.md) creates and decodes a script-safe snapshot without
a runtime or listener. Read [Concepts](concepts.md) next to understand what a page may disclose.

## Choose a task

Follow a guide for an integration task; use the [API index](api/README.md) for exact contracts.

| I want to… | Read |
| --- | --- |
| Register a native GET page and loader | [Add a page](guides/add-a-page.md) |
| Use one route definition on both targets | [Share routes](guides/share-routes.md) |
| Reject mismatched or malformed bootstrap data | [Handle contract mismatch](guides/contract-recovery.md) |
| Hydrate or package shared UI | [UI guides](../../framework-ui/docs/README.md#choose-a-task) |

## Features and integration

Default features are empty. The default contract crate is client-safe and has no Leptos or
native host dependency. Enable `server` only in a native host to expose `PageSpec`, `PageLoader`,
and `ServerBuilderViewExt`. The facade equivalent is `framework-view-server`; `server` on
wasm32 is a compile error. UI's `ssr` feature also enables this native extension.

The [Profile UI example](../../../../examples/profile-ui/README.md) is the canonical integration:
one generated capability feeds SSR, hydrated HTTP actions, and local Tauri actions.
The example owns its DTO validation, query codec, transport, asset serving, and recovery UI.

Source authority: [exports](../src/lib.rs), [policies](../src/policy.rs),
[routes](../src/routes.rs), [native extension](../src/server.rs), and [tests](../tests/).
Markdown examples are compile-checked separately from Rustdoc doctests.
