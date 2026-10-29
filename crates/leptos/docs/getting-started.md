# Render an independent component

Add `stackpress-leptos` from `crates/leptos` with `ssr`, upstream `leptos = =0.8.22`
with defaults off and `ssr`, and Tokio with `macros` and `rt`. No view/server dependency is needed.

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

The synchronous component runs under a fresh Owner, which is cleaned before preparation
returns. Finish injects eager owned Head and admits the fully framed bytes. A shell is trusted
application code: it must escape its own interpolated values. No page seed, bootstrap, route,
listener or persistent setting is inferred. Use buffered async rendering for reactive Head.
