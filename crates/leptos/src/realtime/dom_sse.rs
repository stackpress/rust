//! One native EventSource with browser-owned retry; no competing Rust timer exists.
use super::*;
use web_sys::{EventSource, EventSourceInit, MessageEvent};

/// Listener leases are removed before the EventSource is closed or its callbacks are freed.
pub(in crate::realtime) struct EventSourceLease {
    source: EventSource,
    listeners: Vec<Listener>,
}
impl Drop for EventSourceLease {
    /// Close on explicit stop, constructor failure or owning Leptos root disposal.
    fn drop(&mut self) {
        self.listeners.clear();
        self.source.close();
    }
}
/// Install only after the browser mount effect runs; SSR never enters this module.
pub(in crate::realtime) fn event_source(
    url: String,
    options: EventSourceOptions,
    handle: EventSourceHandle,
) {
    Effect::new(move |_| {
        let running = handle.running.get();
        // A signal change disposes the old instance before any fresh constructor is considered.
        let _ = handle.lease.try_update_value(Option::take);
        if !running {
            return;
        }
        match connect(&url, &options, handle) {
            Ok(lease) => {
                let _ = handle.lease.try_set_value(Some(lease));
            }
            Err(error) => {
                let _ = handle.error.try_set(Some(error));
                let _ = handle.state.try_set(ConnectionState::Disconnected);
            }
        }
    });
}
/// Build and register fallibly; a partially installed lease always closes on an error.
fn connect(
    url: &str,
    options: &EventSourceOptions,
    handle: EventSourceHandle,
) -> RealtimeResult<EventSourceLease> {
    let init = EventSourceInit::new();
    init.set_with_credentials(options.credentials);
    let source = EventSource::new_with_event_source_init_dict(url, &init)
        .map_err(|_| RealtimeError::Browser)?;
    let mut lease = EventSourceLease {
        source,
        listeners: Vec::new(),
    };
    let _ = handle.state.try_set(ConnectionState::Connecting);
    lease.listeners.push(Listener::new(
        lease.source.clone().into(),
        "open",
        move |_| {
            let _ = handle.error.try_set(None);
            let _ = handle.state.try_set(ConnectionState::Open);
        },
    )?);
    let source = lease.source.clone();
    lease
        .listeners
        .push(Listener::new(source.clone().into(), "error", move |_| {
            let _ = handle.error.try_set(Some(RealtimeError::Browser));
            let state = if source.ready_state() == EventSource::CLOSED {
                ConnectionState::Disconnected
            } else {
                ConnectionState::Connecting
            };
            let _ = handle.state.try_set(state);
            if state == ConnectionState::Disconnected {
                let _ = handle.running.try_set(false);
            }
        })?);
    // Default messages are installed once even when explicitly included in the named list.
    for name in std::iter::once("message").chain(
        options
            .events
            .iter()
            .map(String::as_str)
            .filter(|name| *name != "message"),
    ) {
        lease.listeners.push(Listener::new(
            lease.source.clone().into(),
            name,
            move |event| {
                let result = event
                    .dyn_into::<MessageEvent>()
                    .map_err(|_| RealtimeError::Invalid)
                    .and_then(|event| {
                        Ok(EventSourceMessage {
                            data: text(event.data(), 65536)?,
                            event: property_text(event.as_ref(), "type", 256)?,
                            last_event_id: property_text(event.as_ref(), "lastEventId", 1024)?,
                        })
                    });
                match result {
                    Ok(message) => {
                        let _ = handle.latest.try_set(Some(message));
                    }
                    Err(error) => {
                        let _ = handle.error.try_set(Some(error));
                        let _ = handle.state.try_set(ConnectionState::Disconnected);
                        // Cleanup runs in the next effect turn, after this executing closure returns.
                        let _ = handle.running.try_set(false);
                    }
                }
            },
        )?);
    }
    Ok(lease)
}
