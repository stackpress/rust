# Render your first HTML document

Render one page before wiring a browser or HTTP listener. This gives you a complete HTML
document and proves the native executor, public seed, and component agree.

## Create a native consumer

In a small binary crate outside this workspace, add these local dependencies with your actual
checkout path:

```toml
[dependencies]
stackpress-framework-ui = { path = "/path/to/stackpress/rust/crates/framework/framework-ui", features = ["ssr"] }
stackpress-framework-view = { path = "/path/to/stackpress/rust/crates/framework/framework-view" }
leptos = { version = "=0.8.22", default-features = false, features = ["ssr"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

The direct `leptos` dependency is required by Leptos macro expansion, even with framework
re-export imports. Select the same mode on both dependencies. Put this program in
`src/main.rs`; `initialize_ssr` must run inside Tokio before preparation.

```rust
use stackpress_framework_ui::{
    leptos::prelude::*,
    render::{LeptosRenderer, initialize_ssr},
};
use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus, ViewError};

#[tokio::main]
async fn main() -> Result<(), ViewError> {
    initialize_ssr()?;
    let page = PageSnapshot {
        revision: 1,
        build: "demo-1".into(),
        page: PageId::try_from("welcome".to_owned())?,
        location: "/ui/welcome".into(),
        locale: "en".into(),
        theme: "light".into(),
        status: PublicStatus::ok(),
        data: "Welcome".to_owned(),
    };
    let message = page.data.clone();
    let prepared = LeptosRenderer::prepare(&page, || view! { <h1>{message}</h1> })?;
    let html = prepared.finish().await?.into_html();
    assert!(html.contains("Welcome"));
    assert!(html.contains(r#"id="stackpress-page""#));
    println!("Rendered complete HTML");
    Ok(())
}
```

Run `cargo run`; expect `Rendered complete HTML`. `prepare` evaluates the component synchronously,
and `finish` awaits owned Head assembly before checking the complete-document byte limit.
This program does not serve the default `/assets/start.js` URL or hydrate the document.

## What happened

The renderer created a fresh Leptos Owner, provided metadata and route context, rendered the
body, and disposed reactive state before returning `PreparedPage`. Only owned HTML and Head
assembly state survived into the await. `into_html` transferred the checked complete document.

For a real app, use `LeptosRenderer::new` with explicit assets and register it through
[View's page extension](../../framework-view/docs/guides/add-a-page.md). You can now verify
native output without a server; [Concepts](concepts.md) explains the ownership rules that
keep this safe across requests.
