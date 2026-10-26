# PreparedPage

Owned HTML/Head assembly state after the render Owner has been cleaned; native `ssr` only. Source: [render.rs](../../src/render.rs).

```rust
use stackpress_framework_ui::render::PreparedPage;
use stackpress_framework_view::ViewError;
async fn finish(page: PreparedPage) -> Result<String, ViewError> {
    Ok(page.finish().await?.into_html())
}
```

## finish

`async fn finish(self) -> Result<RenderedPage, ViewError>` takes no additional parameters and
consumes the prepared state. Invoke it inside an active Tokio runtime after `initialize_ssr`.

**Returns** One complete `RenderedPage`, or executor, Head assembly, or final HTML limit error.
Only owned metadata/HTML state crosses the await; no render Owner survives in this value.
SSR Head props must own eager values. A Head getter/formatter panic yields `Failure` with a
native cause; the consumed assembly future is discarded rather than resumed or retried.

Dropping a prepared page or unfinished assembly drops its owned buffers; there is no partial
native publication. Signal cleanup already happened during preparation. Fields are private;
obtain the value through `LeptosRenderer::prepare`.

Related: [LeptosRenderer](leptos-renderer.md), [RenderedPage](../../../framework-view/docs/api/rendered-page.md).
