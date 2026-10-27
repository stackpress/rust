//! One supervised Hyper connection: finite phases, strict boundaries and ordered reuse.

use crate::{
    ConnectionReport, HttpDelivery, HttpRequestContext, HttpServer,
    body::TrackedBody,
    external,
    io::SharedIo,
    monitor::{Monitor, Phase},
    report::{self, Exchange, ExchangeSlot, lock},
};
use http::header::{CONNECTION, CONTENT_LENGTH, TRANSFER_ENCODING};
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use stackpress_lib::{Error, Result};
use stackpress_server::{HandleOptions, http::stream::StreamingHttpOutput};
use std::{
    future::{Future, poll_fn},
    net::SocketAddr,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::io::{AsyncRead, AsyncWrite};

impl HttpServer {
    /// Drive one admitted IO; scopes drop Hyper, body, active loader and IO before final reporting.
    pub(crate) async fn serve_connection<I, F>(
        &self,
        io: I,
        scheme: &'static str,
        stop: F,
        context: crate::ConnectionContext,
    ) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let crate::ConnectionContext {
            peer,
            started,
            slots,
        } = context;
        let monitor = Monitor::new(&self.limits);
        let io = SharedIo::new(io, Arc::clone(&monitor));
        let raw = Arc::clone(&io.io);
        let guard = ConnectionCancellation(Arc::clone(&slots));
        let mut shutdown = false;
        let mut forced = false;
        let mut timeout_phase = None;
        let result;
        {
            let registry = Arc::clone(&slots);
            let progress = Arc::clone(&monitor);
            let service = service_fn(move |request| {
                let registry = Arc::clone(&registry);
                let progress = Arc::clone(&progress);
                async move {
                    // Hyper may prepare a successor early, but no successor callback passes the prior flush.
                    progress.turn().await;
                    let slot = {
                        let mut registry = lock(&registry);
                        let slot = Arc::new(Mutex::new(Exchange::new(registry.len() as u64 + 1)));
                        registry.push(Arc::clone(&slot));
                        slot
                    };
                    progress.begin(Arc::clone(&slot));
                    self.handle_request(request, peer, scheme, slot, progress)
                        .await
                }
            });
            let mut builder = http1::Builder::new();
            builder
                .keep_alive(true)
                .half_close(false)
                .max_headers(self.limits.max_headers)
                .max_buf_size(self.limits.parser_buffer_bytes)
                .timer(crate::timer::HeadTimer::new(
                    Arc::clone(&monitor),
                    self.limits.head_idle_timeout,
                ))
                .header_read_timeout(self.limits.head_timeout);
            let connection = builder.serve_connection(TokioIo::new(io), service);
            tokio::pin!(connection);
            tokio::pin!(stop);
            let lifetime = tokio::time::sleep_until(started + self.limits.connection_lifetime);
            tokio::pin!(lifetime);
            result = tokio::select! {
                biased;
                ()=&mut stop=>{
                    shutdown=true;connection.as_mut().graceful_shutdown();
                    drain(&mut connection,&monitor,self.limits.shutdown_grace,&mut forced,&mut timeout_phase).await
                }
                ()=&mut lifetime=>{
                    connection.as_mut().graceful_shutdown();
                    drain(&mut connection,&monitor,self.limits.shutdown_grace,&mut forced,&mut timeout_phase).await
                }
                phase=monitor.expired()=>{
                    timeout_phase=Some(phase);
                    if phase==Phase::Idle {Ok(())}else{Err(deadline_error("HTTP phase deadline exceeded"))}
                }
                result=&mut connection=>result.map_err(hyper_error),
            };
        }
        // Hyper has relinquished IO before timeout cleanup considers native rejection bytes.
        if let Some(phase) = timeout_phase {
            expire(
                phase,
                &raw,
                &monitor,
                &slots,
                self.limits.write_idle_timeout,
            )
            .await;
        }
        drop(guard);
        drop(raw);
        report::finish(slots, result, shutdown, forced)
    }
    /// Reject transport policy before normalization, then load once and run the ordinary G12/G17 path.
    async fn handle_request(
        &self,
        request: http::Request<Incoming>,
        peer: SocketAddr,
        scheme: &'static str,
        slot: ExchangeSlot,
        monitor: Arc<Monitor>,
    ) -> Result<http::Response<TrackedBody>> {
        let index = lock(&slot).report.exchange_id as usize;
        if index > self.limits.max_requests {
            return Err(Error::new("HTTP connection reuse exhausted"));
        }
        let has_transfer = request.headers().contains_key(TRANSFER_ENCODING);
        let head_bytes = crate::input::head_bytes(&request);
        let rejection = crate::input::check_head(&request, &self.limits, head_bytes);
        if let Some((status, message)) = rejection {
            return Ok(reject(status, message, slot, monitor));
        }
        // Deployment authority/trust checks precede normalization, body loading and all callbacks.
        let effective = match &self.policy {
            Some(policy) => match policy.check(request.uri(), request.headers(), peer) {
                Ok(effective) => effective,
                Err(error) => return Ok(reject_error(400, error.with_code(400), slot, monitor)),
            },
            None => crate::proxy::EffectiveRequest {
                origin: self.origin.clone(),
                client: None,
            },
        };
        let close = Arc::new(AtomicBool::new(
            has_transfer || index == self.limits.max_requests,
        ));
        let cancellation = lock(&slot).cancellation.clone();
        let options = HandleOptions {
            adapter_context: Some(Arc::new(HttpRequestContext {
                peer,
                scheme,
                public_origin: effective.origin.clone(),
                cancellation,
                forwarded_client: effective.client,
            })),
        };
        let known_length = request
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|s| s.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let (parts, mut body) = request.into_parts();
        let close_on_error = Arc::clone(&close);
        let progress = Arc::clone(&monitor);
        // A validated forwarded origin affects URL metadata only; native path/operation stays exact.
        let selected;
        let adapter = if effective.origin == self.origin {
            &self.adapter
        } else {
            let mut options = self.http_options.clone();
            options.base_url = effective.origin;
            selected = stackpress_server::http::stream::StreamingHttpAdapter::new(options)?;
            &selected
        };
        let adapter = adapter.handle_body(&self.app, parts, options, || async move {
            let result =
                crate::input::load(&mut body, &self.limits, head_bytes, known_length).await;
            if result.is_err() {
                close_on_error.store(true, Ordering::Release);
            }
            progress.lifecycle();
            result
        });
        let reply = adapter.await?;
        lock(&slot).report.host = Some(reply.report);
        match reply.output {
            StreamingHttpOutput::Response(mut native) => {
                if close.load(Ordering::Acquire) {
                    native
                        .headers_mut()
                        .insert(CONNECTION, http::HeaderValue::from_static("close"));
                }
                {
                    let mut state = lock(&slot);
                    state.report.delivery = HttpDelivery::Published;
                    state.report.status = Some(native.status().as_u16());
                }
                monitor.output(native.body().limits());
                Ok(native.map(|source| TrackedBody::new(source, slot, monitor)))
            }
            StreamingHttpOutput::Suppressed => {
                lock(&slot).report.delivery = HttpDelivery::Suppressed;
                Err(Error::new("HTTP output suppressed"))
            }
            StreamingHttpOutput::Unavailable { .. } => {
                lock(&slot).fail(Arc::new(Error::new("HTTP output unavailable")));
                Err(Error::new("HTTP output unavailable"))
            }
        }
    }
}
/// Drain a graceful connection within one finite absolute budget.
async fn drain<F: Future<Output = std::result::Result<(), hyper::Error>>>(
    connection: &mut Pin<&mut F>,
    monitor: &Monitor,
    grace: std::time::Duration,
    forced: &mut bool,
    timeout_phase: &mut Option<Phase>,
) -> Result<()> {
    // Grace is an additional upper bound; it never replaces an existing phase deadline.
    tokio::select! {
        biased;
        phase=monitor.expired()=>{
            *timeout_phase=Some(phase);
            if phase==Phase::Idle {Ok(())}else{Err(deadline_error("HTTP phase deadline exceeded during drain"))}
        }
        result=tokio::time::timeout(grace,connection)=>match result {
            Ok(result)=>result.map_err(|cause|external("HTTP connection drain failed",cause)),
            Err(cause)=>{*forced=true;Err(external("HTTP graceful shutdown deadline exceeded",cause))}
        }
    }
}

