# Own asynchronous work

A Resource loader can outlive the component that started it. Wrap yielding work inside the
Resource fetcher so its Owner can cancel it cooperatively; keep external side effects and their
retry/rollback policy in your application.

```rust
use leptos::prelude::*;
use stackpress_leptos::resources::cancel_on_cleanup;

#[component]
pub fn PublicDetails() -> impl IntoView {
    let details = Resource::new(|| (), |_| cancel_on_cleanup(async {
        "Explicit public details".to_owned()
    }));
    view! {<Suspense fallback=|| "Loading">
        {move || details.get().map(|result| result.unwrap_or_else(|failure| failure.to_string()))}
    </Suspense>}
}
```

The guard is created while the fetcher's Owner is active. On cleanup, an awakened poll drops
the loader. A native loader panic becomes safe `ResourceFailure` text; the private cause is
available only through explicit native diagnostics and is omitted from replay serialization.
Use native `ssr` for rendering this component and browser `hydrate,async` or `csr,async` for it
on the client. Buffered rendering retains its Owner until complete body/replay/Head assembly.

## Publish only current suggestions

`ComboboxState::request(query, loader)` returns an owned completion future. Await it without
holding an Owner guard or state lock, then call `state.publish(completion)` in that root's UI
context. A newer generation, foreign root or disposed root rejects an old completion. A ticket
also has cooperative abort ownership. Do not interpret a pending result as a successful save.

This is separate from a Resource source change: `cancel_on_cleanup` cancels on Owner cleanup,
not every replaced source. The [independent helper tests](../../../../examples/leptos-independent/tests/values.rs)
prove stale/foreign completion rejection and pending-ticket cancellation; its lifecycle watchdog
proves actual Resource success, cancellation and panic cleanup. See [replay](../api/replay.md)
for the inert wire and its limits.
