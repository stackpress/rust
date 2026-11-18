//! Managed-listener example: SSE greeting and an explicitly Origin-checked WebSocket echo.
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt, StartupContext,
    http::sse::{SseEvent, SseOptions, SseResponseExt},
};
use stackpress_server_http::{
    HttpServer, SseHostLimits, WebSocketHostLimits,
    websocket::{WebSocketOptions, WebSocketResponseExt, origin_allowed},
};
use std::{sync::Arc, time::Duration};

/// Example policy is registered before startup; production apps also select authentication.
struct Origin(String);
/// A finite source demonstrates SSE framing without spawning a producer task.
async fn events(_: &mut Request, response: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    let greeting = SseEvent::data("hello")?.id("1")?;
    response.sse(
        futures_util::stream::iter([Ok(greeting)]),
        SseOptions::default(),
    )?;
    Ok(Flow::Continue)
}
/// Deliberately reject missing/null/cross-origin browser origins before selecting the intent.
async fn websocket(
    request: &mut Request,
    response: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    let expected = invocation.service::<Origin>()?;
    if !origin_allowed(request, &[&expected.0], false, false)? {
        response.status_code(403, None)?;
        return Ok(Flow::Abort);
    }
    response.websocket(
        request,
        WebSocketOptions::default(),
        |mut connection| async move {
            // Real apps recheck message authorization/session expiry at this boundary.
            while let Some(message) = connection.recv().await? {
                connection.send(message).await?;
            }
            Ok(())
        },
    )?;
    Ok(Flow::Continue)
}
/// Read the devmetrics-assigned port, build first, then bind and drain on Ctrl-C or 60 seconds.
#[tokio::main]
async fn main() -> Result<()> {
    let port = std::env::args()
        .nth(1)
        .ok_or_else(|| Error::invalid("pass the managed port"))?
        .parse::<u16>()
        .map_err(|_| Error::invalid("invalid managed port"))?;
    if port == 0 {
        return Err(Error::invalid("managed port must be nonzero"));
    }
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let origin = format!("http://{address}");
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |context: &mut StartupContext| {
        let origin = origin.clone();
        Box::pin(async move {
            context.register(Arc::new(Origin(origin)))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.get("/events", events)?;
    builder.get("/ws", websocket)?;
    let app = Arc::new(builder.build().await.map_err(|failure| failure.error)?);
    let host = HttpServer::new(app, Default::default())?
        .with_sse_limits(SseHostLimits::default())?
        .with_websocket_limits(WebSocketHostLimits::default())?;
    let bound = host.bind_prepared(address).await?;
    println!("ready http://{address}/events; WS endpoint /ws");
    let report = bound.serve_until(async {
        tokio::select! { _=tokio::signal::ctrl_c()=>{}, ()=tokio::time::sleep(Duration::from_secs(60))=>{} }
    }).await;
    if report.failure.is_some() {
        return Err(Error::new("realtime host failed after drain"));
    }
    Ok(())
}
