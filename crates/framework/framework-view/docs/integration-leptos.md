# Add Leptos page rendering explicitly

Use view when a page needs public snapshot/route/asset contracts and host integration. Use
`stackpress-leptos` directly for reusable UI and generic rendering. View's page contracts work
without either Leptos or a server; opt into the adapter for the artifact you build.

## Select the integration

The same pinned upstream Leptos mode must be selected wherever your component macros compile.
Page adapters keep their metadata, admission and host ownership in view.

| View feature | Adds |
| --- | --- |
| `leptos` | Routing context and public-value presentation mapping; no native renderer/server |
| `leptos-async` | `leptos` plus checked page replay wrapper; no server |
| `leptos-ssr` | Native sync/buffered async adapters, generic SSR and view's `server` extension |
| `leptos-stream` | `leptos-ssr` plus explicit page streaming/status framing |
| `leptos-hydrate` | Browser hydration integration; add `leptos-async` when using replay |
| `leptos-csr` | Browser CSR integration; add `leptos-async` when using replay |

Facade features are `framework-view-leptos` and the matching `-async`, `-ssr`, `-stream`,
`-hydrate` or `-csr` suffix. Generic helper features remain `leptos-rich` and `leptos-format`.
Preferences, flash, native files and web registrations retain their separate feature gates;
none enables Leptos by itself. Importing libraries performs no registration.

## Render a checked page

Select view's `leptos-ssr`, direct upstream `leptos = "=0.8.22"` with `ssr`, and a native Tokio
runtime. This complete program uses a checked snapshot and a synchronous eager component.

```rust
use leptos::prelude::*;
use stackpress_framework_view::{BasePath, PageAssets, PageContract, PageRenderer,
    PageSnapshot, PublicLocation, ViewError};
use stackpress_framework_view::leptos::render::{initialize_ssr, LeptosRenderer};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ViewError> {
    initialize_ssr()?;
    let contract = PageContract::new(1, "example-build")?;
    let assets = PageAssets {
        theme_bootstrap: None, build: "example-build".into(), base: BasePath::default(),
        module: PublicLocation::new("/assets/start.js")?, stylesheet: None,
    };
    let renderer = LeptosRenderer::new(contract, assets, |page: &PageSnapshot<String>| {
        Ok(view! { <h1>{page.data.clone()}</h1> })
    })?;
    let page = PageSnapshot::decode(
        r#"{"revision":1,"build":"example-build","page":"home","location":"/","locale":"en","theme":null,"data":"Welcome"}"#,
        1, "example-build",
    )?;
    assert!(renderer.render(page).await?.into_html().contains("Welcome"));
    Ok(())
}
```

The adapter validates page/assets/settings before evaluating a fresh root, then uses the generic
kernel for Owner and metadata execution. View owns its document shell, script-safe seed/assets,
mount joining and output/error types. The module URL is declared data here; this example does
not serve it or start a listener. A host registration still needs an explicit loader/page spec.

Use `AsyncLeptosRenderer` for Resources/reactive Head, and `StreamLeptosRenderer` only when the
host and browser implement its completion/recovery protocol. Their return values retain view's
`ViewError` and rendered-page types. Direct generic kernels use `RenderError<E>` instead.

## Share the boundary with the client

Admit the page and assets before constructing browser state. Enable `leptos` where a component
needs `RouteContext`/`CatalogPath`; `BasePath`, `PageRoute` and `RouteCatalog` remain view root
contracts. Build hydrate and CSR separately, with no native `server`/file feature leakage.

View maps acknowledged Light/Dark values to generic UI theme; absence remains unset. Saved
preference policy, one-shot pre-paint sampling and no-JS behavior belong to view/host adapters,
not UI signals. Flash conversion, dictionaries and allowed-action hints consume explicit public
values; actual authorization remains in the host.

Follow [hydration](guides/leptos-hydrate.md), [routing](api/leptos-routing.md) and
[presentation host composition](presentation-host.md). The [migration map](migration-issue17.md)
lists retired paths; [the independent consumer](../../../../examples/leptos-independent/README.md)
proves the lower-level library without any view/server/facade edge. Base `plugin::register`
remains reserved for issue 14; existing preference/flash registrations remain explicit.
