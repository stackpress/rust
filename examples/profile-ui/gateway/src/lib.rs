//! Example-only buffered HTTP/1 boundary; no socket is opened by this library.
//!
//! Immutable checked assets and generated operations pass through the existing
//! HttpAdapter. Dropping a connection drops its current read/handler future;
//! cancellation never promises rollback or retries completed work.

pub mod assets;
mod options;
mod runner;
pub use options::{GatewayLimits, LaunchOptions};

use http_body_util::{BodyExt, Full};
use hyper::{
    Request, Response,
    body::{Body, Bytes},
    header,
};
use hyper_util::rt::{TokioIo, TokioTimer};
use stackpress_framework_view::BasePath;
use stackpress_server::{
    Server, ServerBuilder,
    http::{HttpAdapter, HttpOptions, HttpOutput},
};
use std::{
    convert::Infallible,
    io,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::watch,
    time::timeout,
};

/// Native wire response remains fully buffered and finite, including static WASM.
pub type WireResponse = Response<Full<Bytes>>;

/// One immutable preview host; each connection owns and cancels its own work.
pub struct Gateway {
    /// Built immutable generated capability/page/asset host.
    app: Server,
    /// Existing conversion/lifecycle/output authority with trusted finite policy.
    adapter: HttpAdapter,
    /// Validated acquisition, admission and deadline bounds.
    limits: GatewayLimits,
    /// Live connection tickets, incremented/decremented exactly once by RAII.
    active: AtomicUsize,
}

impl Gateway {
    /// Build only after manifest/hash validation, before the executable binds a listener.
    ///
    /// # Errors
    /// Returns checked startup/HTTP policy errors; no partially built host is published.
    pub async fn new(
        base: BasePath,
        assets: assets::AssetBundle,
        limits: GatewayLimits,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        limits.validate()?;
        stackpress_example_profile_ui_web::initialize_renderer()?;
        let mut app = ServerBuilder::new();
        stackpress_example_profile_ui_web::configure_application_with_module(
            &mut app,
            base.clone(),
            &assets.module(),
        )?;
        assets.register(&mut app, &base)?;
        let mut options = HttpOptions::default();
        options.limits.max_bytes = limits.body_bytes;
        options.max_header_bytes = limits.header_bytes;
        options.max_headers = limits.headers;
        options.max_request_bytes = limits.body_bytes + limits.header_bytes + 4096;
        // Static WASM is larger than a page. PageRenderer still separately enforces
        // its two-MiB HTML ceiling; only the adapter's static buffer budget expands.
        options.response_limits.max_bytes = assets::MAX_ASSET_BYTES;
        Ok(Self {
            app: app.build().await.map_err(|error| error.error)?,
            adapter: HttpAdapter::new(options)?,
            limits,
            active: AtomicUsize::new(0),
        })
    }

    /// Acquire a bounded body before HttpAdapter; no caller field selects operation authority.
    ///
    /// This service API is also the port-free test/proof entry. Admission failures
    /// close the connection rather than drain an attacker-controlled remainder.
    pub async fn handle<B>(&self, request: Request<B>) -> WireResponse
    where
        B: Body<Data = Bytes> + Unpin,
        B::Error: std::fmt::Display,
    {
        match timeout(self.limits.request_timeout, self.handle_inner(request)).await {
            Ok(response) => response,
            Err(_) => failure(504, "Request timed out"),
        }
    }

