# initialize_ssr

Native `ssr` startup requirement. Source: [render.rs](../../src/render.rs).

```rust
use stackpress_leptos::render::initialize_ssr;
use stackpress_leptos::render::RenderError;
#[tokio::main]
async fn main() -> Result<(), RenderError> {
    initialize_ssr()?;
    initialize_ssr()?;
    Ok(())
}
```

## Function

`initialize_ssr() -> Result<(), RenderError>` takes no parameters. Call during host startup
inside an active Tokio runtime; the renderer and final assembly also need runtime context.
The helper initializes Leptos's process-global Tokio executor and marks this boundary ready.

**Returns** Unit on success, including repeated calls or an already initialized dependency
executor. Outside Tokio it returns `ExecutorUnavailable`. Calling another executor initializer
alone does not replace this helper. It starts no listener. Synchronous eager Head, buffered async and opt-in streaming all use
this startup boundary; their different Owner lifetimes are documented separately.

Related: [native ownership](../concepts.md).
