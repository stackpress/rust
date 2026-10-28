//! Compile-only fixture here; the coordinator runs it through devmetrics for the listener smoke.
use futures_core::Stream;
use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ServerBuilder,
    http::stream::{HttpResponseExt, StreamLimits},
};
use stackpress_server_http::{AssetBundle, AssetLimits, HttpServer, NetworkOptions};
use std::{
    path::PathBuf,
    pin::Pin,
    process::ExitCode,
    sync::Arc,
    task::{Context, Poll},
};

/// Minimal health response reached through the ordinary method/path lifecycle.
async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.set("text/plain; charset=utf-8", Body::Text("healthy\n".into()))?;
    Ok(Flow::Continue)
}
/// One owned demand-driven stream, with no borrowed request/renderer or detached producer task.
struct Greeting(u8);
impl Stream for Greeting {
    type Item = Result<Vec<u8>>;
    /// Each demand yields one bounded chunk; EOF follows without another task/channel.
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let part = self.0;
        self.0 += 1;
        Poll::Ready(match part {
            0 => Some(Ok(b"Stackpress ".to_vec())),
            1 => Some(Ok(b"stream\n".to_vec())),
            _ => None,
        })
    }
}
/// Select an explicit one-shot HTTP capability, published after response hooks finish.
async fn streamed(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.stream(
        "text/plain; charset=utf-8",
        Greeting(0),
        StreamLimits::default(),
    )?;
    Ok(Flow::Continue)
}
/// Validate the managed port before loading assets, building services or binding.
fn port() -> Result<u16> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "--port" {
        return Err(stackpress_lib::Error::invalid(
            "usage: deployment --port <managed-port>",
        ));
    }
    let port = args[1].parse::<u16>().map_err(|cause| {
        stackpress_lib::Error::new("port invalid")
            .with_kind(stackpress_lib::ErrorKind::External)
            .with_cause(cause)
    })?;
    if port == 0 {
        return Err(stackpress_lib::Error::invalid("port must be nonzero"));
    }
    Ok(port)
}
/// Complete app startup before bind, await HTTP drain before dropping process-owned services.
async fn run() -> Result<()> {
    let port = port()?;
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/assets");
    let bundle = AssetBundle::load(&root, root.join("manifest.json"), AssetLimits::default())?;
    let mut builder = ServerBuilder::new();
    builder.get("/health", health)?;
    builder.get("/stream", streamed)?;
    bundle.register(&mut builder)?;
    let app = Arc::new(builder.build().await.map_err(|failure| failure.error)?);
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    // Install application-owned shutdown input before publishing listener readiness.
    let stopping = shutdown_signal()?;
    let bound = HttpServer::bind(
        Arc::clone(&app),
        NetworkOptions::http(address, format!("http://{address}/")),
    )
    .await?;
    println!("ready http://{}/health", bound.local_addr()?);
    let mut stop_error = None;
    let report = bound
        .serve_until(async {
            stop_error = stopping.await.err();
        })
        .await;
    // Close database/process services here, while the runtime is alive and HTTP tasks are reaped.
    drop(app);
    if let Some(cause) = stop_error {
        return Err(
            stackpress_lib::Error::new("process shutdown input failed after drain")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(cause),
        );
    }
    if report.failure.is_some() {
        return Err(stackpress_lib::Error::new(
            "HTTP serving failed after drain",
        ));
    }
    println!("drained {} connections", report.completed);
    Ok(())
}
/// Process signals belong to the application; registration errors retain their native cause.
fn shutdown_signal() -> Result<impl std::future::Future<Output = std::io::Result<()>>> {
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|cause| {
        stackpress_lib::Error::new("process signal registration failed")
            .with_kind(stackpress_lib::ErrorKind::External)
            .with_cause(cause)
    })?;
    Ok(async move {
        #[cfg(unix)]
        {
            tokio::select! {result=tokio::signal::ctrl_c()=>result,_=terminate.recv()=>Ok(())}
        }
        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c().await
        }
    })
}
/// This binary is never run by automated proofs; listener execution belongs to managed smoke.
#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("deployment failed ({:?})", error.kind());
            ExitCode::FAILURE
        }
    }
}
