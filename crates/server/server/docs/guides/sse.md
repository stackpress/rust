# Runtime-neutral SSE

With `http-sse`, a GET handler can install a stream of `Result<SseEvent>` using
`SseResponseExt::sse(events, SseOptions::default())`. Events own checked UTF-8 data;
CRLF/bare CR normalize to LF and every data line gets its own prefix. Cursor and
event-name fields reject CR/LF injection; IDs also reject NUL. Limits charge the
complete encoded event before allocation. Retry values are decimal milliseconds.

```rust
use stackpress_server::http::sse::{channel, SseEvent, SseOptions, SseResponseExt};
use stackpress_lib::Response;
# fn example() -> stackpress_lib::Result<()> {
let (sender, events) = channel(8, 65_536)?;
sender.try_send(SseEvent::data("ready")?.id("cursor-1")?)?;
let mut response = Response::new();
response.sse(events, SseOptions::default())?;
drop(sender); // Consumer drains the accepted event, then reaches EOF.
# Ok(())
# }
```

The channel is optional and runtime-neutral; any owned Send stream works. Full
queues return an error immediately. A response descriptor is taken only once,
after lifecycle hooks finish. Buffered adapters reject it without polling.
In-process streaming yields application events without native timers.
`SseDriver` accepts an injected elapsed-time stream to prove heartbeat, age and
byte rotation without an executor. Native `stackpress-server-http/sse` owns actual
write monitoring and separate session clocks; see its realtime guide.

Use bounded `last_event_id` to inspect a single resume header. Missing and empty
are distinct; duplicate/oversized or CR/LF/NUL-bearing values fail. Apps own
cursor interpretation, authorization, replay storage and duplicate handling.
