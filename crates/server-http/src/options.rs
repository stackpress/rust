//! Validated finite network policy, independent from buffered adapter compatibility limits.

use stackpress_lib::{Error, Result};
use std::time::{Duration, Instant};

/// Native connection, parsing and deadline policy; zero never means unlimited.
///
/// # Examples
/// Tighten native policy before preparing a host; zero never selects an unlimited budget.
/// ```
/// use stackpress_server_http::NetworkLimits;
/// use std::time::Duration;
/// let limits = NetworkLimits {
///     max_connections: 16,
///     max_handshakes: 4,
///     head_timeout: Duration::from_secs(5),
///     ..Default::default()
/// };
/// assert!(limits.max_handshakes <= limits.max_connections);
/// ```
#[derive(Clone, Debug)]
pub struct NetworkLimits {
    /// Simultaneously owned connections, acquired before a host task is spawned.
    pub max_connections: usize,
    /// Simultaneous TLS handshakes, included in the connection budget.
    pub max_handshakes: usize,
    /// Raw parser header occurrence limit, before Hyper merges duplicate lengths.
    pub max_headers: usize,
    /// Hyper read/write buffer ceiling; must be at least Hyper's 8192-byte minimum.
    pub parser_buffer_bytes: usize,
    /// Aggregate normalized header name/value bytes, counting repeated retained values.
    pub max_header_bytes: usize,
    /// Maximum native request target bytes.
    pub max_target_bytes: usize,
    /// Actual decoded body bytes, checked before each retained copy.
    pub max_body_bytes: usize,
    /// Method, target, headers and decoded body aggregate.
    pub max_request_bytes: usize,
    /// Maximum exchanges per connection, including rejected requests.
    pub max_requests: usize,
    /// Absolute accepted connection lifetime; expiry begins bounded graceful drain.
    pub connection_lifetime: Duration,
    /// Absolute TLS handshake budget.
    pub handshake_timeout: Duration,
    /// Absolute complete-head budget, unaffected by progress.
    pub head_timeout: Duration,
    /// Maximum interval without head read progress.
    pub head_idle_timeout: Duration,
    /// Absolute decoded-body load budget.
    pub body_timeout: Duration,
    /// Maximum interval without decoded-body progress.
    pub body_idle_timeout: Duration,
    /// Absolute lifecycle computation budget after loading completes or fails.
    pub lifecycle_timeout: Duration,
    /// Maximum complete response lifetime, also bounding buffered slow writes.
    pub response_timeout: Duration,
    /// Maximum interval without an actual native write.
    pub write_idle_timeout: Duration,
    /// Maximum quiet interval between reusable requests.
    pub keep_alive_idle: Duration,
    /// Stop closes admission first, then drains within this budget before abort/reap.
    pub shutdown_grace: Duration,
}
impl Default for NetworkLimits {
    /// Choose the accepted finite Slice 7 defaults.
    fn default() -> Self {
        Self {
            max_connections: 256,
            max_handshakes: 32,
            max_headers: 64,
            parser_buffer_bytes: 32 * 1024,
            max_header_bytes: 16 * 1024,
            max_target_bytes: 8 * 1024,
            max_body_bytes: 1024 * 1024,
            max_request_bytes: 2 * 1024 * 1024,
            max_requests: 100,
            connection_lifetime: Duration::from_secs(300),
            handshake_timeout: Duration::from_secs(5),
            head_timeout: Duration::from_secs(10),
            head_idle_timeout: Duration::from_secs(5),
            body_timeout: Duration::from_secs(30),
            body_idle_timeout: Duration::from_secs(5),
            lifecycle_timeout: Duration::from_secs(30),
            response_timeout: Duration::from_secs(60),
            write_idle_timeout: Duration::from_secs(10),
            keep_alive_idle: Duration::from_secs(15),
            shutdown_grace: Duration::from_secs(30),
        }
    }
}
impl NetworkLimits {
    /// Validate allocation and monotonic-clock constraints before any IO is admitted.
    pub(crate) fn validate(&self) -> Result<()> {
        // Tokio semaphores have a finite representation independent from the usize range.
        if self.max_connections == 0
            || self.max_connections > tokio::sync::Semaphore::MAX_PERMITS
            || self.max_handshakes == 0
            || self.max_handshakes > self.max_connections
            || self.max_headers == 0
            || self.max_headers > self.parser_buffer_bytes
            || self.parser_buffer_bytes < 8192
            || self.max_header_bytes == 0
            || self.max_target_bytes == 0
            || self.max_body_bytes == 0
            || self.max_request_bytes == 0
            || self.max_requests == 0
        {
            return Err(Error::invalid(
                "HTTP count/byte limits must be positive and representable",
            ));
        }
        // Every deadline is absolute or progress-based, never a zero/unlimited sentinel.
        let now = Instant::now();
        for duration in [
            self.connection_lifetime,
            self.handshake_timeout,
            self.head_timeout,
            self.head_idle_timeout,
            self.body_timeout,
            self.body_idle_timeout,
            self.lifecycle_timeout,
            self.response_timeout,
            self.write_idle_timeout,
            self.keep_alive_idle,
            self.shutdown_grace,
        ] {
            if duration.is_zero() || now.checked_add(duration).is_none() {
                return Err(Error::invalid("HTTP deadlines must be positive and finite"));
            }
        }
        Ok(())
    }
}
