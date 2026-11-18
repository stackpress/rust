//! Protocol admission and finite session policy, separate from ordinary HTTP limits.

use crate::{HttpServer, report::lock};
use stackpress_lib::{Error, Result};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};
#[cfg(feature = "websocket")]
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

/// Validated SSE host policy, inactive until explicitly selected on HttpServer.
#[cfg(feature = "sse")]
#[derive(Clone, Debug)]
pub struct SseHostLimits {
    pub(crate) sessions: usize,
    pub(crate) options: stackpress_server::http::sse::SseOptions,
}
#[cfg(feature = "sse")]
impl Default for SseHostLimits {
    /// Select the approved 64-session and bounded SSE defaults.
    fn default() -> Self {
        Self {
            sessions: 64,
            options: Default::default(),
        }
    }
}
#[cfg(feature = "sse")]
impl SseHostLimits {
    /// Choose a positive session cap and validated finite body policy.
    pub fn new(sessions: usize, options: stackpress_server::http::sse::SseOptions) -> Result<Self> {
        check_cap(sessions)?;
        Ok(Self { sessions, options })
    }
    /// Maximum pending plus published SSE sessions.
    pub fn max_sessions(&self) -> usize {
        self.sessions
    }
    /// Read the host's immutable session ceilings.
    pub fn options(&self) -> stackpress_server::http::sse::SseOptions {
        self.options
    }
}
/// Validated WebSocket host limits; dependency config is never exposed mutably.
#[cfg(feature = "websocket")]
#[derive(Clone, Debug)]
pub struct WebSocketHostLimits {
    pub(crate) sessions: usize,
    pub(crate) age: Duration,
    pub(crate) message: usize,
    pub(crate) frame: usize,
    pub(crate) queue: usize,
    pub(crate) queue_bytes: usize,
    pub(crate) read: usize,
    pub(crate) write: usize,
    pub(crate) write_max: usize,
    pub(crate) ping: Duration,
    pub(crate) pong: Duration,
    pub(crate) idle: Duration,
    pub(crate) send: Duration,
    pub(crate) close: Duration,
}
#[cfg(feature = "websocket")]
impl Default for WebSocketHostLimits {
    /// Apply the approved bounded buffers, queues and clocks.
    fn default() -> Self {
        Self {
            sessions: 64,
            age: Duration::from_secs(3600),
            message: 65536,
            frame: 65536,
            queue: 16,
            queue_bytes: 1048576,
            read: 16384,
            write: 16384,
            write_max: 262144,
            ping: Duration::from_secs(20),
            pong: Duration::from_secs(10),
            idle: Duration::from_secs(60),
            send: Duration::from_secs(10),
            close: Duration::from_secs(5),
        }
    }
}
#[cfg(feature = "websocket")]
impl WebSocketHostLimits {
    /// Change the cap, preserving the validated finite policy.
    pub fn with_max_sessions(mut self, sessions: usize) -> Result<Self> {
        check_cap(sessions)?;
        self.sessions = sessions;
        Ok(self)
    }
    /// Reduce the one-hour maximum session clock.
    pub fn with_max_age(mut self, age: Duration) -> Result<Self> {
        check_duration(age, 3600)?;
        self.age = age;
        Ok(self)
    }
    /// Choose message/frame ceilings up to 64 KiB and validate write-buffer headroom.
    pub fn with_message_limits(mut self, message: usize, frame: usize) -> Result<Self> {
        if message == 0 || message > 65536 || frame == 0 || frame > 65536 {
            return Err(Error::invalid("invalid WebSocket message limits"));
        }
        self.message = message;
        self.frame = frame;
        self.validate()?;
        Ok(self)
    }
    /// Select bounded queue count/bytes; no queue may exceed the approved hard ceiling.
    pub fn with_queue_limits(mut self, count: usize, bytes: usize) -> Result<Self> {
        if count == 0 || count > 16 || bytes == 0 || bytes > 1048576 {
            return Err(Error::invalid("invalid WebSocket queue limits"));
        }
        self.queue = count;
        self.queue_bytes = bytes;
        Ok(self)
    }
    /// Validate private dependency buffer sizes before any asserting constructor is called.
    pub fn with_buffers(mut self, read: usize, write: usize, max: usize) -> Result<Self> {
        self.read = read;
        self.write = write;
        self.write_max = max;
        self.validate()?;
        Ok(self)
    }
    /// Set finite ping, matching-pong, inbound-idle, send and close deadlines.
    pub fn with_timing(
        mut self,
        ping: Duration,
        pong: Duration,
        idle: Duration,
        send: Duration,
        close: Duration,
    ) -> Result<Self> {
        for (value, max) in [(ping, 20), (pong, 10), (idle, 60), (send, 10), (close, 5)] {
            check_duration(value, max)?;
        }
        self.ping = ping;
        self.pong = pong;
        self.idle = idle;
        self.send = send;
        self.close = close;
        Ok(self)
    }
    /// Inspect the admitted connection ceiling.
    pub fn max_sessions(&self) -> usize {
        self.sessions
    }
    /// Reject dependency assertion conditions and excess allocation before configuration conversion.
    pub(crate) fn validate(&self) -> Result<()> {
        check_cap(self.sessions)?;
        if self.read == 0
            || self.read > 16384
            || self.write > 16384
            || self.write_max > 262144
            || self.write_max <= self.write
            || self
                .write
                .checked_add(self.message)
                .and_then(|n| n.checked_add(14))
                .is_none_or(|n| n > self.write_max)
        {
            return Err(Error::invalid("invalid WebSocket buffer headroom"));
        }
        Ok(())
    }
}
/// Shared counters contain no request headers, snapshots or payloads.
#[derive(Clone, Default, Debug)]
pub struct RealtimeStats {
    inner: Arc<Counters>,
}
#[derive(Default, Debug)]
struct Counters {
    active: Mutex<(usize, usize)>,
    rejected: AtomicU64,
    bytes: AtomicU64,
    closed: AtomicU64,
    forced: AtomicU64,
}
impl RealtimeStats {
    /// Read currently reserved SSE and WS slots, including unpublished handoffs.
    pub fn active(&self) -> (usize, usize) {
        *lock(&self.inner.active)
    }
    /// Total rejected protocol admissions.
    pub fn rejected(&self) -> u64 {
        self.inner.rejected.load(Ordering::Relaxed)
    }
    /// Bytes handed to transport drivers, without remote-acknowledgement semantics.
    pub fn bytes(&self) -> u64 {
        self.inner.bytes.load(Ordering::Relaxed)
    }
    /// Terminal session observations.
    pub fn closed(&self) -> u64 {
        self.inner.closed.load(Ordering::Relaxed)
    }
    /// Sessions terminated forcibly at a deadline.
    pub fn forced(&self) -> u64 {
        self.inner.forced.load(Ordering::Relaxed)
    }
    /// Account bounded outgoing bytes without retaining payloads.
    pub(crate) fn add_bytes(&self, n: usize) {
        self.inner.bytes.fetch_add(n as u64, Ordering::Relaxed);
    }
    /// Record a forced terminal observation once, owned by its driver.
    #[cfg(feature = "websocket")]
    pub(crate) fn force(&self) {
        self.inner.forced.fetch_add(1, Ordering::Relaxed);
    }
}
/// RAII permit never lives in a ResponseView clone, only in exclusive host output ownership.
pub(crate) struct SessionPermit {
    _permit: OwnedSemaphorePermit,
    stats: RealtimeStats,
    websocket: bool,
}
impl Drop for SessionPermit {
    /// Return counters/permits on every denied, replaced, failed and terminal path.
    fn drop(&mut self) {
        let mut active = lock(&self.stats.inner.active);
        if self.websocket {
            active.1 = active.1.saturating_sub(1);
        } else {
            active.0 = active.0.saturating_sub(1);
        }
        self.stats.inner.closed.fetch_add(1, Ordering::Relaxed);
    }
}
impl HttpServer {
    /// Activate SSE with its independent finite policy; complete cross-cap validation is at serve.
    #[cfg(feature = "sse")]
    pub fn with_sse_limits(mut self, limits: SseHostLimits) -> Result<Self> {
        check_cap(limits.sessions)?;
        self.sse_slots = Arc::new(Semaphore::new(limits.sessions));
        self.sse = Some(limits);
        Ok(self)
    }
    /// Activate WebSockets, preserving feature-off and unselected-host behavior.
    #[cfg(feature = "websocket")]
    pub fn with_websocket_limits(mut self, limits: WebSocketHostLimits) -> Result<Self> {
        limits.validate()?;
        self.ws_slots = Arc::new(Semaphore::new(limits.sessions));
        self.websocket = Some(limits);
        Ok(self)
    }
    /// Borrow counters without granting admission or exposing payloads.
    pub fn realtime_stats(&self) -> RealtimeStats {
        self.realtime_stats.clone()
    }
    /// Validate the complete policy independent of builder order, before accepting any work.
    pub(crate) fn validate_realtime(&self) -> Result<()> {
        let mut total = 0usize;
        #[cfg(feature = "sse")]
        if let Some(limits) = &self.sse {
            total = total
                .checked_add(limits.sessions)
                .ok_or_else(|| Error::invalid("session cap overflow"))?;
        }
        #[cfg(feature = "websocket")]
        if let Some(limits) = &self.websocket {
            total = total
                .checked_add(limits.sessions)
                .ok_or_else(|| Error::invalid("session cap overflow"))?;
        }
        if total >= self.limits.max_connections {
            return Err(Error::invalid(
                "realtime caps must leave ordinary HTTP capacity",
            ));
        }
        Ok(())
    }
    /// Reserve synchronously after validation; no waiter, callback or source poll on rejection.
    pub(crate) fn reserve_session(
        &self,
        slots: &Arc<Semaphore>,
        websocket: bool,
    ) -> Result<SessionPermit> {
        let permit = Arc::clone(slots).try_acquire_owned().map_err(|_| {
            self.realtime_stats
                .inner
                .rejected
                .fetch_add(1, Ordering::Relaxed);
            Error::limit("realtime admission exhausted").with_code(503)
        })?;
        let mut active = lock(&self.realtime_stats.inner.active);
        if websocket {
            active.1 += 1;
        } else {
            active.0 += 1;
        }
        drop(active);
        Ok(SessionPermit {
            _permit: permit,
            stats: self.realtime_stats.clone(),
            websocket,
        })
    }
}
/// Check semaphore and global counter representability before allocation.
fn check_cap(n: usize) -> Result<()> {
    if n == 0 || n > Semaphore::MAX_PERMITS {
        return Err(Error::invalid("invalid realtime session cap"));
    }
    Ok(())
}
/// Cap all new durations at one hour, preventing zero and monotonic-clock overflow.
#[cfg(feature = "websocket")]
fn check_duration(value: Duration, max: u64) -> Result<()> {
    if value.is_zero() || value > Duration::from_secs(max) {
        return Err(Error::invalid("invalid realtime duration"));
    }
    Ok(())
}

