# LeptosRenderer

Checked synchronous component evaluation followed by owned buffered assembly; native `ssr` only. Source: [render.rs](../../src/render.rs).

```rust
use stackpress_framework_ui::{leptos::prelude::*, render::LeptosRenderer};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageRenderer, PageSnapshot, PublicLocation, ViewError,
};
fn renderer() -> Result<impl PageRenderer<String>, ViewError> {
    LeptosRenderer::new(
        PageContract::new(1, "demo-1")?,
        PageAssets {
            build: "demo-1".into(),
            base: BasePath::default(),
            module: PublicLocation::new("/assets/start.js")?,
            stylesheet: None,
        },
        |page: &PageSnapshot<String>| {
            let title = page.data.clone();
            Ok(
                view! { <stackpress_framework_ui::leptos_meta::Title text=title.clone()/><h1>{title}</h1> },
            )
        },
    )
}
```

Contents: [Construction](#1-construction) · [Preparation](#2-preparation) · [Rendering](#3-rendering)

## 1. Construction

`LeptosRenderer::new<T, F, V>(contract, assets, component)` creates `LeptosRenderer<F>`.

| Parameter | Type/bound | Meaning |
| --- | --- | --- |
| `contract` | `PageContract` | Immutable expected revision/build/settings/limits. |
| `assets` | `PageAssets` | Checked build, mount, module, optional stylesheet. |
| `component` | `F: Fn(&PageSnapshot<T>) -> Result<V, ViewError> + Send + Sync + 'static`, `V: IntoView` | Synchronous factory returning owned view state, never live invocation data. |

**Returns** `Result<LeptosRenderer<F>, ViewError>` after checking assets/contract, including exact
build agreement. Construction needs no active executor; rendering does. `F` is reusable, so
avoid capturing request-specific state in it. `T: Serialize` is required for the rendering impl.

## 2. Preparation

`LeptosRenderer::prepare<T: Serialize, V: IntoView>(snapshot, component)` is the convenience
entry on `LeptosRenderer<()>`.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `snapshot` | `&PageSnapshot<T>` | Public seed, checked before evaluation. |
| `component` | `impl FnOnce() -> V` | Synchronous body factory. |

**Returns** `Result<PreparedPage, ViewError>`. Uses snapshot revision/build/locale/theme,
default limits, root base, `/assets/start.js`, and no stylesheet. Use `new` for custom assets
or limits. See the [quick start](../getting-started.md) for complete preparation/finish usage.

## 3. Rendering

The `PageRenderer<T>` impl exposes `contract(&self) -> &PageContract` and
`render(&self, snapshot: PageSnapshot<T>) -> RenderFuture<'_>`.

**Returns** `render` evaluates synchronously before returning a Send future that finishes
owned Head assembly. It checks executor/startup, assets, exact revision/build and locale/theme,
snapshot metadata, and logical seed limit before evaluating components. Final HTML is bounded
after assembly. A fresh Owner provides meta context, `RouteContext`, and `RequestUrl`; cleanup
and caller-Owner restoration occur before metadata awaits.

SSR Head values and formatters must eagerly own their data, as `Title` does above. Resolve
signals during component evaluation; deferred signal getters may run after disposal. Head
assembly panic becomes safe `ViewError::Failure`; component Result errors propagate as errors,
while trusted synchronous component panics propagate after cleanup. No retry occurs.

The output includes `#app`, inert `#stackpress-page` and `#stackpress-assets`, a module script,
explicit HTML language/theme, canonical link, and optional mounted stylesheet. Title text is
escaped at the native metadata boundary. This is buffered rendering, not streaming, general
async Resource SSR, or a sandbox for component CPU/allocations.

Related: [PreparedPage](prepared-page.md), [View contracts](../../../framework-view/docs/api/README.md), [ownership](../concepts.md).
