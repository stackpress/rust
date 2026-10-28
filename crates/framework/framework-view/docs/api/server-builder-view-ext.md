# ServerBuilderViewExt

Ordinary native GET registration through ServerBuilder; `server` feature only. Source: [server.rs](../../src/server.rs).

```rust
use stackpress_framework_view::{PageLoader, PageRenderer, PageSpec, ServerBuilderViewExt};
use stackpress_lib::{ListenerId, Result};
use stackpress_server::ServerBuilder;
fn register<T, L, R>(
    app: &mut ServerBuilder,
    spec: PageSpec<T, L>,
    renderer: R,
) -> Result<ListenerId>
where
    T: Send + 'static,
    L: PageLoader<T>,
    R: PageRenderer<T>,
{
    app.page(spec, renderer)
}
```

## page

Import the extension trait to make the method available on `ServerBuilder`.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `self` | `&mut ServerBuilder` | Unbuilt host accumulator. |
| `spec` | `PageSpec<T, L>` | Owned checked GET definition and named loader. |
| `renderer` | `R: PageRenderer<T>` | Send + Sync + 'static renderer and validated contract. |

**Returns** Foundation `Result<ListenerId>`, identifying the ordinary route listener. Immediate
errors include invalid renderer policy and normal server registration errors. A Config-phase
named identity claim rejects duplicate page IDs at `build`, including claims from selected
plugins. Duplicate native route listeners remain allowed and all-matching.

Registration is not transactional across builder queues; discard a builder after a registration
error. The adapter checks delivery state before loading and again before rendering, preserves
loader Abort, stages complete HTML, and restores the public status after HTML staging.
It starts no listener and adds no second top-level lifecycle. The
[full lifecycle table](../concepts.md#lifecycle-remains-the-servers) covers response/error hooks.

Related: [page guide](../guides/add-a-page.md), [PageRenderer](page-renderer.md), [server docs](../../../../server/server/docs/README.md).
