# PageSpec

Checked GET definition with an explicit public-location projection; native `server` feature only. Source: [server.rs](../../src/server.rs).

```rust
use stackpress_framework_view::{PageLoad, PageSpec, PublicLocation};
use stackpress_lib::{Invocation, Request, Response, Result};
async fn load(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<PageLoad<String>> {
    Ok(PageLoad::Ready("public".into()))
}
fn configure() -> Result<()> {
    let _page = PageSpec::get("/ui/welcome", "welcome", load)?
        .location(|_: &String| PublicLocation::new("/ui/welcome?greeting=public"));
    Ok(())
}
```

## Construction and projection

Available for `T: Send + 'static`, `L: PageLoader<T>`. The fields are private.

| Operation | Parameters | Behavior |
| --- | --- | --- |
| `get(path, id, loader)` | `&str`, `&str`, `L` | Checks native Express-style path, `PublicLocation`, no query, and PageId. |
| `location(self, projector)` | `Fn(&T) -> Result<PublicLocation, ViewError> + Send + Sync + 'static` | Stores/replaces a projection over loaded public data only. |

**Returns** `get` yields foundation `Result<PageSpec<T, L>>`; invalid native patterns or public
identity/location return a registration error. `location` consumes and returns Self for chaining;
its checked result is evaluated only after a successful load with an Open response. Projector
failure enters the normal operation error lifecycle.

Without a projector, the public location is the actual request pathname, omitting query data.
Use a projector to rebuild approved path/query fields from `T`. It never implicitly receives
the request/session. Neither method registers a route; pass the spec to `ServerBuilder::page`.

Related: [ServerBuilderViewExt](server-builder-view-ext.md), [PublicLocation](public-location.md).
