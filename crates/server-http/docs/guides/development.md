# Run the example through devmetrics

Use the managed example to inspect real HTTP behavior locally. Devmetrics
assigns a port in 3000–3020, records ownership, and stops the process after two
hours by default. The example accepts `--port` so its bind address and public
origin match the assigned port.

## Start from the workspace root

Check `devmetrics list` first and preserve unrelated servers. Then run:

```sh
devmetrics start --wait 120 --summary "Stackpress HTTP development" -- 'CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo run --locked -p stackpress-server-http --example deployment -- --port {port}'
```

Use the reported URL and expiry. The readiness wait allows a build; inspect
`devmetrics logs <port>` if it reports that the listener is not ready. Never
start this repository's development listeners with a bare cargo run, background
shell, or unmanaged wrapper. The example binds loopback only.

## Inspect the three routes

Request `/health`: expect 200, text/plain and `healthy` with a newline.
GET `/assets/hello.txt` returns the checked manifest bytes and an ETag; HEAD
returns the length without bytes. Send the ETag as If-None-Match to receive 304.
GET `/stream` returns `Stackpress stream` with a newline using chunked output.

The [source](../../examples/deployment.rs) keeps asset loading and service
startup before bind. It handles Ctrl-C and, on Unix, SIGTERM in an awaited stop
future, then reports drain completion. Use `devmetrics stop <port>` for the
assigned instance and wait for exit; do not stop unrelated instances.

## Automated verification

Repository behavioral proofs use Tokio duplex IO and the generic acceptor,
including TLS handshakes, limits, pipelines, cancellation, and draining.
Hang-class cases run in child processes with parent watchdogs that kill/reap.
Listener examples and documentation mains are compiled without execution.
Real TCP smoke is a separate managed closeout step; compile checks alone do
not establish socket or process-signal behavior.

See [shutdown](shutdown.md) for service cleanup and [limits](../api/limits.md)
for timeout/status details when exercising slow clients.
