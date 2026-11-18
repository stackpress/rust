//! Owner-scoped EventSource and WebSocket clients, independent of the native HTTP host.
//!
//! Native SSR is inert. Browser effects connect after mount; disposal closes connections,
//! removes listeners and cancels retries. SSE reconnect/cursor handling belongs to EventSource.
//! WebSocket retries never resend messages. Applications own authentication, Origin/CORS,
//! session revocation, replay and duplicate handling. Browser-internal buffers are outside the
//! helper's bounded latest-value retention; EventSource cannot set arbitrary request headers.
use leptos::prelude::*;
use std::fmt;
#[cfg(target_arch = "wasm32")]
mod dom;
#[cfg(feature = "client-sse")]
mod event_source;
#[cfg(feature = "client-sse")]
pub use event_source::{
    EventSourceHandle, EventSourceMessage, EventSourceOptions, use_event_source,
};
#[cfg(feature = "client-websocket")]
mod websocket;
#[cfg(feature = "client-websocket")]
pub use websocket::{BrowserWebSocketOptions, BrowserWsMessage, WebSocketHandle, use_websocket};

/// Browser transport state; native SSR remains Disconnected without touching Window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    /// No active connection, including explicit close, SSR and disposal.
    Disconnected,
    /// Initial connection or browser-managed retry is pending.
    Connecting,
    /// The browser reported a successful open.
    Open,
}
/// Bounded, non-secret client failure categories; native browser exception text is not exposed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RealtimeError {
    /// Invalid URL, options, payload or close metadata.
    Invalid,
    /// The owning Leptos root no longer exists, or no Owner was provided.
    Disposed,
    /// Browser transport is absent or not open.
    Disconnected,
    /// Browser bufferedAmount would exceed the configured bound.
    Backpressure,
    /// A browser constructor, listener, transport or send operation failed.
    Browser,
}
impl fmt::Display for RealtimeError {
    /// Report only the stable category, without credentials or server-supplied payloads.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "Invalid realtime input",
            Self::Disposed => "Realtime owner disposed",
            Self::Disconnected => "Realtime connection disconnected",
            Self::Backpressure => "Realtime send backpressure",
            Self::Browser => "Browser realtime operation failed",
        })
    }
}
impl std::error::Error for RealtimeError {}
/// Client helpers use a library-local result without depending on a server/framework crate.
pub type RealtimeResult<T> = Result<T, RealtimeError>;
/// Validate bounded portable URL policy before the browser performs full URL parsing.
fn validate_url(url: &str, websocket: bool) -> RealtimeResult<()> {
    if url.is_empty() || url.len() > 2048 || url.chars().any(char::is_control) || url.contains('#')
    {
        return Err(RealtimeError::Invalid);
    }
    if let Some((scheme, _)) = url.split_once(':') {
        let valid = if websocket {
            matches!(scheme, "ws" | "wss" | "http" | "https")
        } else {
            matches!(scheme, "http" | "https")
        };
        if !valid {
            return Err(RealtimeError::Invalid);
        }
    }
    Ok(())
}
/// A current Owner is required before allocating arena-backed handles.
fn require_owner() -> RealtimeResult<()> {
    Owner::current().map(|_| ()).ok_or(RealtimeError::Disposed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_url_and_owner_admission() {
        assert_eq!(require_owner(), Err(RealtimeError::Disposed));
        for url in [
            "/events",
            "https://example.test/events",
            "http://example.test/",
        ] {
            assert!(validate_url(url, false).is_ok());
        }
        for url in ["ws://example.test/ws", "wss://example.test/ws"] {
            assert!(validate_url(url, true).is_ok());
            assert!(validate_url(url, false).is_err());
        }
        for url in ["", "javascript:alert(1)", "/a\nb", "/a#fragment"] {
            assert!(validate_url(url, true).is_err());
        }
        assert!(validate_url(&"x".repeat(2049), true).is_err());
        for error in [
            RealtimeError::Invalid,
            RealtimeError::Disposed,
            RealtimeError::Disconnected,
            RealtimeError::Backpressure,
            RealtimeError::Browser,
        ] {
            assert!(!error.to_string().is_empty());
        }
    }
}
