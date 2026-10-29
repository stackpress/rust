# Render native HTML

Choose synchronous staging for eager owned content. If a Resource or reactive Head must finish,
use `AsyncRenderer` so its Owner stays alive through complete assembly. Both require native
`ssr`, explicit startup inside Tokio, and a caller-supplied document shell.

```rust
use leptos::prelude::*;
use stackpress_leptos::async_render::AsyncRenderer;
use stackpress_leptos::render::{initialize_ssr, RenderLimits};
use std::convert::Infallible;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    let render = AsyncRenderer::try_prepare(
        RenderLimits::default(),
        || Ok::<_, Infallible>(view! { <p>"Owned asynchronous result"</p> }),
        || Ok(()),
        |body, replay| Ok(format!(
            "<html><head></head><body>{body}<script type=\"application/json\">{replay}</script></body></html>"
        )),
    )?;
    let html = render.await?;
    assert!(html.contains("Owned asynchronous result"));
    Ok(())
}
```

The third closure runs after the component's stream is created while its Owner is active.
Here it has no extra work. The final closure receives generated body and checked script-safe
replay; its framing is application code. The returned concrete `OwnedRender` is `Send` and owns
its captures. Drop it when the caller disconnects, and keep the executor alive long enough for
cooperative loader cancellation to be polled.

A factory's typed error stays `RenderError::Component(error)`. Native metadata/worker failures
retain causes for explicit diagnostic traversal; public `Display` does not expose those causes.
Component panic during preparation propagates after cleanup; consumed assembly unwind becomes
a native failure. No path retries trusted work.

Use [the rendering reference](../api/rendering.md) for signatures and ceilings, and
[own async work](async-work.md) for loader cancellation. The independent consumer's
[lifecycle tests](../../../../examples/leptos-independent/tests/lifecycle.rs) exercise those exits
without importing application contracts.
