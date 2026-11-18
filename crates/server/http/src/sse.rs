//! Demand-driven native SSE heartbeats; source, clock and admission share body ownership.
use crate::{monitor::Monitor, realtime::SessionPermit};
use futures_core::Stream;
use stackpress_lib::Result;
use stackpress_server::http::{
    sse::{SseDriver, SseOptions},
    stream::HttpBody,
};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{Instant, Sleep};

/// The native wrapper samples time on every demand poll and owns the sole timer.
struct NativeClock;
impl Stream for NativeClock {
    type Item = Duration;
    /// NativeSse samples elapsed time directly; this placeholder never starts a second timer.
    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Duration>> {
        Poll::Pending
    }
}
/// Exclusive source, clock and permit; no descriptor clone can retain native admission.
pub(crate) struct NativeSse {
    driver: SseDriver<HttpBody, NativeClock>,
    _permit: SessionPermit,
    monitor: Arc<Monitor>,
    stats: crate::RealtimeStats,
    started: Instant,
    sleep: Pin<Box<Sleep>>,
    options: SseOptions,
}
impl NativeSse {
    /// Begin clocks at publication; every yielded complete event rearms the quiet interval.
    pub(crate) fn new(
        body: HttpBody,
        options: SseOptions,
        permit: SessionPermit,
        monitor: Arc<Monitor>,
        stats: crate::RealtimeStats,
    ) -> Self {
        let started = Instant::now();
        let deadline = started + options.heartbeat_interval().min(options.max_age());
        Self {
            driver: SseDriver::new(body, NativeClock, options),
            _permit: permit,
            monitor,
            stats,
            started,
            sleep: Box::pin(tokio::time::sleep_until(deadline)),
            options,
        }
    }
}
impl Stream for NativeSse {
    type Item = Result<Vec<u8>>;
    /// Observe stop and age before source demand; register wakes before inspecting sticky stop.
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        *crate::report::lock(&this.monitor.stop_waker) = Some(cx.waker().clone());
        if this.monitor.is_stopping() {
            this.driver.stop();
        }
        let now = Instant::now();
        this.driver
            .observe_elapsed(now.saturating_duration_since(this.started));
        let _ = this.sleep.as_mut().poll(cx);
        let item = Pin::new(&mut this.driver).poll_next(cx);
        if let Poll::Ready(Some(Ok(bytes))) = &item {
            this.stats.add_bytes(bytes.len());
            this.sleep.as_mut().reset(
                (now + this.options.heartbeat_interval())
                    .min(this.started + this.options.max_age()),
            );
        }
        item
    }
}
/// Select the strictest requested/host controls; caller has already validated both policies.
pub(crate) fn selected(request: SseOptions, host: SseOptions) -> Result<SseOptions> {
    let heartbeat = request.heartbeat_interval().min(host.heartbeat_interval());
    let idle = request.write_idle().min(host.write_idle());
    SseOptions::new(
        request.max_age().min(host.max_age()),
        heartbeat,
        idle,
        request.max_bytes().min(host.max_bytes()),
    )
    .map(|options| {
        options.with_encode_limits(request.encode_limits().intersection(host.encode_limits()))
    })
}

impl crate::HttpServer {
    /// Charge a typed, fully converted body before publication; rejected sources are never polled.
    pub(crate) fn prepare_sse(
        &self,
        native: http::Response<HttpBody>,
        stopping: bool,
        pending: &mut Option<(SseOptions, SessionPermit)>,
    ) -> Result<http::Response<HttpBody>> {
        let Some(requested) = native.body().sse_options() else {
            return Ok(native);
        };
        let selected = self
            .sse
            .as_ref()
            .filter(|_| !stopping)
            .map(|policy| selected(requested, policy.options))
            .transpose()?;
        // Admission is immediate; a rejected source is dropped without its first poll.
        if let Some(options) = selected
            && let Ok(permit) = self.reserve_session(&self.sse_slots, false)
        {
            *pending = Some((options, permit));
            return Ok(native);
        }
        drop(native);
        let mut denied = http::Response::new(HttpBody::empty());
        *denied.status_mut() = http::StatusCode::SERVICE_UNAVAILABLE;
        denied
            .headers_mut()
            .insert("content-length", http::HeaderValue::from_static("0"));
        denied
            .headers_mut()
            .insert("connection", http::HeaderValue::from_static("close"));
        Ok(denied)
    }
}
