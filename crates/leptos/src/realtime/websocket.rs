//! Bounded browser sends and optional retry policy; no Rust-side message queue or resend.
use super::*;
/// Explicit browser policy; reconnect defaults off and always has a finite attempt cap.
#[derive(Clone, Copy, Debug)]
pub struct BrowserWebSocketOptions {
    pub(super) message: usize,
    pub(super) buffered: usize,
    pub(super) retries: u8,
}
impl Default for BrowserWebSocketOptions {
    /// Retain at most 64 KiB per message and refuse sends above 256 KiB bufferedAmount.
    fn default() -> Self {
        Self {
            message: 65536,
            buffered: 262144,
            retries: 0,
        }
    }
}
impl BrowserWebSocketOptions {
    /// Reduce the message and browser-send buffer ceilings; zero never means unlimited.
    pub fn with_limits(mut self, message: usize, buffered: usize) -> RealtimeResult<Self> {
        if message == 0 || message > 65536 || buffered == 0 || buffered > 262144 {
            return Err(RealtimeError::Invalid);
        }
        self.message = message;
        self.buffered = buffered;
        Ok(self)
    }
    /// Opt into at most five retries; delays are 1, 2, 4, 8 and 16 seconds, capped at 30.
    pub fn with_reconnect(mut self, attempts: u8) -> RealtimeResult<Self> {
        if attempts > 5 {
            return Err(RealtimeError::Invalid);
        }
        self.retries = attempts;
        Ok(self)
    }
    /// Charge the next send against both the payload and browser's existing buffered bytes.
    pub(super) fn check_send(&self, bytes: usize, buffered: usize) -> RealtimeResult<()> {
        if bytes > self.message {
            return Err(RealtimeError::Invalid);
        }
        if buffered
            .checked_add(bytes)
            .is_none_or(|sum| sum > self.buffered)
        {
            return Err(RealtimeError::Backpressure);
        }
        Ok(())
    }
    /// Return a bounded retry delay; only a successful open resets the caller's attempt count.
    pub fn retry_delay(&self, attempt: u8) -> Option<i32> {
        if attempt >= self.retries {
            return None;
        }
        [1000, 2000, 4000, 8000, 16000, 30000]
            .get(usize::from(attempt))
            .copied()
    }
}
/// Latest complete text or binary message; browser internal receive buffering remains external.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrowserWsMessage {
    /// UTF-8 text within the configured byte bound.
    Text(String),
    /// ArrayBuffer bytes within the configured bound.
    Binary(Vec<u8>),
}
impl BrowserWsMessage {
    /// Inspect owned payload size without copying it.
    pub(super) fn len(&self) -> usize {
        match self {
            Self::Text(text) => text.len(),
            Self::Binary(bytes) => bytes.len(),
        }
    }
}
/// One owner-scoped connection; copies refer to the same native object and retry timer.
#[derive(Clone, Copy)]
pub struct WebSocketHandle {
    pub(super) state: RwSignal<ConnectionState>,
    pub(super) latest: RwSignal<Option<BrowserWsMessage>>,
    pub(super) error: RwSignal<Option<RealtimeError>>,
    pub(super) running: RwSignal<bool>,
    pub(super) generation: RwSignal<u32>,
    pub(super) options: BrowserWebSocketOptions,
    #[cfg(target_arch = "wasm32")]
    pub(super) lease: StoredValue<Option<super::dom::WebSocketLease>, LocalStorage>,
    #[cfg(target_arch = "wasm32")]
    pub(super) timer: StoredValue<Option<super::dom::Timer>, LocalStorage>,
}
impl WebSocketHandle {
    /// Observe transport state; successful open does not authorize privileged messages.
    pub fn state(&self) -> ReadSignal<ConnectionState> {
        self.state.read_only()
    }
    /// Observe the latest complete, bounded message only.
    pub fn latest(&self) -> ReadSignal<Option<BrowserWsMessage>> {
        self.latest.read_only()
    }
    /// Observe sanitized browser/input/backpressure failures.
    pub fn error(&self) -> ReadSignal<Option<RealtimeError>> {
        self.error.read_only()
    }
    /// Send immediately if open and below both byte bounds; no message is queued in Rust.
    pub fn send(&self, message: BrowserWsMessage) -> RealtimeResult<()> {
        self.options.check_send(message.len(), 0)?;
        match self.state.try_get_untracked() {
            None => return Err(RealtimeError::Disposed),
            Some(ConnectionState::Open) => {}
            _ => return Err(RealtimeError::Disconnected),
        }
        #[cfg(target_arch = "wasm32")]
        return super::dom::send(*self, message);
        #[cfg(not(target_arch = "wasm32"))]
        Err(RealtimeError::Disconnected)
    }
    /// Close explicitly, cancel retries and remove listeners. Browser codes are 1000 or 3000..4999.
    pub fn close(&self, code: u16, reason: &str) -> RealtimeResult<()> {
        validate_close(code, reason)?;
        if self.running.try_set(false).is_some() {
            return Err(RealtimeError::Disposed);
        }
        #[cfg(target_arch = "wasm32")]
        super::dom::close(*self, code, reason)?;
        let _ = self.state.try_set(ConnectionState::Disconnected);
        Ok(())
    }
    /// Start a fresh connection after mount; explicit reconnect never replays earlier sends.
    pub fn reconnect(&self) -> RealtimeResult<()> {
        let generation = self
            .generation
            .try_get_untracked()
            .ok_or(RealtimeError::Disposed)?;
        let next = generation.checked_add(1).ok_or(RealtimeError::Invalid)?;
        let _ = self.running.try_set(true);
        let _ = self.generation.try_set(next);
        Ok(())
    }
}
/// Validate browser-supported close metadata before invoking the native browser method.
fn validate_close(code: u16, reason: &str) -> RealtimeResult<()> {
    if (code != 1000 && !(3000..=4999).contains(&code)) || reason.len() > 123 {
        return Err(RealtimeError::Invalid);
    }
    Ok(())
}
/// Construct an owner-scoped WebSocket handle; native SSR opens nothing.
/// # Errors
/// Rejects missing Owner, excessive URL length, fragments or unsupported explicit schemes.
pub fn use_websocket(
    url: String,
    options: BrowserWebSocketOptions,
) -> RealtimeResult<WebSocketHandle> {
    require_owner()?;
    validate_url(&url, true)?;
    let handle = WebSocketHandle {
        state: RwSignal::new(ConnectionState::Disconnected),
        latest: RwSignal::new(None),
        error: RwSignal::new(None),
        running: RwSignal::new(true),
        generation: RwSignal::new(0),
        options,
        #[cfg(target_arch = "wasm32")]
        lease: StoredValue::new_local(None),
        #[cfg(target_arch = "wasm32")]
        timer: StoredValue::new_local(None),
    };
    #[cfg(target_arch = "wasm32")]
    super::dom::websocket(url, handle);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = url;
    Ok(handle)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn send_retry_and_close_boundaries() {
        let options = BrowserWebSocketOptions::default()
            .with_limits(4, 8)
            .unwrap()
            .with_reconnect(5)
            .unwrap();
        assert!(options.check_send(4, 4).is_ok());
        assert_eq!(options.check_send(5, 0), Err(RealtimeError::Invalid));
        assert_eq!(options.check_send(4, 5), Err(RealtimeError::Backpressure));
        assert!(options.check_send(1, usize::MAX).is_err());
        assert_eq!(
            (0..=5).map(|n| options.retry_delay(n)).collect::<Vec<_>>(),
            vec![
                Some(1000),
                Some(2000),
                Some(4000),
                Some(8000),
                Some(16000),
                None
            ]
        );
        assert!(BrowserWebSocketOptions::default().retry_delay(0).is_none());
        assert!(options.with_reconnect(6).is_err());
        for (message, buffered) in [(0, 1), (65537, 1), (1, 0), (1, 262145)] {
            assert!(options.with_limits(message, buffered).is_err());
        }
        for code in [1000, 3000, 4999] {
            assert!(validate_close(code, &"x".repeat(123)).is_ok());
        }
        for code in [999, 1001, 1005, 1006, 1015, 2999, 5000] {
            assert!(validate_close(code, "").is_err());
        }
        assert!(validate_close(1000, &"é".repeat(62)).is_err());
    }
    #[test]
    fn native_ssr_send_close_reconnect_and_disposal() {
        let owner = Owner::new();
        let handle = owner.with(|| use_websocket("/ws".into(), Default::default()).unwrap());
        assert_eq!(
            handle.state().get_untracked(),
            ConnectionState::Disconnected
        );
        assert!(handle.latest().get_untracked().is_none());
        assert!(handle.error().get_untracked().is_none());
        assert_eq!(
            handle.send(BrowserWsMessage::Text("x".into())),
            Err(RealtimeError::Disconnected)
        );
        assert_eq!(
            handle.send(BrowserWsMessage::Binary(vec![1; 65537])),
            Err(RealtimeError::Invalid)
        );
        handle.close(1000, "").unwrap();
        handle.reconnect().unwrap();
        assert_eq!(
            handle.state().get_untracked(),
            ConnectionState::Disconnected
        );
        owner.cleanup();
        assert_eq!(handle.reconnect(), Err(RealtimeError::Disposed));
        assert_eq!(handle.close(1000, ""), Err(RealtimeError::Disposed));
        assert_eq!(
            handle.send(BrowserWsMessage::Binary(vec![1])),
            Err(RealtimeError::Disposed)
        );
    }
}
