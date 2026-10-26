# Add a page to a ServerBuilder app

Register a GET page when an existing capability needs a browser view. Keep loading in a named
Send async function; return only the public data needed by the synchronous component.

## Select the native features

Add local dependencies on `stackpress-lib`, `stackpress-server`, and `stackpress-framework-view`
with feature `server`, plus `stackpress-framework-ui` with feature `ssr`. Add a direct `leptos = "=0.8.22"` dependency with defaults off and feature `ssr` for macro
expansion. Add Tokio with `macros` and `rt-multi-thread`. The following complete program builds and handles one request
in process, without a listener.

```rust
use stackpress_framework_ui::{
    leptos::prelude::*,
    render::{LeptosRenderer, initialize_ssr},
};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageLoad, PageSnapshot, PageSpec, PublicLocation,
    ServerBuilderViewExt,
};
use stackpress_lib::{Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

async fn load_welcome(
    _: &mut Request,
    _: &mut Response,
    _: Invocation<'_>,
) -> Result<PageLoad<String>> {
    Ok(PageLoad::Ready("Welcome".to_owned()))
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    let renderer = LeptosRenderer::new(
        PageContract::new(1, "demo-1")?,
        PageAssets {
            build: "demo-1".into(),
            base: BasePath::default(),
            module: PublicLocation::new("/assets/start.js")?,
            stylesheet: None,
        },
        |page: &PageSnapshot<String>| {
            let message = page.data.clone();
            Ok(view! { <h1>{message}</h1> })
        },
    )?;
    let mut app = ServerBuilder::new();
    app.page(
        PageSpec::get("/ui/welcome", "welcome", load_welcome)?,
        renderer,
    )?;
    let server = app.build().await?;
    let mut request = Request::builder().url("/ui/welcome").build()?;
    let mut response = Response::new();
    let report = server.handle_route(&mut request, &mut response).await?;
    assert!(report.failures.is_empty());
    assert!(
        matches!(response.body(), stackpress_lib::Body::Text(html) if html.contains("Welcome"))
    );
    Ok(())
}
```

The loader returns a public string. The factory owns its view text rather than borrowing the
snapshot. `page` queues the GET handler and a Config-phase identity claim; `build` checks both
before publishing the host. The in-process assertion confirms that the route staged HTML.
The bootstrap URL is metadata here; serving actual JS/WASM remains the application's job.

## Use an existing operation

In a real loader, validate input, call `inv.delegate` with the fixed capability name, preserve
child `Flow::Abort` as `PageLoad::Abort`, and decode an allowlisted reply into `T`.
The [Profile UI example](../../../../../examples/profile-ui/README.md) and its
[web loader](../../../../../examples/profile-ui/web/src/lib.rs) show this full integration.

If public query fields are needed, attach `PageSpec::location` and rebuild the location from
the loaded data. Do not copy the original query wholesale. Use the shared
[route catalog](share-routes.md) for mounted paths.

You have added a page through ordinary server registration. For rendering prerequisites and
owned Head, see [UI concepts](../../../framework-ui/docs/concepts.md); for error and Abort
behavior, see the [lifecycle table](../concepts.md#lifecycle-remains-the-servers).
