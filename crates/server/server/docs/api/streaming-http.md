# Streaming HTTP API

Available with `stackpress-server/http-stream` (default off, includes http).
Source: [http/stream.rs](../../src/http/stream.rs). The
[streaming guide](../guides/streaming-http.md) has a complete producer example.
No executor, listener, TLS or hidden whole-stream collection enters this crate.

## Response extension and limits

Import `stackpress_server::http::stream::{HttpResponseExt, StreamLimits}`.
HttpResponseExt adds two methods to an Open Response:

| Operation | Inputs | Returns and effects |
| --- | --- | --- |
| stream | mime: &str, producer: S, limits: StreamLimits; S: Stream<Item = Result<Vec<u8>>> + Send + 'static | Result<&mut Self>; validates MIME/limits and installs private owned take-once identity, without polling. |
| immutable_bytes | mime: &str, bytes: Arc<[u8]>, limits: StreamLimits | Result<&mut Self>; checks size and shares immutable storage with known length, without a whole-body copy. |

Finalized Response, invalid MIME, zero/nonfinite limits, or descriptor budget
failure returns Error. immutable_bytes also rejects bytes above max_bytes.
StreamLimits is Clone/Copy/Debug; all controls must be positive and durations
representable. Defaults:

| Field | Type | Default and owner |
| --- | --- | --- |
| max_chunk_bytes | usize | 64 KiB; HttpBody rejects an oversized producer chunk before handoff. |
| max_bytes | usize | 64 MiB aggregate; equality accepted, overflow/excess rejected. |
| total_timeout | Duration | 60 s; driving IO host enforces absolute output lifetime. |
| idle_timeout | Duration | 10 s; driving IO host enforces progress deadline. |

The server-http host enforces time controls, tightened by its NetworkLimits.
In-process users of HttpBody must enforce their own timers. Neither this type
nor Stream can preempt a blocking producer poll.

This independent helper shares an immutable representation for one response:

```rust
use stackpress_lib::Response;
use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
use std::sync::Arc;

/// Attach shared immutable storage without collecting an application stream.
fn response(bytes: Arc<[u8]>) -> stackpress_lib::Result<Response> {
    let mut response = Response::new();
    response.immutable_bytes("text/plain; charset=utf-8", bytes, StreamLimits::default())?;
    Ok(response)
}
```

Known immutable output copies at most min(max_chunk_bytes, 64 KiB) per demand
poll. HEAD returns the known length without polling/copying body bytes.
Unknown producer length is never replaced by an application Content-Length.

## StreamingHttpAdapter

`new(HttpOptions) -> Result<Self>` validates the same trusted origin/conversion
policy as HttpAdapter. It opens no IO. The native seam is:

`handle_body(&self, app: &Server, parts: http::request::Parts, options: HandleOptions, load: F)`

It returns `Result<StreamingHttpReply>` asynchronously. F is FnOnce() -> Fut +
Send; Fut is a Send future returning Result<Vec<u8>>. The loader is invoked once
inside Input after head normalization. Loaded/aggregate body failures use G12's
error policy; malformed heads use G17's sanitized 400 and Input report with no
callbacks or loader. Dropping the future drops its active work; nothing is
spawned. Outer errors mean reporting could not start, such as correlation
exhaustion; ordinary failures remain in the reply report.

HttpAdapter's buffered `handle_with_options(app, input, HandleOptions)` accepts
the same adapter metadata without enabling streaming or changing its bytes.
See [buffered HTTP](http.md) for normalization and representation rules.

## Reply and exclusive body

StreamingHttpReply contains `output: StreamingHttpOutput` and
`report: HandleReport`. Output is Response(http::Response<HttpBody>), Suppressed,
or Unavailable { state: DeliveryState }. HttpBody implements
Stream<Item = Result<Vec<u8>>> and cannot be cloned.

`limits()` returns Option<StreamLimits> while a streaming/immutable source
remains. `is_finished()` inspects known local completion without polling;
an application stream is incomplete until EOF/error is observed. A producer
error is yielded once, the producer drops, and all later polls return None.
Byte/chunk limits are checked before exposing each chunk.

## Publication and failure rules

Response hooks finish before the private source is taken. Final response
budgets/status/headers/cookies are preflighted first; failure selects one fixed
empty 500 and records Output failure. Stopped suppresses native output; foreign
finalization grants no second attempt. Arbitrary SharedNative values do not
become streams; buffered/value adapters reject these descriptors.

ResponseView clones cannot poll or transfer a producer. Response/body replacement
detaches it, and fresh error output cannot reuse a discarded success stream.
HEAD/204/205/304 discard without polling; HEAD never dispatches GET implicitly.
205 has zero Content-Length; 204/304 omit it. Native hop fields are removed,
Transfer-Encoding from application output is rejected, and framing is adapter/
transport owned. Ordinary buffered representations retain their value contract.

After publication, delivery failure closes output: no retry, hook replay, error
lifecycle reentry, or appended JSON. Sent means publication, not network
completion. See [HTTP host reports](../../../http/docs/api/reports.md)
for local producer/native-flush completion and cancellation diagnostics.
