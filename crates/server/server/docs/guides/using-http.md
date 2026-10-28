# Using the buffered HTTP adapter

When your transport already owns request bytes, convert them through the host without writing a
second operation implementation. Enable `stackpress-server` feature `http`; this adapter handles
values in process and opens no listener.

## Call a route with buffered input

This example registers one route and inspects its native response. It uses the exported alias so no
direct dependency on the `http` crate is needed:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ServerBuilder,
    http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput},
};

async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"Hello, Ada")?;
    Ok(Flow::Continue)
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    app.get("/hello", hello)?;
    let server = app.build().await?;
    let mut input = BufferedRequest::new(Vec::new());
    *input.uri_mut() = "/hello".parse()?;
    let reply = HttpAdapter::new(HttpOptions::default())?
        .handle(&server, input)
        .await?;
    match reply.output {
        HttpOutput::Response(native) => {
            assert_eq!(native.status(), 200);
            let envelope: serde_json::Value = serde_json::from_slice(native.body())?;
            assert_eq!(envelope["results"], "Hello, Ada");
        }
        other => panic!("expected native output, got {other:?}"),
    }
    assert!(!reply.report.output_eligible);
    Ok(())
}
```

The method defaults to GET. The adapter selects `GET /hello`, loads input inside the common
lifecycle, then converts the public envelope into one buffered response. Explicitly match Suppressed
and Unavailable in a real transport; neither contains bytes to send.

## Set the transport policy

Supply a trusted HTTP(S) origin with HttpOptions for origin-form requests; Host and forwarded
headers remain untrusted data. Absolute web targets keep their own checked authority. Only exact
method/path selects routes; normalization-changing targets, non-path targets, ambiguous content
types, and unrepresentable headers fail before callbacks. Structured body parse failure after
normalization may be recovered through Error.

The caller must bound network reads before assembling this buffer. Adapter limits bound retained
values and parsing, not upstream network buffering. It supplies no TLS, streams, gateway,
backpressure, or implicit authorization. HEAD needs a HEAD/Any route; it does not reuse GET
automatically.

You have converted one in-process route to native HTTP values. See [HTTP reference](../api/http.md)
for budgets, text/binary output, framing, cookie-list limitations, and sanitized conversion
fallback. The [canonical HTTP test](../../../../../examples/capability-host/tests/http.rs) compares
this path with the same capability's direct call.

For demand-driven response output, enable http-stream and follow
[Streaming HTTP](streaming-http.md). For TCP, HTTPS, admission and deadlines,
use the separate [HTTP host](../../../http/docs/getting-started.md).
The facade's existing server-http feature continues to select only this
buffered conversion path.
