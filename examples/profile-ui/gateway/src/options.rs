//! Explicit finite listener inputs and trusted byte/time/admission policy.

use stackpress_framework_view::BasePath;
use std::{io, path::PathBuf, time::Duration};

/// Finite independent ceilings; buffered input limits apply before HttpAdapter parsing.
#[derive(Clone, Debug)]
pub struct GatewayLimits {
    /// Total bytes in body frames, including chunked requests without Content-Length.
    pub body_bytes: usize,
    /// Header name/value bytes; HTTP/1 parser also bounds its raw head buffer.
    pub header_bytes: usize,
    /// Maximum native header occurrences.
    pub headers: usize,
    /// Deadline for receiving complete request headers.
    pub header_timeout: Duration,
    /// One total body-acquisition deadline, never reset on progress.
    pub body_timeout: Duration,
    /// Entire buffered request, including operation/render execution.
    pub request_timeout: Duration,
    /// Maximum keep-alive connection lifetime.
    pub connection_timeout: Duration,
    /// Graceful in-flight work deadline after shutdown/lifetime expiry.
    pub drain_timeout: Duration,
    /// Live accepted connections; excess admissions are closed immediately.
    pub connections: usize,
}
impl Default for GatewayLimits {
    /// Public read-only preview defaults, independent of caller headers or DTO fields.
    fn default() -> Self {
        Self {
            body_bytes: 64 * 1024,
            header_bytes: 16 * 1024,
            headers: 64,
            header_timeout: Duration::from_secs(3),
            body_timeout: Duration::from_secs(2),
            request_timeout: Duration::from_secs(5),
            connection_timeout: Duration::from_secs(30),
            drain_timeout: Duration::from_secs(2),
            connections: 8,
        }
    }
}
impl GatewayLimits {
    /// Reject expanded/unbounded policies before publishing the gateway.
    ///
    /// # Errors
    /// Returns invalid input for excessive counts/bytes or zero/excessive deadlines.
    pub fn validate(&self) -> io::Result<()> {
        if self.body_bytes > 64 * 1024
            || self.header_bytes > 16 * 1024
            || self.headers > 64
            || self.connections == 0
            || self.connections > 8
            || [
                self.header_timeout,
                self.body_timeout,
                self.request_timeout,
                self.connection_timeout,
                self.drain_timeout,
            ]
            .iter()
            .any(|d| d.is_zero() || *d > Duration::from_secs(30))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "gateway requires finite byte/time/admission limits",
            ));
        }
        Ok(())
    }
}

/// Trusted executable startup inputs; there is deliberately no default port.
#[derive(Debug)]
pub struct LaunchOptions {
    /// Explicit argv value or PORT environment value, never a hardcoded fallback.
    pub port: u16,
    /// Previously built hydrate assets and manifest.
    pub assets: PathBuf,
    /// Explicit public application mount.
    pub base: BasePath,
    /// Finite process lifetime; devmetrics may stop it sooner.
    pub duration: Duration,
}
impl LaunchOptions {
    /// Parse named argv fields; pass PORT explicitly for deterministic tests.
    ///
    /// # Examples
    /// ```
    /// use stackpress_example_profile_ui_gateway::LaunchOptions;
    /// let args = ["--port", "3001", "--base", "/demo"].map(String::from);
    /// let options = LaunchOptions::parse(&args, None)?;
    /// assert_eq!(options.port, 3001);
    /// assert_eq!(options.base.as_str(), "/demo");
    /// # Ok::<(), std::io::Error>(())
    /// ```
    ///
    /// # Errors
    /// Rejects unknown/duplicate/missing fields, zero port and invalid finite duration/base.
    pub fn parse(args: &[String], port_env: Option<&str>) -> io::Result<Self> {
        let mut values = std::collections::BTreeMap::new();
        for pair in args.chunks(2) {
            if pair.len() != 2
                || !["--port", "--assets", "--base", "--duration-seconds"]
                    .contains(&pair[0].as_str())
                || values.insert(pair[0].as_str(), pair[1].as_str()).is_some()
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "usage: --port N [--assets DIR] [--base /demo] [--duration-seconds N]",
                ));
            }
        }
        let invalid = || {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "explicit nonzero port and 1..=7200 second duration required",
            )
        };
        let port = values
            .get("--port")
            .copied()
            .or(port_env)
            .ok_or_else(invalid)?
            .parse::<u16>()
            .map_err(|_| invalid())?;
        let seconds = values
            .get("--duration-seconds")
            .copied()
            .unwrap_or("7200")
            .parse::<u64>()
            .map_err(|_| invalid())?;
        if port == 0 || !(1..=7200).contains(&seconds) {
            return Err(invalid());
        }
        Ok(Self {
            port,
            assets: values.get("--assets").map_or_else(
                || PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../dist/hydrate"),
                PathBuf::from,
            ),
            base: BasePath::new(values.get("--base").copied().unwrap_or(""))
                .map_err(io::Error::other)?,
            duration: Duration::from_secs(seconds),
        })
    }
}