#[cfg(all(test, feature = "sse", feature = "websocket"))]
mod tests {
    use super::*;
    #[tokio::test]
    async fn complete_caps_validate_independently_of_builder_order() {
        let app = Arc::new(
            stackpress_server::ServerBuilder::new()
                .build()
                .await
                .unwrap(),
        );
        let host = HttpServer::new(app.clone(), Default::default()).unwrap();
        let limit = host.limits.max_connections;
        let sse = SseHostLimits::new(limit - 1, Default::default()).unwrap();
        let ws = WebSocketHostLimits::default().with_max_sessions(1).unwrap();
        for reverse in [false, true] {
            let host = HttpServer::new(app.clone(), Default::default()).unwrap();
            let host = if reverse {
                host.with_websocket_limits(ws.clone())
                    .unwrap()
                    .with_sse_limits(sse.clone())
                    .unwrap()
            } else {
                host.with_sse_limits(sse.clone())
                    .unwrap()
                    .with_websocket_limits(ws.clone())
                    .unwrap()
            };
            assert!(host.validate_realtime().is_err());
        }
        assert!(SseHostLimits::new(0, Default::default()).is_err());
        let sse = SseHostLimits::new(1, Default::default()).unwrap();
        assert_eq!(sse.max_sessions(), 1);
        assert_eq!(sse.options().max_bytes(), 64 * 1024 * 1024);
        let host = HttpServer::new(app, Default::default())
            .unwrap()
            .with_sse_limits(sse)
            .unwrap();
        assert!(host.validate_realtime().is_ok());
        let permit = host.reserve_session(&host.sse_slots, false).unwrap();
        assert!(host.reserve_session(&host.sse_slots, false).is_err());
        let stats = host.realtime_stats();
        assert_eq!(stats.active(), (1, 0));
        assert_eq!(stats.rejected(), 1);
        drop(permit);
        assert_eq!(stats.active(), (0, 0));
        assert_eq!(stats.closed(), 1);
    }
}
