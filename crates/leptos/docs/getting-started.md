# Render your first component

A shared component needs a renderer and an initial value before it needs an HTTP server.
This walkthrough produces a complete HTML string using the generic native kernel.

## Try the maintained consumer

From the repository root, run the finite example with the installed toolchain and cached lock:

```sh
cargo run --manifest-path examples/leptos-independent/Cargo.toml --locked --offline \
  --features ssr --example document -- hydrate light en
```

It prints one document and exits. Look for `Welcome`, a labelled draft input and an escaped
`Independent < & >` title. The consumer owns its shell; it has no view/server dependency.
Change the final choices to `dark fr` to render an independent French root.

## Render a small component yourself

In that consumer, a native example can use the following complete program. Its manifest selects
`ssr` on both libraries and supplies Tokio as a native development dependency for examples.

```rust
use leptos::prelude::*;
use stackpress_leptos::render::{initialize_ssr, Renderer, RenderLimits};
use std::convert::Infallible;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    let prepared = Renderer::try_prepare(
        RenderLimits::default(),
        || Ok::<_, Infallible>(view! { <h1>"Welcome"</h1> }),
        |body| Ok(format!("<html><head></head><body>{body}</body></html>")),
    )?;
    let html = prepared.finish().await?;
    assert!(html.contains("Welcome"));
    Ok(())
}
```

`try_prepare` evaluates the component in a fresh Owner and passes its generated body to your
shell closure. `finish` injects eager owned Head values and checks the complete document's
byte limit. The runtime and explicit startup make the executor available; they open no port.
Escape any untrusted text you interpolate into your own shell. Leptos escapes component text.

## What the first render establishes

The synchronous Owner is already cleaned when preparation returns. Use eager owned title/Head
values here; [buffered async rendering](guides/render-native.md) retains a root for Resources and
reactive Head. A successful render does not create routing, bootstrap data or persistence.

You now have a real owned HTML result. [Compose controls](guides/compose-a-root.md) next, then
[hydrate or mount](guides/hydrate-and-mount.md) the same tree from explicit initial values.
The [API index](api/README.md) is the lookup path for types, features and failure contracts.