/// Fixed empty native response at the pre-normalization boundary, closing unread input.
fn reject(
    status: u16,
    message: &str,
    slot: ExchangeSlot,
    monitor: Arc<Monitor>,
) -> http::Response<TrackedBody> {
    reject_error(status, boundary_error(status, message), slot, monitor)
}
/// Retain the first typed policy cause while the native wire response stays fixed and empty.
fn reject_error(
    status: u16,
    error: Error,
    slot: ExchangeSlot,
    monitor: Arc<Monitor>,
) -> http::Response<TrackedBody> {
    {
        let mut state = lock(&slot);
        state.report.delivery = HttpDelivery::RejectedBeforeHost;
        state.report.status = Some(status);
        state.report.failure = Some(Arc::new(error));
    }
    monitor.output(None);
    let mut response = http::Response::new(TrackedBody::empty(slot, monitor));
    *response.status_mut() = http::StatusCode::from_u16(status).expect("fixed boundary status");
    response
        .headers_mut()
        .insert(CONNECTION, http::HeaderValue::from_static("close"));
    response
        .headers_mut()
        .insert(CONTENT_LENGTH, http::HeaderValue::from_static("0"));
    response
}
/// Deadline cleanup never retries a published response or waits indefinitely on a stopped peer.
async fn write_timeout<I: AsyncWrite + Unpin>(io: &Arc<Mutex<I>>, timeout: std::time::Duration) {
    let bytes = b"HTTP/1.1 408 Request Timeout\r\nConnection: close\r\nContent-Length: 0\r\n\r\n";
    let write = async {
        let mut offset = 0;
        while offset < bytes.len() {
            let count =
                poll_fn(|cx| Pin::new(&mut *lock(io)).poll_write(cx, &bytes[offset..])).await?;
            if count == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::WriteZero));
            }
            offset += count;
        }
        poll_fn(|cx| Pin::new(&mut *lock(io)).poll_flush(cx)).await?;
        poll_fn(|cx| Pin::new(&mut *lock(io)).poll_shutdown(cx)).await
    };
    let _ = tokio::time::timeout(timeout, write).await;
}
/// Scope guard marks cancellation even if the serving future or handler unwinds.
struct ConnectionCancellation(Arc<Mutex<Vec<ExchangeSlot>>>);
impl Drop for ConnectionCancellation {
    /// Cleanup performs only bounded diagnostic updates, never user callbacks under a lock.
    fn drop(&mut self) {
        for slot in lock(&self.0).iter() {
            lock(slot).cancellation.cancel();
        }
    }
}
/// Public error bodies remain sanitized; the status is selected only at this native boundary.
fn boundary_error(status: u16, message: &str) -> Error {
    Error::invalid(message).with_code(status)
}
/// Lifecycle/output deadlines are cancellation, not another error-hook dispatch.
fn deadline_error(message: &str) -> Error {
    Error::new(message)
        .with_kind(stackpress_lib::ErrorKind::External)
        .with_code(408)
}

