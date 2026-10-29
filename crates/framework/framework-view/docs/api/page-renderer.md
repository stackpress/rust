# PageRenderer and RenderFuture

Renderer/host boundary over owned public data. Source: [lib.rs](../../src/lib.rs).

```rust
use stackpress_framework_view::{
    PageContract, PageRenderer, PageSnapshot, RenderFuture, RenderedPage,
};
struct StaticRenderer {
    contract: PageContract,
}
impl PageRenderer<()> for StaticRenderer {
    fn contract(&self) -> &PageContract {
        &self.contract
    }
    fn render(&self, _page: PageSnapshot<()>) -> RenderFuture<'_> {
        Box::pin(async { RenderedPage::new("<!DOCTYPE html><p>Welcome</p>".to_owned()) })
    }
}
```

## Trait contract

`PageRenderer<T>: Send + Sync + 'static` has two required methods. Implementations own their
public metadata and do not receive a request, response, invocation, or native writer.

| Method | Parameters | Responsibility |
| --- | --- | --- |
| `contract(&self)` | None | Borrow immutable `&PageContract`, validated before native registration. |
| `render(&self, page)` | Owned `PageSnapshot<T>` | Produce complete staged output; enforce the renderer's selected contract/limits. |

**Returns** `contract` returns `&PageContract`. `render` returns `RenderFuture<'_>`, an alias for
`Pin<Box<dyn Future<Output = Result<RenderedPage, ViewError>> + Send + 'a>>`. The future may borrow
the renderer for `'a`. No partial native output is published through this trait.

The static example deliberately ignores page data and emits fixed HTML. A custom renderer
that embeds data must validate, escape, and bound it itself; the trait signature does not
automatically implement those policies. The supported Leptos adapter does implement them.
Never carry live reactive state or native output authority across this boundary.

Related: [LeptosRenderer](leptos-renderer.md), [RenderedPage](rendered-page.md).