    /// Validate native headers, collect finite frames and enter exactly one HttpAdapter lifecycle.
    async fn handle_inner<B>(&self, request: Request<B>) -> WireResponse
    where
        B: Body<Data = Bytes> + Unpin,
        B::Error: std::fmt::Display,
    {
        let headers = request.headers();
        let header_bytes = headers
            .iter()
            .map(|(name, value)| name.as_str().len() + value.len())
            .sum::<usize>();
        if headers.len() > self.limits.headers
            || header_bytes > self.limits.header_bytes
            || request.uri().to_string().len() > 4096
        {
            return failure(431, "Request headers too large");
        }
        let (parts, mut body) = request.into_parts();
        if body.size_hint().lower() > self.limits.body_bytes as u64 {
            return failure(413, "Request body too large");
        }
        let acquire = async {
            let mut bytes = Vec::new();
            while let Some(frame) = body.frame().await {
                let frame = frame.map_err(|_| (400, "Invalid request body"))?;
                match frame.into_data() {
                    Ok(chunk) => {
                        if chunk.len() > self.limits.body_bytes.saturating_sub(bytes.len()) {
                            return Err((413, "Request body too large"));
                        }
                        bytes.extend_from_slice(&chunk);
                    }
                    // No trailer headers bypass the single initial-header budget.
                    Err(_) => return Err((400, "Request trailers are unsupported")),
                }
            }
            Ok(bytes)
        };
        let bytes = match timeout(self.limits.body_timeout, acquire).await {
            Ok(Ok(bytes)) => bytes,
            Ok(Err((code, message))) => return failure(code, message),
            Err(_) => return failure(408, "Request body timed out"),
        };
        let request = Request::from_parts(parts, bytes);
        match self.adapter.handle(&self.app, request).await {
            Ok(reply) => match reply.output {
                HttpOutput::Response(response) => {
                    let (mut parts, body) = response.into_parts();
                    // Demo data is public, but HTML/API responses have no implicit cache.
                    if !parts.headers.contains_key(header::CACHE_CONTROL) {
                        parts.headers.insert(
                            header::CACHE_CONTROL,
                            header::HeaderValue::from_static("no-store"),
                        );
                    }
                    Response::from_parts(parts, Full::new(Bytes::from(body)))
                }
                _ => failure(500, "Output unavailable"),
            },
            Err(_) => failure(500, "Request unavailable"),
        }
    }

    /// Serve one owned HTTP/1 connection, with finite header/body/request/lifetime/drain limits.
    ///
    /// # Errors
    /// Returns parser/I/O/deadline errors. Shutdown stops new requests and drains
    /// in-flight work only until its deadline, then drops the connection future.
    pub async fn serve<I>(
        self: Arc<Self>,
        io: I,
        mut shutdown: watch::Receiver<bool>,
    ) -> io::Result<()>
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        self.active.fetch_add(1, Ordering::SeqCst);
        let _ticket = ConnectionTicket(&self.active);
        let host = Arc::clone(&self);
        let service = hyper::service::service_fn(move |request| {
            let host = Arc::clone(&host);
            async move { Ok::<_, Infallible>(host.handle(request).await) }
        });
        let mut builder = hyper::server::conn::http1::Builder::new();
        builder
            .timer(TokioTimer::new())
            .header_read_timeout(self.limits.header_timeout)
            .max_headers(self.limits.headers)
            .max_buf_size(self.limits.header_bytes.max(8192));
        let connection = builder.serve_connection(TokioIo::new(io), service);
        tokio::pin!(connection);
        tokio::select! {
            result = &mut connection => return result.map_err(io::Error::other),
            _ = async {
                // Release watch's read guard before waiting; an already-stopped
                // receiver must never wait for a second shutdown notification.
                let stopped = *shutdown.borrow();
                if !stopped {
                    let _ = shutdown.changed().await;
                }
            } => {},
            _ = tokio::time::sleep(self.limits.connection_timeout) => {},
        }
        connection.as_mut().graceful_shutdown();
        timeout(self.limits.drain_timeout, &mut connection)
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "connection drain deadline"))?
            .map_err(io::Error::other)
    }

    /// Observe retained connections in cancellation/shutdown tests without exposing host state.
    pub fn active_connections(&self) -> usize {
        self.active.load(Ordering::SeqCst)
    }
}

/// RAII makes cancellation of a pending connection release its admission count.
struct ConnectionTicket<'a>(&'a AtomicUsize);
impl Drop for ConnectionTicket<'_> {
    /// Dropping any completion/error/cancellation path releases the same ticket once.
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Emit only fixed public admission errors; never format body/parser/native causes.
fn failure(code: u16, message: &str) -> WireResponse {
    let reason = hyper::StatusCode::from_u16(code)
        .ok()
        .and_then(|status| status.canonical_reason())
        .unwrap_or("Error");
    let json = serde_json::json!({
        "code": code, "status": reason, "error": message, "errors": {}, "total": 0
    });
    let mut response = Response::new(Full::new(Bytes::from(json.to_string())));
    *response.status_mut() =
        hyper::StatusCode::from_u16(code).unwrap_or(hyper::StatusCode::INTERNAL_SERVER_ERROR);
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    response.headers_mut().insert(
        header::CONNECTION,
        header::HeaderValue::from_static("close"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

#[cfg(test)]
mod tests;
