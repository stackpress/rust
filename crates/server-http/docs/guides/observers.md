# Collect final exchange observations

Use an observer to count delivery outcomes without exposing request or error
contents to a logging pipeline. The library supplies fixed-size redacted
metadata and creates no queue or detached sender.

## Attach a nonblocking sink

This sink counts locally completed exchanges without retaining observations:

```rust
use stackpress_server_http::{ExchangeObservation, ExchangeObserver, HttpDelivery, NetworkOptions};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

/// Retain only a count; never block an HTTP cleanup callback.
#[derive(Default)]
struct Completed(AtomicU64);
impl ExchangeObserver for Completed {
    fn observe(&self, observation: ExchangeObservation) {
        // Completion means producer handoff and native flush, not a remote ACK.
        if observation.delivery == HttpDelivery::Completed {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Select observation policy before preparing or binding the shared host.
fn options(counter: Arc<Completed>) -> Result<NetworkOptions, Box<dyn std::error::Error>> {
    Ok(
        NetworkOptions::http("127.0.0.1:8080".parse()?, "http://localhost:8080/")
            .with_observer(counter),
    )
}
```

The application can keep another Arc to read the count. A closure implementing
Fn(ExchangeObservation) + Send + Sync also implements ExchangeObserver.

## Understand delivery timing

Snapshots arrive at connection ownership end, rather than at each live flush.
Callbacks run outside internal locks after active IO/future/admission resources
drop; retained snapshots are bounded by connection/request limits. Caller
retention or forwarding needs its own bound. A callback must be synchronous and
nonblocking.

Observer panic follows ordinary unwinding and is not retried. A task poll panic
drops native work, emits redacted failure metadata, then resumes the original
panic payload for normal supervision. Drop that is already unwinding skips
callbacks to avoid a second panic.

ExchangeObservation contains local IDs, optional status, delivery, handed-off
bytes, elapsed time, optional ErrorKind, and host-report presence. It contains
no URI/path, request headers, cookies, authorization, body, client IP, SQL,
or native source message. Original typed causes remain explicitly available in
[reports](../api/reports.md); default report Debug redacts them.
