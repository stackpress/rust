# Hydrate or mount the same root

Hydration keeps useful native HTML and adopts its nodes. CSR creates a fresh tree. Build each
mode separately from the same component source and explicit initial values; do not combine
features in one artifact.

## Build matching artifacts

The maintained [independent consumer](../../../../examples/leptos-independent/README.md) has
`default=[]`, selects modes on both libraries, and places WASM ABI dependencies under the
WASM target. Its finite build proof copies each matching artifact before building the next.

```sh
python3 examples/leptos-independent/proof.py build
```

Run proof commands through the repository benchmark recorder with the task target/output
settings described in that consumer's README. The proof requires the installed WASM target
and matching `wasm-bindgen`; it never installs tools or starts a listener.

## Adopt native nodes

A minimal browser entry receives a caller-owned element and explicit component. Select
`hydrate` on Leptos and the library, with the consumer's target-specific WASM dependencies.

```rust
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub fn adopt(element: web_sys::Element) -> Result<(), wasm_bindgen::JsValue> {
    let target = element.dyn_into::<web_sys::HtmlElement>()?;
    let handle = leptos::mount::hydrate_from(target, || view! { <h1>"Welcome"</h1> });
    // This example disposes immediately; a real caller must retain the handle until unmount.
    drop(handle);
    Ok(())
}
```

The native renderer must have produced that same `h1` inside the target. If the root uses
metadata or `LayoutProvider` styles, its document head must retain upstream Leptos metadata's
`<!--HEAD-->` marker (normally produced by `MetaTags`). Generic shell closures own this
requirement; omitting it can trap when hydrating metadata. The independent
fixture instead returns a JS disposer that retains the handle, adopts early input/focus,
and validates its explicit choices before constructing its real root.

## Mount CSR and own teardown

With `csr`, use `leptos::mount::mount_to` on an empty target. Its returned handle has the same
ownership rule: keep it while interactive and drop it once during teardown. A repeated wrapper
can use `Option::take` to make later disposal harmless. A fresh CSR root must start from fresh
signals rather than reuse handles belonging to a disposed Owner.

Resource replay is optional and requires `async` on the browser build. Decode through
`ResourceReplay` before populating upstream Leptos replay arrays; do not evaluate replay text.
The fixture's [bootstrap](../../../../examples/leptos-independent/bootstrap.js) and
[WASM adapter](../../../../examples/leptos-independent/src/browser.rs) show the complete boundary.
