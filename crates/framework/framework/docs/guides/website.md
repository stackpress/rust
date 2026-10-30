# Compose a website and browser artifacts

Build native SSR and browser artifacts separately. The native host owns identity verification, SQL, mail and render/compiler providers; the browser receives deliberately constructed page props and snapshots. The [website consumer](../../consumers/website/Cargo.toml) already defines compatible `ssr`, `hydrate` and `csr` selections.

## Select UI families and one renderer

On the native graph select the runtime aggregate, the required `framework-aggregate-ui-*` families and `framework-aggregate-ssr`. On each browser graph select UI families and exactly one of `framework-aggregate-hydrate` or `framework-aggregate-csr`, without native aggregate leaves. Renderer forwarding alone selects no UI family. See [features](../api/features.md) for both compile-error guards.

This portable fence compiles in the website's SSR, hydrate and CSR graphs. It decodes the same public snapshot contract as the existing browser entry, without importing a native catalog or provider.

<!-- example: browser -->
```rust
use stackpress::{
    framework_session::views::{Page, PageProps},
    framework_view::PageSnapshot,
};

pub fn public_page(text: &str) -> Result<(Page, PageProps), String> {
    PageSnapshot::<(Page, PageProps)>::decode(text, 1, "session-v1")
        .map(|snapshot| snapshot.data)
        .map_err(|error| error.to_string())
}
```

Use the consumer's [browser start function](../../consumers/website/src/lib.rs) to hydrate/mount and retain its disposal action. The [native matrix](../../consumers/website/src/session_matrix.rs) shows actual SQLite-backed loaders and hook/Engine shutdown. Browser compilation, Chromium execution and installed Tauri acceptance are distinct evidence.

## Supply rendering and build IO

`view::Provider` owns render callbacks. `view::BuildProvider` constructs a command-owned BuildEngine with `set`, `size`, clients, assets and pages callbacks. Entry loading precedes path selection; the build skips an empty engine, then runs only truthy output paths in clients → assets → pages order. The [view reference](../api/view.md) describes response skips and source props.

View activation tests explicit ConfigPresence: missing `view` is inactive; explicit null, false and an empty object are present and activate it. Non-production mode adds only the literal request middleware hook during Route. It requires native HTTP adapter context and a `view-http` binding when invoked; general Route-phase event registration rules remain unchanged.

## Own optional packaging tasks

Retain the supervision handle when supplying an optional packaging callback. Construction does not spawn work; build submission follows the source's non-awaited packaging point.

<!-- example: full -->
```rust
use stackpress_framework::view::{PackagePaths, Packaging};
use stackpress_lib::Result;
use std::sync::Arc;

pub async fn packaging_lifecycle() -> Result<()> {
    let callback = Arc::new(|paths: PackagePaths| {
        Box::pin(async move {
            assert!(!paths.cwd.as_os_str().is_empty());
            Ok(())
        }) as stackpress_framework::view::NativeFuture<()>
    });
    let packaging = Packaging::new(tokio::runtime::Handle::current(), callback);
    assert!(packaging.drain().await.is_empty());
    packaging.shutdown().await;
    Ok(())
}
```

Publish an Arc of that handle as a Config service for a real build host. `drain` returns provider results or task panic/cancellation outcomes, without changing an already returned build response. `shutdown` closes submissions, aborts pending tasks and reaps them. Source `build.ts:21` does not await packaging; Rust supplies supervision, not an npm package.json writer. See [view API](../api/view.md) and [timing tests](../../tests/cases/view.rs).
