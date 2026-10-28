# Serve a Stackpress application

A built Server can already handle operations in process. Add a network host
when clients need HTTP routes, then await its drain before closing shared
resources. This walkthrough produces a plain-text health endpoint using the
same lifecycle as any other route.

## Prepare the application

Use Rust 1.96 and edition 2024. In a consumer manifest, add these dependencies;
resolve the paths relative to that manifest (shown here from the repository root):

```toml
[dependencies]
stackpress-lib = { path = "crates/lib" }
stackpress-server = { path = "crates/server/server" }
stackpress-server-http = { path = "crates/server/http" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "signal"] }
```

The HTTP host selects `stackpress-server/http-stream` itself. Direct use of the
workspace facade instead needs its opt-in `server-http-host` feature.

## Write the main

The handler sets a real text representation. Startup completes before bind;
`serve_until` owns connection supervision until the application's stop input
resolves. This program is compiled, never run as a documentation test:

```rust
use stackpress_lib::{Body, Error, ErrorKind, Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;
use stackpress_server_http::{HttpServer, NetworkOptions};
use std::sync::Arc;

/// Return text through the ordinary application lifecycle.
async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.set("text/plain; charset=utf-8", Body::Text("healthy\n".into()))?;
    Ok(Flow::Continue)
}

/// Register SIGTERM before bind; keep signal handling inside the awaited stop future.
fn shutdown_signal() -> Result<impl std::future::Future<Output = std::io::Result<()>>> {
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|cause| {
        Error::new("signal registration failed")
            .with_kind(ErrorKind::External)
            .with_cause(cause)
    })?;
    Ok(async move {
        #[cfg(unix)]
        {
            tokio::select! {
                result = tokio::signal::ctrl_c() => result,
                _ = terminate.recv() => Ok(()),
            }
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await
        }
    })
}

/// Await network cleanup while the runtime and process services remain alive.
#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = ServerBuilder::new();
    builder.get("/health", health)?;
    let app = Arc::new(builder.build().await.map_err(|failure| failure.error)?);
    let stop = shutdown_signal()?;
    let address = "127.0.0.1:8080".parse().map_err(|cause| {
        Error::new("bind address invalid")
            .with_kind(ErrorKind::External)
            .with_cause(cause)
    })?;
    let bound = HttpServer::bind(
        Arc::clone(&app),
        NetworkOptions::http(address, "http://localhost:8080/"),
    )
    .await?;
    let mut signal_error = None;
    let report = bound
        .serve_until(async {
            signal_error = stop.await.err();
        })
        .await;
    // Explicitly close any retained database/process services here, even on serving failure.
    drop(app);
    if let Some(cause) = report.failure {
        return Err(Error::new("HTTP serving failed after drain")
            .with_kind(ErrorKind::External)
            .with_cause(cause));
    }
    if let Some(cause) = signal_error {
        return Err(Error::new("stop input failed after drain")
            .with_kind(ErrorKind::External)
            .with_cause(cause));
    }
    Ok(())
}
```

`NetworkOptions::http` selects plaintext and configures the trusted public
origin. Send `Host: localhost:8080` for this origin; the native bind address
and allowed public authority serve different purposes. `bind` validates policy
before opening TCP and does not serve until `serve_until` is polled.

## Verify with managed development

In this repository, use the [devmetrics recipe](guides/development.md) with the
compiled [deployment example](../examples/deployment.rs). It takes the assigned
`--port`, serves `/health`, `/assets/hello.txt`, and `/stream`, and handles
SIGTERM as well as Ctrl-C on Unix. Do not launch a listener directly.

Request `/health` at devmetrics' reported URL. Expect status 200,
`text/plain; charset=utf-8`, and `healthy` followed by a newline. Stop through
devmetrics and wait for the drain message and process exit.

## What happened

ServerBuilder selected routes and finished shared-service startup once. The
HTTP host then validated authority, limits, and optional deployment policy.
Each accepted request entered one ordinary lifecycle before the adapter
published native output. The application awaited all owned HTTP task cleanup
before reaching its service-close point.

You now have the production bind/serve path and explicit cleanup order. Read
[concepts](concepts.md), then add [TLS](guides/tls.md) or a deliberate
[proxy policy](guides/proxy.md) for your deployment.
