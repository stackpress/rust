# Stream owned HTML bytes

Choose `stream` only when the consumer can handle partial output and a late failure. The kernel
keeps local view state on one worker and sends bounded owned bytes to your transport. You own
the document/replay framing, failure marker and meaning of completion.

```rust
use futures::StreamExt;
use leptos::prelude::*;
use stackpress_leptos::{leptos_meta::ServerMetaContext,
    render::{initialize_ssr, RenderLimits}, stream::{StreamLimits, StreamRenderer}};
use std::convert::Infallible;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_ssr()?;
    let mut body = StreamRenderer::render(
        RenderLimits::default(), StreamLimits::default(), "Stream failed".into(),
        |writer| async move {
            let (metadata, head) = ServerMetaContext::new();
            provide_context(metadata);
            let chunks = view! { <p>"One local view"</p> }.to_html_stream_in_order();
            writer.render(chunks, head, "<html><head></head><body>".into(),
                |body, _replay| Ok::<_, Infallible>(body), "</body></html>".into()).await
        },
    ).await?;
    let mut html = Vec::new();
    while let Some(chunk) = body.next().await { html.extend(chunk?); }
    assert!(String::from_utf8(html)?.contains("One local view"));
    Ok(())
}
```

This component has no Resource, so the framing callback deliberately ignores its empty replay.
If you add Resources, frame each checked replay value inertly and define a matching browser
admission/bootstrap protocol. The fixed failure text is reserved inside the aggregate byte budget.
The factory crosses threads; its constructed local view never does.

The first poll receives staged opening/Head, and demand starts production. Slow consumers
bound producer lookahead through the two-slot bridge. Dropping the body cancels and joins the
worker; successful EOF follows explicit completion. A deadline or post-publication error yields
the supplied failure bytes and a typed error. A host must distinguish that from successful EOF.
Use [the stream reference](../api/streaming.md) for exact bounds and cancellation limits.
