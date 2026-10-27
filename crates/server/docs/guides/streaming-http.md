# Publish an owned HTTP response stream

Use streaming when output should be produced on demand after the ordinary
lifecycle finishes. Enable `stackpress-server/http-stream`; the feature includes
`http` and adds a typed one-shot response extension without a runtime or listener.
The network host enables it automatically.

## Attach an owned producer

This independent example installs a single-chunk producer in a named handler,
then drives it in process through the streaming adapter. It needs stackpress-lib,
stackpress-server with http-stream, and futures-core:

```rust
use futures_core::Stream;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    HandleOptions, ServerBuilder,
    http::{
        BufferedRequest, HttpOptions,
        stream::{HttpResponseExt, StreamLimits, StreamingHttpAdapter, StreamingHttpOutput},
    },
};
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// Own all state that must survive the handler; yield once without spawning a task.
struct Greeting(Option<Vec<u8>>);
impl Stream for Greeting {
    type Item = Result<Vec<u8>>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.0.take().map(Ok))
    }
}

/// Select the representation; no producer polling happens inside this call.
async fn greeting(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.stream(
        "text/plain; charset=utf-8",
        Greeting(Some(b"hello\n".to_vec())),
        StreamLimits::default(),
    )?;
    Ok(Flow::Continue)
}

/// Drive native output without opening IO; a real host also owns the time controls.
async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut builder = ServerBuilder::new();
    builder.get("/stream", greeting)?;
    let app = builder.build().await?;
    let mut input = BufferedRequest::new(Vec::new());
    *input.uri_mut() = "/stream".parse()?;
    let (parts, _) = input.into_parts();
    let reply = StreamingHttpAdapter::new(HttpOptions::default())?
        .handle_body(&app, parts, HandleOptions::default(), || async {
            Ok(Vec::new())
        })
        .await?;
    let StreamingHttpOutput::Response(mut native) = reply.output else {
        panic!("expected available native response");
    };
    let chunk = std::future::poll_fn(|cx| Pin::new(native.body_mut()).poll_next(cx)).await;
    assert_eq!(chunk.transpose()?, Some(b"hello\n".to_vec()));
    assert!(
        std::future::poll_fn(|cx| Pin::new(native.body_mut()).poll_next(cx))
            .await
            .is_none()
    );
    Ok(())
}
```

The producer owns its bytes. A producer must implement
`Stream<Item = stackpress_lib::Result<Vec<u8>>> + Send + 'static`; it cannot borrow
Request, ResponseView or Invocation state. Response hooks run before the adapter
takes the source and before the first poll. For an actual connection, use
[the HTTP host](../../../server-http/docs/getting-started.md), which enforces
stream total/idle deadlines as well as native write limits.

## Preserve one-shot ownership

A ResponseView clone shares private descriptor identity, not another producer
or output permit. Replacing the response or its body detaches the original
producer; retained views cannot poll or extract it. If a handler errors after
selecting a success stream, the fresh error response cannot leak the previous
success bytes, headers or cookies.

HEAD requires its own HEAD/Any route; it never invokes GET automatically. HEAD
and statuses 204, 205, 304 take and drop the producer without polling it.
Unknown-length streams omit Content-Length; the transport owns framing.
`immutable_bytes` shares owned Arc<[u8]> storage with a known representation
length and is useful for verified asset responses. See
[streaming reference](../api/streaming-http.md).

## Handle failure at the correct boundary

Before publication, final response budgets, status, headers and cookies are
preflighted. Conversion failure records Output failure and chooses one fixed
empty 500. Stopped yields Suppressed. A response finalized outside adapter
authority yields Unavailable, without another output attempt.

After publication, producer failure or limit/deadline failure ends delivery.
No retry, lifecycle replay, or JSON error append is permitted. Dropping the
exclusive body drops its producer; cancellation drops active loading/handling
without detached completion work. The buffered HttpAdapter and other value
adapters do not collect this native descriptor into bytes.

Framework Sent reports publication only. The network host's separate exchange
report establishes producer handoff and native flush, still without a remote
acknowledgement. You now have an owned producer and one publication boundary;
use [HTTP delivery reports](../../../server-http/docs/api/reports.md) when
tracking a real connection.
