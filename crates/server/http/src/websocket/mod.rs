//! Opt-in, bounded WebSocket connections through ordinary ordered GET routes.
//!
//! Applications own Origin, authentication, CSRF and per-message authorization. Cookie-bearing
//! browser handshakes can be sent cross-site: absent app Origin checks can permit cross-site
//! WebSocket hijacking. CORS does not authorize a WebSocket, and a GET CSRF hook may do nothing.
//! No automatic message-to-event mapping, session persistence or credential refresh is implied.
//! Proxy deployments must forward Upgrade/Connection and select suitable idle timeouts. HTTPS
//! uses the host's existing Rustls transport; no client connector or compression is enabled.

mod connection;
mod driver;
mod handshake;
use crate::{HttpServer, WebSocketHostLimits, realtime::SessionPermit};
pub use connection::{WsClose, WsCloseInfo, WsConnection, WsMessage, WsRequestSnapshot};
use stackpress_lib::{Body, CookieOptions, Error, Request, Response, Result, SharedNative, Value};
use stackpress_server::http::upgrade::prepare_headers;
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};

/// Private owned callback; borrowed/non-Send request and invocation captures cannot escape.
type Callback =
    Box<dyn FnOnce(WsConnection) -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send>;
/// Explicit snapshot/subprotocol selection, independent of host admission policy.
#[derive(Clone, Default, Debug)]
pub struct WebSocketOptions {
    protocol: Option<String>,
    headers: Vec<String>,
    sessions: Vec<String>,
}
impl WebSocketOptions {
    /// Select one offered subprotocol; invalid/unoffered values fail before 101.
    pub fn with_subprotocol(mut self, protocol: impl AsRef<str>) -> Result<Self> {
        let protocol = protocol.as_ref();
        if !handshake::token(protocol) {
            return Err(Error::invalid("invalid WebSocket subprotocol"));
        }
        self.protocol = Some(protocol.into());
        Ok(self)
    }
    /// Explicitly select <=32 header names and <=32 session keys for a <=16 KiB snapshot.
    pub fn with_snapshot(mut self, headers: &[&str], sessions: &[&str]) -> Result<Self> {
        if headers.len() > 32
            || sessions.len() > 32
            || headers
                .iter()
                .chain(sessions)
                .any(|s| s.len() > 256 || s.is_empty())
        {
            return Err(Error::invalid("invalid WebSocket snapshot allowlist"));
        }
        for name in headers {
            http::HeaderName::from_bytes(name.as_bytes())
                .map_err(|_| Error::invalid("invalid snapshot header name"))?;
        }
        self.headers = headers.iter().map(|s| s.to_ascii_lowercase()).collect();
        self.sessions = sessions.iter().map(|s| (*s).into()).collect();
        Ok(self)
    }
}
/// Private descriptor identity carries no admission permit, even through a ResponseView clone.
struct Intent {
    callback: Mutex<Option<Callback>>,
    snapshot: Arc<WsRequestSnapshot>,
    key: String,
    protocol: Option<String>,
}
/// An exclusive selected handshake; it never calls the callback until successful IO handoff.
pub(crate) struct Prepared {
    pub(crate) response: http::Response<()>,
    pub(crate) session: Option<Session>,
}
/// Owned action awaited inside the serving future; no detached upgrade task exists.
pub(crate) struct Session {
    callback: Callback,
    snapshot: Arc<WsRequestSnapshot>,
    limits: WebSocketHostLimits,
    published: tokio::time::Instant,
    _permit: SessionPermit,
}
/// Select a one-shot WebSocket intent through the ordinary response body seam.
pub trait WebSocketResponseExt {
    /// Capture only owned bounded state; hooks may replace or deny before native publication.
    /// Buffered/non-upgrade adapters reject this descriptor without invoking the callback.
    ///
    /// Borrowed state cannot escape the HTTP lifecycle:
    /// ```compile_fail
    /// use stackpress_lib::{Request,Response};
    /// use stackpress_server_http::websocket::{WebSocketOptions,WebSocketResponseExt};
    /// fn borrowed(res:&mut Response,req:&Request,text:&str) {
    ///     let _=res.websocket(req,WebSocketOptions::default(),move |_| async move {
    ///         let _=text.len();Ok(())
    ///     });
    /// }
    /// ```
    /// Non-Send service captures cannot enter the native serving future:
    /// ```compile_fail
    /// use stackpress_lib::{Request,Response};
    /// use stackpress_server_http::websocket::{WebSocketOptions,WebSocketResponseExt};
    /// fn non_send(res:&mut Response,req:&Request) {
    ///     let state=std::rc::Rc::new(String::from("private"));
    ///     let _=res.websocket(req,WebSocketOptions::default(),move |_| async move {
    ///         drop(state);Ok(())
    ///     });
    /// }
    /// ```
    fn websocket<F, Fut>(
        &mut self,
        request: &Request,
        options: WebSocketOptions,
        connect: F,
    ) -> Result<&mut Self>
    where
        F: FnOnce(WsConnection) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static;
}
impl WebSocketResponseExt for Response {
    /// Check the selected input/snapshot before retaining the owned callback.
    fn websocket<F, Fut>(
        &mut self,
        request: &Request,
        options: WebSocketOptions,
        connect: F,
    ) -> Result<&mut Self>
    where
        F: FnOnce(WsConnection) -> Fut + Send + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        // Normalized headers are revalidated; the host later matches the original trusted key.
        let mut native = http::Request::builder()
            .method(request.method.as_str())
            .uri("/")
            .body(())
            .map_err(|e| crate::external("WebSocket request conversion failed", e))?;
        for (name, values) in request.headers.entries() {
            for value in values {
                native.headers_mut().append(
                    http::HeaderName::from_bytes(name.as_bytes())
                        .map_err(|e| crate::external("WebSocket header name invalid", e))?,
                    http::HeaderValue::from_str(value)
                        .map_err(|e| crate::external("WebSocket header value invalid", e))?,
                );
            }
        }
        let checked = handshake::validate(&native)?;
        if options
            .protocol
            .as_ref()
            .is_some_and(|p| !checked.protocols.contains(p))
        {
            return Err(Error::invalid("unoffered WebSocket subprotocol"));
        }
        let snapshot = Arc::new(WsRequestSnapshot::capture(request, &options)?);
        self.set(
            "application/websocket",
            Body::Value(Value::Shared(SharedNative::new(Intent {
                callback: Mutex::new(Some(Box::new(move |connection| {
                    Box::pin(connect(connection))
                }))),
                snapshot,
                key: checked.key,
                protocol: options.protocol,
            }))),
        )?;
        // Response::set selects 200; the protocol intent deliberately selects 101 afterward.
        self.status_code(101, None)
    }
}
/// Optional exact web-Origin comparison; absent/null policies must be chosen by the app.
/// No guard is installed automatically. Inputs and allowlist are bounded before parsing.
pub fn origin_allowed(
    request: &Request,
    allowed: &[&str],
    allow_absent: bool,
    allow_null: bool,
) -> Result<bool> {
    if allowed.len() > 32 {
        return Err(Error::invalid("Origin allowlist too large"));
    }
    let values = request.headers.get_all("origin").unwrap_or_default();
    if values.is_empty() {
        return Ok(allow_absent);
    }
    if values.len() != 1 || values[0].len() > 2048 {
        return Err(Error::invalid("invalid Origin header"));
    }
    if values[0] == "null" {
        return Ok(allow_null);
    }
    let origin = origin(&values[0])?;
    for candidate in allowed {
        if origin == self::origin(candidate)? {
            return Ok(true);
        }
    }
    Ok(false)
}
/// Parse a bounded scheme/host/port origin without allowing paths, query, credentials or fragments.
fn origin(value: &str) -> Result<(String, String, u16)> {
    if value.len() > 2048 || value.contains(['@', '#', '?']) {
        return Err(Error::invalid("invalid web Origin"));
    }
    let uri = value
        .parse::<http::Uri>()
        .map_err(|_| Error::invalid("invalid web Origin"))?;
    let scheme = uri
        .scheme_str()
        .ok_or_else(|| Error::invalid("Origin scheme missing"))?
        .to_ascii_lowercase();
    if !matches!(scheme.as_str(), "http" | "https")
        || uri.path_and_query().is_some_and(|s| s.as_str() != "/")
    {
        return Err(Error::invalid("invalid web Origin scheme/path"));
    }
    let raw_host = uri
        .host()
        .ok_or_else(|| Error::invalid("Origin host missing"))?;
    // http::Uri permits an authority with a non-u16 port, reporting port_u16 as None.
    // Only a truly absent port may receive the web scheme default.
    if uri.authority().is_some_and(|a| a.as_str() != raw_host) && uri.port_u16().is_none() {
        return Err(Error::invalid("invalid Origin port"));
    }
    let host = raw_host.to_ascii_lowercase();
    Ok((
        scheme.clone(),
        host,
        uri.port_u16()
            .unwrap_or(if scheme == "https" { 443 } else { 80 }),
    ))
}
/// Classify candidates only; strict validation runs before any lifecycle callbacks.
pub(crate) fn candidate<B>(request: &http::Request<B>) -> bool {
    request.headers().contains_key("upgrade")
        || request.headers().contains_key("sec-websocket-key")
        || request.headers().contains_key("sec-websocket-version")
}
pub(crate) use handshake::validate;
impl HttpServer {
    /// Preflight the final private descriptor, all headers/cookies, trusted key and admission.
    pub(crate) fn prepare_websocket(
        &self,
        response: &Response,
        cookie: &CookieOptions,
        checked: Option<&handshake::Handshake>,
        stopping: bool,
    ) -> Result<Option<Prepared>> {
        let intent = match response.body() {
            Body::Value(Value::Shared(value)) => value.downcast_ref::<Intent>(),
            _ => None,
        };
        let Some(intent) = intent else {
            return Ok(None);
        };
        // A response hook may deny by status without needing to know the descriptor internals.
        if let Some(status) = response.status().filter(|s| s.code != 101) {
            let headers = prepare_headers(response, cookie)?;
            if !(200..=599).contains(&status.code) {
                return Err(Error::invalid("invalid WebSocket denial status"));
            }
            let mut denied = http::Response::new(());
            *denied.status_mut() = http::StatusCode::from_u16(status.code)
                .map_err(|e| crate::external("invalid denial status", e))?;
            *denied.headers_mut() = headers;
            denied.headers_mut().remove("content-length");
            denied
                .headers_mut()
                .insert("connection", http::HeaderValue::from_static("close"));
            drop(
                intent
                    .callback
                    .lock()
                    .map_err(|_| Error::new("WebSocket ownership lock poisoned"))?
                    .take(),
            );
            return Ok(Some(Prepared {
                response: denied,
                session: None,
            }));
        }
        let checked = checked
            .ok_or_else(|| Error::invalid("WebSocket intent lacks a valid native handshake"))?;
        if checked.key != intent.key
            || intent
                .protocol
                .as_ref()
                .is_some_and(|p| !checked.protocols.contains(p))
        {
            return Err(Error::invalid(
                "WebSocket intent does not match native request",
            ));
        }
        let mut headers = prepare_headers(response, cookie)?;
        // The host regenerates protected fields; application hooks cannot forge framing/accept.
        for name in [
            "content-length",
            "transfer-encoding",
            "sec-websocket-extensions",
            "sec-websocket-protocol",
            "connection",
            "upgrade",
            "sec-websocket-accept",
        ] {
            headers.remove(name);
        }
        headers.insert("connection", http::HeaderValue::from_static("Upgrade"));
        headers.insert("upgrade", http::HeaderValue::from_static("websocket"));
        headers.insert("sec-websocket-accept", checked.accept.clone());
        if let Some(protocol) = &intent.protocol {
            headers.insert(
                "sec-websocket-protocol",
                http::HeaderValue::from_str(protocol)
                    .map_err(|e| crate::external("WebSocket protocol conversion failed", e))?,
            );
        }
        let limits = match &self.websocket {
            Some(limits) if !stopping => limits.clone(),
            _ => return Ok(Some(denied())),
        };
        let permit = match self.reserve_session(&self.ws_slots, true) {
            Ok(permit) => permit,
            Err(_) => return Ok(Some(denied())),
        };
        let callback = intent
            .callback
            .lock()
            .map_err(|_| Error::new("WebSocket ownership lock poisoned"))?
            .take()
            .ok_or_else(|| Error::new("WebSocket callback already consumed"))?;
        let mut response = http::Response::new(());
        *response.status_mut() = http::StatusCode::SWITCHING_PROTOCOLS;
        *response.headers_mut() = headers;
        Ok(Some(Prepared {
            response,
            session: Some(Session {
                callback,
                snapshot: Arc::clone(&intent.snapshot),
                limits,
                published: tokio::time::Instant::now(),
                _permit: permit,
            }),
        }))
    }
}
/// Reject native admission without executing or polling the application callback.
fn denied() -> Prepared {
    let mut response = http::Response::new(());
    *response.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
    response
        .headers_mut()
        .insert("connection", http::HeaderValue::from_static("close"));
    response
        .headers_mut()
        .insert("content-length", http::HeaderValue::from_static("0"));
    Prepared {
        response,
        session: None,
    }
}
impl Session {
    /// Begin the finite session clock at native 101 publication, including handoff time.
    pub(crate) fn commit(&mut self) {
        self.published = tokio::time::Instant::now();
    }
    /// Drive the handed-off IO and callback as children of the serving future.
    pub(crate) async fn run<I, F>(
        self,
        io: I,
        stop: F,
        grace: std::time::Duration,
        stats: crate::RealtimeStats,
    ) -> driver::Outcome
    where
        I: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let mut limits = self.limits;
        limits.age = limits.age.saturating_sub(self.published.elapsed());
        driver::run(io, self.callback, self.snapshot, limits, stop, grace, stats).await
    }
}

#[cfg(test)]
mod tests;