/// Preserve parser error identity while identifying its native-boundary status for reporting.
fn hyper_error(cause: hyper::Error) -> Error {
    let code = if cause.is_parse_too_large() {
        431
    } else if cause.is_parse() {
        400
    } else if cause.is_timeout() {
        408
    } else {
        500
    };
    external("HTTP connection failed", cause).with_code(code)
}

/// Cancel one phase and record one final observation after the native connection was dropped.
async fn expire<I: AsyncWrite + Unpin>(
    phase: Phase,
    raw: &Arc<Mutex<I>>,
    monitor: &Arc<Monitor>,
    slots: &Arc<Mutex<Vec<ExchangeSlot>>>,
    write_idle: std::time::Duration,
) {
    let can_reject = {
        let state = lock(&monitor.state);
        !state.wrote && matches!(phase, Phase::Head | Phase::Lifecycle)
    };
    if can_reject {
        write_timeout(raw, write_idle).await;
    }
    if phase != Phase::Idle {
        let slot = lock(&monitor.state).current.clone();
        let slot = slot.unwrap_or_else(|| {
            let mut slots = lock(slots);
            let slot = Arc::new(Mutex::new(Exchange::new(slots.len() as u64 + 1)));
            slots.push(Arc::clone(&slot));
            slot
        });
        let mut state = lock(&slot);
        state.report.status = if can_reject {
            Some(408)
        } else {
            state.report.status
        };
        state.report.delivery = if phase == Phase::Head {
            HttpDelivery::RejectedBeforeHost
        } else {
            HttpDelivery::Cancelled
        };
        state.report.failure = Some(Arc::new(deadline_error("HTTP phase deadline exceeded")));
        state.cancellation.cancel();
    }
}
