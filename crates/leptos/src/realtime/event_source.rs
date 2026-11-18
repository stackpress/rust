//! SSE state and options; the browser alone owns retry and Last-Event-ID replay mechanics.
use super::*;

/// Explicit EventSource credentials and a bounded set of named event listeners.
#[derive(Clone, Debug, Default)]
pub struct EventSourceOptions {
    pub(super) credentials: bool,
    pub(super) events: Vec<String>,
}
impl EventSourceOptions {
    /// Opt into browser credentials for an application-configured cross-origin CORS policy.
    pub fn with_credentials(mut self, enabled: bool) -> Self {
        self.credentials = enabled;
        self
    }
    /// Add at most 32 unique event names of 1..=256 bytes, excluding CR/LF/NUL.
    pub fn with_events(mut self, events: &[&str]) -> RealtimeResult<Self> {
        if events.len() > 32
            || events
                .iter()
                .any(|v| v.is_empty() || v.len() > 256 || v.contains(['\r', '\n', '\0']))
        {
            return Err(RealtimeError::Invalid);
        }
        self.events.clear();
        for event in events {
            if !self.events.iter().any(|value| value == event) {
                self.events.push((*event).into());
            }
        }
        Ok(self)
    }
}
/// Latest complete browser message; retaining it does not provide durable replay.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventSourceMessage {
    /// UTF-8 event data, at most 64 KiB.
    pub data: String,
    /// Browser event type, at most 256 bytes.
    pub event: String,
    /// Opaque cursor, at most 1 KiB; an empty value represents a reset.
    pub last_event_id: String,
}
/// Copyable arena handle; copying never creates another native connection.
#[derive(Clone, Copy)]
pub struct EventSourceHandle {
    pub(super) state: RwSignal<ConnectionState>,
    pub(super) latest: RwSignal<Option<EventSourceMessage>>,
    pub(super) error: RwSignal<Option<RealtimeError>>,
    pub(super) running: RwSignal<bool>,
    #[cfg(target_arch = "wasm32")]
    pub(super) lease: StoredValue<Option<super::dom::EventSourceLease>, LocalStorage>,
}
impl EventSourceHandle {
    /// Observe the browser transport state without gaining mutation authority.
    pub fn state(&self) -> ReadSignal<ConnectionState> {
        self.state.read_only()
    }
    /// Observe only the latest bounded message, including named type and resume cursor.
    pub fn latest(&self) -> ReadSignal<Option<EventSourceMessage>> {
        self.latest.read_only()
    }
    /// Observe a stable error category; EventSource may still be reconnecting after an error.
    pub fn error(&self) -> ReadSignal<Option<RealtimeError>> {
        self.error.read_only()
    }
    /// Permanently close this instance and remove its listeners; disposal performs the same cleanup.
    pub fn close(&self) -> RealtimeResult<()> {
        if self.running.try_set(false).is_some() {
            return Err(RealtimeError::Disposed);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = self.lease.try_update_value(Option::take);
        let _ = self.state.try_set(ConnectionState::Disconnected);
        Ok(())
    }
}
/// Create one owner-scoped SSE handle. Native SSR returns inert Disconnected state.
/// Browser connection construction runs in an Effect after mount, never during rendering.
/// # Errors
/// Rejects missing Owner, excessive URL length, fragments or unsupported explicit schemes.
pub fn use_event_source(
    url: String,
    options: EventSourceOptions,
) -> RealtimeResult<EventSourceHandle> {
    require_owner()?;
    validate_url(&url, false)?;
    let handle = EventSourceHandle {
        state: RwSignal::new(ConnectionState::Disconnected),
        latest: RwSignal::new(None),
        error: RwSignal::new(None),
        running: RwSignal::new(true),
        #[cfg(target_arch = "wasm32")]
        lease: StoredValue::new_local(None),
    };
    #[cfg(target_arch = "wasm32")]
    super::dom::event_source(url, options, handle);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (url, options);
    Ok(handle)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_ssr_is_inert_and_disposal_is_fallible() {
        let owner = Owner::new();
        let handle = owner.with(|| use_event_source("/events".into(), Default::default()).unwrap());
        assert_eq!(
            handle.state().get_untracked(),
            ConnectionState::Disconnected
        );
        assert!(handle.latest().get_untracked().is_none());
        assert!(handle.error().get_untracked().is_none());
        handle.close().unwrap();
        owner.cleanup();
        assert_eq!(handle.close(), Err(RealtimeError::Disposed));
    }
    #[test]
    fn named_event_limits_and_explicit_credentials() {
        let options = EventSourceOptions::default()
            .with_credentials(true)
            .with_events(&["message", "ready", "ready"])
            .unwrap();
        assert!(options.credentials);
        assert_eq!(options.events.len(), 2);
        for value in ["", "a\n", "a\r", "a\0"] {
            assert!(EventSourceOptions::default().with_events(&[value]).is_err());
        }
        assert!(
            EventSourceOptions::default()
                .with_events(&[&"x".repeat(257)])
                .is_err()
        );
        assert!(
            EventSourceOptions::default()
                .with_events(&["x"; 33])
                .is_err()
        );
    }
}
