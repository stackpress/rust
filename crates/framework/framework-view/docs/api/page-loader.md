# PageLoader

Named asynchronous loader contract; native `server` feature only. Source: [server.rs](../../src/server.rs).

```rust
use stackpress_framework_view::PageLoad;
use stackpress_lib::{Invocation, Request, Response, Result};
async fn load_welcome(
    _: &mut Request,
    _: &mut Response,
    _: Invocation<'_>,
) -> Result<PageLoad<String>> {
    Ok(PageLoad::Ready("Welcome".into()))
}
```

## Accepted functions

Supply a named async function as in the example. The actual future for every common invocation
borrow must be Send and borrow-bounded; the callable is Send + Sync + 'static. Returning data
for native page registration additionally requires `T: Send + 'static`.

| Parameter | Type | Purpose |
| --- | --- | --- |
| Request | `&mut Request` | Validate app input; never serialize it wholesale. |
| Response | `&mut Response` | Use existing public status/error and delivery policy. |
| Invocation | `Invocation<'_>` | Delegate a fixed child operation inside the same lifecycle. |

**Returns** The function future yields foundation `Result<PageLoad<T>>`. `Ready` must own the
public projection. Child Abort should map to `PageLoad::Abort`; errors stay Result failures.
An `Rc` held across await is rejected by the real registered-loader bounds.

`PageLoader<T>` is a marker with a blanket implementation for accepted callables. Use the
named function path rather than trying to implement its private adaptation machinery. Do not
retain the live invocation in reactive state or start another top-level handle for nested work.

Related: [PageSpec](page-spec.md), [PageLoad](page-load.md), [page guide](../guides/add-a-page.md).
