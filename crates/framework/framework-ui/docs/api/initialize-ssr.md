# initialize_ssr

Native `ssr` startup requirement. Source: [render.rs](../../src/render.rs).

```rust
use stackpress_framework_ui::render::initialize_ssr;
use stackpress_framework_view::ViewError;
#[tokio::main]
async fn main() -> Result<(), ViewError> {
    initialize_ssr()?;
    initialize_ssr()?;
    Ok(())
}
```

## Function

`initialize_ssr() -> Result<(), ViewError>` takes no parameters. Call during host startup
inside an active Tokio runtime; the renderer and final assembly also need runtime context.
The helper initializes Leptos's process-global Tokio executor and marks this boundary ready.

**Returns** Unit on success, including repeated calls or an already initialized dependency
executor. Outside Tokio it returns `ExecutorUnavailable`. Calling another executor initializer
alone does not replace this helper. It starts no listener and enables buffered owned Head
assembly only; general asynchronous component SSR and reactive SSR Head remain S5-F6.

Related: [LeptosRenderer](leptos-renderer.md), [PreparedPage](prepared-page.md).
