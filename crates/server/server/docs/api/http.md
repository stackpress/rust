# Buffered HTTP API

Available only with `stackpress-server` feature `http` (default off). These are HTTP value
conversions, not a network transport. Source: [http.rs](../../src/http.rs).

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ServerBuilder,
    http::{BufferedRequest, HttpAdapter, HttpOptions, HttpOutput},
};

async fn hello(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"hello")?;
    Ok(Flow::Continue)
}
let mut app = ServerBuilder::new();
app.get("/hello", hello)?;
let server = app.build().await?;
```

Each following Rust snippet is independent, using this setup inside an async function returning
`std::result::Result<(), Box<dyn std::error::Error>>`. Named helpers remain ordinary functions.

Contents: [1. Types and options](#1-types-and-options) · [2. Adapter
operations](#2-adapter-operations) · [3. Input contract](#3-input-contract) · [4. Output
contract](#4-output-contract)

## 1. Types and options

BufferedRequest aliases `http::Request<Vec<u8>>`; BufferedResponse aliases
`http::Response<Vec<u8>>`. The caller owns input buffering and network-read limits. HttpOptions is
Clone/Debug/Default with public policy fields; every ceiling includes equality and zero is strict.

| Field | Type | Default and meaning |
| --- | --- | --- |
| base_url | String | http://unknownhost/; trusted absolute HTTP(S) origin, root path, no credentials/query/fragment. |
| limits | ParseLimits | Lib defaults; 1 MiB, 10,000 entries, depth 64, multipart-part header bytes 8,192; also bounds URI/body separately. |
| response_limits | ParseLimits | Lib defaults for initial Response; a replacement owns its own limits. |
| max_request_bytes | usize | 2 MiB across method, whole URI, every header-name/value occurrence, and body. |
| max_header_bytes | usize | 64 KiB, counting a repeated name on each occurrence. |
| max_headers | usize | 256 header occurrences. |

HttpReply (Debug) has `output: HttpOutput` and `report: HandleReport`. HttpOutput (Debug) is
Response(BufferedResponse), Suppressed, or Unavailable { state: DeliveryState }. Suppressed means
explicit stop; Unavailable means finalization elsewhere supplied no adapter-produced native value.

## 2. Adapter operations

HttpAdapter keeps validated policy private and creates independent output authority per handle. It
creates no socket, runtime, task, or stream. Native publication here is a move of buffered values,
not evidence that bytes reached a client.

### 2.1. new

`HttpAdapter::new(options: HttpOptions) -> Result<Self>`

Validate trusted origin policy before handling any input.

```rust
let adapter = HttpAdapter::new(HttpOptions {
    base_url: "https://profiles.example/".into(),
    ..Default::default()
})?;
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `HttpOptions` | Owned finite policies and trusted origin. |

**Returns** An immutable adapter or base-URL construction/policy Error. Rejects relative, non-web,
credential-bearing, non-root, query/fragment bases. Origin-form input ignores Host/forwarded headers
for authority.

### 2.2. handle

`async fn handle(&self, app: &Server, input: BufferedRequest) -> Result<HttpReply>`

Normalize one buffer, enter one lifecycle, then prepare/publish output once.

```rust
let adapter = HttpAdapter::new(HttpOptions::default())?;
let mut input = BufferedRequest::new(Vec::<u8>::new());
*input.uri_mut() = "/hello".parse()?;
let reply = adapter.handle(&server, input).await?;
assert!(matches!(reply.output, HttpOutput::Response(native) if native.status() == 200));
```

| Parameter | Type | Meaning |
| --- | --- | --- |
| `app` | `&Server` | Fully constructed host with explicit routes. |
| `input` | `BufferedRequest` | Consumed method/URI/multi-value headers/body; no event override. |

**Returns** HttpReply for ordinary normalization/parse/handler/conversion failures; outer Error only
when host reporting cannot start, e.g. correlation exhaustion. Pre-normalization failure yields
public/native 400 with no callbacks. Normalized body parse errors use Error/Response rules. Panics
propagate; dropping stops further work and no background task completes it.

## 3. Input contract

The selector is only checked method plus exact URI pathname. Query/body/header data never selects a
capability. Absolute HTTP(S) targets retain checked authority; origin-form uses configured base.
Path escapes, case, and repeated slashes are retained. A URL normalization change (including dot or
backslash rewrites), credentials, unsupported scheme, non-path target such as `*`/authority-form
CONNECT, binary-unrepresentable header, or multiple content-type occurrences rejects normalization.
Repeated ordinary headers/cookies remain separate inputs; cookie construction follows Lib policy.

JSON (application/json, text/json, +json), urlencoded, and multipart use strict Lib parsing; JSON
requires an object root. Other MIME types keep raw bytes without structured post fields. Loading
stages body and post together; failure installs neither. Absent Content-Type uses text/plain. Input
budgets precede normalization copies; decoded/aggregate budgets still apply afterward.

## 4. Output contract

Text/Bytes preserve exact representation, including empty/non-UTF-8 buffers. Empty/Value output uses
Response::try_to_json public envelope and application/json; explicit response MIME overrides stale
content-type for raw output. Repeated headers and separate Set-Cookie values are not folded. Scalar
session revisions serialize normally, including scoped removals. List cookie revisions have no
adapter codec option: they produce an Output diagnostic naming the cookie and fixed empty 500.

Completed native status must be 200–599. Transfer-Encoding is rejected. Handler Content-Length
cannot control framing: the adapter computes the actual representation length. Status 204/205/304
has no body; 204/304 omit Content-Length, 205 uses zero. HEAD selects only HEAD/Any routes, computes
representation length then suppresses bytes, and never falls back to GET. Forbidden-body statuses
skip representation conversion but still validate relevant framework state/headers/cookies.

Output preflight checks final Response budgets, native status/header legality, and cookie codecs
before publication. Conversion failure records terminal Output failure and publishes one fixed empty
500 with Content-Length: 0; no hook or output retry occurs. Stopped produces Suppressed; foreign
Sent/Attempted/Failed produces Unavailable plus an Output failure. Handler-owned external writes
remain outside adapter authority. Streams, TLS, gateway/network buffering, passthrough native
responses, backpressure, and transport authorization are outside this API.

Related: [HTTP guide](../guides/using-http.md), [lifecycle](../concepts.md), [Lib
parsers](../../../../lib/docs/api/parsers.md).

## Optional stream seam

Feature `http-stream` includes `http` and adds owned one-shot response output.
See the [streaming guide](../guides/streaming-http.md) for a complete producer,
and [streaming reference](streaming-http.md) for HttpResponseExt, StreamLimits,
StreamingHttpAdapter::handle_body, HttpBody, publication and failure rules.
The buffered methods above retain their existing bytes and limits.

The separate [HTTP host](../../../http/docs/README.md) drives real IO,
enforces finite stream/write deadlines, and returns final exchange reports.
Its opt-in facade features are server-http-host and server-http-host-tls;
server-http keeps its buffered-only meaning. No network host is pulled into
this crate by http-stream alone.
