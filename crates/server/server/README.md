# stackpress-server

**Beta:** 0.2.x is the supported release series; 0.1.x users should upgrade. The API may still change before 1.0.

Compile-time plugins configure a transport-neutral Stackpress host. Top-level handling runs the request, operation, response and error lifecycle; nested dispatch stays explicit.

```rust
use stackpress_server::{BuildResult, ServerBuilder};
# async fn example() -> BuildResult<()> {
let server = ServerBuilder::new().build().await?;
# let _ = server;
# Ok(())
# }
```

Building freezes registrations. The caller supplies the runtime and transport; construction starts no listener.

Licence: [Apache-2.0](LICENSE).

Optional `http-sse` builds on `http-stream` with checked `SseEvent`,
`SseResponseExt::sse`, bounded `channel`, `last_event_id`, and `SseDriver`.
It adds no Tokio, Hyper or WebSocket dependency. See the
[SSE guide](docs/guides/sse.md). The sibling `http-upgrade` feature supplies
`NativeHttpOutput`/`NativeHttpReply` and preflight methods for native adapters;
published streaming enums and extension-trait requirements remain unchanged.
