# Hydrate a browser page

Adopt server markup only after the client can recreate the same first tree. Compile the
shared component in a separate WASM consumer with `stackpress-leptos/hydrate`, plus
View's default contracts. Add direct `leptos = "=0.8.22"` with defaults off and feature
`hydrate` for macro expansion. Never enable native `ssr` in that consumer.

## Admit the bootstrap first

Read `textContent` from the inert `stackpress-page` and `stackpress-assets` scripts. Use
compiled revision/build expectations, the same lower snapshot budget, and application DTO
validation. [View's recovery guide](contract-recovery.md)
owns those checks and the explicit reload policy.

The following small mount fragment assumes the admitted page is `PageSnapshot<String>`,
`app` is the server's `#app` HtmlElement, and the supplied base came from admitted assets.
In the WASM consumer, add `web-sys` with feature `HtmlElement`.

```rust
use stackpress_leptos::{leptos::{mount::hydrate_from, prelude::*}};
use stackpress_framework_view::leptos::{routing::provide_route_context};
use stackpress_framework_view::{BasePath, PageSnapshot};

pub fn hydrate(app: web_sys::HtmlElement, page: PageSnapshot<String>, base: BasePath) {
    let message = page.data;
    let handle = hydrate_from(app, move || {
        provide_route_context(base.clone());
        view! { <h1>{message.clone()}</h1> }
    });
    handle.forget();
}
```

Compile this block for `wasm32-unknown-unknown` with `hydrate`, not with `csr`. The component
matches the quick start's body. `forget` deliberately keeps this minimal root for the document
lifetime; an application with unmounting should retain the handle and drop it during disposal.
For components using Head, provide one client metadata context before evaluating the tree.

## Preserve initial agreement

The real root must use the same page identity, data, status, locale/theme, route base, and
component structure as SSR. Seed its controller from the snapshot without another initial
operation call. Subsequent user actions may use a fixed typed HTTP adapter.

Build matching JS/WASM and serve the bootstrap URLs declared by the server. The
[canonical example](../../../../../examples/profile-ui/README.md) owns pinned bindgen commands,
asset manifests, browser entrypoint/disposer, and proofs of unchanged initial DOM and focus.
Its integration is the recommended next step after this minimal mount compiles.
